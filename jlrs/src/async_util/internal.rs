use super::{channel::Channel, task::PersistentTask};
use crate::memory::context::Stack;
use crate::memory::ledger::Ledger;
use crate::{async_util::channel::ChannelReceiver, memory::global::Global};
use crate::{async_util::channel::OneshotSender, memory::frame::AsyncGcFrame};
use crate::{async_util::task::AsyncTask, runtime::async_rt::PersistentMessage};
use crate::{error::JlrsResult};
use crate::{memory::frame::GcFrame, runtime::async_rt::PersistentHandle};
use async_trait::async_trait;
use futures::{future::LocalBoxFuture, FutureExt};
use std::cell::RefCell;
use std::{marker::PhantomData, num::NonZeroUsize, sync::Arc};

pub(crate) type InnerPersistentMessage<P> = Box<
    dyn CallPersistentTaskEnvelope<
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

pub(crate) struct CallPersistentTask<I, O, S>
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
trait AsyncTaskEnvelope: Send {
    type A: AsyncTask + Send;

    async fn call_run<'inner>(
        &'inner mut self,
        global: Global<'static>,
        frame: AsyncGcFrame<'static>,
    ) -> JlrsResult<<Self::A as AsyncTask>::Output>;
}

#[async_trait(?Send)]
impl<A: AsyncTask> AsyncTaskEnvelope for A {
    type A = Self;
    async fn call_run<'inner>(
        &'inner mut self,
        global: Global<'static>,
        frame: AsyncGcFrame<'static>,
    ) -> JlrsResult<<Self::A as AsyncTask>::Output> {
        self.run(global, frame).await
    }
}

#[async_trait(?Send)]
trait PersistentTaskEnvelope: Send {
    type P: PersistentTask + Send;

    async fn call_init<'inner>(
        &'inner mut self,
        global: Global<'static>,
        frame: AsyncGcFrame<'static>,
    ) -> JlrsResult<<Self::P as PersistentTask>::State>;

    async fn call_run<'inner>(
        &'inner mut self,
        global: Global<'static>,
        frame: AsyncGcFrame<'static>,
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
        mut frame: AsyncGcFrame<'static>,
    ) -> JlrsResult<<Self::P as PersistentTask>::State> {
        {
            self.init(global, &mut frame).await
        }
    }

    async fn call_run<'inner>(
        &'inner mut self,
        global: Global<'static>,
        mut frame: AsyncGcFrame<'static>,
        state: &'inner mut <Self::P as PersistentTask>::State,
        input: <Self::P as PersistentTask>::Input,
    ) -> JlrsResult<<Self::P as PersistentTask>::Output> {
        {
            let output = {
                let (nested, owner) = frame.new_async();
                let res = self.run(global, nested, state, input).await;
                std::mem::drop(owner);
                res
            };

            output
        }
    }
}

#[async_trait(?Send)]
pub(crate) trait CallPersistentTaskEnvelope: Send + Sync {
    type Input;
    type Output;

    async fn respond(self: Box<Self>, result: JlrsResult<Self::Output>);
    fn input(&mut self) -> Self::Input;
}

#[async_trait(?Send)]
impl<I, O, S> CallPersistentTaskEnvelope for CallPersistentTask<I, O, S>
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

pub(crate) struct PersistentComms<C, P, O> {
    sender: O,
    _task: PhantomData<P>,
    _channel: PhantomData<C>,
}

impl<C, P, O> PersistentComms<C, P, O>
where
    C: Channel<PersistentMessage<P>>,
    P: PersistentTask,
    O: OneshotSender<JlrsResult<PersistentHandle<P>>>,
{
    pub(crate) fn new(sender: O) -> Self {
        PersistentComms {
            sender,
            _task: PhantomData,
            _channel: PhantomData,
        }
    }
}

impl<C, P, O> PendingTask<PersistentComms<C, P, O>, P, Persistent>
where
    C: Channel<PersistentMessage<P>>,
    P: PersistentTask,
    O: OneshotSender<JlrsResult<PersistentHandle<P>>>,
{
    pub(crate) fn new(task: P, sender: PersistentComms<C, P, O>) -> Self {
        PendingTask {
            task: Some(task),
            sender,
            _kind: PhantomData,
        }
    }

    fn split(self) -> (P, PersistentComms<C, P, O>) {
        (self.task.unwrap(), self.sender)
    }
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
    async fn call(mut self: Box<Self>, mut stack: &'static Stack, ledger: &'static  RefCell<Ledger>);
}

#[async_trait(?Send)]
impl<O, A> PendingTaskEnvelope for PendingTask<O, A, Task>
where
    O: OneshotSender<JlrsResult<A::Output>>,
    A: AsyncTask,
{
    async fn call(mut self: Box<Self>, stack: &'static Stack, ledger: &'static  RefCell<Ledger>) {
        let (mut task, result_sender) = self.split();

        // Safety: the stack slots can be reallocated because it doesn't contain any frames
        // yet. The frame is dropped at the end of the scope, the nested hierarchy of scopes is
        // maintained.
        let res = unsafe {
            let (frame, owner) = AsyncGcFrame::base_async(stack, ledger);
            let global = Global::new();

            let res = task.call_run(global, frame).await;
            std::mem::drop(owner);
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
    async fn call(mut self: Box<Self>, stack: &'static Stack, ledger: &'static RefCell<Ledger>) {
        let sender = self.sender();

        // Safety: the stack slots can be reallocated because it doesn't contain any frames
        // yet. The frame is dropped at the end of the scope, the nested hierarchy of scopes is
        // maintained.
        let res = unsafe {
            let (frame, owner) = AsyncGcFrame::base_async(stack, ledger);
            let global = Global::new();

            let res = A::register(global, frame).await;
            std::mem::drop(owner);
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
    async fn call(mut self: Box<Self>, stack: &'static Stack, ledger: &'static RefCell<Ledger>) {
        let sender = self.sender();

        // Safety: the stack slots can be reallocated because it doesn't contain any frames
        // yet. The frame is dropped at the end of the scope, the nested hierarchy of scopes is
        // maintained.
        let res = unsafe {
            let (frame, owner) = AsyncGcFrame::base_async(stack, ledger);
            let global = Global::new();

            let res = P::register(global, frame).await;
            std::mem::drop(owner);
            res
        };

        Box::new(sender).send(res).await;
    }
}

#[async_trait(?Send)]
impl<C, P, O> PendingTaskEnvelope for PendingTask<PersistentComms<C, P, O>, P, Persistent>
where
    C: Channel<PersistentMessage<P>>,
    O: OneshotSender<JlrsResult<PersistentHandle<P>>>,
    P: PersistentTask,
{
    async fn call(mut self: Box<Self>, stack: &'static Stack, ledger: &'static RefCell<Ledger>) {
        let (mut persistent, handle_sender) = self.split();
        let handle_sender = handle_sender.sender;
        let (sender, mut receiver) = C::channel(NonZeroUsize::new(P::CHANNEL_CAPACITY));
        // Safety: the stack slots can be reallocated because it doesn't contain any frames
        // yet. The frame is dropped at the end of the scope, the nested hierarchy of scopes is
        // maintained.
        unsafe {
            let (frame, mut owner) = AsyncGcFrame::base_async(stack, ledger);
            let global = Global::new();

            match persistent.call_init(global, frame).await {
                Ok(mut state) => {
                    handle_sender
                        .send(Ok(PersistentHandle::new(Arc::new(sender))))
                        .await;

                    owner.set_offset(stack.size());

                    loop {
                        let mut msg = match receiver.recv().await {
                            Ok(msg) => msg.msg,
                            Err(_) => break,
                        };

                        let frame = owner.reconstruct();
                        let res = persistent
                            .call_run(global, frame, &mut state, msg.input())
                            .await;

                        msg.respond(res).await;
                    }

                    let frame = owner.reconstruct();
                    persistent.exit(global, frame, &mut state).await;
                }
                Err(e) => handle_sender.send(Err(e)).await,
            }

            std::mem::drop(owner);
        }
    }
}

pub(crate) struct BlockingTask<F, O, T> {
    func: F,
    sender: O,
    _res: PhantomData<T>,
}

impl<F, O, T> BlockingTask<F, O, T>
where
    for<'base> F:
        Send + Sync + FnOnce(Global<'base>, GcFrame<'base>) -> JlrsResult<T>,
    O: OneshotSender<JlrsResult<T>>,
    T: Send + Sync + 'static,
{
    pub(crate) fn new(func: F, sender: O) -> Self {
        Self {
            func,
            sender,
            _res: PhantomData,
        }
    }

    fn call<'scope>(self: Box<Self>, frame: GcFrame<'scope>) -> (JlrsResult<T>, O) {
        // Safety: this method is called from a thread known to Julia, the lifetime is limited to
        // 'scope.
        let global = unsafe { Global::new() };
        let func = self.func;
        let res = func(global, frame);
        (res, self.sender)
    }
}

pub(crate) trait BlockingTaskEnvelope: Send + Sync {
    fn call(self: Box<Self>, stack: &'static Stack, ledger: &'static RefCell<Ledger>) -> LocalBoxFuture<'static, ()>;
}

impl<F, O, T> BlockingTaskEnvelope for BlockingTask<F, O, T>
where
    for<'base> F:
        Send + Sync + FnOnce(Global<'base>, GcFrame<'base>) -> JlrsResult<T>,
    O: OneshotSender<JlrsResult<T>>,
    T: Send + Sync + 'static,
{
    fn call(self: Box<Self>, stack: &'static Stack, ledger: &'static RefCell<Ledger>) -> LocalBoxFuture<'static, ()> {
        // Safety: the stack slots can be reallocated because it doesn't contain any frames
        // yet. The frame is dropped at the end of the scope, the nested hierarchy of scopes is
        // maintained.
        let (res, ch) = unsafe {
            let (frame, owner) = GcFrame::base(stack, ledger);
            let res = self.call(frame);
            std::mem::drop(owner);
            res
        };

        async {
            OneshotSender::send(ch, res).await;
        }
        .boxed_local()
    }
}
