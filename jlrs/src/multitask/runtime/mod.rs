//! Async runtimes
//!
//! Two async runtimes are available in jls, one using on tokio and one using async-std. To use
//! one of these runtimes, either the `tokio-rt` or `async-std-rt` feature must be enabled. Only
//! one of these features can be enabled, which shouldn't be a problem because Julia can't be
//! initialized multiple times.

use std::fmt;
use std::{error::Error, marker::PhantomData};

#[cfg(feature = "async-std-rt")]
pub(crate) mod async_std_rt;

#[cfg(any(feature = "tokio-rt", feature = "docs"))]
pub(crate) mod tokio_rt;

#[derive(Debug)]
pub struct SendError<T>(pub T);

impl<T> fmt::Display for SendError<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "channel closed")
    }
}

impl<T: fmt::Debug> std::error::Error for SendError<T> {}

#[derive(Debug)]
pub enum TrySendError<T> {
    Full(T),
    Closed(T),
}

impl<T: fmt::Debug> Error for TrySendError<T> {}

impl<T> fmt::Display for TrySendError<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            "{}",
            match self {
                TrySendError::Full(..) => "no available capacity",
                TrySendError::Closed(..) => "channel closed",
            }
        )
    }
}

use crate::{
    error::{JlrsError, JlrsResult},
    info::Info,
    init_jlrs,
    memory::{frame::GcFrame, global::Global, stack_page::StackPage},
    multitask::{
        async_task::internal::{GenericBlockingTask, GenericPendingTask},
        async_task::PersistentTask,
        mode::Async,
        result_sender::ResultSender,
    },
    wrappers::ptr::Wrapper,
    wrappers::ptr::{call::Call, module::Module, string::JuliaString, value::Value},
    INIT,
};
use jl_sys::{jl_atexit_hook, jl_init, jl_init_with_image, jl_is_initialized, jl_process_events};
#[cfg(not(feature = "lts"))]
use jl_sys::{jl_get_current_task, jl_task_t};
use std::{
    collections::VecDeque,
    env,
    ffi::c_void,
    io::{Error as IOError, ErrorKind},
    path::{Path, PathBuf},
    ptr::null_mut,
    ptr::NonNull,
    sync::atomic::Ordering,
    {cell::Cell, pin::Pin},
};

// Ensure AsyncJulia can be sent to other threads.
pub(crate) trait RequireSendSync: 'static + Send + Sync {}

init_fn!(init_multitask, JLRS_MULTITASK_JL, "JlrsMultitask.jl");

#[cfg(feature = "async-std-rt")]
mod impl_async_std {
    use self::{
        async_std_rt::{channel, oneshot_channel},
        TrySendError,
    };
    use super::*;
    use crate::error::JlrsResult;
    use crate::multitask::async_task::internal::{
        BlockingTask, PendingTask, Persistent, RegisterPersistent, RegisterTask, Task,
    };
    use crate::multitask::async_task::{AsyncTask, PersistentTask};
    use async_std::{
        channel::{Receiver, Sender},
        future::timeout,
        task::{self, JoinHandle},
    };
    use std::{
        sync::Arc,
        thread::{self, JoinHandle as ThreadHandle},
        time::Duration,
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
        pub(crate) sender: Arc<Sender<Message>>,
    }

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
        /// [struct-level]: crate::multitask::runtime::AsyncJulia
        pub unsafe fn init(
            max_n_tasks: usize,
            channel_capacity: usize,
            recv_timeout: Duration,
        ) -> JlrsResult<(Self, ThreadHandle<JlrsResult<()>>)> {
            check_threads_var()?;

            let (sender, receiver) = channel(channel_capacity);
            let sender = Arc::new(sender);
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
        /// [struct-level]: crate::multitask::runtime::AsyncJulia
        pub async unsafe fn init_async(
            max_n_tasks: usize,
            channel_capacity: usize,
            recv_timeout: Duration,
        ) -> JlrsResult<(Self, JoinHandle<JlrsResult<()>>)> {
            check_threads_var()?;

            let (sender, receiver) = channel(channel_capacity);
            let sender = Arc::new(sender);
            let julia = AsyncJulia { sender };
            let handle =
                task::spawn_blocking(move || run_async(max_n_tasks, recv_timeout, receiver));
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
            let sender = Arc::new(sender);
            let julia = AsyncJulia { sender };
            let handle = thread::spawn(move || {
                run_async_with_image::<_, _>(
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
        ) -> JlrsResult<(Self, JoinHandle<JlrsResult<()>>)>
        where
            P: AsRef<Path> + Send + 'static,
            Q: AsRef<Path> + Send + 'static,
        {
            check_threads_var()?;

            let (sender, receiver) = channel(channel_capacity);
            let sender = Arc::new(sender);
            let julia = AsyncJulia { sender };
            let handle = task::spawn_blocking(move || {
                run_async_with_image::<_, _>(
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
            R: ResultSender<JlrsResult<AT::Output>>,
        {
            let sender = self.sender.clone();
            let msg = PendingTask::<_, _, Task>::new(task, res_sender);
            let boxed = Box::new(msg);
            self.sender
                .send(MessageInner::Task(boxed, sender).wrap())
                .await
                .expect("Channel was closed");
        }

        /// Try to send a new task to the runtime, if there's no room in the channel an error is
        /// returned immediately. This method takes two arguments, the task and the sending half of a
        /// channel which is used to send the result back after the task has completed.
        pub fn try_task<AT, R>(&self, task: AT, res_sender: R) -> JlrsResult<()>
        where
            AT: AsyncTask,
            R: ResultSender<JlrsResult<AT::Output>>,
        {
            let sender = self.sender.clone();
            let msg = PendingTask::<_, _, Task>::new(task, res_sender);
            let boxed = Box::new(msg);
            match self
                .sender
                .try_send(MessageInner::Task(boxed, sender).wrap())
            {
                Ok(_) => Ok(()),
                Err(::async_std::channel::TrySendError::Full(_)) => {
                    Err(Box::new(JlrsError::other(TrySendError::Full(()))))?
                }
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
            R: ResultSender<JlrsResult<T>>,
            T: Send + Sync + 'static,
        {
            let msg = BlockingTask::<_, _, _>::new(task, res_sender, 0);
            let boxed = Box::new(msg);
            self.sender
                .send(MessageInner::BlockingTask(boxed).wrap())
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
            R: ResultSender<JlrsResult<T>>,
            T: Send + Sync + 'static,
        {
            let msg = BlockingTask::<_, _, _>::new(task, res_sender, 0);
            let boxed = Box::new(msg);
            match self
                .sender
                .try_send(MessageInner::BlockingTask(boxed).wrap())
            {
                Ok(_) => Ok(()),
                Err(::async_std::channel::TrySendError::Full(_)) => {
                    Err(Box::new(JlrsError::other(TrySendError::Full(()))))?
                }
                Err(_) => Err(Box::new(JlrsError::other(TrySendError::Closed(()))))?,
            }
        }

        /// Sends a blocking task to the async runtime, if there's no room in the channel an error is
        /// returned immediately. A blocking task is a closure that takes two arguments, a `Global`
        /// and mutable reference to a `GcFrame`, and must return a `JlrsResult` whose inner type is
        /// both `Send` and `Sync`. This task is executed as soon as possible and can't call async
        /// methods, so it block the runtime. The result of the closure is sent to `res_sender`.
        pub async fn blocking_task_with_slots<T, R, F>(&self, task: F, res_sender: R, slots: usize)
        where
            for<'base> F: 'static
                + Send
                + Sync
                + FnOnce(Global<'base>, &mut GcFrame<'base, Async<'base>>) -> JlrsResult<T>,
            R: ResultSender<JlrsResult<T>>,
            T: Send + Sync + 'static,
        {
            let msg = BlockingTask::<_, _, _>::new(task, res_sender, slots);
            let boxed = Box::new(msg);
            self.sender
                .send(MessageInner::BlockingTask(boxed).wrap())
                .await
                .expect("Channel was closed");
        }

        /// Sends a blocking task to the async runtime, this method waits if there's no room in the
        /// channel. A blocking task is a closure that takes two arguments, a `Global` and mutable
        /// reference to a `GcFrame`, and must return a `JlrsResult` whose inner type is both `Send`
        /// and `Sync`. This task is executed as soon as possible and can't call async methods, so it
        /// block the runtime. The result of the closure is sent to `res_sender`.
        pub fn try_blocking_task_with_slots<T, R, F>(
            &self,
            task: F,
            res_sender: R,
            slots: usize,
        ) -> JlrsResult<()>
        where
            for<'base> F: 'static
                + Send
                + Sync
                + FnOnce(Global<'base>, &mut GcFrame<'base, Async<'base>>) -> JlrsResult<T>,
            R: ResultSender<JlrsResult<T>>,
            T: Send + Sync + 'static,
        {
            let msg = BlockingTask::<_, _, _>::new(task, res_sender, slots);
            let boxed = Box::new(msg);
            match self
                .sender
                .try_send(MessageInner::BlockingTask(boxed).wrap())
            {
                Ok(_) => Ok(()),
                Err(::async_std::channel::TrySendError::Full(_)) => {
                    Err(Box::new(JlrsError::other(TrySendError::Full(()))))?
                }
                Err(_) => Err(Box::new(JlrsError::other(TrySendError::Closed(()))))?,
            }
        }

        /// Register a task, this method waits if there's no room in the channel. This method takes
        /// one argument, the sending half of a channel which is used to send the result back after
        /// the registration has completed.
        pub async fn register_task<AT, R>(&self, res_sender: R)
        where
            AT: AsyncTask,
            R: ResultSender<JlrsResult<()>>,
        {
            let sender = self.sender.clone();
            let msg = PendingTask::<_, AT, RegisterTask>::new(res_sender);
            let boxed = Box::new(msg);
            self.sender
                .send(MessageInner::Task(boxed, sender).wrap())
                .await
                .expect("Channel was closed");
        }

        /// Try to register a task, if there's no room in the channel an error is returned
        /// immediately. This method takes one argument, the sending half of a channel which is used
        /// to send the result back after the registration has completed.
        pub fn try_register_task<AT, R>(&self, res_sender: R) -> JlrsResult<()>
        where
            AT: AsyncTask,
            R: ResultSender<JlrsResult<()>>,
        {
            let sender = self.sender.clone();
            let msg = PendingTask::<_, AT, RegisterTask>::new(res_sender);
            let boxed = Box::new(msg);
            match self
                .sender
                .try_send(MessageInner::Task(boxed, sender).wrap())
            {
                Ok(_) => (),
                Err(::async_std::channel::TrySendError::Full(_)) => {
                    Err(Box::new(JlrsError::other(TrySendError::Full(()))))?
                }
                Err(_) => Err(Box::new(JlrsError::other(TrySendError::Closed(()))))?,
            }

            Ok(())
        }

        /// Send a new persistent to the runtime, this method waits if there's no room in the channel.
        /// This method takes a single argument, the task. It returns after
        /// [`PersistentTask::init`] has been called, a [`PersistentHandle`] to the task is
        /// returned if this method completes successfully, and the error if it doesn't.
        pub async fn persistent<GT>(&self, task: GT) -> JlrsResult<PersistentHandle<GT>>
        where
            GT: PersistentTask,
        {
            let (init_sender, init_recv) = oneshot_channel();
            let rt_sender = self.sender.clone();
            let msg = PendingTask::<_, _, Persistent>::new(task, init_sender);
            let boxed = Box::new(msg);

            self.sender
                .send(MessageInner::Task(boxed, rt_sender).wrap())
                .await
                .expect("Channel was closed");

            match init_recv.recv().await {
                Ok(handle) => handle,
                Err(e) => Err(JlrsError::other(e))?,
            }
        }

        /// Send a new persistent to the runtime, if there's no room in the channel an error is
        /// returned immediately. This method takes a single argument, the task. It returns after
        /// [`PersistentTask::init`] has been called, a [`PersistentHandle`] to the task is
        /// returned if this method completes successfully, and the error if it doesn't.
        pub fn try_persistent<GT>(&self, task: GT) -> JlrsResult<PersistentHandle<GT>>
        where
            GT: PersistentTask,
        {
            let (init_sender, init_receiver) = crossbeam_channel::bounded(1);
            let sender = self.sender.clone();
            let msg = PendingTask::<_, _, Persistent>::new(task, init_sender);
            let boxed = Box::new(msg);
            match self
                .sender
                .try_send(MessageInner::Task(boxed, sender).wrap())
            {
                Ok(_) => (),
                Err(::async_std::channel::TrySendError::Full(_)) => {
                    Err(Box::new(JlrsError::other(TrySendError::Full(()))))?
                }
                Err(_) => Err(Box::new(JlrsError::other(TrySendError::Closed(()))))?,
            };

            match init_receiver.recv() {
                Ok(handle) => handle,
                Err(e) => Err(JlrsError::other(e))?,
            }
        }

        /// Register a persistent, this method waits if there's no room in the channel. This method
        /// takes one argument, the sending half of a channel which is used to send the result back
        /// after the registration has completed.
        pub async fn register_persistent<GT, R>(&self, res_sender: R)
        where
            GT: PersistentTask,
            R: ResultSender<JlrsResult<()>>,
        {
            let sender = self.sender.clone();
            let msg = PendingTask::<_, GT, RegisterPersistent>::new(res_sender);
            let boxed = Box::new(msg);
            self.sender
                .send(MessageInner::Task(boxed, sender).wrap())
                .await
                .expect("Channel was closed");
        }

        /// Try to register a persistent, if there's no room in the channel an error is returned
        /// immediately. This method takes one argument, the sending half of a channel which is used
        /// to send the result back after the registration has completed.
        pub fn try_register_persistent<GT, R>(&self, res_sender: R) -> JlrsResult<()>
        where
            GT: PersistentTask,
            R: ResultSender<JlrsResult<()>>,
        {
            let sender = self.sender.clone();
            let msg = PendingTask::<_, GT, RegisterPersistent>::new(res_sender);
            let boxed = Box::new(msg);
            match self
                .sender
                .try_send(MessageInner::Task(boxed, sender).wrap())
            {
                Ok(_) => (),
                Err(::async_std::channel::TrySendError::Full(_)) => {
                    Err(Box::new(JlrsError::other(TrySendError::Full(()))))?
                }
                Err(_) => Err(Box::new(JlrsError::other(TrySendError::Closed(()))))?,
            }

            Ok(())
        }

        /// Include a Julia file. This method waits until the call to `Main.include` in Julia has been
        /// completed. It returns an error if the path doesn't exist or the call to `Main.include`
        /// throws an exception.
        pub async unsafe fn include<P: AsRef<Path>>(&self, path: P) -> JlrsResult<()> {
            if !path.as_ref().exists() {
                Err(JlrsError::IncludeNotFound {
                    path: path.as_ref().to_string_lossy().into(),
                })?
            }

            let (sender, receiver) = oneshot_channel();
            self.sender
                .send(MessageInner::Include(path.as_ref().to_path_buf(), Box::new(sender)).wrap())
                .await
                .expect("Channel was closed");

            receiver.recv().await.expect("Result channel was closed")
        }

        /// Include a Julia file. This method waits until the call `Main.include` in Julia has been
        /// completed. It returns an error if the path doesn't  exist, the channel is full, or the
        /// call to `Main.include` throws an exception.
        pub unsafe fn try_include<P: AsRef<Path>>(&self, path: P) -> JlrsResult<()> {
            if !path.as_ref().exists() {
                Err(JlrsError::IncludeNotFound {
                    path: path.as_ref().to_string_lossy().into(),
                })?
            }

            let (sender, receiver) = crossbeam_channel::bounded(1);
            self.sender
                .try_send(MessageInner::TryInclude(path.as_ref().to_path_buf(), sender).wrap())
                .map_err(|e| match e {
                    ::async_std::channel::TrySendError::Full(_) => {
                        Box::new(JlrsError::other(TrySendError::Full(())))
                    }
                    ::async_std::channel::TrySendError::Closed(_) => {
                        Box::new(JlrsError::other(TrySendError::Closed(())))
                    }
                })
                .and_then(|_| receiver.recv().expect("Result channel was closed"))
        }

        /// Enable or disable colored error messages originating from Julia. If this is enabled the
        /// error message in [`JlrsError::Exception`] can contain ANSI color codes. This feature is
        /// disabled by default.
        pub async fn error_color(&self, enable: bool) -> JlrsResult<()> {
            let (sender, receiver) = oneshot_channel();
            self.sender
                .send(MessageInner::ErrorColor(enable, Box::new(sender)).wrap())
                .await
                .expect("Channel was closed");

            receiver.recv().await.expect("Result channel was closed")
        }

        /// Enable or disable colored error messages originating from Julia. If this is enabled the
        /// error message in [`JlrsError::Exception`] can contain ANSI color codes. This feature is
        /// disabled by default.
        pub fn try_error_color(&self, enable: bool) -> JlrsResult<()> {
            let (sender, receiver) = crossbeam_channel::bounded(1);
            self.sender
                .try_send(MessageInner::TryErrorColor(enable, sender).wrap())
                .map_err(|e| match e {
                    ::async_std::channel::TrySendError::Full(_) => {
                        Box::new(JlrsError::other(TrySendError::Full(())))
                    }
                    ::async_std::channel::TrySendError::Closed(_) => {
                        Box::new(JlrsError::other(TrySendError::Closed(())))
                    }
                })
                .and_then(|_| receiver.recv().expect("Result channel was closed"))
        }

        /// Returns `true` if the channel is full.
        pub fn is_full(&self) -> bool {
            self.sender.is_full()
        }

        fn try_set_custom_fns(&self) -> JlrsResult<()> {
            let (sender, receiver) = crossbeam_channel::bounded(1);
            self.sender
                .try_send(MessageInner::TrySetCustomFns(sender).wrap())
                .map_err(|e| match e {
                    ::async_std::channel::TrySendError::Full(_) => {
                        Box::new(JlrsError::other(TrySendError::Full(())))
                    }
                    ::async_std::channel::TrySendError::Closed(_) => {
                        Box::new(JlrsError::other(TrySendError::Closed(())))
                    }
                })
                .and_then(|_| receiver.recv().expect("Result channel was closed"))
        }

        async fn set_custom_fns(&self) -> JlrsResult<()> {
            let (sender, receiver) = oneshot_channel();
            self.sender
                .send(MessageInner::SetCustomFns(Box::new(sender)).wrap())
                .await
                .expect("Channel was closed");

            receiver.recv().await.expect("Result channel was closed")
        }
    }

    fn run_async(
        max_n_tasks: usize,
        recv_timeout: Duration,
        receiver: Receiver<Message>,
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
                        Ok(Ok(msg)) => match msg.unwrap() {
                            MessageInner::Task(task, sender) => {
                                if let Some(idx) = free_stacks.pop_front() {
                                    let mut stack =
                                        stacks[idx].take().expect("Async stack corrupted");
                                    let task = task::spawn_local(async move {
                                        task.call(&mut stack).await;
                                        sender
                                            .send(MessageInner::Complete(idx, stack).wrap())
                                            .await
                                            .ok();
                                    });
                                    n_running += 1;
                                    running_tasks[idx] = Some(task);
                                } else {
                                    pending_tasks.push_back((task, sender));
                                }
                            }
                            MessageInner::Complete(idx, mut stack) => {
                                if let Some((jl_task, sender)) = pending_tasks.pop_front() {
                                    let task = task::spawn_local(async move {
                                        jl_task.call(&mut stack).await;
                                        sender
                                            .send(MessageInner::Complete(idx, stack).wrap())
                                            .await
                                            .ok();
                                    });
                                    running_tasks[idx] = Some(task);
                                } else {
                                    stacks[idx] = Some(stack);
                                    free_stacks.push_front(idx);
                                    n_running -= 1;
                                    running_tasks[idx] = None;
                                }
                            }
                            MessageInner::Include(path, sender) => {
                                let stack = stacks[0].as_mut().expect("Async stack corrupted");
                                let res = call_include(stack, path);
                                sender.send(res).await;
                            }
                            MessageInner::TryInclude(path, sender) => {
                                let stack = stacks[0].as_mut().expect("Async stack corrupted");
                                let res = call_include(stack, path);
                                (&sender).send(res).ok();
                            }
                            MessageInner::BlockingTask(task) => {
                                let stack = stacks[0].as_mut().expect("Async stack corrupted");
                                task.call(stack)
                            }
                            MessageInner::ErrorColor(enable, sender) => {
                                let res = call_error_color(enable);
                                sender.send(res).await;
                            }
                            MessageInner::TryErrorColor(enable, sender) => {
                                let res = call_error_color(enable);
                                (&sender).send(res).ok();
                            }
                            MessageInner::SetCustomFns(sender) => {
                                let stack = stacks[0].as_mut().expect("Async stack corrupted");
                                let res = call_set_custom_fns(stack);
                                sender.send(res).await;
                            }
                            MessageInner::TrySetCustomFns(sender) => {
                                let stack = stacks[0].as_mut().expect("Async stack corrupted");
                                let res = call_set_custom_fns(stack);
                                (&sender).send(res).ok();
                            }
                        },
                        Ok(Err(_)) => break,
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
        receiver: Receiver<Message>,
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
                        Ok(Ok(msg)) => match msg.unwrap() {
                            MessageInner::Task(task, sender) => {
                                if let Some(idx) = free_stacks.pop_front() {
                                    let mut stack =
                                        stacks[idx].take().expect("Async stack corrupted");
                                    let task = task::spawn_local(async move {
                                        task.call(&mut stack).await;
                                        sender
                                            .send(MessageInner::Complete(idx, stack).wrap())
                                            .await
                                            .ok();
                                    });
                                    n_running += 1;
                                    running_tasks[idx] = Some(task);
                                } else {
                                    pending_tasks.push_back((task, sender));
                                }
                            }
                            MessageInner::Complete(idx, mut stack) => {
                                if let Some((jl_task, sender)) = pending_tasks.pop_front() {
                                    let task = task::spawn_local(async move {
                                        jl_task.call(&mut stack).await;
                                        sender
                                            .send(MessageInner::Complete(idx, stack).wrap())
                                            .await
                                            .ok();
                                    });
                                    running_tasks[idx] = Some(task);
                                } else {
                                    stacks[idx] = Some(stack);
                                    n_running -= 1;
                                    free_stacks.push_front(idx);
                                    running_tasks[idx] = None;
                                }
                            }
                            MessageInner::Include(path, sender) => {
                                let stack = stacks[0].as_mut().expect("Async stack corrupted");
                                let res = call_include(stack, path);
                                sender.send(res).await;
                            }
                            MessageInner::TryInclude(path, sender) => {
                                let stack = stacks[0].as_mut().expect("Async stack corrupted");
                                let res = call_include(stack, path);
                                (&sender).send(res).ok();
                            }
                            MessageInner::BlockingTask(task) => {
                                let stack = stacks[0].as_mut().expect("Async stack corrupted");
                                task.call(stack)
                            }
                            MessageInner::ErrorColor(enable, sender) => {
                                let res = call_error_color(enable);
                                sender.send(res).await;
                            }
                            MessageInner::TryErrorColor(enable, sender) => {
                                let res = call_error_color(enable);
                                (&sender).send(res).ok();
                            }
                            MessageInner::SetCustomFns(sender) => {
                                let stack = stacks[0].as_mut().expect("Async stack corrupted");
                                let res = call_set_custom_fns(stack);
                                sender.send(res).await;
                            }
                            MessageInner::TrySetCustomFns(sender) => {
                                let stack = stacks[0].as_mut().expect("Async stack corrupted");
                                let res = call_set_custom_fns(stack);
                                (&sender).send(res).ok();
                            }
                        },
                        Ok(Err(_)) => break,
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

    pub(crate) enum MessageInner {
        Task(Box<dyn GenericPendingTask>, Arc<Sender<Message>>),
        BlockingTask(Box<dyn GenericBlockingTask>),
        Include(PathBuf, Box<dyn ResultSender<JlrsResult<()>>>),
        TryInclude(PathBuf, crossbeam_channel::Sender<JlrsResult<()>>),
        ErrorColor(bool, Box<dyn ResultSender<JlrsResult<()>>>),
        TryErrorColor(bool, crossbeam_channel::Sender<JlrsResult<()>>),
        Complete(usize, Pin<Box<AsyncStackPage>>),
        SetCustomFns(Box<dyn ResultSender<JlrsResult<()>>>),
        TrySetCustomFns(crossbeam_channel::Sender<JlrsResult<()>>),
    }
}

#[cfg(feature = "tokio-rt")]
pub mod impl_tokio {
    use self::{
        tokio_rt::{channel, oneshot_channel, MaybeUnboundedReceiver, MaybeUnboundedSender, Tokio},
        TrySendError,
    };
    use super::super::async_task::internal::{
        BlockingTask, PendingTask, Persistent, RegisterPersistent, RegisterTask, Task,
    };
    use super::super::async_task::{AsyncTask, PersistentTask};
    use super::*;
    use crate::error::JlrsResult;
    use std::{
        sync::Arc,
        thread::{self, JoinHandle as ThreadHandle},
        time::Duration,
    };
    use tokio::{
        task::{self, JoinHandle},
        time::timeout,
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
    pub struct AsyncJuliaTokio {
        pub(crate) sender: Arc<MaybeUnboundedSender<Message>>,
    }

    impl RequireSendSync for AsyncJuliaTokio {}

    impl AsyncJuliaTokio {
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
        /// [struct-level]: crate::multitask::runtime::AsyncJulia
        pub unsafe fn init(
            max_n_tasks: usize,
            channel_capacity: usize,
            recv_timeout: Duration,
        ) -> JlrsResult<(Self, ThreadHandle<JlrsResult<()>>)> {
            check_threads_var()?;

            let (sender, receiver) = channel(channel_capacity);
            let sender = Arc::new(sender);
            let julia = AsyncJuliaTokio { sender };
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
        /// [struct-level]: crate::multitask::runtime::AsyncJulia
        pub async unsafe fn init_async(
            max_n_tasks: usize,
            channel_capacity: usize,
            recv_timeout: Duration,
        ) -> JlrsResult<(Self, JoinHandle<JlrsResult<()>>)> {
            check_threads_var()?;

            let (sender, receiver) = channel(channel_capacity);
            let sender = Arc::new(sender);
            let julia = AsyncJuliaTokio { sender };
            let handle =
                task::spawn_blocking(move || run_async(max_n_tasks, recv_timeout, receiver));
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
            let sender = Arc::new(sender);
            let julia = AsyncJuliaTokio { sender };
            let handle = thread::spawn(move || {
                run_async_with_image::<_, _>(
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
        ) -> JlrsResult<(Self, JoinHandle<JlrsResult<()>>)>
        where
            P: AsRef<Path> + Send + 'static,
            Q: AsRef<Path> + Send + 'static,
        {
            check_threads_var()?;

            let (sender, receiver) = channel(channel_capacity);
            let sender = Arc::new(sender);
            let julia = AsyncJuliaTokio { sender };
            let handle = task::spawn_blocking(move || {
                run_async_with_image::<_, _>(
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
            R: ResultSender<JlrsResult<AT::Output>>,
        {
            let sender = self.sender.clone();
            let msg = PendingTask::<_, _, Task>::new(task, res_sender);
            let boxed = Box::new(msg);
            self.sender
                .send(MessageInner::Task(boxed, sender).wrap())
                .await
                .expect("Channel was closed");
        }

        /// Try to send a new task to the runtime, if there's no room in the channel an error is
        /// returned immediately. This method takes two arguments, the task and the sending half of a
        /// channel which is used to send the result back after the task has completed.
        pub fn try_task<AT, R>(&self, task: AT, res_sender: R) -> JlrsResult<()>
        where
            AT: AsyncTask,
            R: ResultSender<JlrsResult<AT::Output>>,
        {
            let sender = self.sender.clone();
            let msg = PendingTask::<_, _, Task>::new(task, res_sender);
            let boxed = Box::new(msg);
            match self
                .sender
                .try_send(MessageInner::Task(boxed, sender).wrap())
            {
                Ok(_) => Ok(()),
                Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                    Err(Box::new(JlrsError::other(TrySendError::Full(()))))?
                }
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
            R: ResultSender<JlrsResult<T>>,
            T: Send + Sync + 'static,
        {
            let msg = BlockingTask::<_, _, _>::new(task, res_sender, 0);
            let boxed = Box::new(msg);
            self.sender
                .send(MessageInner::BlockingTask(boxed).wrap())
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
            R: ResultSender<JlrsResult<T>>,
            T: Send + Sync + 'static,
        {
            let msg = BlockingTask::<_, _, _>::new(task, res_sender, 0);
            let boxed = Box::new(msg);
            match self
                .sender
                .try_send(MessageInner::BlockingTask(boxed).wrap())
            {
                Ok(_) => Ok(()),
                Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                    Err(Box::new(JlrsError::other(TrySendError::Full(()))))?
                }
                Err(_) => Err(Box::new(JlrsError::other(TrySendError::Closed(()))))?,
            }
        }

        /// Sends a blocking task to the async runtime, if there's no room in the channel an error is
        /// returned immediately. A blocking task is a closure that takes two arguments, a `Global`
        /// and mutable reference to a `GcFrame`, and must return a `JlrsResult` whose inner type is
        /// both `Send` and `Sync`. This task is executed as soon as possible and can't call async
        /// methods, so it block the runtime. The result of the closure is sent to `res_sender`.
        pub async fn blocking_task_with_slots<T, R, F>(&self, task: F, res_sender: R, slots: usize)
        where
            for<'base> F: 'static
                + Send
                + Sync
                + FnOnce(Global<'base>, &mut GcFrame<'base, Async<'base>>) -> JlrsResult<T>,
            R: ResultSender<JlrsResult<T>>,
            T: Send + Sync + 'static,
        {
            let msg = BlockingTask::<_, _, _>::new(task, res_sender, slots);
            let boxed = Box::new(msg);
            self.sender
                .send(MessageInner::BlockingTask(boxed).wrap())
                .await
                .expect("Channel was closed");
        }

        /// Sends a blocking task to the async runtime, this method waits if there's no room in the
        /// channel. A blocking task is a closure that takes two arguments, a `Global` and mutable
        /// reference to a `GcFrame`, and must return a `JlrsResult` whose inner type is both `Send`
        /// and `Sync`. This task is executed as soon as possible and can't call async methods, so it
        /// block the runtime. The result of the closure is sent to `res_sender`.
        pub fn try_blocking_task_with_slots<T, R, F>(
            &self,
            task: F,
            res_sender: R,
            slots: usize,
        ) -> JlrsResult<()>
        where
            for<'base> F: 'static
                + Send
                + Sync
                + FnOnce(Global<'base>, &mut GcFrame<'base, Async<'base>>) -> JlrsResult<T>,
            R: ResultSender<JlrsResult<T>>,
            T: Send + Sync + 'static,
        {
            let msg = BlockingTask::<_, _, _>::new(task, res_sender, slots);
            let boxed = Box::new(msg);
            match self
                .sender
                .try_send(MessageInner::BlockingTask(boxed).wrap())
            {
                Ok(_) => Ok(()),
                Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                    Err(Box::new(JlrsError::other(TrySendError::Full(()))))?
                }
                Err(_) => Err(Box::new(JlrsError::other(TrySendError::Closed(()))))?,
            }
        }

        /// Register a task, this method waits if there's no room in the channel. This method takes
        /// one argument, the sending half of a channel which is used to send the result back after
        /// the registration has completed.
        pub async fn register_task<AT, R>(&self, res_sender: R)
        where
            AT: AsyncTask,
            R: ResultSender<JlrsResult<()>>,
        {
            let sender = self.sender.clone();
            let msg = PendingTask::<_, AT, RegisterTask>::new(res_sender);
            let boxed = Box::new(msg);
            self.sender
                .send(MessageInner::Task(boxed, sender).wrap())
                .await
                .expect("Channel was closed");
        }

        /// Try to register a task, if there's no room in the channel an error is returned
        /// immediately. This method takes one argument, the sending half of a channel which is used
        /// to send the result back after the registration has completed.
        pub fn try_register_task<AT, R>(&self, res_sender: R) -> JlrsResult<()>
        where
            AT: AsyncTask,
            R: ResultSender<JlrsResult<()>>,
        {
            let sender = self.sender.clone();
            let msg = PendingTask::<_, AT, RegisterTask>::new(res_sender);
            let boxed = Box::new(msg);
            match self
                .sender
                .try_send(MessageInner::Task(boxed, sender).wrap())
            {
                Ok(_) => (),
                Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                    Err(Box::new(JlrsError::other(TrySendError::Full(()))))?
                }
                Err(_) => Err(Box::new(JlrsError::other(TrySendError::Closed(()))))?,
            }

            Ok(())
        }

        /// Send a new persistent to the runtime, this method waits if there's no room in the channel.
        /// This method takes a single argument, the task. It returns after
        /// [`PersistentTask::init`] has been called, a [`PersistentHandle`] to the task is
        /// returned if this method completes successfully, and the error if it doesn't.
        pub async fn persistent<GT>(&self, task: GT) -> JlrsResult<PersistentHandle<GT>>
        where
            GT: PersistentTask,
        {
            let (init_sender, init_recv) = oneshot_channel();
            let rt_sender = self.sender.clone();
            let msg = PendingTask::<_, _, Persistent>::new(task, init_sender);
            let boxed = Box::new(msg);

            self.sender
                .send(MessageInner::Task(boxed, rt_sender).wrap())
                .await
                .expect("Channel was closed");

            match init_recv.await {
                Ok(handle) => handle,
                Err(_) => panic!("Channel was closed"),
            }
        }

        /// Send a new persistent to the runtime, if there's no room in the channel an error is
        /// returned immediately. This method takes a single argument, the task. It returns after
        /// [`PersistentTask::init`] has been called, a [`PersistentHandle`] to the task is
        /// returned if this method completes successfully, and the error if it doesn't.
        pub fn try_persistent<GT>(&self, task: GT) -> JlrsResult<PersistentHandle<GT>>
        where
            GT: PersistentTask,
        {
            let (init_sender, init_receiver) = crossbeam_channel::bounded(1);
            let sender = self.sender.clone();
            let msg = PendingTask::<_, _, Persistent>::new(task, init_sender);
            let boxed = Box::new(msg);
            match self
                .sender
                .try_send(MessageInner::Task(boxed, sender).wrap())
            {
                Ok(_) => (),
                Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                    Err(Box::new(JlrsError::other(TrySendError::Full(()))))?
                }
                Err(_) => Err(Box::new(JlrsError::other(TrySendError::Closed(()))))?,
            };

            match init_receiver.recv() {
                Ok(handle) => handle,
                Err(e) => Err(JlrsError::other(e))?,
            }
        }

        /// Register a persistent, this method waits if there's no room in the channel. This method
        /// takes one argument, the sending half of a channel which is used to send the result back
        /// after the registration has completed.
        pub async fn register_persistent<GT, R>(&self, res_sender: R)
        where
            GT: PersistentTask,
            R: ResultSender<JlrsResult<()>>,
        {
            let sender = self.sender.clone();
            let msg = PendingTask::<_, GT, RegisterPersistent>::new(res_sender);
            let boxed = Box::new(msg);
            self.sender
                .send(MessageInner::Task(boxed, sender).wrap())
                .await
                .expect("Channel was closed");
        }

        /// Try to register a persistent, if there's no room in the channel an error is returned
        /// immediately. This method takes one argument, the sending half of a channel which is used
        /// to send the result back after the registration has completed.
        pub fn try_register_persistent<GT, R>(&self, res_sender: R) -> JlrsResult<()>
        where
            GT: PersistentTask,
            R: ResultSender<JlrsResult<()>>,
        {
            let sender = self.sender.clone();
            let msg = PendingTask::<_, GT, RegisterPersistent>::new(res_sender);
            let boxed = Box::new(msg);
            match self
                .sender
                .try_send(MessageInner::Task(boxed, sender).wrap())
            {
                Ok(_) => (),
                Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                    Err(Box::new(JlrsError::other(TrySendError::Full(()))))?
                }
                Err(_) => Err(Box::new(JlrsError::other(TrySendError::Closed(()))))?,
            }

            Ok(())
        }

        /// Include a Julia file. This method waits until the call to `Main.include` in Julia has been
        /// completed. It returns an error if the path doesn't exist or the call to `Main.include`
        /// throws an exception.
        pub async unsafe fn include<P: AsRef<Path>>(&self, path: P) -> JlrsResult<()> {
            if !path.as_ref().exists() {
                Err(JlrsError::IncludeNotFound {
                    path: path.as_ref().to_string_lossy().into(),
                })?
            }

            let (sender, receiver) = oneshot_channel();
            self.sender
                .send(MessageInner::Include(path.as_ref().to_path_buf(), Box::new(sender)).wrap())
                .await
                .expect("Channel was closed");

            receiver.await.expect("Result channel was closed")
        }

        /// Include a Julia file. This method waits until the call `Main.include` in Julia has been
        /// completed. It returns an error if the path doesn't  exist, the channel is full, or the
        /// call to `Main.include` throws an exception.
        pub unsafe fn try_include<P: AsRef<Path>>(&self, path: P) -> JlrsResult<()> {
            if !path.as_ref().exists() {
                Err(JlrsError::IncludeNotFound {
                    path: path.as_ref().to_string_lossy().into(),
                })?
            }

            let (sender, receiver) = crossbeam_channel::bounded(1);
            self.sender
                .try_send(MessageInner::TryInclude(path.as_ref().to_path_buf(), sender).wrap())
                .map_err(|e| match e {
                    tokio::sync::mpsc::error::TrySendError::Full(_) => {
                        Box::new(JlrsError::other(TrySendError::Full(())))
                    }
                    tokio::sync::mpsc::error::TrySendError::Closed(_) => {
                        Box::new(JlrsError::other(TrySendError::Closed(())))
                    }
                })
                .and_then(|_| receiver.recv().expect("Result channel was closed"))
        }

        /// Enable or disable colored error messages originating from Julia. If this is enabled the
        /// error message in [`JlrsError::Exception`] can contain ANSI color codes. This feature is
        /// disabled by default.
        pub async fn error_color(&self, enable: bool) -> JlrsResult<()> {
            let (sender, receiver) = oneshot_channel();
            self.sender
                .send(MessageInner::ErrorColor(enable, Box::new(sender)).wrap())
                .await
                .expect("Channel was closed");

            receiver.await.expect("Result channel was closed")
        }

        /// Enable or disable colored error messages originating from Julia. If this is enabled the
        /// error message in [`JlrsError::Exception`] can contain ANSI color codes. This feature is
        /// disabled by default.
        pub fn try_error_color(&self, enable: bool) -> JlrsResult<()> {
            let (sender, receiver) = crossbeam_channel::bounded(1);
            self.sender
                .try_send(MessageInner::TryErrorColor(enable, sender).wrap())
                .map_err(|e| match e {
                    tokio::sync::mpsc::error::TrySendError::Full(_) => {
                        Box::new(JlrsError::other(TrySendError::Full(())))
                    }
                    tokio::sync::mpsc::error::TrySendError::Closed(_) => {
                        Box::new(JlrsError::other(TrySendError::Closed(())))
                    }
                })
                .and_then(|_| receiver.recv().expect("Result channel was closed"))
        }

        fn try_set_custom_fns(&self) -> JlrsResult<()> {
            let (sender, receiver) = crossbeam_channel::bounded(1);
            self.sender
                .try_send(MessageInner::TrySetCustomFns(sender).wrap())
                .map_err(|e| match e {
                    tokio::sync::mpsc::error::TrySendError::Full(_) => {
                        Box::new(JlrsError::other(TrySendError::Full(())))
                    }
                    tokio::sync::mpsc::error::TrySendError::Closed(_) => {
                        Box::new(JlrsError::other(TrySendError::Closed(())))
                    }
                })
                .and_then(|_| receiver.recv().expect("Result channel was closed"))
        }

        async fn set_custom_fns(&self) -> JlrsResult<()> {
            let (sender, receiver) = oneshot_channel();
            self.sender
                .send(MessageInner::SetCustomFns(Box::new(sender)).wrap())
                .await
                .expect("Channel was closed");

            receiver.await.expect("Result channel was closed")
        }
    }

    fn run_async(
        max_n_tasks: usize,
        recv_timeout: Duration,
        receiver: MaybeUnboundedReceiver<Message>,
    ) -> JlrsResult<()> {
        let rt = Tokio::new();
        rt.block_on(async {
            unsafe {
                if jl_is_initialized() != 0 || INIT.swap(true, Ordering::SeqCst) {
                    return Err(JlrsError::AlreadyInitialized.into());
                }

                jl_init();
                run_inner(max_n_tasks, recv_timeout, receiver).await?;
            }

            Ok(())
        })
    }

    fn run_async_with_image<P, Q>(
        max_n_tasks: usize,
        recv_timeout: Duration,
        receiver: MaybeUnboundedReceiver<Message>,
        julia_bindir: P,
        image_path: Q,
    ) -> JlrsResult<()>
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        let rt = Tokio::new();
        rt.block_on(async {
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
                run_inner(max_n_tasks, recv_timeout, receiver).await?;
            }

            Ok(())
        })
    }

    async unsafe fn run_inner(
        max_n_tasks: usize,
        recv_timeout: Duration,
        mut receiver: MaybeUnboundedReceiver<Message>,
    ) -> Result<(), Box<JlrsError>> {
        if Info::new().n_threads() < 3 {
            Err(JlrsError::MoreThreadsRequired)?;
        }

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
                    jl_sys::jl_yield();
                }
                Ok(Some(msg)) => match msg.unwrap() {
                    MessageInner::Task(task, sender) => {
                        if let Some(idx) = free_stacks.pop_front() {
                            let mut stack = stacks[idx].take().expect("Async stack corrupted");
                            let task = task::spawn_local(async move {
                                task.call(&mut stack).await;
                                sender
                                    .as_ref()
                                    .send(MessageInner::Complete(idx, stack).wrap())
                                    .await
                                    .ok();
                            });
                            n_running += 1;
                            running_tasks[idx] = Some(task);
                        } else {
                            pending_tasks.push_back((task, sender));
                        }
                    }
                    MessageInner::Complete(idx, mut stack) => {
                        if let Some((jl_task, sender)) = pending_tasks.pop_front() {
                            let task = task::spawn_local(async move {
                                jl_task.call(&mut stack).await;
                                sender
                                    .send(MessageInner::Complete(idx, stack).wrap())
                                    .await
                                    .ok();
                            });
                            running_tasks[idx] = Some(task);
                        } else {
                            stacks[idx] = Some(stack);
                            n_running -= 1;
                            free_stacks.push_front(idx);
                            running_tasks[idx] = None;
                        }
                    }
                    MessageInner::Include(path, sender) => {
                        let stack = stacks[0].as_mut().expect("Async stack corrupted");
                        let res = call_include(stack, path);
                        sender.send(res).await;
                    }
                    MessageInner::TryInclude(path, sender) => {
                        let stack = stacks[0].as_mut().expect("Async stack corrupted");
                        let res = call_include(stack, path);
                        (&sender).send(res).ok();
                    }
                    MessageInner::BlockingTask(task) => {
                        let stack = stacks[0].as_mut().expect("Async stack corrupted");
                        task.call(stack)
                    }
                    MessageInner::ErrorColor(enable, sender) => {
                        let res = call_error_color(enable);
                        sender.send(res).await;
                    }
                    MessageInner::TryErrorColor(enable, sender) => {
                        let res = call_error_color(enable);
                        (&sender).send(res).ok();
                    }
                    MessageInner::SetCustomFns(sender) => {
                        let stack = stacks[0].as_mut().expect("Async stack corrupted");
                        let res = call_set_custom_fns(stack);
                        sender.send(res).await;
                    }
                    MessageInner::TrySetCustomFns(sender) => {
                        let stack = stacks[0].as_mut().expect("Async stack corrupted");
                        let res = call_set_custom_fns(stack);
                        (&sender).send(res).ok();
                    }
                },
                Ok(None) => break,
            }
        }

        for running in running_tasks.iter_mut() {
            if let Some(handle) = running.take() {
                handle.await.ok();
            }
        }

        jl_atexit_hook(0);
        Ok(())
    }

    pub(crate) enum MessageInner {
        Task(
            Box<dyn GenericPendingTask>,
            Arc<MaybeUnboundedSender<Message>>,
        ),
        BlockingTask(Box<dyn GenericBlockingTask>),
        Include(PathBuf, Box<dyn ResultSender<JlrsResult<()>>>),
        TryInclude(PathBuf, crossbeam_channel::Sender<JlrsResult<()>>),
        ErrorColor(bool, Box<dyn ResultSender<JlrsResult<()>>>),
        TryErrorColor(bool, crossbeam_channel::Sender<JlrsResult<()>>),
        Complete(usize, Pin<Box<AsyncStackPage>>),
        SetCustomFns(Box<dyn ResultSender<JlrsResult<()>>>),
        TrySetCustomFns(crossbeam_channel::Sender<JlrsResult<()>>),
    }
}

#[cfg(feature = "async-std-rt")]
pub use impl_async_std::*;

#[cfg(feature = "tokio-rt")]
pub use impl_tokio::AsyncJuliaTokio as AsyncJulia;

#[cfg(feature = "tokio-rt")]
use impl_tokio::MessageInner;

use super::julia_future::wake_task;

pub struct Message {
    inner: MessageInner,
}

impl Message {
    pub(crate) fn unwrap(self) -> MessageInner {
        self.inner
    }
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

#[derive(Debug)]
pub(crate) struct AsyncStackPage {
    pub(crate) top: Pin<Box<[Cell<*mut c_void>; 2]>>,
    pub(crate) page: StackPage,
}

// Not actually true, but we need to be able to send a page back after completing a task. The page
// is never (and must never be) shared across threads.
unsafe impl Send for AsyncStackPage {}
unsafe impl Sync for AsyncStackPage {}

impl AsyncStackPage {
    unsafe fn new() -> Pin<Box<Self>> {
        let stack = AsyncStackPage {
            top: Box::pin([Cell::new(null_mut()), Cell::new(null_mut())]),
            page: StackPage::default(),
        };

        Box::pin(stack)
    }
}

#[cfg(not(feature = "lts"))]
unsafe fn link_stacks(stacks: &mut [Option<Pin<Box<AsyncStackPage>>>]) {
    for stack in stacks.iter_mut() {
        let stack = stack.as_mut().unwrap();
        let task = NonNull::new_unchecked(jl_get_current_task().cast::<jl_task_t>()).as_mut();

        stack.top[1].set(task.gcstack.cast());

        task.gcstack = stack.top[0..].as_mut_ptr().cast();
    }
}

#[cfg(feature = "lts")]
unsafe fn link_stacks(stacks: &mut [Option<Pin<Box<AsyncStackPage>>>]) {
    for stack in stacks.iter_mut() {
        let stack = stack.as_mut().unwrap();
        let rtls = NonNull::new_unchecked(jl_sys::jl_get_ptls_states()).as_mut();
        stack.top[1].set(rtls.pgcstack.cast());
        rtls.pgcstack = stack.top[0..].as_mut_ptr().cast();
    }
}

fn call_include(stack: &mut AsyncStackPage, path: PathBuf) -> JlrsResult<()> {
    unsafe {
        let global = Global::new();
        let mode = Async(&stack.top[1]);
        let raw = stack.page.as_mut();
        let mut frame = GcFrame::new(raw, mode);

        match path.to_str() {
            Some(path) => {
                let path = JuliaString::new(&mut frame, path)?;
                Module::main(global)
                    .function_ref("include")?
                    .wrapper_unchecked()
                    .call1_unrooted(global, path.as_value())
                    .map_err(|e| JlrsError::Exception {
                        msg: format!("Include error: {:?}", e.value_unchecked()),
                    })?;
            }
            None => {}
        }

        Ok(())
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

fn call_set_custom_fns(stack: &mut AsyncStackPage) -> JlrsResult<()> {
    unsafe {
        let global = Global::new();
        let mode = Async(&stack.top[1]);
        let raw = stack.page.as_mut();
        let mut frame = GcFrame::new(raw, mode);

        init_jlrs(&mut frame);
        init_multitask(&mut frame);

        let jlrs_mod = Module::main(global)
            .submodule_ref("JlrsMultitask")?
            .wrapper_unchecked();

        let wake_rust = Value::new(&mut frame, wake_task as *mut c_void)?;
        jlrs_mod
            .global_ref("wakerust")?
            .wrapper_unchecked()
            .set_nth_field_unchecked(0, wake_rust);

        #[cfg(feature = "pyplot")]
        crate::pyplot::init_jlrs_py_plot(&mut frame);

        Ok(())
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
            Err(JlrsError::ThreadsVarRequired)?;
        }
    };

    Ok(())
}

#[cfg(feature = "async-std-rt")]
use async_std_rt::HandleSender;
#[cfg(feature = "tokio-rt")]
use tokio_rt::HandleSender;

use super::async_task::internal::{CallPersistentMessage, PersistentMessage};

/// A handle to a [`PersistentTask`]. This handle can be used to call the task and shared
/// across threads. The `PersistentTask` is dropped when its final handle has been dropped and all
/// remaining pending calls have completed.
#[derive(Clone)]
pub struct PersistentHandle<GT>
where
    GT: PersistentTask,
{
    sender: HandleSender<GT>,
}

impl<GT> PersistentHandle<GT>
where
    GT: PersistentTask,
{
    pub(crate) fn new(sender: HandleSender<GT>) -> Self {
        PersistentHandle { sender }
    }

    /// Call the task, this method waits until there's room available in the channel.
    pub async fn call<R>(&self, input: GT::Input, sender: R)
    where
        R: ResultSender<JlrsResult<GT::Output>>,
    {
        self.sender
            .send(PersistentMessage {
                msg: Box::new(CallPersistentMessage {
                    input: Some(input),
                    sender,
                    _marker: PhantomData,
                }),
            })
            .await
            .expect("Channel was closed")
    }

    /// Call the task, this method returns an error immediately if there's NO room available
    /// in the channel.
    pub fn try_call<R>(&self, input: GT::Input, sender: R) -> JlrsResult<()>
    where
        R: ResultSender<JlrsResult<GT::Output>>,
    {
        match self.sender.try_send(PersistentMessage {
            msg: Box::new(CallPersistentMessage {
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
impl<GT: PersistentTask> RequireSendSync for PersistentHandle<GT> {}
