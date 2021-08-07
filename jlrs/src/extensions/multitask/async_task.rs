//! Traits used to implement tasks for the async runtime.
//!
//! While the sync runtime can call Julia inside a closure, using the async runtime takes a bit
//! more effort. The async runtime must be sent tasks which fall into two categories: tasks that
//! can be called once implement [`AsyncTask`], tasks that can be called multiple times implement
//! [`GeneratorTask`].
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

use std::marker::PhantomData;

use super::async_frame::AsyncGcFrame;
use super::mode::Async;
use super::return_channel::ReturnChannel;
use super::{wake_julia, RequireSendSync};
use super::{AsyncStackPage, Message};
use crate::error::{JlrsError, JlrsResult};
use crate::memory::global::Global;
use async_std::channel::Sender as AsyncStdSender;
use async_trait::async_trait;

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
    /// like loading packages or defining a type.
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
    /// successfully, like loading packages or defining a type.
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
}

type HandleSender<GT> = AsyncStdSender<
    Box<
        dyn GenericCallGeneratorMessage<
            Input = <GT as GeneratorTask>::Input,
            Output = <GT as GeneratorTask>::Output,
        >,
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
        R: ReturnChannel<Success = GT::Output>,
    {
        self.sender
            .send(Box::new(CallGeneratorMessage {
                input: Some(input),
                sender,
            }))
            .await
            .map(|_| unsafe { wake_julia() })
            .expect("Channel was closed")
    }

    /// Call the generator, this method returns an error immediately if there's NO room available
    /// in the channel.
    pub fn try_call<R>(&self, input: GT::Input, sender: R) -> JlrsResult<()>
    where
        R: ReturnChannel<Success = GT::Output>,
    {
        match self.sender.try_send(Box::new(CallGeneratorMessage {
            input: Some(input),
            sender,
        })) {
            Ok(_) => unsafe {
                wake_julia();
                Ok(())
            },
            Err(e) => Err(JlrsError::other(e))?,
        }
    }

    /// Returns the capacity of the backing channel if a bounded channel is used, or `None` if
    /// it's unbounded.
    pub fn capacity(&self) -> Option<usize> {
        self.sender.capacity()
    }

    /// Returns the number of messages in the backing channel.
    pub fn len(&self) -> usize {
        self.sender.len()
    }

    /// Returns the number of handles that exist for this generator.
    pub fn handle_count(&self) -> usize {
        self.sender.sender_count()
    }

    /// Returns `true` if the backing channel is empty.
    pub fn is_empty(&self) -> bool {
        self.sender.is_empty()
    }

    /// Returns `true` if the backing channel is full.
    pub fn is_full(&self) -> bool {
        self.sender.is_full()
    }

    /// Closes the backing channel.
    ///
    /// Returns `true` if this call has closed the channel and it was not closed already.
    ///
    /// Pending calls will be executed before the generator completes, but it can't be called
    /// again.
    pub fn close(&self) -> bool {
        self.sender.close()
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
    O: Send + Sync,
    RC: ReturnChannel<Success = O>,
{
    sender: RC,
    input: Option<I>,
}

#[async_trait(?Send)]
trait GenericCallGeneratorMessage: Send + Sync {
    type Input;
    type Output;

    async fn respond(&self, result: JlrsResult<Self::Output>);
    fn input(&mut self) -> Self::Input;
}

#[async_trait(?Send)]
impl<I, O, RC> GenericCallGeneratorMessage for CallGeneratorMessage<I, O, RC>
where
    I: Send + Sync,
    O: Send + Sync,
    RC: ReturnChannel<Success = O>,
{
    type Input = I;
    type Output = O;

    async fn respond(&self, result: JlrsResult<Self::Output>) {
        self.sender.send(result).await
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
        unsafe { self.run(global, frame).await }
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
    RC: ReturnChannel<Success = AT::Output>,
    AT: AsyncTask,
{
    pub(crate) fn new(task: AT, sender: RC) -> Self {
        PendingTask {
            task: Some(task),
            sender,
            _kind: PhantomData,
        }
    }

    pub(crate) fn split(self) -> (AT, RC) {
        (self.task.unwrap(), self.sender)
    }
}

impl<IRC, GT> PendingTask<IRC, GT, Generator>
where
    IRC: ReturnChannel<Success = GeneratorHandle<GT>>,
    GT: GeneratorTask,
{
    pub(crate) fn new(task: GT, sender: IRC) -> Self {
        PendingTask {
            task: Some(task),
            sender,
            _kind: PhantomData,
        }
    }

    pub(crate) fn split(self) -> (GT, IRC) {
        (self.task.unwrap(), self.sender)
    }
}

impl<RC, AT> PendingTask<RC, AT, RegisterTask>
where
    RC: ReturnChannel<Success = ()>,
    AT: AsyncTask,
{
    pub(crate) fn new(sender: RC) -> Self {
        PendingTask {
            task: None,
            sender,
            _kind: PhantomData,
        }
    }

    pub(crate) fn sender(self) -> RC {
        self.sender
    }
}

impl<RC, GT> PendingTask<RC, GT, RegisterGenerator>
where
    RC: ReturnChannel<Success = ()>,
    GT: GeneratorTask,
{
    pub(crate) fn new(sender: RC) -> Self {
        PendingTask {
            task: None,
            sender,
            _kind: PhantomData,
        }
    }

    pub(crate) fn sender(self) -> RC {
        self.sender
    }
}

#[async_trait(?Send)]
pub(crate) trait GenericPendingTask: Send + Sync {
    async fn call(
        mut self: Box<Self>,
        task_idx: usize,
        mut stack: Box<AsyncStackPage>,
        rt_sender: AsyncStdSender<Message>,
    );
}

#[async_trait(?Send)]
impl<RC, AT> GenericPendingTask for PendingTask<RC, AT, Task>
where
    RC: ReturnChannel<Success = AT::Output>,
    AT: AsyncTask,
{
    async fn call(
        mut self: Box<Self>,
        task_idx: usize,
        mut stack: Box<AsyncStackPage>,
        rt_sender: AsyncStdSender<Message>,
    ) {
        unsafe {
            let (mut task, result_sender) = self.split();

            // Transmute to get static lifetimes. Should be okay because tasks can't leak
            // Julia data and the frame is not dropped until the generator is dropped.
            let mode = Async(std::mem::transmute(&stack.top[1]));
            let raw = std::mem::transmute(stack.page.as_mut());
            let mut frame = AsyncGcFrame::new(raw, 0, mode);
            let global = Global::new();

            let res = task.call_run(global, &mut frame).await;
            result_sender.send(res).await;
        }

        rt_sender
            .send(Message::Complete(task_idx, stack))
            .await
            .expect("Channel was closed");
    }
}

#[async_trait(?Send)]
impl<RC, AT> GenericPendingTask for PendingTask<RC, AT, RegisterTask>
where
    RC: ReturnChannel<Success = ()>,
    AT: AsyncTask,
{
    async fn call(
        mut self: Box<Self>,
        task_idx: usize,
        mut stack: Box<AsyncStackPage>,
        rt_sender: AsyncStdSender<Message>,
    ) {
        unsafe {
            let sender = self.sender();

            let mode = Async(&stack.top[1]);
            let raw = stack.page.as_mut();
            let mut frame = AsyncGcFrame::new(raw, 0, mode);
            let global = Global::new();

            let res = AT::register(global, &mut frame).await;
            sender.send(res).await;
        }

        rt_sender
            .send(Message::Complete(task_idx, stack))
            .await
            .expect("Channel was closed");
    }
}

#[async_trait(?Send)]
impl<RC, GT> GenericPendingTask for PendingTask<RC, GT, RegisterGenerator>
where
    RC: ReturnChannel<Success = ()>,
    GT: GeneratorTask,
{
    async fn call(
        mut self: Box<Self>,
        task_idx: usize,
        mut stack: Box<AsyncStackPage>,
        rt_sender: AsyncStdSender<Message>,
    ) {
        unsafe {
            let sender = self.sender();

            let mode = Async(&stack.top[1]);
            let raw = stack.page.as_mut();
            let mut frame = AsyncGcFrame::new(raw, 0, mode);
            let global = Global::new();

            let res = GT::register(global, &mut frame).await;
            sender.send(res).await;
        }

        rt_sender
            .send(Message::Complete(task_idx, stack))
            .await
            .expect("Channel was closed");
    }
}

#[async_trait(?Send)]
impl<IRC, GT> GenericPendingTask for PendingTask<IRC, GT, Generator>
where
    IRC: ReturnChannel<Success = GeneratorHandle<GT>>,
    GT: GeneratorTask,
{
    async fn call(
        mut self: Box<Self>,
        task_idx: usize,
        mut stack: Box<AsyncStackPage>,
        rt_sender: AsyncStdSender<Message>,
    ) {
        unsafe {
            let (mut generator, handle_sender) = self.split();

            // Transmute to get static lifetimes. Should be okay because tasks can't leak
            // Julia data and the frame is not dropped until the generator is dropped.
            let mode = Async(std::mem::transmute(&stack.top[1]));
            let raw = std::mem::transmute(stack.page.as_mut());
            let mut frame = AsyncGcFrame::new(raw, GT::INIT_SLOTS, mode);
            let global = Global::new();

            match generator.call_init(global, &mut frame).await {
                Ok(mut state) => {
                    let (sender, receiver) = if GT::CHANNEL_CAPACITY == 0 {
                        async_std::channel::unbounded()
                    } else {
                        async_std::channel::bounded(GT::CHANNEL_CAPACITY)
                    };

                    let handle = generator.create_handle(sender);
                    handle_sender.send(Ok(handle)).await;

                    loop {
                        let mut msg = match receiver.recv().await {
                            Ok(msg) => msg,
                            Err(_) => break,
                        };

                        let res = generator
                            .call_run(global, &mut frame, &mut state, msg.input())
                            .await;

                        msg.respond(res).await;
                    }
                }
                Err(e) => {
                    handle_sender.send(Err(e)).await;
                }
            }
        }

        rt_sender
            .send(Message::Complete(task_idx, stack))
            .await
            .expect("Channel was closed");
    }
}
