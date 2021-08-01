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
use super::RequireSendSync;
use super::{AsyncStackPage, Message};
use crate::error::{JlrsError, JlrsResult};
use crate::memory::frame::GcFrame;
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
    /// The type of the result which is returned if `init` completes successfully. It's provided
    /// to every call of `run`. Because `init` takes a frame with the `'static` lifetime, this
    /// type can contain Julia data.
    type InitData: 'static + Clone;

    /// The type of the data that must be provided when calling this generator.
    type CallData: 'static + Send + Sync;

    /// The type of the result which is returned if `run` completes successfully.
    type Output: 'static + Send + Sync;

    /// The capacity of the channel the `GeneratorHandle` uses to communicate with this generator.
    /// If it's set to 0, the channel is unbounded.
    const CHANNEL_CAPACITY: usize = 0;

    /// The number of slots preallocated for the `GcFrame` provided to `init`.
    const INIT_SLOTS: usize = 0;

    /// The number of slots preallocated for the `AsyncGcFrame` provided to `run`.
    const RUN_SLOTS: usize = 0;

    /// Initialize the generator. You can interact with Julia inside this method, the frame is
    /// not dropped until the generator itself is dropped. This means that `InitData` can contain
    /// arbitrary Julia data rooted in this frame. This data is provided to every call to `run`.
    fn init(
        &mut self,
        global: Global<'static>,
        frame: &mut GcFrame<'static, Async<'static>>,
    ) -> JlrsResult<Self::InitData>;

    /// Run the generator. This method takes a `Global` and a mutable reference to an
    /// `AsyncGcFrame`, which lets you interact with Julia. It's also provided with the result
    /// of `init` and the `call_data` provided by the caller.
    async fn run<'nested>(
        &mut self,
        global: Global<'nested>,
        frame: &mut AsyncGcFrame<'nested>,
        init_data: Self::InitData,
        call_data: Self::CallData,
    ) -> JlrsResult<Self::Output>;
}

type HandleSender<GT> = AsyncStdSender<
    Box<
        dyn GenericCallGeneratorMessage<
            Data = <GT as GeneratorTask>::CallData,
            Output = <GT as GeneratorTask>::Output,
        >,
    >,
>;

/// A handle to a `GeneratorTask`. This handle can be used to call the generator, and can be used
/// from multiple threads. The `GeneratorTask` is dropped when its final handle has been dropped.
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
    pub async fn call<R>(&self, data: GT::CallData, sender: R)
    where
        R: ReturnChannel<T = GT::Output>,
    {
        self.sender
            .send(Box::new(CallGeneratorMessage {
                data: Some(data),
                sender,
            }))
            .await
            .expect("Channel was closed")
    }

    /// Call the generator, this method returns an error immediately if there's room available in
    /// the channel.
    pub fn try_call<R>(&self, data: GT::CallData, sender: R) -> JlrsResult<()>
    where
        R: ReturnChannel<T = GT::Output>,
    {
        match self.sender.try_send(Box::new(CallGeneratorMessage {
            data: Some(data),
            sender,
        })) {
            Ok(_) => Ok(()),
            Err(e) => Err(JlrsError::other(e))?,
        }
    }

    /// Returns the capacity of the channel used to communicate with the generator if a bounded
    /// channel is used, or `None` if it's unbounded.
    pub fn capacity(&self) -> Option<usize> {
        self.sender.capacity()
    }

    /// Returns the number of messages in the channel used to communicate with the generator.
    pub fn len(&self) -> usize {
        self.sender.len()
    }
}

impl<GT: GeneratorTask> RequireSendSync for GeneratorHandle<GT> {}

pub(crate) enum Generator {}
pub(crate) enum Task {}

pub(crate) struct PendingTask<RC, AT, Kind> {
    task: AT,
    sender: RC,
    _kind: PhantomData<Kind>,
}

impl<RC, AT> PendingTask<RC, AT, Task>
where
    RC: ReturnChannel<T = AT::Output>,
    AT: AsyncTask,
{
    pub(crate) fn new(task: AT, sender: RC) -> Self {
        PendingTask {
            task,
            sender,
            _kind: PhantomData,
        }
    }

    pub(crate) fn split(self) -> (AT, RC) {
        (self.task, self.sender)
    }
}

impl<IRC, GT> PendingTask<IRC, GT, Generator>
where
    IRC: ReturnChannel<T = GeneratorHandle<GT>>,
    GT: GeneratorTask,
{
    pub(crate) fn new(task: GT, sender: IRC) -> Self {
        PendingTask {
            task,
            sender,
            _kind: PhantomData,
        }
    }

    pub(crate) fn split(self) -> (GT, IRC) {
        (self.task, self.sender)
    }
}

struct CallGeneratorMessage<CD, O, RC>
where
    CD: 'static + Send + Sync,
    O: 'static + Send + Sync,
    RC: ReturnChannel<T = O>,
{
    sender: RC,
    data: Option<CD>,
}

#[async_trait(?Send)]
trait GenericCallGeneratorMessage: 'static + Send + Sync {
    type Data;
    type Output;

    async fn send(&self, result: JlrsResult<Self::Output>);
    fn data(&mut self) -> Self::Data;
}

#[async_trait(?Send)]
impl<CD, O, RC> GenericCallGeneratorMessage for CallGeneratorMessage<CD, O, RC>
where
    CD: 'static + Send + Sync,
    O: 'static + Send + Sync,
    RC: ReturnChannel<T = O>,
{
    type Data = CD;
    type Output = O;

    async fn send(&self, result: JlrsResult<Self::Output>) {
        self.sender.send(result).await
    }

    fn data(&mut self) -> Self::Data {
        self.data.take().unwrap()
    }
}

#[async_trait(?Send)]
trait GenericAsyncTask: Send + Sync + Sized {
    type AT: AsyncTask + Send + Sync;

    async fn call_run(
        &mut self,
        global: Global<'static>,
        frame: GcFrame<'static, Async<'static>>,
    ) -> JlrsResult<<Self::AT as AsyncTask>::Output>;
}

#[async_trait(?Send)]
impl<AT: AsyncTask> GenericAsyncTask for AT {
    type AT = Self;
    async fn call_run(
        &mut self,
        global: Global<'static>,
        mut frame: GcFrame<'static, Async<'static>>,
    ) -> JlrsResult<<Self::AT as AsyncTask>::Output> {
        unsafe {
            let mut frame = AsyncGcFrame::new_from(&mut frame, AT::RUN_SLOTS);
            self.run(global, &mut frame).await
        }
    }
}

#[async_trait(?Send)]
trait GenericGeneratorTask: Send + Sync + Sized {
    type GT: GeneratorTask + Send + Sync;

    unsafe fn call_init(
        &mut self,
        global: Global<'static>,
        frame: &mut GcFrame<'static, Async<'static>>,
    ) -> JlrsResult<<Self::GT as GeneratorTask>::InitData>;

    async fn call_run(
        &mut self,
        global: Global<'static>,
        frame: GcFrame<'static, Async<'static>>,
        init_data: <Self::GT as GeneratorTask>::InitData,
        call_data: <Self::GT as GeneratorTask>::CallData,
    ) -> (
        GcFrame<'static, Async<'static>>,
        JlrsResult<<Self::GT as GeneratorTask>::Output>,
    );

    fn create_handle(&self, sender: HandleSender<Self::GT>) -> GeneratorHandle<Self::GT>;
}

#[async_trait(?Send)]
impl<GT> GenericGeneratorTask for GT
where
    GT: GeneratorTask,
{
    type GT = Self;

    unsafe fn call_init(
        &mut self,
        global: Global<'static>,
        frame: &mut GcFrame<'static, Async<'static>>,
    ) -> JlrsResult<<Self::GT as GeneratorTask>::InitData> {
        self.init(global, frame)
    }

    async fn call_run(
        &mut self,
        global: Global<'static>,
        mut frame: GcFrame<'static, Async<'static>>,
        init_data: <Self::GT as GeneratorTask>::InitData,
        call_data: <Self::GT as GeneratorTask>::CallData,
    ) -> (
        GcFrame<'static, Async<'static>>,
        JlrsResult<<Self::GT as GeneratorTask>::Output>,
    ) {
        unsafe {
            let output = {
                let mut frame = AsyncGcFrame::new_from(&mut frame, GT::RUN_SLOTS);
                self.run(global, &mut frame, init_data, call_data).await
            };

            (frame, output)
        }
    }

    fn create_handle(&self, sender: HandleSender<Self>) -> GeneratorHandle<Self> {
        GeneratorHandle::new(sender)
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
    RC: ReturnChannel<T = AT::Output>,
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
            // Julia data and the frame is not dropped until the task is dropped.
            // TODO: call_run creates a new frame, "upgrade" this one instead.
            let mode = Async(std::mem::transmute(&stack.top[1]));
            let raw = std::mem::transmute(stack.page.as_mut());
            let frame = GcFrame::new(raw, 0, mode);
            let global = Global::new();

            let res = GenericAsyncTask::call_run(&mut task, global, frame).await;
            result_sender.send(res).await;
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
    IRC: ReturnChannel<T = GeneratorHandle<GT>>,
    GT: GeneratorTask,
{
    async fn call(
        mut self: Box<Self>,
        task_idx: usize,
        mut stack: Box<AsyncStackPage>,
        rt_sender: AsyncStdSender<Message>,
    ) {
        unsafe {
            {
                let (mut generator, handle_sender) = self.split();

                // Transmute to get static lifetimes. Should be okay because tasks can't leak
                // Julia data and the frame is not dropped until the generator is dropped.
                let mode = Async(std::mem::transmute(&stack.top[1]));
                let raw = std::mem::transmute(stack.page.as_mut());
                let mut frame = GcFrame::new(raw, GT::INIT_SLOTS, mode);
                let global = Global::new();

                match GenericGeneratorTask::call_init(&mut generator, global, &mut frame) {
                    Ok(init_data) => {
                        let (sender, receiver) = if GT::CHANNEL_CAPACITY == 0 {
                            async_std::channel::unbounded()
                        } else {
                            async_std::channel::bounded(GT::CHANNEL_CAPACITY)
                        };

                        let handle = GenericGeneratorTask::create_handle(&generator, sender);
                        handle_sender.send(Ok(handle)).await;

                        loop {
                            let mut msg = match receiver.recv().await {
                                Ok(msg) => msg,
                                Err(_) => break,
                            };

                            let data = msg.data();
                            match GenericGeneratorTask::call_run(
                                &mut generator,
                                global,
                                frame,
                                init_data.clone(),
                                data,
                            )
                            .await
                            {
                                (fr, Ok(res)) => {
                                    frame = fr;
                                    msg.send(Ok(res)).await;
                                }
                                (fr, Err(e)) => {
                                    frame = fr;
                                    msg.send(Err(e)).await;
                                }
                            }
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
}
