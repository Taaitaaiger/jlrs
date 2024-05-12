use std::{marker::PhantomData, path::PathBuf};

use async_trait::async_trait;

use super::{
    channel::{channel, OneshotSender},
    persistent::PersistentHandle,
};
use crate::{
    async_util::task::{AsyncTask, PersistentTask, Register},
    call::Call,
    data::managed::module::{JlrsCore, Main},
    error::JlrsError,
    memory::{context::stack::Stack, target::frame::GcFrame},
    prelude::{AsyncGcFrame, JlrsResult, JuliaString, Managed, Value},
};

pub(crate) struct PendingTask<T, U, Kind> {
    task: Option<T>,
    sender: U,
    _kind: PhantomData<Kind>,
}

impl<T> PendingTask<T, OneshotSender<T::Output>, Task>
where
    T: AsyncTask,
{
    #[inline]
    pub(crate) fn new(task: T, sender: OneshotSender<T::Output>) -> Self {
        PendingTask {
            task: Some(task),
            sender,
            _kind: PhantomData,
        }
    }

    #[inline]
    fn split(self) -> (T, OneshotSender<T::Output>) {
        (self.task.unwrap(), self.sender)
    }
}

impl<T> PendingTask<T, OneshotSender<JlrsResult<()>>, RegisterTask>
where
    T: Register,
{
    #[inline]
    pub(crate) fn new(sender: OneshotSender<JlrsResult<()>>) -> Self {
        PendingTask {
            task: None,
            sender,
            _kind: PhantomData,
        }
    }

    #[inline]
    fn sender(self) -> OneshotSender<JlrsResult<()>> {
        self.sender
    }
}

// Must be object-safe, so `async_trait` is required.
#[async_trait(?Send)]
pub(crate) trait PendingTaskEnvelope: Send {
    async fn call(self: Box<Self>, stack: &'static Stack);
}

#[async_trait(?Send)]
impl<A> PendingTaskEnvelope for PendingTask<A, OneshotSender<A::Output>, Task>
where
    A: AsyncTask,
{
    async fn call(self: Box<Self>, stack: &'static Stack) {
        let (mut task, sender) = self.split();

        // Safety: the stack slots can be reallocated because it doesn't contain any frames
        // yet. The frame is dropped at the end of the scope, the nested hierarchy of scopes is
        // maintained.
        let res = unsafe {
            let frame = AsyncGcFrame::base(&stack);
            let res = task.call_run(frame).await;
            stack.pop_roots(0);
            res
        };

        sender.send(res).ok();
    }
}

#[async_trait(?Send)]
impl<A> PendingTaskEnvelope for PendingTask<A, OneshotSender<JlrsResult<()>>, RegisterTask>
where
    A: Register,
{
    async fn call(mut self: Box<Self>, stack: &'static Stack) {
        let sender = self.sender();

        // Safety: the stack slots can be reallocated because it doesn't contain any frames
        // yet. The frame is dropped at the end of the scope, the nested hierarchy of scopes is
        // maintained.
        let res = unsafe {
            let frame = AsyncGcFrame::base(&stack);
            let res = A::register(frame).await;
            stack.pop_roots(0);
            res
        };

        sender.send(res).ok();
    }
}

#[async_trait(?Send)]
impl<P> PendingTaskEnvelope
    for PendingTask<P, OneshotSender<JlrsResult<PersistentHandle<P>>>, Persistent>
where
    P: PersistentTask,
{
    async fn call(mut self: Box<Self>, stack: &'static Stack) {
        let (mut persistent, handle_sender) = self.split();
        let handle_sender = handle_sender;
        let (sender, receiver) = channel(P::CHANNEL_CAPACITY);
        // Safety: the stack slots can be reallocated because it doesn't contain any frames
        // yet. The frame is dropped at the end of the scope, the nested hierarchy of scopes is
        // maintained.
        unsafe {
            let frame = AsyncGcFrame::base(&stack);

            match persistent.call_init(frame).await {
                Ok(mut state) => {
                    if let Err(_) = handle_sender.send(Ok(PersistentHandle::new(sender))) {
                        stack.pop_roots(0);
                        return;
                    }

                    loop {
                        let mut msg = match receiver.recv().await {
                            Ok(msg) => msg.msg,
                            Err(_) => break,
                        };

                        let frame = AsyncGcFrame::base(&stack);
                        let res = persistent.call_run(frame, &mut state, msg.input()).await;

                        msg.respond(res);
                    }

                    let frame = AsyncGcFrame::base(&stack);
                    persistent.exit(frame, &mut state).await;
                }
                Err(e) => {
                    handle_sender.send(Err(e)).ok();
                }
            }

            stack.pop_roots(0);
        }
    }
}

trait AsyncTaskEnvelope: Send {
    type A: AsyncTask + Send;

    async fn call_run<'inner>(
        &'inner mut self,
        frame: AsyncGcFrame<'static>,
    ) -> <Self::A as AsyncTask>::Output;
}

impl<A: AsyncTask> AsyncTaskEnvelope for A {
    type A = Self;
    #[inline]
    async fn call_run<'inner>(
        &'inner mut self,
        frame: AsyncGcFrame<'static>,
    ) -> <Self::A as AsyncTask>::Output {
        self.run(frame).await
    }
}

trait PersistentTaskEnvelope: Send {
    type P: PersistentTask + Send;

    async fn call_init<'inner>(
        &'inner mut self,
        frame: AsyncGcFrame<'static>,
    ) -> JlrsResult<<Self::P as PersistentTask>::State<'static>>;

    async fn call_run<'inner>(
        &'inner mut self,
        frame: AsyncGcFrame<'static>,
        state: &'inner mut <Self::P as PersistentTask>::State<'static>,
        input: <Self::P as PersistentTask>::Input,
    ) -> <Self::P as PersistentTask>::Output;
}

impl<P> PersistentTaskEnvelope for P
where
    P: PersistentTask,
{
    type P = Self;

    #[inline]
    async fn call_init<'inner>(
        &'inner mut self,
        frame: AsyncGcFrame<'static>,
    ) -> JlrsResult<<Self::P as PersistentTask>::State<'static>> {
        {
            self.init(frame).await
        }
    }

    async fn call_run<'inner>(
        &'inner mut self,
        mut frame: AsyncGcFrame<'static>,
        state: &'inner mut <Self::P as PersistentTask>::State<'static>,
        input: <Self::P as PersistentTask>::Input,
    ) -> <Self::P as PersistentTask>::Output {
        {
            let output = unsafe {
                let stack = frame.stack();
                let (offset, nested) = frame.nest_async();
                let res = self.run(nested, state, input).await;
                stack.pop_roots(offset);
                res
            };

            output
        }
    }
}

pub(crate) struct BlockingTask<F, T> {
    func: F,
    sender: OneshotSender<T>,
}

impl<F, T> BlockingTask<F, T>
where
    for<'base> F: Send + FnOnce(GcFrame<'base>) -> T,
    T: Send + 'static,
{
    #[inline]
    pub(crate) fn new(func: F, sender: OneshotSender<T>) -> Self {
        Self { func, sender }
    }

    #[inline]
    fn call<'scope>(self: Box<Self>, frame: GcFrame<'scope>) {
        // Safety: this method is called from a thread known to Julia, the lifetime is limited to
        // 'scope.
        let func = self.func;
        let res = func(frame);
        self.sender.send(res).ok();
    }
}

pub(crate) trait BlockingTaskEnvelope: Send {
    fn call<'scope>(self: Box<Self>, stack: &'scope Stack);
}

impl<F, T> BlockingTaskEnvelope for BlockingTask<F, T>
where
    for<'base> F: Send + FnOnce(GcFrame<'base>) -> T,
    T: Send + 'static,
{
    fn call<'scope>(self: Box<Self>, stack: &'scope Stack) {
        // Safety: the stack slots can be reallocated because it doesn't contain any frames
        // yet. The frame is dropped at the end of the scope, the nested hierarchy of scopes is
        // maintained.
        unsafe {
            let frame = GcFrame::base(&stack);
            self.call(frame);
            stack.pop_roots(0);
        }
    }
}

pub(crate) type InnerPersistentMessage<P> = Box<
    dyn CallPersistentTaskEnvelope<
        Input = <P as PersistentTask>::Input,
        Output = <P as PersistentTask>::Output,
    >,
>;

pub(crate) struct CallPersistentTask<I, O>
where
    I: Send,
    O: Send + 'static,
{
    pub(crate) sender: OneshotSender<O>,
    pub(crate) input: Option<I>,
}

pub(crate) trait CallPersistentTaskEnvelope: Send {
    type Input;
    type Output;

    fn respond(self: Box<Self>, result: Self::Output);
    fn input(&mut self) -> Self::Input;
}

impl<I, O> CallPersistentTaskEnvelope for CallPersistentTask<I, O>
where
    I: Send,
    O: Send,
{
    type Input = I;
    type Output = O;

    #[inline]
    fn respond(self: Box<Self>, result: Self::Output) {
        self.sender.send(result).ok();
    }

    #[inline]
    fn input(&mut self) -> Self::Input {
        self.input.take().unwrap()
    }
}

// pub(crate) struct PersistentComms<P: PersistentTask> {
//     sender: OneshotSender<JlrsResult<PersistentHandle<P>>>,
// }

// impl<P> OneshotSender<JlrsResult<PersistentHandle<P>>>
// where
//     P: PersistentTask,
// {
//     #[inline]
//     pub(crate) fn new(sender: OneshotSender<JlrsResult<PersistentHandle<P>>>) -> Self {
//         PersistentComms {
//             sender,
//         }
//     }
// }

impl<P> PendingTask<P, OneshotSender<JlrsResult<PersistentHandle<P>>>, Persistent>
where
    P: PersistentTask,
{
    #[inline]
    pub(crate) fn new(task: P, sender: OneshotSender<JlrsResult<PersistentHandle<P>>>) -> Self {
        PendingTask {
            task: Some(task),
            sender: sender,
            _kind: PhantomData,
        }
    }

    #[inline]
    fn split(self) -> (P, OneshotSender<JlrsResult<PersistentHandle<P>>>) {
        (self.task.unwrap(), self.sender)
    }
}

pub(crate) struct IncludeTask {
    path: PathBuf,
    sender: OneshotSender<JlrsResult<()>>,
}

impl IncludeTask {
    #[inline]
    pub(crate) fn new(path: PathBuf, sender: OneshotSender<JlrsResult<()>>) -> Self {
        Self { path, sender }
    }

    #[inline]
    unsafe fn call_inner<'scope>(mut frame: GcFrame<'scope>, path: PathBuf) -> JlrsResult<()> {
        match path.to_str() {
            Some(path) => {
                let path = JuliaString::new(&mut frame, path);
                Main::include(&frame)
                    .call1(&frame, path.as_value())
                    .map_err(|e| {
                        JlrsError::exception(format!("include error: {:?}", e.as_value()))
                    })?;
            }
            None => {}
        }

        Ok(())
    }

    fn call<'scope>(self: Box<Self>, frame: GcFrame<'scope>) {
        // Safety: this method is called from a thread known to Julia, the lifetime is limited to
        // 'scope.
        let path = self.path;
        let res = unsafe { Self::call_inner(frame, path) };
        self.sender.send(res).ok();
    }
}

pub(crate) trait IncludeTaskEnvelope: Send {
    fn call(self: Box<Self>, stack: &'static Stack);
}

impl IncludeTaskEnvelope for IncludeTask {
    fn call(self: Box<Self>, stack: &'static Stack) {
        // Safety: the stack slots can be reallocated because it doesn't contain any frames
        // yet. The frame is dropped at the end of the scope, the nested hierarchy of scopes is
        // maintained.
        unsafe {
            let frame = GcFrame::base(&stack);
            self.call(frame);
            stack.pop_roots(0);
        }
    }
}

pub(crate) struct SetErrorColorTask {
    enable: bool,
    sender: OneshotSender<()>,
}

impl SetErrorColorTask {
    #[inline]
    pub(crate) fn new(enable: bool, sender: OneshotSender<()>) -> Self {
        Self { enable, sender }
    }

    fn call<'scope>(self: Box<Self>, frame: GcFrame<'scope>) {
        // Safety: this method is called from a thread known to Julia, the lifetime is limited to
        // 'scope.
        let enable = self.enable;
        unsafe {
            let unrooted = frame.unrooted();

            let enable = if enable {
                Value::true_v(&unrooted)
            } else {
                Value::false_v(&unrooted)
            };

            JlrsCore::color(&unrooted).set_nth_field_unchecked(0, enable);
        };
        self.sender.send(()).ok();
    }
}

pub(crate) trait SetErrorColorTaskEnvelope: Send {
    fn call(self: Box<Self>, stack: &'static Stack);
}

impl SetErrorColorTaskEnvelope for SetErrorColorTask {
    fn call(self: Box<Self>, stack: &'static Stack) {
        // Safety: the stack slots can be reallocated because it doesn't contain any frames
        // yet. The frame is dropped at the end of the scope, the nested hierarchy of scopes is
        // maintained.
        unsafe {
            let frame = GcFrame::base(&stack);
            self.call(frame);
            stack.pop_roots(0);
        }
    }
}

// What follows is a significant amount of indirection to allow different tasks to have a
// different Output types and be unaware of the used channels.
pub(crate) enum Task {}
pub(crate) enum RegisterTask {}
pub(crate) enum Persistent {}
