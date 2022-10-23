//! Use Julia with support for multitasking.
//!
//! This module is only available if the `async-rt` feature is enabled.
//!
//! While access to the Julia C API is not thread-safe, it is possible to create and schedule new
//! tasks from the thread that has intialized Julia. To do so from Rust you must use an async
//! runtime rather than the sync runtime.
//!
//! In order to use an async runtime, you'll have to choose a backing runtime. By default, tokio
//! and async-std can be used by enabling the `tokio-rt` or `async-std-rt` feature respectively.
//! To use a custom runtime, you can implement the `AsyncRuntime` trait.
//!
//! After initialization a handle to the runtime, [`AsyncJulia`], is returned which can be shared
//! across threads and can be used to send new tasks to the runtime. Three kinds of task exist:
//! blocking, async, and persistent tasks. Blocking tasks block the runtime, the other two kinds
//! of tasks can schedule Julia function calls and wait for them to complete. While the scheduled
//! Julia function hasn't returned the async runtime handles other tasks. Blocking tasks can be
//! expressed as closures, the other two kinds of task require implementing the [`AsyncTask`] and
//! [`PersistentTask`] traits respectively.

#[cfg(feature = "nightly")]
pub mod adopted;
#[cfg(feature = "async-std-rt")]
pub mod async_std_rt;
pub mod queue;
#[cfg(feature = "tokio-rt")]
pub mod tokio_rt;

use crate::{
    async_util::{
        channel::{Channel, ChannelSender, OneshotSender, TrySendError},
        future::wake_task,
        internal::{
            BlockingTask, BlockingTaskEnvelope, CallPersistentTask, IncludeTask,
            IncludeTaskEnvelope, InnerPersistentMessage, PendingTask, PendingTaskEnvelope,
            Persistent, PersistentComms, RegisterPersistent, RegisterTask, SetErrorColorTask,
            SetErrorColorTaskEnvelope, Task,
        },
        task::{AsyncTask, PersistentTask},
    },
    error::{IOError, JlrsError, JlrsResult, RuntimeError},
    memory::{context::stack::Stack, stack_frame::StackFrame, target::frame::GcFrame},
    runtime::{builder::AsyncRuntimeBuilder, init_jlrs, INIT},
    wrappers::ptr::{module::Module, value::Value},
};
use async_trait::async_trait;
use futures::Future;
use jl_sys::{
    jl_atexit_hook, jl_enter_threaded_region, jl_exit_threaded_region, jl_gc_safepoint, jl_init,
    jl_init_with_image, jl_is_initialized, jl_process_events, jl_yield,
};

use jl_sys::jl_options;

use std::{
    cell::RefCell,
    collections::VecDeque,
    ffi::c_void,
    fmt,
    marker::PhantomData,
    path::Path,
    rc::Rc,
    sync::{atomic::Ordering, Arc},
    time::Duration,
};

use self::{
    adopted::init_worker,
    queue::{channel, Receiver, Sender},
};

#[cfg(feature = "nightly")]
init_fn!(init_multitask, JLRS_MULTITASK_JL, "JlrsMultitaskNightly.jl");

#[cfg(not(feature = "nightly"))]
init_fn!(init_multitask, JLRS_MULTITASK_JL, "JlrsMultitask.jl");

/// Convert `Self` to a `Result`.
pub trait IntoResult<T, E> {
    /// Convert `self` to a `Result`.
    fn into_result(self) -> Result<T, E>;
}

impl<E> IntoResult<(), E> for () {
    fn into_result(self) -> Result<(), E> {
        Ok(self)
    }
}

impl<E> IntoResult<JlrsResult<()>, E> for JlrsResult<()> {
    fn into_result(self) -> Result<JlrsResult<()>, E> {
        Ok(self)
    }
}

impl<E> IntoResult<(), E> for Result<(), E> {
    fn into_result(self) -> Result<(), E> {
        self
    }
}

impl<E> IntoResult<JlrsResult<()>, E> for Result<JlrsResult<()>, E> {
    fn into_result(self) -> Result<JlrsResult<()>, E> {
        self
    }
}

/// Functionality that is necessary to use an async runtime with jlrs.
///
/// If you want to use async-std or tokio, you can use one of the implementations provided by
/// jlrs. If you want to use another crate you can implement this trait.
#[async_trait(?Send)]
pub trait AsyncRuntime: Send + 'static {
    /// Error that is returned when a task can't be joined because it has panicked.
    type JoinError;

    /// The output type of a task spawned by `AsyncRuntime::spawn_local`.
    type TaskOutput: IntoResult<(), Self::JoinError>;

    /// The output type of the runtime task spawned by `AsyncRuntime::spawn_blocking`.
    type RuntimeOutput: IntoResult<JlrsResult<()>, Self::JoinError>;

    /// The handle type of a task spawned by `AsyncRuntime::spawn_local`.
    type JoinHandle: Future<Output = Self::TaskOutput>;

    /// The handle type of the runtime task spawned by `AsyncRuntime::spawn_local`.
    type RuntimeHandle: Future<Output = Self::RuntimeOutput>;

    /// Spawn the async runtime a new thread, this method called if `AsyncBuilder::start` is
    /// called.
    fn spawn_thread<F>(rt_fn: F) -> std::thread::JoinHandle<JlrsResult<()>>
    where
        F: FnOnce() -> JlrsResult<()> + Send + 'static,
    {
        std::thread::spawn(rt_fn)
    }

    /// Spawn the async runtime a blocking task, this method called if `AsyncBuilder::start_async`
    /// is called.
    fn spawn_blocking<F>(rt_fn: F) -> Self::RuntimeHandle
    where
        F: FnOnce() -> JlrsResult<()> + Send + 'static;

    /// Block on a future, this method is called to start the runtime loop.
    fn block_on<F>(loop_fn: F, worker_id: Option<usize>) -> JlrsResult<()>
    where
        F: Future<Output = JlrsResult<()>>;

    async fn yield_now();

    /// Spawn a local task, this method is called from the loop task to spawn an [`AsyncTask`] or
    /// [`PersistentTask`].
    fn spawn_local<F>(future: F) -> Self::JoinHandle
    where
        F: Future<Output = ()> + 'static;

    /// Wait on `future` until it resolves or `duration` has elapsed. If the future times out it
    /// must return `None`.
    async fn timeout<F>(duration: Duration, future: F) -> Option<JlrsResult<Message>>
    where
        F: Future<Output = JlrsResult<Message>>;
}

/// A handle to the async runtime.
///
/// This handle can be used to include files and send new tasks to the runtime. The runtime shuts
/// down when the last handle is dropped.
pub struct AsyncJulia<R>
where
    R: AsyncRuntime,
{
    sender: Sender<Message>,
    _runtime: PhantomData<R>,
}

impl<R> AsyncJulia<R>
where
    R: AsyncRuntime,
{
    pub fn resize_queue<'own>(&'own self, capacity: usize) -> impl 'own + Future<Output = ()> {
        self.sender.resize_queue(capacity)
    }

    pub async fn resize_queue_async(&self, capacity: usize) {
        self.resize_queue(capacity).await
    }

    /// Send a new async task to the runtime.
    ///
    /// This method waits if there's no room in the channel. It takes two arguments, the task and
    /// the sending half of a channel which is used to send the result back after the task has
    /// completed.
    pub async fn task<A, O>(&self, task: A, res_sender: O)
    where
        A: AsyncTask,
        O: OneshotSender<JlrsResult<A::Output>>,
    {
        let msg = PendingTask::<_, _, Task>::new(task, res_sender);
        let boxed = Box::new(msg);
        self.sender.send(MessageInner::Task(boxed).wrap()).await
    }

    /// Try to send a new async task to the runtime.
    ///
    /// If there's no room in the backing channel an error is returned immediately. This method
    /// takes two arguments, the task and the sending half of a channel which is used to send the
    /// result back after the task has completed.
    pub fn try_task<A, O>(&self, task: A, res_sender: O) -> JlrsResult<()>
    where
        A: AsyncTask,
        O: OneshotSender<JlrsResult<A::Output>>,
    {
        let msg = PendingTask::<_, _, Task>::new(task, res_sender);
        let boxed = Box::new(msg);
        self.sender.try_send(MessageInner::Task(boxed).wrap())
    }

    /// Register an async task.
    ///
    /// This method waits if there's no room in the channel. It takes one argument, the sending
    /// half of a channel which is used to send the result back after the registration has
    /// completed.
    pub async fn register_task<A, O>(&self, res_sender: O)
    where
        A: AsyncTask,
        O: OneshotSender<JlrsResult<()>>,
    {
        let msg = PendingTask::<_, A, RegisterTask>::new(res_sender);
        let boxed = Box::new(msg);
        self.sender.send(MessageInner::Task(boxed).wrap()).await
    }

    /// Try to register an async task.
    ///
    /// If there's no room in the channel an error is returned immediately. This method takes one
    /// argument, the sending half of a channel which is used to send the result back after the
    /// registration has completed.
    pub fn try_register_task<A, O>(&self, res_sender: O) -> JlrsResult<()>
    where
        A: AsyncTask,
        O: OneshotSender<JlrsResult<()>>,
    {
        let msg = PendingTask::<_, A, RegisterTask>::new(res_sender);
        let boxed = Box::new(msg);
        self.sender.try_send(MessageInner::Task(boxed).wrap())
    }

    /// Send a new blocking task to the runtime.
    ///
    /// This method waits if there's no room in the channel. It takes two arguments, the first is
    /// a closure that takes a `GcFrame` and must return a `JlrsResult` whose inner type is both
    /// `Send` and `Sync`. The second is the sending half of a channel which is used to send the
    /// result back after the task has completed. This task is executed as soon as possible and
    /// can't call async methods, so it blocks the runtime.
    pub async fn blocking_task<T, O, F>(&self, task: F, res_sender: O)
    where
        for<'base> F: 'static + Send + Sync + FnOnce(GcFrame<'base>) -> JlrsResult<T>,
        O: OneshotSender<JlrsResult<T>>,
        T: Send + Sync + 'static,
    {
        let msg = BlockingTask::new(task, res_sender);
        let boxed = Box::new(msg);
        self.sender
            .send(MessageInner::BlockingTask(boxed).wrap())
            .await
    }

    /// Try to send a new blocking task to the runtime.
    ///
    /// If there's no room in the backing channel an error is returned immediately. This method
    /// takes two arguments, the first is a closure that takes a `GcFrame` and must return a
    /// `JlrsResult` whose inner type is both `Send` and `Sync`. The second is the sending half of
    /// a channel which is used to send the  result back after the task has completed. This task
    /// is executed as soon as possible and can't call async methods, so it blocks the runtime.
    pub fn try_blocking_task<T, O, F>(&self, task: F, res_sender: O) -> JlrsResult<()>
    where
        for<'base> F: 'static + Send + Sync + FnOnce(GcFrame<'base>) -> JlrsResult<T>,
        O: OneshotSender<JlrsResult<T>>,
        T: Send + Sync + 'static,
    {
        let msg = BlockingTask::new(task, res_sender);
        let boxed = Box::new(msg);
        self.sender
            .try_send(MessageInner::BlockingTask(boxed).wrap())
    }

    /// Send a new persistent task to the runtime.
    ///
    /// This method waits if there's no room in the channel. It takes a two arguments, the task
    /// and a `OneshotSender` to send a [`PersistentHandle`] after the task's `init` method has
    /// completed. You must also provide an implementation of [`Channel`] as a type parameter.
    /// This channel is used by the handle to communicate with the persistent task.
    pub async fn persistent<C, P, O>(&self, task: P, handle_sender: O)
    where
        C: Channel<PersistentMessage<P>>,
        P: PersistentTask,
        O: OneshotSender<JlrsResult<PersistentHandle<P>>>,
    {
        let msg = PendingTask::<_, _, Persistent>::new(
            task,
            PersistentComms::<C, _, _>::new(handle_sender),
        );
        let boxed = Box::new(msg);

        self.sender.send(MessageInner::Task(boxed).wrap()).await
    }

    /// Try to send a new persistent task to the runtime.
    ///
    /// If there's no room in the backing channel an error is returned immediately. This method
    /// takes a two arguments, the task  and a `OneshotSender` to send a [`PersistentHandle`]
    /// after the task's `init` method has completed. You must also provide an implementation of
    /// [`Channel`] as a type parameter. This channel is used by the handle to communicate with
    /// the persistent task.
    pub fn try_persistent<C, P, O>(&self, task: P, handle_sender: O) -> JlrsResult<()>
    where
        C: Channel<PersistentMessage<P>>,
        P: PersistentTask,
        O: OneshotSender<JlrsResult<PersistentHandle<P>>>,
    {
        let msg = PendingTask::<_, _, Persistent>::new(
            task,
            PersistentComms::<C, _, _>::new(handle_sender),
        );
        let boxed = Box::new(msg);
        self.sender.try_send(MessageInner::Task(boxed).wrap())
    }

    /// Register a persistent task.
    ///
    /// This method waits if there's no room in the channel. It takes one argument, the sending
    /// half of a channel which is used to send the result back after the registration has
    /// completed.
    pub async fn register_persistent<P, O>(&self, res_sender: O)
    where
        P: PersistentTask,
        O: OneshotSender<JlrsResult<()>>,
    {
        let msg = PendingTask::<_, P, RegisterPersistent>::new(res_sender);
        let boxed = Box::new(msg);
        self.sender.send(MessageInner::Task(boxed).wrap()).await
    }

    /// Try to register a persistent task.
    ///
    /// If there's no room in the channel an error is returned immediately. This method takes one
    /// argument, the sending half of a channel which is used to send the result back after the
    /// registration has completed.
    pub fn try_register_persistent<P, O>(&self, res_sender: O) -> JlrsResult<()>
    where
        P: PersistentTask,
        O: OneshotSender<JlrsResult<()>>,
    {
        let msg = PendingTask::<_, P, RegisterPersistent>::new(res_sender);
        let boxed = Box::new(msg);
        self.sender.try_send(MessageInner::Task(boxed).wrap())
    }

    /// Include a Julia file by calling `Main.include` as a blocking task.
    ///
    /// This method waits if there's no room in the channel. It takes two arguments, the path to
    /// the file and the sending half of a channel which is used to send the result back after the
    /// file has been included.
    ///
    /// Safety: this method evaluates the contents of the file if it exists, which can't be
    /// checked for correctness.
    pub async unsafe fn include<P, O>(&self, path: P, res_sender: O) -> JlrsResult<()>
    where
        P: AsRef<Path>,
        O: OneshotSender<JlrsResult<()>>,
    {
        if !path.as_ref().exists() {
            Err(IOError::NotFound {
                path: path.as_ref().to_string_lossy().into(),
            })?
        }

        let msg = IncludeTask::new(path.as_ref().into(), res_sender);

        self.sender
            .send(MessageInner::Include(Box::new(msg)).wrap())
            .await;

        Ok(())
    }

    /// Try to include a Julia file by calling `Main.include` as a blocking task.
    ///
    /// If there's no room in the channel an error is returned immediately. This method takes two
    /// arguments, the path to the file and the sending half of a channel which is used to send
    /// the result back after the file has been included.
    ///
    /// Safety: this method evaluates the contents of the file if it exists, which can't be
    /// checked for correctness.
    pub unsafe fn try_include<P, O>(&self, path: P, res_sender: O) -> JlrsResult<()>
    where
        P: AsRef<Path>,
        O: OneshotSender<JlrsResult<()>>,
    {
        if !path.as_ref().exists() {
            Err(IOError::NotFound {
                path: path.as_ref().to_string_lossy().into(),
            })?
        }

        let msg = IncludeTask::new(path.as_ref().into(), res_sender);

        self.sender
            .try_send(MessageInner::Include(Box::new(msg)).wrap())
    }

    /// Enable or disable colored error messages originating from Julia as a blocking task.
    ///
    /// This method waits if there's no room in the channel. It takes two arguments, a `bool` to
    /// enable or disable colored error messages and the sending half of a channel which is used
    /// to send the result back after the option is set.
    ///
    /// This feature is disabled by default.
    pub async fn error_color<O>(&self, enable: bool, res_sender: O)
    where
        O: OneshotSender<JlrsResult<()>>,
    {
        let msg = SetErrorColorTask::new(enable, res_sender);

        self.sender
            .send(MessageInner::ErrorColor(Box::new(msg)).wrap())
            .await
    }

    /// Try to enable or disable colored error messages originating from Julia as a blocking task.
    ///
    /// If there's no room in the channel an error is returned immediately. This method takes two
    /// arguments, a `bool` to enable or disable colored error messages and the sending half of a
    /// channel which is used to send the result back after the option is set.
    ///
    /// This feature is disabled by default.
    pub fn try_error_color<O>(&self, enable: bool, res_sender: O) -> JlrsResult<()>
    where
        O: OneshotSender<JlrsResult<()>>,
    {
        let msg = SetErrorColorTask::new(enable, res_sender);
        self.sender
            .try_send(MessageInner::ErrorColor(Box::new(msg)).wrap())
    }

    pub(crate) unsafe fn init<C, const N: usize>(
        builder: AsyncRuntimeBuilder<R, C>,
    ) -> JlrsResult<(Self, std::thread::JoinHandle<JlrsResult<()>>)>
    where
        C: Channel<Message>,
    {
        let (sender, receiver) = channel(builder.channel_capacity);
        let handle = R::spawn_thread(move || Self::run_async::<_, N>(builder, receiver));

        let julia = AsyncJulia {
            sender,
            _runtime: PhantomData,
        };

        Ok((julia, handle))
    }

    pub(crate) unsafe fn init_async<C, const N: usize>(
        builder: AsyncRuntimeBuilder<R, C>,
    ) -> JlrsResult<(Self, R::RuntimeHandle)>
    where
        C: Channel<Message>,
    {
        let (sender, receiver) = channel(builder.channel_capacity);
        let handle = R::spawn_blocking(move || Self::run_async::<_, N>(builder, receiver));

        let julia = AsyncJulia {
            sender,
            _runtime: PhantomData,
        };

        Ok((julia, handle))
    }

    fn run_async<C, const N: usize>(
        builder: AsyncRuntimeBuilder<R, C>,
        receiver: Receiver<Message>,
    ) -> JlrsResult<()>
    where
        C: Channel<Message>,
    {
        unsafe {
            if jl_is_initialized() != 0 || INIT.swap(true, Ordering::SeqCst) {
                Err(RuntimeError::AlreadyInitialized)?;
            }
            #[cfg(not(feature = "nightly"))]
            {
                if builder.n_threads == 0 {
                    jl_options.nthreads = -1;
                } else {
                    jl_options.nthreads = builder.n_threads as _;
                }
            }

            #[cfg(feature = "nightly")]
            {
                if builder.n_threadsi != 0 {
                    if builder.n_threads == 0 {
                        jl_options.nthreads = -1;
                        jl_options.nthreadpools = 2;
                        let perthread = Box::new([-1i16, builder.n_threadsi as _]);
                        jl_options.nthreads_per_pool = Box::leak(perthread) as *const _;
                    } else {
                        let nthreads = builder.n_threads as i16;
                        let nthreadsi = builder.n_threadsi as i16;
                        jl_options.nthreads = nthreads + nthreadsi;
                        jl_options.nthreadpools = 2;
                        let perthread = Box::new([nthreads, builder.n_threadsi as _]);
                        jl_options.nthreads_per_pool = Box::leak(perthread) as *const _;
                    }
                } else if builder.n_threads == 0 {
                    jl_options.nthreads = -1;
                    jl_options.nthreadpools = 1;
                    let perthread = Box::new(-1i16);
                    jl_options.nthreads_per_pool = Box::leak(perthread) as *const _;
                } else {
                    let n_threads = builder.n_threads as _;
                    jl_options.nthreads = n_threads;
                    jl_options.nthreadpools = 1;
                    let perthread = Box::new(n_threads);
                    jl_options.nthreads_per_pool = Box::leak(perthread) as *const _;
                }
            }

            if let Some((ref julia_bindir, ref image_path)) = builder.builder.image {
                let julia_bindir_str = julia_bindir.to_string_lossy().to_string();
                let image_path_str = image_path.to_string_lossy().to_string();

                if !julia_bindir.exists() {
                    return Err(IOError::NotFound {
                        path: julia_bindir_str,
                    })?;
                }

                if !image_path.exists() {
                    return Err(IOError::NotFound {
                        path: image_path_str,
                    })?;
                }

                let bindir = std::ffi::CString::new(julia_bindir_str).unwrap();
                let im_rel_path = std::ffi::CString::new(image_path_str).unwrap();

                jl_init_with_image(bindir.as_ptr(), im_rel_path.as_ptr());
            } else {
                jl_init();
            }
        }

        let mut base_frame = StackFrame::<N>::new_n();
        R::block_on(
            unsafe { Self::run_inner(builder, receiver, &mut base_frame) },
            None,
        )
    }

    async unsafe fn run_inner<'ctx, C, const N: usize>(
        builder: AsyncRuntimeBuilder<R, C>,
        receiver: Receiver<Message>,
        base_frame: &'ctx mut StackFrame<N>,
    ) -> Result<(), Box<JlrsError>>
    where
        C: Channel<Message>,
    {
        let base_frame: &'static mut StackFrame<N> = std::mem::transmute(base_frame);
        let mut pinned = base_frame.pin();
        let base_frame = pinned.stack_frame();

        set_custom_fns(base_frame.sync_stack())?;

        let free_stacks = {
            let mut free_stacks = VecDeque::with_capacity(N);
            for i in 0..N {
                free_stacks.push_back(i);
            }

            Rc::new(RefCell::new(free_stacks))
        };

        let running_tasks = {
            let mut running_tasks = Vec::with_capacity(N);
            for _ in 0..N {
                running_tasks.push(None);
            }

            Rc::new(RefCell::new(running_tasks.into_boxed_slice()))
        };

        let recv_timeout = builder.recv_timeout;

        #[cfg(feature = "nightly")]
        let mut workers = Vec::with_capacity(builder.n_workers);
        #[cfg(feature = "nightly")]
        for i in 0..builder.n_workers {
            let worker = init_worker::<R, N>(i, recv_timeout, receiver.clone());
            workers.push(worker)
        }

        jl_enter_threaded_region();

        loop {
            if free_stacks.borrow().len() == 0 {
                jl_process_events();
                jl_yield();
                R::yield_now().await;

                continue;
            }

            match R::timeout(recv_timeout, receiver.recv()).await {
                None => {
                    jl_process_events();
                    jl_yield();
                }
                Some(Ok(msg)) => match msg.inner {
                    MessageInner::Task(task) => {
                        // TODO: only receive if free space available
                        let idx = free_stacks.borrow_mut().pop_front().unwrap();
                        let stack = base_frame.nth_stack(idx);

                        let task = {
                            let free_stacks = free_stacks.clone();
                            let running_tasks = running_tasks.clone();

                            R::spawn_local(async move {
                                task.call(stack).await;
                                free_stacks.borrow_mut().push_back(idx);
                                running_tasks.borrow_mut()[idx] = None;
                            })
                        };

                        running_tasks.borrow_mut()[idx] = Some(task);
                    }
                    // TODO: single trait
                    MessageInner::BlockingTask(task) => {
                        let stack = base_frame.sync_stack();
                        task.call(stack);
                    }
                    MessageInner::Include(task) => {
                        let stack = base_frame.sync_stack();
                        task.call(stack);
                    }
                    MessageInner::ErrorColor(task) => {
                        let stack = base_frame.sync_stack();
                        task.call(stack);
                    }
                },
                Some(Err(_)) => break,
            }
        }

        for i in 0..N {
            let task = running_tasks.borrow_mut()[i].take();
            if let Some(task) = task {
                task.await;
            }
        }

        #[cfg(feature = "nightly")]
        for worker in workers.into_iter() {
            loop {
                if worker.is_finished() {
                    let _ = worker.join();
                    break;
                }

                jl_gc_safepoint();
                // TODO: julia sleep?
            }
        }

        jl_exit_threaded_region();

        jl_atexit_hook(0);
        Ok(())
    }
}

/// The message type used by the async runtime for communication.
pub struct Message {
    inner: MessageInner,
}

pub(crate) enum MessageInner {
    Task(Box<dyn PendingTaskEnvelope>),
    BlockingTask(Box<dyn BlockingTaskEnvelope>),
    Include(Box<dyn IncludeTaskEnvelope>),
    ErrorColor(Box<dyn SetErrorColorTaskEnvelope>),
}

impl fmt::Debug for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Message")
    }
}

impl MessageInner {
    pub(crate) fn wrap(self) -> Message {
        Message { inner: self }
    }
}

fn set_custom_fns(stack: &Stack) -> JlrsResult<()> {
    unsafe {
        let (owner, mut frame) = GcFrame::base(stack);

        init_jlrs(&mut frame);
        init_multitask(&mut frame);

        let jlrs_mod = Module::main(&frame)
            .submodule(&frame, "JlrsMultitask")?
            .wrapper_unchecked();

        let wake_rust = Value::new(&mut frame, wake_task as *mut c_void);
        jlrs_mod
            .global(&frame, "wakerust")?
            .wrapper_unchecked()
            .set_nth_field_unchecked(0, wake_rust);

        std::mem::drop(owner);
        Ok(())
    }
}

/// The message type used by persistent handles for communication with persistent tasks.
pub struct PersistentMessage<P>
where
    P: PersistentTask,
{
    pub(crate) msg: InnerPersistentMessage<P>,
}

impl<P> fmt::Debug for PersistentMessage<P>
where
    P: PersistentTask,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PersistentMessage")
    }
}

/// A handle to a [`PersistentTask`].
///
/// This handle can be used to call the task and shared across threads. The `PersistentTask` is
/// dropped when its final handle has been dropped and all remaining pending calls have completed.
#[derive(Clone)]
pub struct PersistentHandle<P>
where
    P: PersistentTask,
{
    sender: Arc<dyn ChannelSender<PersistentMessage<P>>>,
}

impl<P> PersistentHandle<P>
where
    P: PersistentTask,
{
    pub(crate) fn new(sender: Arc<dyn ChannelSender<PersistentMessage<P>>>) -> Self {
        PersistentHandle { sender }
    }

    /// Call the persistent task with the provided input.
    ///
    /// This method waits until there's room available in the channel. In addition to the input
    /// data, it also takes the sending half of a channel which is used to send the result back
    /// after the call has completed.
    pub async fn call<R>(&self, input: P::Input, sender: R) -> JlrsResult<()>
    where
        R: OneshotSender<JlrsResult<P::Output>>,
    {
        self.sender
            .send(PersistentMessage {
                msg: Box::new(CallPersistentTask {
                    input: Some(input),
                    sender,
                    _marker: PhantomData,
                }),
            })
            .await
            .map_err(|_| RuntimeError::ChannelClosed)?;

        Ok(())
    }

    /// Try to call the persistent task with the provided input.
    ///
    /// If there's no room in the backing channel an error is returned immediately. In addition to
    /// the input data, it also takes the sending half of a channel which is used to send the
    /// result back after the call has completed.
    pub fn try_call<R>(&self, input: P::Input, sender: R) -> JlrsResult<()>
    where
        R: OneshotSender<JlrsResult<P::Output>>,
    {
        self.sender
            .try_send(PersistentMessage {
                msg: Box::new(CallPersistentTask {
                    input: Some(input),
                    sender,
                    _marker: PhantomData,
                }),
            })
            .map_err(|e| match e {
                TrySendError::Full(_) => RuntimeError::ChannelFull,
                TrySendError::Closed(_) => RuntimeError::ChannelClosed,
            })?;

        Ok(())
    }
}

pub trait RequireSendSync: 'static + Send {}

// Ensure the handle can be shared across threads
impl<P: PersistentTask> RequireSendSync for PersistentHandle<P> {}
