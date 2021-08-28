//! Run Julia in a separate thread and execute tasks in parallel.
//!
//! While the Julia C API can only be used from a single thread, it is possible to schedule
//! multiple [`Task`]s to run in parallel. This doesn't work nicely with the sync runtime provided
//! by jlrs's [`Julia`] struct because the garbage collector is unable to run and async events in
//! Julia aren't handled. The async runtime initializes Julia in a new thread and returns a
//! handle, [`AsyncJulia`], that can be cloned and shared across threads; async events in Julia
//! are handled periodically, which also allows the garbage collector to run.
//!
//! In order to use the async runtime Julia must be started with three or more threads by setting
//! the `JULIA_NUM_THREADS` environment variable. If this environment variable is not set it's set
//! to `auto`, which starts Julia with as many threads as the CPU supports.
//!
//! The easiest way to interact with the async runtime is sending a blocking task. A blocking task
//! is executed on the main thread as soon as it's received. Any closure that takes a [`Global`]
//! and a mutable reference to a [`GcFrame`], returns a [`JlrsResult`], and implements `Send` and
//! `Sync` is a valid blocking task. This is essentially the same interface as [`Julia::scope`]
//! provides, the main difference is that the requirements are a bit more strict because the
//! closure and return type must implement `Send` and `Sync`. In order to send the result back, a
//! channel must be provided whenever a new task sent to the runtime. The sending half of this
//! channel must implement the [`ReturnChannel`] trait, by default this trait is implemented for
//! the `Sender`s from async-std and crossbeam-channel, the empty tuple `()` can be used if you're
//! not interested in the result and the task returns `()` if successful.
//!
//! Because blocking tasks are essentially equivalent to using [`Julia::scope`], using blocking
//! threads to schedule new `Task`s doesn't work. It's also not possible to have multiple blocking
//! tasks running at the same time because they block the main thread.
//!
//! In order to write a non-blocking task the [`AsyncTask`] trait must be implemented. This is an
//! async trait with two async methods, `register` and `run`. Only `run` has to be implemented, it
//! takes a mutable reference to an [`AsyncGcFrame`] rather than a `GcFrame`, the major difference
//! between these two frame types is that `AsyncGcFrame` can be used to call the methods provided
//! by the [`CallAsync`] trait.
//!
//! The `CallAsync` trait extends [`Call`]. Its methods let you schedule a Julia function call as
//! a new `Task`, either by using `Base.Thread.@spawn` or `@async` internally. Note that tasks are
//! never scheduled on the main thread, even if `@async` is used this happens on another thread to
//! ensure the main thread isn't blocked. A sync and async variation of each method is available,
//! the async method resolves when the function call has completed. While it's `await`ed the async
//! runtime can handle other tasks. The sync variants simply schedule the function call and return
//! the `Task`.
//!
//! Like a blocking task, an `AsyncTask` runs once and eventually sends back its result through a
//! provided channel. In many cases it's more useful to set up some initial state and interact
//! with this task. For this purpose the [`GeneratorTask`] trait can be implemented. It has three
//! async methods: `register`, `init`, and `run`; both `init` and `run` must be implemented. When
//! a `GeneratorTask` starts executing `init` is called, which returns the initial state of the
//! generator. Because the frame provided to this method isn't dropped after it has completed, the
//! initial state can contain Julia data rooted in that frame. After `init` has completed a handle
//! to the generator is returned. This handle can be used to call the generator's `run` method.
//! This method is similar to [`AsyncTask::run`], except that it's also provided with a mutable
//! reference to the generator's state and the additional data that must be provided when calling
//! the generator using its handle.
//!
//! Examples that show how to use the async runtime and implement tasks can be found in the
//! [`examples`] directory of the repository.
//!
//! [`examples`]: https://github.com/Taaitaaiger/jlrs/tree/master/examples
//! [`Julia`]: crate::Julia
//! [`Julia::scope`]: crate::Julia::scope
//! [`AsyncGcFrame`]: crate::extensions::multitask::async_frame::AsyncGcFrame
//! [`CallAsync`]: crate::extensions::multitask::call_async::CallAsync
//! [`Task`]: crate::wrappers::ptr::task::Task

pub mod async_frame;
pub mod async_task;
pub mod call_async;
pub(crate) mod julia_future;
pub mod mode;
pub(crate) mod output_result_ext;
pub mod return_channel;

use self::async_task::GenericBlockingTask;
use self::mode::Async;
use self::return_channel::ReturnChannel;
use crate::error::{JlrsError, JlrsResult};
use crate::extensions::multitask::async_task::{
    AsyncTask, BlockingTask, Generator, GeneratorHandle, GeneratorTask, GenericPendingTask,
    PendingTask, RegisterGenerator, RegisterTask, Task,
};
use crate::info::Info;
use crate::memory::global::Global;
use crate::memory::stack_page::StackPage;
use crate::wrappers::ptr::module::Module;
use crate::wrappers::ptr::string::JuliaString;
use crate::wrappers::ptr::value::Value;
use crate::{memory::frame::GcFrame, wrappers::ptr::call::Call};
use crate::{INIT, JLRS_JL};
use async_std::channel::{
    bounded, unbounded, Receiver as AsyncStdReceiver, RecvError, Sender as AsyncStdSender,
    TrySendError,
};
use async_std::future::timeout;
use async_std::sync::{Condvar as AsyncStdCondvar, Mutex as AsyncStdMutex};
use async_std::task::{self, JoinHandle as AsyncStdHandle};
use jl_sys::{
    jl_atexit_hook, jl_eval_string, jl_get_ptls_states, jl_init, jl_init_with_image,
    jl_is_initialized, jl_process_events,
};
use std::cell::Cell;
use std::io::{Error as IOError, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle as ThreadHandle};
use std::time::Duration;
use std::{
    collections::VecDeque,
    env,
    ffi::{c_void, CString},
    ptr::null_mut,
};

/// A handle to the async runtime. It can be used to include files and send new tasks. The
/// runtime shuts down when the last handle is dropped.
///
/// All initialization methods share three arguments:
///
///  - `max_n_tasks`: the maximum number of tasks that can run at the same time.
///  - `channel_capacity`: the capacity of the channel used to communicate with the runtime. If it's 0
///    an unbounded channel is used.
///  - `recv_timeout`: timeout used when receiving messages on the communication channel. If no
///    new message is received before the timeout and tasks are running, events are processed.
#[derive(Clone)]
pub struct AsyncJulia {
    sender: AsyncStdSender<Message>,
}

// Ensure AsyncJulia can be sent to other threads.
trait RequireSendSync: 'static + Send + Sync {}
impl RequireSendSync for AsyncJulia {}

impl AsyncJulia {
    /// Initialize Julia in a new thread.
    ///
    /// This function returns an error if the `JULIA_NUM_THREADS` environment variable is set
    /// to a value smaller than 3 or some invalid value, or if Julia has already been initialized.
    /// If the environment variable is not set it's set to `auto`. See the [struct-level]
    /// documentation for more info about this method's arguments.
    ///
    /// Safety: this method can race with other crates that try to initialize Julia at the same
    /// time.
    ///
    /// [struct-level]: crate::extensions::multitask::AsyncJulia
    pub unsafe fn init(
        max_n_tasks: usize,
        channel_capacity: usize,
        recv_timeout: Duration,
    ) -> JlrsResult<(Self, ThreadHandle<JlrsResult<()>>)> {
        check_threads_var()?;

        let (sender, receiver) = channel(channel_capacity);
        let julia = AsyncJulia { sender };
        let handle = thread::spawn(move || run_async(max_n_tasks, recv_timeout, receiver));
        julia.try_set_custom_fns()?;

        Ok((julia, handle))
    }

    /// Initialize Julia as a blocking task.
    ///
    /// This function returns an error if the `JULIA_NUM_THREADS` environment variable is set
    /// to a value smaller than 3 or some invalid value, or if Julia has already been initialized.
    /// If the environment variable is not set it's set to `auto`. See the [struct-level]
    /// documentation for more info about this method's arguments.
    ///
    /// Safety: this method can race with other crates that try to initialize Julia at the same
    /// time.
    ///
    /// [struct-level]: crate::extensions::multitask::AsyncJulia
    pub async unsafe fn init_async(
        max_n_tasks: usize,
        channel_capacity: usize,
        recv_timeout: Duration,
    ) -> JlrsResult<(Self, AsyncStdHandle<JlrsResult<()>>)> {
        check_threads_var()?;

        let (sender, receiver) = channel(channel_capacity);
        let julia = AsyncJulia { sender };
        let handle = task::spawn_blocking(move || run_async(max_n_tasks, recv_timeout, receiver));
        julia.set_custom_fns().await?;

        Ok((julia, handle))
    }

    /// This function is similar to [`AsyncJulia::init`] except that it loads a custom system
    /// image. A custom image can be generated with the [`PackageCompiler`] package for Julia. The
    /// main advantage of using a custom image over the default one is that it allows you to avoid
    /// much of the compilation overhead often associated with Julia.
    ///
    /// In addition to the common arguments to initialize the async runtime, you need to provide
    /// `julia_bindir` and `image_path`. The first must be the absolute path to a directory that
    /// contains a compatible Julia binary (eg `${JULIA_DIR}/bin`), the second must be either an
    /// absolute or a relative path to a system image.
    ///
    /// This function returns an error if either of the two paths doesn't exist, if the
    /// `JULIA_NUM_THREADS` environment variable is set to a value smaller than 3 or some invalid
    /// value, or if Julia has already been initialized. If the environment variable is not set
    /// it's set to `auto`. See the [struct-level] documentation for more info about this
    /// method's arguments.
    ///
    /// Safety: this method can race with other crates that try to initialize Julia at the same
    /// time.
    ///
    /// [`PackageCompiler`]: https://julialang.github.io/PackageCompiler.jl/dev/
    pub unsafe fn init_with_image<P, Q>(
        max_n_tasks: usize,
        channel_capacity: usize,
        recv_timeout: Duration,
        julia_bindir: P,
        image_path: Q,
    ) -> JlrsResult<(Self, ThreadHandle<JlrsResult<()>>)>
    where
        P: AsRef<Path> + Send + 'static,
        Q: AsRef<Path> + Send + 'static,
    {
        check_threads_var()?;

        let (sender, receiver) = channel(channel_capacity);
        let julia = AsyncJulia { sender };
        let handle = thread::spawn(move || {
            run_async_with_image(
                max_n_tasks,
                recv_timeout,
                receiver,
                julia_bindir,
                image_path,
            )
        });
        julia.try_set_custom_fns()?;

        Ok((julia, handle))
    }

    /// This function is similar to [`AsyncJulia::init_async`] except that it loads a custom
    /// system image. A custom image can be generated with the [`PackageCompiler`] package for
    /// Julia. The main advantage of using a custom image over the default one is that it allows
    /// you to avoid much of the compilation overhead often associated with Julia.
    ///
    /// In addition to the common arguments to initialize the async runtime, you need to provide
    /// `julia_bindir` and `image_path`. The first must be the absolute path to a directory that
    /// contains a compatible Julia binary (eg `${JULIA_DIR}/bin`), the second must be either an
    /// absolute or a relative path to a system image.
    ///
    /// This function returns an error if either of the two paths doesn't exist, if the
    /// `JULIA_NUM_THREADS` environment variable is set to a value smaller than 3 or some invalid
    /// value, or if Julia has already been initialized. If the environment variable is not set
    /// it's set to `auto`. See the [struct-level] documentation for more info about this
    /// method's arguments.
    ///
    /// Safety: this method can race with other crates that try to initialize Julia at the same
    /// time.
    ///
    /// [`PackageCompiler`]: https://julialang.github.io/PackageCompiler.jl/dev/
    pub async unsafe fn init_with_image_async<P, Q>(
        max_n_tasks: usize,
        channel_capacity: usize,
        recv_timeout: Duration,
        julia_bindir: P,
        image_path: Q,
    ) -> JlrsResult<(Self, AsyncStdHandle<JlrsResult<()>>)>
    where
        P: AsRef<Path> + Send + 'static,
        Q: AsRef<Path> + Send + 'static,
    {
        check_threads_var()?;

        let (sender, receiver) = channel(channel_capacity);
        let julia = AsyncJulia { sender };
        let handle = task::spawn_blocking(move || {
            run_async_with_image(
                max_n_tasks,
                recv_timeout,
                receiver,
                julia_bindir,
                image_path,
            )
        });
        julia.set_custom_fns().await?;

        Ok((julia, handle))
    }

    /// Send a new task to the runtime, this method waits if there's no room in the channel. This
    /// method takes two arguments, the task and the sending half of a channel which is used to
    /// send the result back after the task has completed.
    pub async fn task<AT, R>(&self, task: AT, res_sender: R)
    where
        AT: AsyncTask,
        R: ReturnChannel<Ok = AT::Output>,
    {
        let sender = self.sender.clone();
        let msg = PendingTask::<_, _, Task>::new(task, res_sender);
        let boxed = Box::new(msg);
        self.sender
            .send(Message::Task(boxed, sender))
            .await
            .expect("Channel was closed");
    }

    /// Try to send a new task to the runtime, if there's no room in the channel an error is
    /// returned immediately. This method takes two arguments, the task and the sending half of a
    /// channel which is used to send the result back after the task has completed.
    pub fn try_task<AT, R>(&self, task: AT, res_sender: R) -> JlrsResult<()>
    where
        AT: AsyncTask,
        R: ReturnChannel<Ok = AT::Output>,
    {
        let sender = self.sender.clone();
        let msg = PendingTask::<_, _, Task>::new(task, res_sender);
        let boxed = Box::new(msg);
        match self.sender.try_send(Message::Task(boxed, sender)) {
            Ok(_) => Ok(()),
            Err(TrySendError::Full(_)) => Err(Box::new(JlrsError::other(TrySendError::Full(()))))?,
            Err(_) => Err(Box::new(JlrsError::other(TrySendError::Closed(()))))?,
        }
    }

    /// Sends a blocking task to the async runtime, if there's no room in the channel an error is
    /// returned immediately. A blocking task is a closure that takes two arguments, a `Global`
    /// and mutable reference to a `GcFrame`, and must return a `JlrsResult` whose inner type is
    /// both `Send` and `Sync`. This task is executed as soon as possible and can't call async
    /// methods, so it block the runtime. The result of the closure is sent to `res_sender`.
    pub async fn blocking_task<T, R, F>(&self, task: F, res_sender: R)
    where
        for<'base> F: 'static
            + Send
            + Sync
            + FnOnce(Global<'base>, &mut GcFrame<'base, Async<'base>>) -> JlrsResult<T>,
        R: ReturnChannel<Ok = T>,
        T: Send + Sync + 'static,
    {
        let msg = BlockingTask::new(task, res_sender);
        let boxed = Box::new(msg);
        self.sender
            .send(Message::BlockingTask(boxed))
            .await
            .expect("Channel was closed");
    }

    /// Sends a blocking task to the async runtime, this method waits if there's no room in the
    /// channel. A blocking task is a closure that takes two arguments, a `Global` and mutable
    /// reference to a `GcFrame`, and must return a `JlrsResult` whose inner type is both `Send`
    /// and `Sync`. This task is executed as soon as possible and can't call async methods, so it
    /// block the runtime. The result of the closure is sent to `res_sender`.
    pub fn try_blocking_task<T, R, F>(&self, task: F, res_sender: R) -> JlrsResult<()>
    where
        for<'base> F: 'static
            + Send
            + Sync
            + FnOnce(Global<'base>, &mut GcFrame<'base, Async<'base>>) -> JlrsResult<T>,
        R: ReturnChannel<Ok = T>,
        T: Send + Sync + 'static,
    {
        let msg = BlockingTask::new(task, res_sender);
        let boxed = Box::new(msg);
        match self.sender.try_send(Message::BlockingTask(boxed)) {
            Ok(_) => Ok(()),
            Err(TrySendError::Full(_)) => Err(Box::new(JlrsError::other(TrySendError::Full(()))))?,
            Err(_) => Err(Box::new(JlrsError::other(TrySendError::Closed(()))))?,
        }
    }

    /// Register a task, this method waits if there's no room in the channel. This method takes
    /// one argument, the sending half of a channel which is used to send the result back after
    /// the registration has completed.
    pub async fn register_task<AT, R>(&self, res_sender: R)
    where
        AT: AsyncTask,
        R: ReturnChannel<Ok = ()>,
    {
        let sender = self.sender.clone();
        let msg = PendingTask::<_, AT, RegisterTask>::new(res_sender);
        let boxed = Box::new(msg);
        self.sender
            .send(Message::Task(boxed, sender))
            .await
            .expect("Channel was closed");
    }

    /// Try to register a task, if there's no room in the channel an error is returned
    /// immediately. This method takes one argument, the sending half of a channel which is used
    /// to send the result back after the registration has completed.
    pub fn try_register_task<AT, R>(&self, res_sender: R) -> JlrsResult<()>
    where
        AT: AsyncTask,
        R: ReturnChannel<Ok = ()>,
    {
        let sender = self.sender.clone();
        let msg = PendingTask::<_, AT, RegisterTask>::new(res_sender);
        let boxed = Box::new(msg);
        match self.sender.try_send(Message::Task(boxed, sender)) {
            Ok(_) => (),
            Err(TrySendError::Full(_)) => Err(Box::new(JlrsError::other(TrySendError::Full(()))))?,
            Err(_) => Err(Box::new(JlrsError::other(TrySendError::Closed(()))))?,
        }

        Ok(())
    }

    /// Send a new generator to the runtime, this method waits if there's no room in the channel.
    /// This method takes a single argument, the generator. It returns after
    /// [`GeneratorTask::init`] has been called, a [`GeneratorHandle`] to the generator is
    /// returned if this method completes successfully, and the error if it doesn't.
    pub async fn generator<GT>(&self, task: GT) -> JlrsResult<GeneratorHandle<GT>>
    where
        GT: GeneratorTask,
    {
        let rt_sender = self.sender.clone();
        let (init_sender, init_recv) = async_std::channel::bounded(1);
        let msg = PendingTask::<_, _, Generator>::new(task, init_sender);
        let boxed = Box::new(msg);

        self.sender
            .send(Message::Task(boxed, rt_sender))
            .await
            .expect("Channel was closed");

        match init_recv.recv().await {
            Ok(Ok(gh)) => Ok(gh),
            Ok(Err(e)) => Err(e)?,
            Err(e) => Err(JlrsError::other(e))?,
        }
    }

    /// Send a new generator to the runtime, if there's no room in the channel an error is
    /// returned immediately. This method takes a single argument, the generator. It returns after
    /// [`GeneratorTask::init`] has been called, a [`GeneratorHandle`] to the generator is
    /// returned if this method completes successfully, and the error if it doesn't.
    pub fn try_generator<GT>(&self, task: GT) -> JlrsResult<GeneratorHandle<GT>>
    where
        GT: GeneratorTask,
    {
        let sender = self.sender.clone();
        let (init_sender, init_recv) = crossbeam_channel::bounded(1);
        let msg = PendingTask::<_, _, Generator>::new(task, init_sender);
        let boxed = Box::new(msg);
        match self.sender.try_send(Message::Task(boxed, sender)) {
            Ok(_) => (),
            Err(TrySendError::Full(_)) => Err(Box::new(JlrsError::other(TrySendError::Full(()))))?,
            Err(_) => Err(Box::new(JlrsError::other(TrySendError::Closed(()))))?,
        }

        match init_recv.recv() {
            Ok(Ok(gh)) => Ok(gh),
            Ok(Err(e)) => Err(e)?,
            Err(e) => Err(JlrsError::other(e))?,
        }
    }

    /// Register a generator, this method waits if there's no room in the channel. This method
    /// takes one argument, the sending half of a channel which is used to send the result back
    /// after the registration has completed.
    pub async fn register_generator<GT, R>(&self, res_sender: R)
    where
        GT: GeneratorTask,
        R: ReturnChannel<Ok = ()>,
    {
        let sender = self.sender.clone();
        let msg = PendingTask::<_, GT, RegisterGenerator>::new(res_sender);
        let boxed = Box::new(msg);
        self.sender
            .send(Message::Task(boxed, sender))
            .await
            .expect("Channel was closed");
    }

    /// Try to register a generator, if there's no room in the channel an error is returned
    /// immediately. This method takes one argument, the sending half of a channel which is used
    /// to send the result back after the registration has completed.
    pub fn try_register_generator<GT, R>(&self, res_sender: R) -> JlrsResult<()>
    where
        GT: GeneratorTask,
        R: ReturnChannel<Ok = ()>,
    {
        let sender = self.sender.clone();
        let msg = PendingTask::<_, GT, RegisterGenerator>::new(res_sender);
        let boxed = Box::new(msg);
        match self.sender.try_send(Message::Task(boxed, sender)) {
            Ok(_) => (),
            Err(TrySendError::Full(_)) => Err(Box::new(JlrsError::other(TrySendError::Full(()))))?,
            Err(_) => Err(Box::new(JlrsError::other(TrySendError::Closed(()))))?,
        }

        Ok(())
    }

    /// Include a Julia file. This method waits until the call to `Main.include` in Julia has been
    /// completed. It returns an error if the path doesn't exist or the call to `Main.include`
    /// throws an exception.
    pub async fn include<P: AsRef<Path>>(&self, path: P) -> JlrsResult<()> {
        if !path.as_ref().exists() {
            Err(JlrsError::IncludeNotFound {
                path: path.as_ref().to_string_lossy().into(),
            })?
        }

        let completed = Arc::new((AsyncStdMutex::new(Status::Pending), AsyncStdCondvar::new()));
        self.sender
            .send(Message::Include(
                path.as_ref().to_path_buf(),
                completed.clone(),
            ))
            .await
            .expect("Channel was closed");

        let (lock, cvar) = &*completed;
        let mut completed = lock.lock().await;
        while (&*completed).is_pending() {
            completed = cvar.wait(completed).await;
        }

        (&mut *completed).as_jlrs_result()
    }

    /// Include a Julia file. This method waits until the call `Main.include` in Julia has been
    /// completed. It returns an error if the path doesn't  exist, the channel is full, or the
    /// call to `Main.include` throws an exception.
    pub fn try_include<P: AsRef<Path>>(&self, path: P) -> JlrsResult<()> {
        if !path.as_ref().exists() {
            Err(JlrsError::IncludeNotFound {
                path: path.as_ref().to_string_lossy().into(),
            })?
        }

        let completed = Arc::new((Mutex::new(Status::Pending), Condvar::new()));
        self.sender
            .try_send(Message::TryInclude(
                path.as_ref().to_path_buf(),
                completed.clone(),
            ))
            .map_err(|e| match e {
                TrySendError::Full(_) => Box::new(JlrsError::other(TrySendError::Full(()))),
                TrySendError::Closed(_) => Box::new(JlrsError::other(TrySendError::Closed(()))),
            })
            .and_then(|_| {
                let (lock, cvar) = &*completed;
                let mut completed = lock.lock().unwrap();
                while (&*completed).is_pending() {
                    completed = cvar.wait(completed).unwrap();
                }
                (&mut *completed).as_jlrs_result()
            })
    }

    /// Enable or disable colored error messages originating from Julia. If this is enabled the
    /// error message in [`JlrsError::Exception`] can contain ANSI color codes. This feature is
    /// disabled by default.
    pub async fn error_color(&self, enable: bool) -> JlrsResult<()> {
        let completed = Arc::new((AsyncStdMutex::new(Status::Pending), AsyncStdCondvar::new()));
        self.sender
            .send(Message::ErrorColor(enable, completed.clone()))
            .await
            .expect("Channel was closed");

        let (lock, cvar) = &*completed;
        let mut completed = lock.lock().await;
        while (&*completed).is_pending() {
            completed = cvar.wait(completed).await;
        }

        (&mut *completed).as_jlrs_result()
    }

    /// Enable or disable colored error messages originating from Julia. If this is enabled the
    /// error message in [`JlrsError::Exception`] can contain ANSI color codes. This feature is
    /// disabled by default.
    pub fn try_error_color(&self, enable: bool) -> JlrsResult<()> {
        let completed = Arc::new((Mutex::new(Status::Pending), Condvar::new()));
        self.sender
            .try_send(Message::TryErrorColor(enable, completed.clone()))
            .map_err(|e| match e {
                TrySendError::Full(_) => Box::new(JlrsError::other(TrySendError::Full(()))),
                TrySendError::Closed(_) => Box::new(JlrsError::other(TrySendError::Closed(()))),
            })
            .and_then(|_| {
                let (lock, cvar) = &*completed;
                let mut completed = lock.lock().unwrap();
                while (&*completed).is_pending() {
                    completed = cvar.wait(completed).unwrap();
                }
                (&mut *completed).as_jlrs_result()
            })
    }

    /// Returns the capacity of the channel.
    pub fn capacity(&self) -> usize {
        self.sender.capacity().unwrap()
    }

    /// Returns the number of messages in the channel.
    pub fn len(&self) -> usize {
        self.sender.len()
    }

    /// Returns `true` if the channel is empty.
    pub fn is_empty(&self) -> bool {
        self.sender.is_empty()
    }

    /// Returns `true` if the channel is full.
    pub fn is_full(&self) -> bool {
        self.sender.is_full()
    }

    fn try_set_custom_fns(&self) -> JlrsResult<()> {
        let completed = Arc::new((Mutex::new(Status::Pending), Condvar::new()));
        self.sender
            .try_send(Message::TrySetCustomFns(completed.clone()))
            .map_err(|e| match e {
                TrySendError::Full(_) => Box::new(JlrsError::other(TrySendError::Full(()))),
                TrySendError::Closed(_) => Box::new(JlrsError::other(TrySendError::Closed(()))),
            })
            .and_then(|_| {
                let (lock, cvar) = &*completed;
                let mut completed = lock.lock().unwrap();
                while (&*completed).is_pending() {
                    completed = cvar.wait(completed).unwrap();
                }
                (&mut *completed).as_jlrs_result()
            })
    }

    async fn set_custom_fns(&self) -> JlrsResult<()> {
        let completed = Arc::new((AsyncStdMutex::new(Status::Pending), AsyncStdCondvar::new()));
        self.sender
            .send(Message::SetCustomFns(completed.clone()))
            .await
            .expect("Channel was closed");

        {
            let (lock, cvar) = &*completed;
            let mut completed = lock.lock().await;
            while (&*completed).is_pending() {
                completed = cvar.wait(completed).await;
            }

            (&mut *completed).as_jlrs_result()
        }
    }
}

fn channel(channel_capacity: usize) -> (AsyncStdSender<Message>, AsyncStdReceiver<Message>) {
    if channel_capacity == 0 {
        unbounded()
    } else {
        bounded(channel_capacity)
    }
}

pub(crate) enum Status {
    Pending,
    Ok,
    Err(Option<Box<JlrsError>>),
}

impl Status {
    fn is_pending(&self) -> bool {
        match self {
            Status::Pending => true,
            _ => false,
        }
    }

    fn as_jlrs_result(&mut self) -> JlrsResult<()> {
        match self {
            Status::Ok => Ok(()),
            Status::Err(ref mut e) => Err(e.take().expect("Status is Err, but no error is set")),
            Status::Pending => panic!("Cannot convert Status::Pending to JlrsResult"),
        }
    }
}

pub(crate) enum Message {
    Task(Box<dyn GenericPendingTask>, AsyncStdSender<Message>),
    BlockingTask(Box<dyn GenericBlockingTask>),
    Include(PathBuf, Arc<(AsyncStdMutex<Status>, AsyncStdCondvar)>),
    TryInclude(PathBuf, Arc<(Mutex<Status>, Condvar)>),
    ErrorColor(bool, Arc<(AsyncStdMutex<Status>, AsyncStdCondvar)>),
    TryErrorColor(bool, Arc<(Mutex<Status>, Condvar)>),
    Complete(usize, Box<AsyncStackPage>),
    SetCustomFns(Arc<(AsyncStdMutex<Status>, AsyncStdCondvar)>),
    TrySetCustomFns(Arc<(Mutex<Status>, Condvar)>),
}

#[derive(Debug)]
pub(crate) struct AsyncStackPage {
    top: [Cell<*mut c_void>; 2],
    page: StackPage,
}

// Not actually true, but we need to be able to send a page back after completing a task. The page
//is (and must) never shared across threads.
unsafe impl Send for AsyncStackPage {}
unsafe impl Sync for AsyncStackPage {}

impl AsyncStackPage {
    unsafe fn new() -> Box<Self> {
        let stack = AsyncStackPage {
            top: [Cell::new(null_mut()), Cell::new(null_mut())],
            page: StackPage::default(),
        };

        Box::new(stack)
    }
}

unsafe fn link_stacks(stacks: &mut [Option<Box<AsyncStackPage>>]) {
    for stack in stacks.iter_mut() {
        let stack = stack.as_mut().unwrap();
        let rtls = &mut *jl_get_ptls_states();
        stack.top[1].set(rtls.pgcstack.cast());
        rtls.pgcstack = stack.top[0..1].as_mut_ptr().cast();
    }
}

fn run_async(
    max_n_tasks: usize,
    recv_timeout: Duration,
    receiver: AsyncStdReceiver<Message>,
) -> JlrsResult<()> {
    task::block_on(async {
        unsafe {
            if jl_is_initialized() != 0 || INIT.swap(true, Ordering::SeqCst) {
                return Err(JlrsError::AlreadyInitialized.into());
            }

            jl_init();
            if Info::new().n_threads() < 3 {
                Err(JlrsError::MoreThreadsRequired)?;
            }

            let jlrs_jl = CString::new(JLRS_JL).expect("Invalid Jlrs module");
            jl_eval_string(jlrs_jl.as_ptr());

            let mut free_stacks = VecDeque::with_capacity(max_n_tasks);
            for i in 1..max_n_tasks {
                free_stacks.push_back(i);
            }

            let mut stacks = {
                let mut stacks = Vec::with_capacity(max_n_tasks);
                for _ in 0..max_n_tasks {
                    stacks.push(Some(AsyncStackPage::new()));
                }
                link_stacks(&mut stacks);
                stacks.into_boxed_slice()
            };

            let mut running_tasks = Vec::with_capacity(max_n_tasks);
            for _ in 0..max_n_tasks {
                running_tasks.push(None);
            }
            let mut running_tasks = running_tasks.into_boxed_slice();
            let mut pending_tasks = VecDeque::new();

            let mut n_running = 0;

            loop {
                let wait_time = if n_running > 0 {
                    recv_timeout
                } else {
                    Duration::from_secs(2 << 32)
                };

                match timeout(wait_time, receiver.recv()).await {
                    Err(_) => {
                        jl_process_events();
                    }
                    Ok(Ok(Message::Task(task, sender))) => {
                        if let Some(idx) = free_stacks.pop_front() {
                            let stack = stacks[idx].take().expect("Async stack corrupted");
                            let task = task::spawn_local(task.call(idx, stack, sender));
                            n_running += 1;
                            running_tasks[idx] = Some(task);
                        } else {
                            pending_tasks.push_back((task, sender));
                        }
                    }
                    Ok(Ok(Message::Complete(idx, stack))) => {
                        if let Some((jl_task, sender)) = pending_tasks.pop_front() {
                            let task = task::spawn_local(jl_task.call(idx, stack, sender));
                            running_tasks[idx] = Some(task);
                        } else {
                            stacks[idx] = Some(stack);
                            free_stacks.push_front(idx);
                            n_running -= 1;
                            running_tasks[idx] = None;
                        }
                    }
                    Ok(Ok(Message::Include(path, completed))) => {
                        let stack = stacks[0].as_mut().expect("Async stack corrupted");
                        include(stack, path, completed).await
                    }
                    Ok(Ok(Message::TryInclude(path, completed))) => {
                        let stack = stacks[0].as_mut().expect("Async stack corrupted");
                        try_include(stack, path, completed)
                    }
                    Ok(Ok(Message::BlockingTask(task))) => {
                        let stack = stacks[0].as_mut().expect("Async stack corrupted");
                        task.call(stack)
                    }
                    Ok(Ok(Message::ErrorColor(enable, completed))) => {
                        error_color(enable, completed).await
                    }
                    Ok(Ok(Message::TryErrorColor(enable, completed))) => {
                        try_error_color(enable, completed)
                    }
                    Ok(Ok(Message::SetCustomFns(completed))) => {
                        let stack = stacks[0].as_mut().expect("Async stack corrupted");
                        set_custom_fns(stack, completed).await;
                    }
                    Ok(Ok(Message::TrySetCustomFns(completed))) => {
                        let stack = stacks[0].as_mut().expect("Async stack corrupted");
                        try_set_custom_fns(stack, completed)
                    }
                    Ok(Err(RecvError)) => break,
                }
            }

            for running in running_tasks.iter_mut() {
                if let Some(handle) = running.take() {
                    handle.await;
                }
            }

            jl_atexit_hook(0);
        }

        Ok(())
    })
}

fn run_async_with_image<P, Q>(
    max_n_tasks: usize,
    recv_timeout: Duration,
    receiver: AsyncStdReceiver<Message>,
    julia_bindir: P,
    image_path: Q,
) -> JlrsResult<()>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    task::block_on(async {
        unsafe {
            if jl_is_initialized() != 0 || INIT.swap(true, Ordering::SeqCst) {
                return Err(JlrsError::AlreadyInitialized.into());
            }

            let julia_bindir_str = julia_bindir.as_ref().to_string_lossy().to_string();
            let image_path_str = image_path.as_ref().to_string_lossy().to_string();

            if !julia_bindir.as_ref().exists() {
                let io_err = IOError::new(ErrorKind::NotFound, julia_bindir_str);
                return Err(JlrsError::other(io_err))?;
            }

            if !image_path.as_ref().exists() {
                let io_err = IOError::new(ErrorKind::NotFound, image_path_str);
                return Err(JlrsError::other(io_err))?;
            }

            let bindir = std::ffi::CString::new(julia_bindir_str).unwrap();
            let im_rel_path = std::ffi::CString::new(image_path_str).unwrap();

            jl_init_with_image(bindir.as_ptr(), im_rel_path.as_ptr());
            if Info::new().n_threads() < 3 {
                Err(JlrsError::MoreThreadsRequired)?;
            }

            let jlrs_jl = CString::new(JLRS_JL).expect("Invalid Jlrs module");
            jl_eval_string(jlrs_jl.as_ptr());

            let mut free_stacks = VecDeque::with_capacity(max_n_tasks);
            for i in 1..max_n_tasks {
                free_stacks.push_back(i);
            }

            let mut stacks = {
                let mut stacks = Vec::with_capacity(max_n_tasks);
                for _ in 0..max_n_tasks {
                    stacks.push(Some(AsyncStackPage::new()));
                }
                link_stacks(&mut stacks);
                stacks.into_boxed_slice()
            };

            let mut running_tasks = Vec::with_capacity(max_n_tasks);
            for _ in 0..max_n_tasks {
                running_tasks.push(None);
            }
            let mut running_tasks = running_tasks.into_boxed_slice();
            let mut pending_tasks = VecDeque::new();

            let mut n_running = 0usize;

            loop {
                let wait_time = if n_running > 0 {
                    recv_timeout
                } else {
                    Duration::from_secs(u64::MAX)
                };

                match timeout(wait_time, receiver.recv()).await {
                    Err(_) => {
                        jl_process_events();
                    }
                    Ok(Ok(Message::Task(task, sender))) => {
                        if let Some(idx) = free_stacks.pop_front() {
                            let stack = stacks[idx].take().expect("Async stack corrupted");
                            let task = task::spawn_local(task.call(idx, stack, sender));
                            n_running += 1;
                            running_tasks[idx] = Some(task);
                        } else {
                            pending_tasks.push_back((task, sender));
                        }
                    }
                    Ok(Ok(Message::Complete(idx, stack))) => {
                        if let Some((jl_task, sender)) = pending_tasks.pop_front() {
                            let task = task::spawn_local(jl_task.call(idx, stack, sender));
                            running_tasks[idx] = Some(task);
                        } else {
                            stacks[idx] = Some(stack);
                            n_running -= 1;
                            free_stacks.push_front(idx);
                            running_tasks[idx] = None;
                        }
                    }
                    Ok(Ok(Message::Include(path, completed))) => {
                        let stack = stacks[0].as_mut().expect("Async stack corrupted");
                        include(stack, path, completed).await
                    }
                    Ok(Ok(Message::TryInclude(path, completed))) => {
                        let stack = stacks[0].as_mut().expect("Async stack corrupted");
                        try_include(stack, path, completed)
                    }
                    Ok(Ok(Message::BlockingTask(task))) => {
                        let stack = stacks[0].as_mut().expect("Async stack corrupted");
                        task.call(stack)
                    }
                    Ok(Ok(Message::ErrorColor(enable, completed))) => {
                        error_color(enable, completed).await
                    }
                    Ok(Ok(Message::TryErrorColor(enable, completed))) => {
                        try_error_color(enable, completed)
                    }
                    Ok(Ok(Message::SetCustomFns(completed))) => {
                        let stack = stacks[0].as_mut().expect("Async stack corrupted");
                        set_custom_fns(stack, completed).await;
                    }
                    Ok(Ok(Message::TrySetCustomFns(completed))) => {
                        let stack = stacks[0].as_mut().expect("Async stack corrupted");
                        try_set_custom_fns(stack, completed);
                    }
                    Ok(Err(RecvError)) => break,
                }
            }

            for running in running_tasks.iter_mut() {
                if let Some(handle) = running.take() {
                    handle.await;
                }
            }

            jl_atexit_hook(0);
        }

        Ok(())
    })
}

fn call_include(stack: &mut AsyncStackPage, path: PathBuf) -> JlrsResult<()> {
    unsafe {
        let global = Global::new();
        let mode = Async(&stack.top[1]);
        let raw = stack.page.as_mut();
        let mut frame = GcFrame::new(raw, 1, mode);

        match path.to_str() {
            Some(path) => {
                let path = JuliaString::new(&mut frame, path)?;
                Module::main(global)
                    .function_ref("include")?
                    .wrapper_unchecked()
                    .call1_unrooted(global, path)
                    .map_err(|e| JlrsError::Exception {
                        msg: format!("Include error: {:?}", e.value_unchecked()),
                    })?;
            }
            None => {}
        }

        Ok(())
    }
}

async fn include(
    stack: &mut AsyncStackPage,
    path: PathBuf,
    completed: Arc<(AsyncStdMutex<Status>, AsyncStdCondvar)>,
) {
    let res = call_include(stack, path);
    {
        let (lock, condvar) = &*completed;
        let mut completed = lock.lock().await;
        if res.is_ok() {
            *completed = Status::Ok;
        } else {
            *completed = Status::Err(Some(res.unwrap_err()));
        }

        condvar.notify_one();
    }
}

fn try_include(
    stack: &mut AsyncStackPage,
    path: PathBuf,
    completed: Arc<(Mutex<Status>, Condvar)>,
) {
    let res = call_include(stack, path);

    {
        let (lock, condvar) = &*completed;
        let mut completed = lock.lock().expect("Cannot lock");
        if res.is_ok() {
            *completed = Status::Ok;
        } else {
            *completed = Status::Err(Some(res.unwrap_err()));
        }

        condvar.notify_one();
    }
}

fn call_error_color(enable: bool) -> JlrsResult<()> {
    unsafe {
        let global = Global::new();

        let enable = if enable {
            Value::true_v(global)
        } else {
            Value::false_v(global)
        };

        Module::main(global)
            .submodule_ref("Jlrs")?
            .wrapper_unchecked()
            .global_ref("color")?
            .value_unchecked()
            .set_field_unchecked("x", enable)?;

        Ok(())
    }
}

async fn error_color(enable: bool, completed: Arc<(AsyncStdMutex<Status>, AsyncStdCondvar)>) {
    let res = call_error_color(enable);
    {
        let (lock, condvar) = &*completed;
        let mut completed = lock.lock().await;
        if res.is_ok() {
            *completed = Status::Ok;
        } else {
            *completed = Status::Err(Some(res.unwrap_err()));
        }

        condvar.notify_one();
    }
}

fn try_error_color(enable: bool, completed: Arc<(Mutex<Status>, Condvar)>) {
    let res = call_error_color(enable);

    {
        let (lock, condvar) = &*completed;
        let mut completed = lock.lock().expect("Cannot lock");
        if res.is_ok() {
            *completed = Status::Ok;
        } else {
            *completed = Status::Err(Some(res.unwrap_err()));
        }

        condvar.notify_one();
    }
}

fn call_set_custom_fns(stack: &mut AsyncStackPage) -> JlrsResult<()> {
    unsafe {
        let global = Global::new();
        let mode = Async(&stack.top[1]);
        let raw = stack.page.as_mut();
        let mut frame = GcFrame::new(raw, 1, mode);

        let jlrs_mod = Module::main(global)
            .submodule_ref("Jlrs")?
            .wrapper_unchecked();

        let wake_rust = Value::new(&mut frame, julia_future::wake_task as *mut c_void)?;
        jlrs_mod
            .global_ref("wakerust")?
            .wrapper_unchecked()
            .set_nth_field_unchecked(0, wake_rust);

        Ok(())
    }
}

async fn set_custom_fns(
    stack: &mut AsyncStackPage,
    completed: Arc<(AsyncStdMutex<Status>, AsyncStdCondvar)>,
) {
    match call_set_custom_fns(stack) {
        Ok(()) => {
            let (lock, condvar) = &*completed;
            let mut completed = lock.lock().await;
            *completed = Status::Ok;
            condvar.notify_one();
        }
        Err(e) => {
            let (lock, condvar) = &*completed;
            let mut completed = lock.lock().await;
            *completed = Status::Err(Some(e));
            condvar.notify_one();
        }
    }
}

fn try_set_custom_fns(stack: &mut AsyncStackPage, completed: Arc<(Mutex<Status>, Condvar)>) {
    match call_set_custom_fns(stack) {
        Ok(_) => {
            let (lock, condvar) = &*completed;
            let mut completed = lock.lock().expect("Cannot lock");
            *completed = Status::Ok;
            condvar.notify_one();
        }
        Err(e) => {
            let (lock, condvar) = &*completed;
            let mut completed = lock.lock().expect("Cannot lock");
            *completed = Status::Err(Some(e));
            condvar.notify_one();
        }
    }
}

fn check_threads_var() -> JlrsResult<()> {
    match env::var("JULIA_NUM_THREADS") {
        Ok(n_threads) => {
            if n_threads != "auto" {
                let n_threads = n_threads
                    .parse::<usize>()
                    .map_err(|_| JlrsError::NumThreadsVar { value: n_threads })?;

                if n_threads < 3 {
                    Err(JlrsError::MoreThreadsRequired)?;
                }
            }
        }
        Err(_) => {
            env::set_var("JULIA_NUM_THREADS", "auto");
        }
    };

    Ok(())
}
