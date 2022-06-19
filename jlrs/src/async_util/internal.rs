use super::task::PersistentTask;
use crate::memory::frame::GcFrame;
use crate::memory::mode::Async;
use crate::memory::stack_page::StackPage;
use crate::{async_util::channel::ChannelReceiver, memory::global::Global};
use crate::{async_util::channel::OneshotSender, memory::frame::AsyncGcFrame};
use crate::{async_util::task::AsyncTask, runtime::async_rt::PersistentMessage};
use crate::{error::JlrsResult, memory::stack_page::AsyncStackPage};
use async_trait::async_trait;
use futures::{future::LocalBoxFuture, FutureExt};
use std::marker::PhantomData;

pub(crate) type InnerPersistentMessage<P> = Box<
    dyn CallPersistentMessageEnvelope<
        Input = <P as PersistentTask>::Input,
        Output = <P as PersistentTask>::Output,
    >,
>;

// What follows is a significant amount of indirection to allow different tasks to have a
// different Output, and allow users to provide an arbitrary sender that implements ReturnChannel
// to return some result.
pub(crate) enum Task {}
pub(crate) enum RegisterTask {}
pub(crate) enum Persistent {}
pub(crate) enum RegisterPersistent {}

pub(crate) struct CallPersistentMessage<I, O, S>
where
    I: Send + Sync,
    O: Send + Sync + 'static,
    S: OneshotSender<JlrsResult<O>>,
{
    pub(crate) sender: S,

    pub(crate) input: Option<I>,

    pub(crate) _marker: PhantomData<O>,
}

#[async_trait(?Send)]
pub(crate) trait CallPersistentMessageEnvelope: Send + Sync {
    type Input;
    type Output;

    async fn respond(self: Box<Self>, result: JlrsResult<Self::Output>);
    fn input(&mut self) -> Self::Input;
}

#[async_trait(?Send)]
impl<I, O, S> CallPersistentMessageEnvelope for CallPersistentMessage<I, O, S>
where
    I: Send + Sync,
    O: Send + Sync,
    S: OneshotSender<JlrsResult<O>>,
{
    type Input = I;
    type Output = O;

    async fn respond(self: Box<Self>, result: JlrsResult<Self::Output>) {
        Box::new(self.sender).send(result).await
    }

    fn input(&mut self) -> Self::Input {
        self.input.take().unwrap()
    }
}

#[async_trait(?Send)]
trait AsyncTaskEnvelope: Send + Sync {
    type A: AsyncTask + Send + Sync;

    async fn call_run<'inner>(
        &'inner mut self,
        global: Global<'static>,
        frame: &'inner mut AsyncGcFrame<'static>,
    ) -> JlrsResult<<Self::A as AsyncTask>::Output>;
}

#[async_trait(?Send)]
impl<A: AsyncTask> AsyncTaskEnvelope for A {
    type A = Self;
    async fn call_run<'inner>(
        &'inner mut self,
        global: Global<'static>,
        frame: &'inner mut AsyncGcFrame<'static>,
    ) -> JlrsResult<<Self::A as AsyncTask>::Output> {
        self.run(global, frame).await
    }
}

trait RegisterAsyncTaskEnvelope: Send + Sync {
    type A: AsyncTask + Send + Sync;
}

impl<A: AsyncTask> RegisterAsyncTaskEnvelope for A {
    type A = Self;
}

#[async_trait(?Send)]
trait PersistentTaskEnvelope: Send + Sync {
    type P: PersistentTask + Send + Sync;

    async fn call_init<'inner>(
        &'inner mut self,
        global: Global<'static>,
        frame: &'inner mut AsyncGcFrame<'static>,
    ) -> JlrsResult<<Self::P as PersistentTask>::State>;

    async fn call_run<'inner>(
        &'inner mut self,
        global: Global<'static>,
        frame: &'inner mut AsyncGcFrame<'static>,
        state: &'inner mut <Self::P as PersistentTask>::State,
        input: <Self::P as PersistentTask>::Input,
    ) -> JlrsResult<<Self::P as PersistentTask>::Output>;
}

#[async_trait(?Send)]
impl<P> PersistentTaskEnvelope for P
where
    P: PersistentTask,
{
    type P = Self;

    async fn call_init<'inner>(
        &'inner mut self,
        global: Global<'static>,
        frame: &'inner mut AsyncGcFrame<'static>,
    ) -> JlrsResult<<Self::P as PersistentTask>::State> {
        {
            self.init(global, frame).await
        }
    }

    async fn call_run<'inner>(
        &'inner mut self,
        global: Global<'static>,
        frame: &'inner mut AsyncGcFrame<'static>,
        state: &'inner mut <Self::P as PersistentTask>::State,
        input: <Self::P as PersistentTask>::Input,
    ) -> JlrsResult<<Self::P as PersistentTask>::Output> {
        {
            let output = {
                let mut nested = frame.nest_async(Self::RUN_CAPACITY);
                let res = self.run(global, &mut nested, state, input).await;
                std::mem::drop(nested);
                res
            };

            output
        }
    }
}

trait RegisterPersistentTaskEnvelope: Send + Sync {
    type P: PersistentTask + Send + Sync;
}

impl<P: PersistentTask> RegisterPersistentTaskEnvelope for P {
    type P = Self;
}

pub(crate) struct PendingTask<O, T, Kind> {
    task: Option<T>,
    sender: O,
    _kind: PhantomData<Kind>,
}

impl<O, A> PendingTask<O, A, Task>
where
    O: OneshotSender<JlrsResult<A::Output>>,
    A: AsyncTask,
{
    pub(crate) fn new(task: A, sender: O) -> Self {
        PendingTask {
            task: Some(task),
            sender,
            _kind: PhantomData,
        }
    }

    fn split(self) -> (A, O) {
        (self.task.unwrap(), self.sender)
    }
}

impl<C, P> PendingTask<C, P, Persistent>
where
    C: ChannelReceiver<PersistentMessage<P>>,
    P: PersistentTask,
{
    pub(crate) fn new(task: P, sender: C) -> Self {
        PendingTask {
            task: Some(task),
            sender,
            _kind: PhantomData,
        }
    }

    fn split(self) -> (P, C) {
        (self.task.unwrap(), self.sender)
    }
}

impl<O, A> PendingTask<O, A, RegisterTask>
where
    O: OneshotSender<JlrsResult<()>>,
    A: AsyncTask,
{
    pub(crate) fn new(sender: O) -> Self {
        PendingTask {
            task: None,
            sender,
            _kind: PhantomData,
        }
    }

    fn sender(self) -> O {
        self.sender
    }
}

impl<O, P> PendingTask<O, P, RegisterPersistent>
where
    O: OneshotSender<JlrsResult<()>>,

    P: PersistentTask,
{
    pub(crate) fn new(sender: O) -> Self {
        PendingTask {
            task: None,
            sender,
            _kind: PhantomData,
        }
    }

    fn sender(self) -> O {
        self.sender
    }
}

#[async_trait(?Send)]
pub(crate) trait PendingTaskEnvelope: Send + Sync {
    async fn call(mut self: Box<Self>, mut stack: &mut AsyncStackPage);
}

#[async_trait(?Send)]
impl<O, A> PendingTaskEnvelope for PendingTask<O, A, Task>
where
    O: OneshotSender<JlrsResult<A::Output>>,
    A: AsyncTask,
{
    async fn call(mut self: Box<Self>, stack: &mut AsyncStackPage) {
        let (mut task, result_sender) = self.split();

        // Safety: the stack page can be reallocated because it doesn't contain any frames
        // yet. The frame is dropped at the end of the scope, the nested hierarchy of scopes is
        // maintained.
        let res = unsafe {
            if stack.size() < A::RUN_CAPACITY + 2 {
                *stack.page_mut() = StackPage::new(A::RUN_CAPACITY + 2);
            }

            // Transmute to get static lifetimes. Should be okay because tasks can't leak
            // Julia data and the frame is not dropped until the task has completed.
            let mode = Async::new(std::mem::transmute(stack.top()));
            let raw = std::mem::transmute(stack.page());
            let mut frame = AsyncGcFrame::new(raw, mode);
            let global = Global::new();

            let res = task.call_run(global, &mut frame).await;
            std::mem::drop(frame);
            res
        };

        Box::new(result_sender).send(res).await;
    }
}

#[async_trait(?Send)]
impl<O, A> PendingTaskEnvelope for PendingTask<O, A, RegisterTask>
where
    O: OneshotSender<JlrsResult<()>>,

    A: AsyncTask,
{
    async fn call(mut self: Box<Self>, stack: &mut AsyncStackPage) {
        let sender = self.sender();

        // Safety: the stack page can be reallocated because it doesn't contain any frames
        // yet. The frame is dropped at the end of the scope, the nested hierarchy of scopes is
        // maintained.
        let res = unsafe {
            if stack.size() < A::REGISTER_CAPACITY + 2 {
                *stack.page_mut() = StackPage::new(A::REGISTER_CAPACITY + 2);
            }

            let mode = Async::new(stack.top());
            let raw = stack.page();
            let mut frame = AsyncGcFrame::new(raw, mode);
            let global = Global::new();

            let res = A::register(global, &mut frame).await;
            std::mem::drop(frame);
            res
        };

        Box::new(sender).send(res).await;
    }
}

#[async_trait(?Send)]
impl<O, P> PendingTaskEnvelope for PendingTask<O, P, RegisterPersistent>
where
    O: OneshotSender<JlrsResult<()>>,

    P: PersistentTask,
{
    async fn call(mut self: Box<Self>, stack: &mut AsyncStackPage) {
        let sender = self.sender();

        // Safety: the stack page can be reallocated because it doesn't contain any frames
        // yet. The frame is dropped at the end of the scope, the nested hierarchy of scopes is
        // maintained.
        let res = unsafe {
            if stack.size() < P::REGISTER_CAPACITY + 2 {
                *stack.page_mut() = StackPage::new(P::REGISTER_CAPACITY + 2);
            }

            let mode = Async::new(stack.top());
            let raw = stack.page();
            let mut frame = AsyncGcFrame::new(raw, mode);
            let global = Global::new();

            let res = P::register(global, &mut frame).await;
            std::mem::drop(frame);
            res
        };

        Box::new(sender).send(res).await;
    }
}

#[async_trait(?Send)]
impl<C, P> PendingTaskEnvelope for PendingTask<C, P, Persistent>
where
    C: ChannelReceiver<PersistentMessage<P>>,
    P: PersistentTask,
{
    async fn call(mut self: Box<Self>, stack: &mut AsyncStackPage) {
        let (mut persistent, mut receiver) = self.split();

        // Safety: the stack page can be reallocated because it doesn't contain any frames
        // yet. The frame is dropped at the end of the scope, the nested hierarchy of scopes is
        // maintained.
        unsafe {
            if stack.size() < P::INIT_CAPACITY + 2 {
                *stack.page_mut() = StackPage::new(P::INIT_CAPACITY + 2);
            }

            // Transmute to get static lifetimes. Should be okay because tasks can't leak
            // Julia data and the frame is not dropped until the task is dropped.
            let mode = Async::new(std::mem::transmute(stack.top()));
            let raw = std::mem::transmute(stack.page());
            let mut frame = AsyncGcFrame::new(raw, mode);
            let global = Global::new();

            match persistent.call_init(global, &mut frame).await {
                Ok(mut state) => {
                    loop {
                        let mut msg = match receiver.recv().await {
                            Ok(msg) => msg.msg,
                            Err(_) => break,
                        };

                        let res = persistent
                            .call_run(global, &mut frame, &mut state, msg.input())
                            .await;

                        msg.respond(res).await;
                    }

                    persistent.exit(global, &mut frame, &mut state).await;
                }
                _ => (), // TODO: don't just drop it.
            }

            std::mem::drop(frame);
        }
    }
}

pub(crate) struct BlockingTask<F, O, T> {
    func: F,
    sender: O,
    slots: usize,
    _res: PhantomData<T>,
}

impl<F, O, T> BlockingTask<F, O, T>
where
    for<'base> F:
        Send + Sync + FnOnce(Global<'base>, &mut GcFrame<'base, Async<'base>>) -> JlrsResult<T>,
    O: OneshotSender<JlrsResult<T>>,
    T: Send + Sync + 'static,
{
    pub(crate) fn new(func: F, sender: O, slots: usize) -> Self {
        Self {
            func,
            sender,
            slots,
            _res: PhantomData,
        }
    }

    fn call<'scope>(
        self: Box<Self>,
        frame: &mut GcFrame<'scope, Async<'scope>>,
    ) -> (JlrsResult<T>, O) {
        // Safety: this method is called from a thread known to Julia, the lifetime is limited to
        // 'scope.
        let global = unsafe { Global::new() };
        let func = self.func;
        let res = func(global, frame);
        (res, self.sender)
    }
}

pub(crate) trait BlockingTaskEnvelope: Send + Sync {
    fn call(self: Box<Self>, stack: &mut AsyncStackPage) -> LocalBoxFuture<()>;
}

impl<F, O, T> BlockingTaskEnvelope for BlockingTask<F, O, T>
where
    for<'base> F:
        Send + Sync + FnOnce(Global<'base>, &mut GcFrame<'base, Async<'base>>) -> JlrsResult<T>,
    O: OneshotSender<JlrsResult<T>>,
    T: Send + Sync + 'static,
{
    fn call(self: Box<Self>, stack: &mut AsyncStackPage) -> LocalBoxFuture<'static, ()> {
        // Safety: the stack page can be reallocated because it doesn't contain any frames
        // yet. The frame is dropped at the end of the scope, the nested hierarchy of scopes is
        // maintained.
        let (res, ch) = unsafe {
            if stack.size() < self.slots + 2 {
                *stack.page_mut() = StackPage::new(self.slots + 2);
            }

            let mode = Async::new(stack.top());
            let raw = stack.page();
            let mut frame = GcFrame::new(raw, mode);
            let res = self.call(&mut frame);
            std::mem::drop(frame);
            res
        };

        async {
            OneshotSender::send(ch, res).await;
        }
        .boxed_local()
    }
}
