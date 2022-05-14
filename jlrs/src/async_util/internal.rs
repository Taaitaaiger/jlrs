use super::task::PersistentTask;
use crate::memory::frame::GcFrame;
use crate::memory::mode::Async;
use crate::memory::stack_page::StackPage;
use crate::{async_util::channel::ChannelReceiver, memory::global::Global};
use crate::{async_util::channel::OneshotSender, memory::frame::AsyncGcFrame};
use crate::{
    async_util::task::AsyncTask,
    runtime::async_rt::{AsyncRuntime, PersistentMessage},
};
use crate::{error::JlrsResult, memory::stack_page::AsyncStackPage};
use async_trait::async_trait;
use std::marker::PhantomData;

pub(crate) type InnerPersistentMessage<GT> = Box<
    dyn GenericCallPersistentMessage<
        Input = <GT as PersistentTask>::Input,
        Output = <GT as PersistentTask>::Output,
    >,
>;

// What follows is a significant amount of indirection to allow different tasks to have a
// different Output, and allow users to provide an arbitrary sender that implements ReturnChannel
// to return some result.
pub(crate) enum Task {}
pub(crate) enum RegisterTask {}
pub(crate) enum Persistent {}
pub(crate) enum RegisterPersistent {}

pub(crate) struct CallPersistentMessage<I, O, RC>
where
    I: Send + Sync,
    O: Send + Sync + 'static,
    RC: OneshotSender<JlrsResult<O>>,
{
    pub(crate) sender: RC,

    pub(crate) input: Option<I>,

    pub(crate) _marker: PhantomData<O>,
}

#[async_trait(?Send)]
pub(crate) trait GenericCallPersistentMessage: Send + Sync {
    type Input;
    type Output;

    async fn respond(self: Box<Self>, result: JlrsResult<Self::Output>);
    fn input(&mut self) -> Self::Input;
}

#[async_trait(?Send)]
impl<I, O, RC> GenericCallPersistentMessage for CallPersistentMessage<I, O, RC>
where
    I: Send + Sync,
    O: Send + Sync,
    RC: OneshotSender<JlrsResult<O>>,
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
trait GenericAsyncTask: Send + Sync {
    type AT: AsyncTask + Send + Sync;

    async fn call_run<'inner>(
        &'inner mut self,
        global: Global<'static>,
        frame: &'inner mut AsyncGcFrame<'static>,
    ) -> JlrsResult<<Self::AT as AsyncTask>::Output>;
}

#[async_trait(?Send)]
impl<AT: AsyncTask> GenericAsyncTask for AT {
    type AT = Self;
    async fn call_run<'inner>(
        &'inner mut self,
        global: Global<'static>,
        frame: &'inner mut AsyncGcFrame<'static>,
    ) -> JlrsResult<<Self::AT as AsyncTask>::Output> {
        self.run(global, frame).await
    }
}

trait GenericRegisterAsyncTask: Send + Sync {
    type AT: AsyncTask + Send + Sync;
}

impl<AT: AsyncTask> GenericRegisterAsyncTask for AT {
    type AT = Self;
}

#[async_trait(?Send)]
trait GenericPersistentTask: Send + Sync {
    type GT: PersistentTask + Send + Sync;

    async unsafe fn call_init<'inner>(
        &'inner mut self,
        global: Global<'static>,
        frame: &'inner mut AsyncGcFrame<'static>,
    ) -> JlrsResult<<Self::GT as PersistentTask>::State>;

    async unsafe fn call_run<'inner>(
        &'inner mut self,
        global: Global<'static>,
        frame: &'inner mut AsyncGcFrame<'static>,
        state: &'inner mut <Self::GT as PersistentTask>::State,
        input: <Self::GT as PersistentTask>::Input,
    ) -> JlrsResult<<Self::GT as PersistentTask>::Output>;
}

#[async_trait(?Send)]
impl<GT> GenericPersistentTask for GT
where
    GT: PersistentTask,
{
    type GT = Self;

    async unsafe fn call_init<'inner>(
        &'inner mut self,
        global: Global<'static>,
        frame: &'inner mut AsyncGcFrame<'static>,
    ) -> JlrsResult<<Self::GT as PersistentTask>::State> {
        {
            self.init(global, frame).await
        }
    }

    async unsafe fn call_run<'inner>(
        &'inner mut self,
        global: Global<'static>,
        frame: &'inner mut AsyncGcFrame<'static>,
        state: &'inner mut <Self::GT as PersistentTask>::State,
        input: <Self::GT as PersistentTask>::Input,
    ) -> JlrsResult<<Self::GT as PersistentTask>::Output> {
        {
            let output = {
                let mut nested = frame.nest_async(Self::RUN_SLOTS);
                self.run(global, &mut nested, state, input).await
            };

            output
        }
    }
}

trait GenericRegisterPersistentTask: Send + Sync {
    type GT: PersistentTask + Send + Sync;
}

impl<GT: PersistentTask> GenericRegisterPersistentTask for GT {
    type GT = Self;
}

pub(crate) struct PendingTask<RC, T, Kind> {
    task: Option<T>,
    sender: RC,
    _kind: PhantomData<Kind>,
}

impl<RC, AT> PendingTask<RC, AT, Task>
where
    RC: OneshotSender<JlrsResult<AT::Output>>,

    AT: AsyncTask,
{
    pub(crate) fn new(task: AT, sender: RC) -> Self {
        PendingTask {
            task: Some(task),
            sender,
            _kind: PhantomData,
        }
    }

    fn split(self) -> (AT, RC) {
        (self.task.unwrap(), self.sender)
    }
}

impl<IRC, GT> PendingTask<IRC, GT, Persistent>
where
    // IRC: OneshotSender<JlrsResult<PersistentHandle<GT>>>,
    IRC: ChannelReceiver<PersistentMessage<GT>>,

    GT: PersistentTask,
{
    pub(crate) fn new(task: GT, sender: IRC) -> Self {
        PendingTask {
            task: Some(task),
            sender,
            _kind: PhantomData,
        }
    }

    fn split(self) -> (GT, IRC) {
        (self.task.unwrap(), self.sender)
    }
}

impl<RC, AT> PendingTask<RC, AT, RegisterTask>
where
    RC: OneshotSender<JlrsResult<()>>,

    AT: AsyncTask,
{
    pub(crate) fn new(sender: RC) -> Self {
        PendingTask {
            task: None,
            sender,
            _kind: PhantomData,
        }
    }

    fn sender(self) -> RC {
        self.sender
    }
}

impl<RC, GT> PendingTask<RC, GT, RegisterPersistent>
where
    RC: OneshotSender<JlrsResult<()>>,

    GT: PersistentTask,
{
    pub(crate) fn new(sender: RC) -> Self {
        PendingTask {
            task: None,
            sender,
            _kind: PhantomData,
        }
    }

    fn sender(self) -> RC {
        self.sender
    }
}

#[async_trait(?Send)]
pub(crate) trait GenericPendingTask: Send + Sync {
    async fn call(mut self: Box<Self>, mut stack: &mut AsyncStackPage);
}

#[async_trait(?Send)]
impl<RC, AT> GenericPendingTask for PendingTask<RC, AT, Task>
where
    RC: OneshotSender<JlrsResult<AT::Output>>,

    AT: AsyncTask,
{
    async fn call(mut self: Box<Self>, mut stack: &mut AsyncStackPage) {
        unsafe {
            let (mut task, result_sender) = self.split();

            // Transmute to get static lifetimes. Should be okay because tasks can't leak
            // Julia data and the frame is not dropped until the task is dropped.
            let mode = Async(std::mem::transmute(&stack.top[1]));
            if stack.page.size() < AT::RUN_SLOTS + 2 {
                stack.page = StackPage::new(AT::RUN_SLOTS + 2);
            }
            let raw = std::mem::transmute(stack.page.as_mut());
            let mut frame = AsyncGcFrame::new(raw, mode);
            let global = Global::new();

            let res = task.call_run(global, &mut frame).await;
            Box::new(result_sender).send(res).await;
        }
    }
}

#[async_trait(?Send)]
impl<RC, AT> GenericPendingTask for PendingTask<RC, AT, RegisterTask>
where
    RC: OneshotSender<JlrsResult<()>>,

    AT: AsyncTask,
{
    async fn call(mut self: Box<Self>, mut stack: &mut AsyncStackPage) {
        unsafe {
            let sender = self.sender();

            let mode = Async(&stack.top[1]);
            if stack.page.size() < AT::REGISTER_SLOTS + 2 {
                stack.page = StackPage::new(AT::REGISTER_SLOTS + 2);
            }

            let raw = stack.page.as_mut();
            let mut frame = AsyncGcFrame::new(raw, mode);
            let global = Global::new();

            let res = AT::register(global, &mut frame).await;
            Box::new(sender).send(res).await;
        }
    }
}

#[async_trait(?Send)]
impl<RC, GT> GenericPendingTask for PendingTask<RC, GT, RegisterPersistent>
where
    RC: OneshotSender<JlrsResult<()>>,

    GT: PersistentTask,
{
    async fn call(mut self: Box<Self>, mut stack: &mut AsyncStackPage) {
        unsafe {
            let sender = self.sender();

            let mode = Async(&stack.top[1]);
            if stack.page.size() < GT::REGISTER_SLOTS + 2 {
                stack.page = StackPage::new(GT::REGISTER_SLOTS + 2);
            }

            let raw = stack.page.as_mut();
            let mut frame = AsyncGcFrame::new(raw, mode);
            let global = Global::new();

            let res = GT::register(global, &mut frame).await;
            Box::new(sender).send(res).await;
        }
    }
}

#[async_trait(?Send)]
impl<IRC, GT> GenericPendingTask for PendingTask<IRC, GT, Persistent>
where
    IRC: ChannelReceiver<PersistentMessage<GT>>,

    GT: PersistentTask,
{
    async fn call(mut self: Box<Self>, mut stack: &mut AsyncStackPage) {
        unsafe {
            {
                let (mut persistent, mut receiver) = self.split();
                // Transmute to get static lifetimes. Should be okay because tasks can't leak
                // Julia data and the frame is not dropped until the task is dropped.
                let mode = Async(std::mem::transmute(&stack.top[1]));
                if stack.page.size() < GT::INIT_SLOTS + 2 {
                    stack.page = StackPage::new(GT::INIT_SLOTS + 2);
                }

                let raw = std::mem::transmute(stack.page.as_mut());
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
                    _ => (),
                }
            }
        }
    }
}

pub(crate) struct BlockingTask<F, RC, R, T> {
    func: F,
    sender: RC,
    slots: usize,
    _runtime: PhantomData<R>,
    _res: PhantomData<T>,
}

impl<F, RC, R, T> BlockingTask<F, RC, R, T>
where
    for<'base> F:
        Send + Sync + FnOnce(Global<'base>, &mut GcFrame<'base, Async<'base>>) -> JlrsResult<T>,

    RC: OneshotSender<JlrsResult<T>>,
    R: AsyncRuntime,
    T: Send + Sync + 'static,
{
    pub(crate) fn new(func: F, sender: RC, slots: usize) -> Self {
        Self {
            func,
            sender,
            slots,
            _runtime: PhantomData,
            _res: PhantomData,
        }
    }

    fn call<'scope>(
        self: Box<Self>,
        frame: &mut GcFrame<'scope, Async<'scope>>,
    ) -> (JlrsResult<T>, RC) {
        let global = unsafe { Global::new() };
        let func = self.func;
        let res = func(global, frame);
        (res, self.sender)
    }
}

pub(crate) trait GenericBlockingTask: Send + Sync {
    fn call(self: Box<Self>, stack: &mut AsyncStackPage);
}

impl<F, RC, R, T> GenericBlockingTask for BlockingTask<F, RC, R, T>
where
    for<'base> F:
        Send + Sync + FnOnce(Global<'base>, &mut GcFrame<'base, Async<'base>>) -> JlrsResult<T>,
    RC: OneshotSender<JlrsResult<T>>,
    R: AsyncRuntime,
    T: Send + Sync + 'static,
{
    fn call(self: Box<Self>, stack: &mut AsyncStackPage) {
        let mode = Async(&stack.top[1]);
        if stack.page.size() < self.slots + 2 {
            stack.page = StackPage::new(self.slots + 2);
        }
        let raw = stack.page.as_mut();
        let mut frame = unsafe { GcFrame::new(raw, mode) };
        let (res, ch) = self.call(&mut frame);

        R::spawn_local(async {
            OneshotSender::send(Box::new(ch), res).await;
        });
    }
}
