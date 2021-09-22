//! Traits to implement non-blocking tasks for the async runtime.
//!
//! In addition to blocking tasks, the async runtime supports non-blocking tasks which fall into
//! two categories: tasks that can be called once implement [`AsyncTask`], tasks that can be
//! called multiple times implement [`GeneratorTask`].
//!
//! Both of these traits require that you implement an async `run` method. This method essentially
//! replaces the closures of the sync runtime. Rather than a mutable reference to a [`GcFrame`] it
//! takes a mutable reference to an [`AsyncGcFrame`]. This frame type provides the same
//! functionality as `GcFrame`, and can be used in combination with several async methods. Most
//! importantly, the methods of the trait [`CallAsync`] which let you schedule a Julia function
//! call as a new Julia task and await its completion.
//!
//! [`GcFrame`]: crate::memory::frame::GcFrame
//! [`CallAsync`]: crate::extensions::multitask::call_async::CallAsync

use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;

use super::async_frame::AsyncGcFrame;
use super::mode::Async;
use super::result_sender::ResultSender;
use super::AsyncStackPage;
use super::RequireSendSync;
use crate::error::{JlrsError, JlrsResult};
use crate::memory::frame::GcFrame;
use crate::memory::global::Global;
use crate::memory::stack_page::StackPage;
use async_trait::async_trait;

#[cfg(feature = "async-std-rt")]
use crate::extensions::multitask::runtime::async_std_rt::{channel, HandleSender};
#[cfg(feature = "tokio-rt")]
use crate::extensions::multitask::runtime::tokio_rt::{channel, HandleSender};

/// A task that returns once. In order to schedule the task you must use [`AsyncJulia::task`] or
/// [`AsyncJulia::try_task`].
///
/// [`AsyncJulia::task`]: crate::extensions::multitask::AsyncJulia::task
/// [`AsyncJulia::try_task`]: crate::extensions::multitask::AsyncJulia::try_task
#[async_trait(?Send)]
pub trait AsyncTask: 'static + Send + Sync {
    /// The type of the result which is returned if `run` completes successfully.
    type Output: 'static + Send + Sync;

    /// The number of slots preallocated for the `AsyncGcFrame` provided to `run`.
    const RUN_SLOTS: usize = 0;

    /// The number of slots preallocated for the `AsyncGcFrame` provided to `register`.
    const REGISTER_SLOTS: usize = 0;

    /// Register the task. Note that this method is not called automatically, but only if
    /// [`AsyncJulia::register_task`] or [`AsyncJulia::try_register_task`] is used. This method
    /// can be implemented to take care of everything required to execute the task successfully,
    /// like loading packages.
    ///
    /// [`AsyncJulia::register_task`]: crate::extensions::multitask::AsyncJulia::register_task
    /// [`AsyncJulia::try_register_task`]: crate::extensions::multitask::AsyncJulia::try_register_task
    async fn register<'frame>(
        _global: Global<'frame>,
        _frame: &mut AsyncGcFrame<'frame>,
    ) -> JlrsResult<()> {
        Ok(())
    }

    /// Run this task. This method takes a `Global` and a mutable reference to an `AsyncGcFrame`,
    /// which lets you interact with Julia.
    async fn run<'base>(
        &mut self,
        global: Global<'base>,
        frame: &mut AsyncGcFrame<'base>,
    ) -> JlrsResult<Self::Output>;
}

/// A task that can be called multiple times. In order to schedule the task you must use
/// [`AsyncJulia::generator`] or [`AsyncJulia::try_generator`].
///
/// [`AsyncJulia::generator`]: crate::extensions::multitask::AsyncJulia::generator
/// [`AsyncJulia::try_generator`]: crate::extensions::multitask::AsyncJulia::try_generator
#[async_trait(?Send)]
pub trait GeneratorTask: 'static + Send + Sync {
    /// The type of the result which is returned if `init` completes successfully. This data is
    /// provided to every call of `run`. Because `init` takes a frame with the `'static` lifetime,
    /// this type can contain Julia data.
    type State: 'static;

    /// The type of the data that must be provided when calling this generator through its handle.
    type Input: 'static + Send + Sync;

    /// The type of the result which is returned if `run` completes successfully.
    type Output: 'static + Send + Sync;

    /// The capacity of the channel the [`GeneratorHandle`] uses to communicate with this
    /// generator.
    ///
    /// If it's set to 0, the channel is unbounded.
    const CHANNEL_CAPACITY: usize = 0;

    /// The number of slots preallocated for the `AsyncGcFrame` provided to `register`.
    const REGISTER_SLOTS: usize = 0;

    /// The number of slots preallocated for the `AsyncGcFrame` provided to `init`.
    const INIT_SLOTS: usize = 0;

    /// The number of slots preallocated for the `AsyncGcFrame` provided to `run`.
    const RUN_SLOTS: usize = 0;

    // NB: `init` and `run` have an explicit 'inner lifetime . If this lifetime is elided
    // `GeneratorTask`s can be implemented in bin crates but not in lib crates (rustc 1.54.0)

    /// Register this generator. Note that this method is not called automatically, but only if
    /// [`AsyncJulia::register_generator`] or [`AsyncJulia::try_register_generator`] is used. This
    /// method can be implemented to take care of everything required to execute the task
    /// successfully, like loading packages.
    ///
    /// [`AsyncJulia::register_generator`]: crate::extensions::multitask::AsyncJulia::register_generator
    /// [`AsyncJulia::try_register_generator`]: crate::extensions::multitask::AsyncJulia::try_register_generator
    async fn register<'frame>(
        _global: Global<'frame>,
        _frame: &mut AsyncGcFrame<'frame>,
    ) -> JlrsResult<()> {
        Ok(())
    }

    /// Initialize the generator. You can interact with Julia inside this method, the frame is
    /// not dropped until the generator itself is dropped. This means that `State` can contain
    /// arbitrary Julia data rooted in this frame. This data is provided to every call to `run`.
    async fn init<'inner>(
        &'inner mut self,
        global: Global<'static>,
        frame: &'inner mut AsyncGcFrame<'static>,
    ) -> JlrsResult<Self::State>;

    /// Run the generator. This method takes a `Global` and a mutable reference to an
    /// `AsyncGcFrame`, which lets you interact with Julia. It's also provided with a mutable
    /// reference to its `state` and the `input` provided by the caller. While the state is
    /// mutable, it's not possible to allocate a new Julia value in `run` and assign it to the
    /// state because the frame doesn't live long enough.
    async fn run<'inner, 'frame>(
        &'inner mut self,
        global: Global<'frame>,
        frame: &'inner mut AsyncGcFrame<'frame>,
        state: &'inner mut Self::State,
        input: Self::Input,
    ) -> JlrsResult<Self::Output>;

    async fn exit<'inner>(
        &'inner mut self,
        _global: Global<'static>,
        _frame: &'inner mut AsyncGcFrame<'static>,
        _state: &'inner mut Self::State,
    ) {
    }
}

pub struct GeneratorMessage<GT>
where
    GT: GeneratorTask,
{
    msg: InnerGeneratorMessage<GT>,
}

impl<GT> fmt::Debug for GeneratorMessage<GT>
where
    GT: GeneratorTask,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("GeneratorMessage")
    }
}

type InnerGeneratorMessage<GT> = Box<
    dyn GenericCallGeneratorMessage<
        Input = <GT as GeneratorTask>::Input,
        Output = <GT as GeneratorTask>::Output,
    >,
>;

/// A handle to a [`GeneratorTask`]. This handle can be used to call the generator and shared
/// across threads. The `GeneratorTask` is dropped when its final handle has been dropped and all
/// remaining pending calls have completed.
#[derive(Clone)]
pub struct GeneratorHandle<GT>
where
    GT: GeneratorTask,
{
    sender: HandleSender<GT>,
}

impl<GT> GeneratorHandle<GT>
where
    GT: GeneratorTask,
{
    fn new(sender: HandleSender<GT>) -> Self {
        GeneratorHandle { sender }
    }

    /// Call the generator, this method waits until there's room available in the channel.
    pub async fn call<R>(&self, input: GT::Input, sender: R)
    where
        R: ResultSender<JlrsResult<GT::Output>>,
    {
        self.sender
            .send(GeneratorMessage {
                msg: Box::new(CallGeneratorMessage {
                    input: Some(input),
                    sender,
                    _marker: PhantomData,
                }),
            })
            .await
            .expect("Channel was closed")
    }

    /// Call the generator, this method returns an error immediately if there's NO room available
    /// in the channel.
    pub fn try_call<R>(&self, input: GT::Input, sender: R) -> JlrsResult<()>
    where
        R: ResultSender<JlrsResult<GT::Output>>,
    {
        match self.sender.try_send(GeneratorMessage {
            msg: Box::new(CallGeneratorMessage {
                input: Some(input),
                sender,
                _marker: PhantomData,
            }),
        }) {
            Ok(_) => Ok(()),
            Err(e) => Err(JlrsError::other(e))?,
        }
    }
}

// Ensure the handle can be shared across threads
impl<GT: GeneratorTask> RequireSendSync for GeneratorHandle<GT> {}

// What follows is a significant amount of indirection to allow different tasks to have a
// different Output, and allow users to provide an arbitrary sender that implements ReturnChannel
// to return some result.
pub(crate) enum Task {}
pub(crate) enum RegisterTask {}
pub(crate) enum Generator {}
pub(crate) enum RegisterGenerator {}

struct CallGeneratorMessage<I, O, RC>
where
    I: Send + Sync,
    O: Send + Sync + 'static,
    RC: ResultSender<JlrsResult<O>>,
{
    sender: RC,
    input: Option<I>,
    _marker: PhantomData<O>,
}

#[async_trait(?Send)]
trait GenericCallGeneratorMessage: Send + Sync {
    type Input;
    type Output;

    async fn respond(self: Box<Self>, result: JlrsResult<Self::Output>);
    fn input(&mut self) -> Self::Input;
}

#[async_trait(?Send)]
impl<I, O, RC> GenericCallGeneratorMessage for CallGeneratorMessage<I, O, RC>
where
    I: Send + Sync,
    O: Send + Sync,
    RC: ResultSender<JlrsResult<O>>,
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
trait GenericGeneratorTask: Send + Sync {
    type GT: GeneratorTask + Send + Sync;

    async unsafe fn call_init<'inner>(
        &'inner mut self,
        global: Global<'static>,
        frame: &'inner mut AsyncGcFrame<'static>,
    ) -> JlrsResult<<Self::GT as GeneratorTask>::State>;

    async unsafe fn call_run<'inner>(
        &'inner mut self,
        global: Global<'static>,
        frame: &'inner mut AsyncGcFrame<'static>,
        state: &'inner mut <Self::GT as GeneratorTask>::State,
        input: <Self::GT as GeneratorTask>::Input,
    ) -> JlrsResult<<Self::GT as GeneratorTask>::Output>;

    fn create_handle(&self, sender: HandleSender<Self::GT>) -> GeneratorHandle<Self::GT>;
}

#[async_trait(?Send)]
impl<GT> GenericGeneratorTask for GT
where
    GT: GeneratorTask,
{
    type GT = Self;

    async unsafe fn call_init<'inner>(
        &'inner mut self,
        global: Global<'static>,
        frame: &'inner mut AsyncGcFrame<'static>,
    ) -> JlrsResult<<Self::GT as GeneratorTask>::State> {
        {
            self.init(global, frame).await
        }
    }

    async unsafe fn call_run<'inner>(
        &'inner mut self,
        global: Global<'static>,
        frame: &'inner mut AsyncGcFrame<'static>,
        state: &'inner mut <Self::GT as GeneratorTask>::State,
        input: <Self::GT as GeneratorTask>::Input,
    ) -> JlrsResult<<Self::GT as GeneratorTask>::Output> {
        unsafe {
            let output = {
                let mut nested = frame.nest_async(Self::RUN_SLOTS);
                self.run(global, &mut nested, state, input).await
            };

            output
        }
    }

    fn create_handle(&self, sender: HandleSender<Self>) -> GeneratorHandle<Self> {
        GeneratorHandle::new(sender)
    }
}

trait GenericRegisterGeneratorTask: Send + Sync {
    type GT: GeneratorTask + Send + Sync;
}

impl<GT: GeneratorTask> GenericRegisterGeneratorTask for GT {
    type GT = Self;
}

pub(crate) struct PendingTask<RC, T, Kind> {
    task: Option<T>,
    sender: RC,
    _kind: PhantomData<Kind>,
}

impl<RC, AT> PendingTask<RC, AT, Task>
where
    RC: ResultSender<JlrsResult<AT::Output>>,
    AT: AsyncTask,
{
    pub(super) fn new(task: AT, sender: RC) -> Self {
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

impl<IRC, GT> PendingTask<IRC, GT, Generator>
where
    IRC: ResultSender<JlrsResult<GeneratorHandle<GT>>>,
    GT: GeneratorTask,
{
    pub(super) fn new(task: GT, sender: IRC) -> Self {
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
    RC: ResultSender<JlrsResult<()>>,
    AT: AsyncTask,
{
    pub(super) fn new(sender: RC) -> Self {
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

impl<RC, GT> PendingTask<RC, GT, RegisterGenerator>
where
    RC: ResultSender<JlrsResult<()>>,
    GT: GeneratorTask,
{
    pub(super) fn new(sender: RC) -> Self {
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
    RC: ResultSender<JlrsResult<AT::Output>>,
    AT: AsyncTask,
{
    async fn call(mut self: Box<Self>, mut stack: &mut AsyncStackPage) {
        unsafe {
            let (mut task, result_sender) = self.split();

            // Transmute to get static lifetimes. Should be okay because tasks can't leak
            // Julia data and the frame is not dropped until the generator is dropped.
            let mode = Async(std::mem::transmute(&stack.top[1]));
            if stack.page.size() < AT::RUN_SLOTS + 2 {
                stack.page = StackPage::new(AT::RUN_SLOTS + 2);
            }
            let raw = std::mem::transmute(stack.page.as_mut());
            let mut frame = AsyncGcFrame::new(raw, 0, mode);
            let global = Global::new();

            let res = task.call_run(global, &mut frame).await;
            Box::new(result_sender).send(res).await;
        }
    }
}

#[async_trait(?Send)]
impl<RC, AT> GenericPendingTask for PendingTask<RC, AT, RegisterTask>
where
    RC: ResultSender<JlrsResult<()>>,
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
            let mut frame = AsyncGcFrame::new(raw, 0, mode);
            let global = Global::new();

            let res = AT::register(global, &mut frame).await;
            Box::new(sender).send(res).await;
        }
    }
}

#[async_trait(?Send)]
impl<RC, GT> GenericPendingTask for PendingTask<RC, GT, RegisterGenerator>
where
    RC: ResultSender<JlrsResult<()>>,
    GT: GeneratorTask,
{
    async fn call(mut self: Box<Self>, mut stack: &mut AsyncStackPage) {
        unsafe {
            let sender = self.sender();

            let mode = Async(&stack.top[1]);
            if stack.page.size() < GT::REGISTER_SLOTS + 2 {
                stack.page = StackPage::new(GT::REGISTER_SLOTS + 2);
            }

            let raw = stack.page.as_mut();
            let mut frame = AsyncGcFrame::new(raw, 0, mode);
            let global = Global::new();

            let res = GT::register(global, &mut frame).await;
            Box::new(sender).send(res).await;
        }
    }
}

#[async_trait(?Send)]
impl<IRC, GT> GenericPendingTask for PendingTask<IRC, GT, Generator>
where
    IRC: ResultSender<JlrsResult<GeneratorHandle<GT>>>,
    GT: GeneratorTask,
{
    async fn call(mut self: Box<Self>, mut stack: &mut AsyncStackPage) {
        unsafe {
            let (mut generator, handle_sender) = self.split();
            let handle_sender = Box::new(handle_sender);

            // Transmute to get static lifetimes. Should be okay because tasks can't leak
            // Julia data and the frame is not dropped until the generator is dropped.
            let mode = Async(std::mem::transmute(&stack.top[1]));
            if stack.page.size() < GT::INIT_SLOTS + 2 {
                stack.page = StackPage::new(GT::INIT_SLOTS + 2);
            }

            let raw = std::mem::transmute(stack.page.as_mut());
            let mut frame = AsyncGcFrame::new(raw, GT::INIT_SLOTS, mode);
            let global = Global::new();

            match generator.call_init(global, &mut frame).await {
                Ok(mut state) => {
                    #[allow(unused_mut)]
                    let (sender, mut receiver) = channel(GT::CHANNEL_CAPACITY);

                    let handle = generator.create_handle(Arc::new(sender));
                    handle_sender.send(Ok(handle)).await;

                    loop {
                        #[cfg(feature = "async-std-rt")]
                        let mut msg = match receiver.recv().await {
                            Ok(msg) => msg.msg,
                            Err(_) => break,
                        };

                        #[cfg(feature = "tokio-rt")]
                        let mut msg = match receiver.recv().await {
                            Some(msg) => msg.msg,
                            None => break,
                        };

                        let res = generator
                            .call_run(global, &mut frame, &mut state, msg.input())
                            .await;

                        msg.respond(res).await;
                    }

                    generator.exit(global, &mut frame, &mut state).await
                }
                Err(e) => {
                    handle_sender.send(Err(e)).await;
                }
            }
        }
    }
}

pub(crate) struct BlockingTask<F, RC, T> {
    func: F,
    sender: RC,
    slots: usize,
    _res: PhantomData<T>,
}

impl<F, RC, T> BlockingTask<F, RC, T>
where
    for<'base> F:
        Send + Sync + FnOnce(Global<'base>, &mut GcFrame<'base, Async<'base>>) -> JlrsResult<T>,
    RC: ResultSender<JlrsResult<T>>,
    T: Send + Sync + 'static,
{
    pub(crate) fn new(func: F, sender: RC, slots: usize) -> Self {
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

impl<F, RC, T> GenericBlockingTask for BlockingTask<F, RC, T>
where
    for<'base> F:
        Send + Sync + FnOnce(Global<'base>, &mut GcFrame<'base, Async<'base>>) -> JlrsResult<T>,
    RC: ResultSender<JlrsResult<T>>,
    T: Send + Sync + 'static,
{
    fn call(self: Box<Self>, stack: &mut AsyncStackPage) {
        let mode = Async(&stack.top[1]);
        if stack.page.size() < self.slots + 2 {
            stack.page = StackPage::new(self.slots + 2);
        }
        let raw = stack.page.as_mut();
        let mut frame = unsafe { GcFrame::new(raw, self.slots, mode) };
        let (res, ch) = self.call(&mut frame);

        #[cfg(feature = "tokio-rt")]
        {
            tokio::task::spawn_local(async {
                Box::new(ch).send(res).await;
            });
        }

        #[cfg(feature = "async-std-rt")]
        {
            async_std::task::spawn_local(async {
                Box::new(ch).send(res).await;
            });
        }
    }
}
