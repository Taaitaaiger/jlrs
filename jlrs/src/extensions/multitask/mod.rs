//! Run Julia in a separate thread and execute tasks in parallel.
//!
//! The Julia C API can only be called from a single thread, and the interface provided by
//! [`Julia`] doesn't play nicely with Julia's task system. The async runtime provided by this
//! extension initializes Julia in a new thread and returns a handle, [`AsyncJulia`], that can
//! be cloned and used across threads. Unlike `Julia` there's no `scope` method, rather you will
//!  need to implement the [`AsyncTask`] and [`GeneratorTask`] traits and use the handle to send
//! such tasks to the async runtime.
//!
//! The `AsyncTask` and `GeneratorTask` traits are async traits, their async `run` methods take
//! the place of the closure that `scope` methods take in the sync runtime. Like the `scope`
//! methods, `run` takes a [`Global`] and an [`AsyncGcFrame`]. The latter provides the same
//! functionality as a [`GcFrame`] and adds a few methods to create nested async scopes. An
//! [`AsyncTask`] is run once, while a [`GeneratorTask`] can be called multiple times. In addition
//! to a run method, the `GeneratorTask` trait requires that you implement the `init` method; this
//! method can also call Julia, its result is provided to `run` whenever the `GeneratorTask`
//! is called. The frame of the `init` method is not dropped until the `GeneratorTask` is dropped,
//! rather each time `run` is called a nested scope is created, so the result of `init` can
//! contain Julia data.
//!
//! The [`CallAsync`] trait extends [`Call`], it provides methods that can be used to create new
//! Julia tasks and schedule them to run on a new thread or the main thread. These methods return
//! a `Future` that can be awaited. While the `Future` hasn't resolved the async runtime doesn't
//! block but can handle other tasks. If there's nothing to do, control of the thread is yielded
//! to Julia allowing the garbage collector and scheduler to run. Note that a task scheduled on
//! the main thread will block the runtime when it's running, so this should only be used with
//! functions that do very little computational work but are mostly waiting for something like IO.
//!
//! In order to use the async runtime, Julia must be started with three or more threads by setting
//! the `JULIA_NUM_THREADS` environment variable. If this environment variable is not set it's set
//! to `auto`. If it's set to `auto` it's assumed this results in Julia using three or more
//! threads.
//!
//! Examples that show how to use the async runtime and implement async tasks can be found in the
//! [`examples`] directory of the repository.
//!
//! [`examples`]: https://github.com/Taaitaaiger/jlrs/tree/master/examples
//! [`Julia`]: crate::Julia
//! [`AsyncGcFrame`]: crate::extensions::multitask::async_frame::AsyncGcFrame
//! [`CallAsync`]: crate::extensions::multitask::call_async::CallAsync

pub mod async_frame;
pub mod async_task;
pub mod call_async;
pub(crate) mod julia_future;
pub mod mode;
pub(crate) mod output_result_ext;
pub mod return_channel;

use crate::error::{JlrsError, JlrsResult};
use crate::extensions::multitask::async_task::{PendingTask, Task};
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
    jl_atexit_hook, jl_call0, jl_eval_string, jl_init, jl_init_with_image, jl_is_initialized,
    jl_value_t, jlrs_current_task, uv_async_send,
};
use mode::Async;
use std::cell::Cell;
use std::io::{Error as IOError, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicPtr, AtomicU8, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle as ThreadHandle};
use std::time::Duration;
use std::{
    collections::VecDeque,
    env,
    ffi::{c_void, CString},
    ptr::null_mut,
};

use self::async_task::{AsyncTask, Generator, GeneratorHandle, GeneratorTask, GenericPendingTask};
use self::return_channel::ReturnChannel;

static ASYNC_CONDITION_HANDLE: AtomicPtr<c_void> = AtomicPtr::new(null_mut());
static ASYNC_STATUS: AtomicU8 = AtomicU8::new(IN_RUST);
const IN_RUST: u8 = 0;
const IN_JULIA: u8 = 1;

/// A handle to the async runtime. It can be used to include files and send new tasks. The
/// runtime shuts down when the last handle is dropped.
///
/// All initialization methods share two arguments:
///
///  - `max_n_tasks`: the maximum number of tasks that can run at the same time.
///  - `channel_capacity`: the capacity of the channel used to communicate with the runtime. If it's 0
///    an unbounded channel is used.
///  - `recv_timeout_ms`: timeout used when receiving messages on the communication channel. If no
///    new message is received before the timeout, control of the thread is yielded to Julia if
///    uncompleted tasks exist.
#[derive(Clone)]
pub struct AsyncJulia {
    sender: AsyncStdSender<Message>,
}

// Ensure AsyncJulia can be sent to other threads.
trait RequireSendSync: Send + Sync {}
impl RequireSendSync for AsyncJulia {}

impl AsyncJulia {
    /// Initialize Julia in a new thread.
    ///
    /// This function returns an error if the `JULIA_NUM_THREADS` environment variable is set
    /// to a value smaller than 3 or some invalid value, or if Julia has already been initialized.
    /// If the `JULIA_NUM_THREADS` environment variable is set to `auto`, it's assumed that three
    /// or more threads will be available. If the environment variable is not set, it's set to
    /// `auto`. See the [struct-level] documentation for more info about this method's arguments.
    ///
    /// Safety: this method can race with other crates that try to initialize Julia at the same
    /// time.
    ///
    /// [struct-level]: crate::extensions::multitask::AsyncJulia
    pub unsafe fn init(
        max_n_tasks: usize,
        channel_capacity: usize,
        recv_timeout_ms: u64,
    ) -> JlrsResult<(Self, ThreadHandle<JlrsResult<()>>)> {
        check_threads_var()?;

        let (sender, receiver) = if channel_capacity == 0 {
            unbounded()
        } else {
            bounded(channel_capacity)
        };
        let julia = AsyncJulia { sender };
        let handle = thread::spawn(move || run_async(max_n_tasks, recv_timeout_ms, receiver));
        julia.try_set_custom_fns()?;

        Ok((julia, handle))
    }

    /// Initialize Julia as a blocking task.
    ///
    /// This function returns an error if the `JULIA_NUM_THREADS` environment variable is set
    /// to a value smaller than 3 or some invalid value, or if Julia has already been initialized.
    /// If the `JULIA_NUM_THREADS` environment variable is set to `auto`, it's assumed that three
    /// or more threads will be available. If the environment variable is not set, it's set to
    /// `auto`. See the [struct-level] documentation for more info about this method's arguments.
    ///
    /// Safety: this method can race with other crates that try to initialize Julia at the same
    /// time.
    ///
    /// [struct-level]: crate::extensions::multitask::AsyncJulia
    pub async unsafe fn init_async(
        max_n_tasks: usize,
        channel_capacity: usize,
        recv_timeout_ms: u64,
    ) -> JlrsResult<(Self, AsyncStdHandle<JlrsResult<()>>)> {
        check_threads_var()?;

        let (sender, receiver) = if channel_capacity == 0 {
            unbounded()
        } else {
            bounded(channel_capacity)
        };

        let julia = AsyncJulia { sender };
        let handle =
            task::spawn_blocking(move || run_async(max_n_tasks, recv_timeout_ms, receiver));
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
    /// value, or if Julia has already been initialized. If the `JULIA_NUM_THREADS` environment
    /// variable is set to `auto`, it's assumed that three or more threads will be available. If
    /// the environment variable is not set, it's set to `auto`. See the [struct-level]
    /// documentation for more info about this method's arguments.
    ///
    /// Safety: this method can race with other crates that try to initialize Julia at the same
    /// time.
    ///
    /// [`PackageCompiler`]: https://julialang.github.io/PackageCompiler.jl/dev/
    pub unsafe fn init_with_image<P, Q>(
        max_n_tasks: usize,
        channel_capacity: usize,
        recv_timeout_ms: u64,
        julia_bindir: P,
        image_path: Q,
    ) -> JlrsResult<(Self, ThreadHandle<JlrsResult<()>>)>
    where
        P: AsRef<Path> + Send + 'static,
        Q: AsRef<Path> + Send + 'static,
    {
        check_threads_var()?;

        let (sender, receiver) = if channel_capacity == 0 {
            unbounded()
        } else {
            bounded(channel_capacity)
        };
        let julia = AsyncJulia { sender };
        let handle = thread::spawn(move || {
            run_async_with_image(
                max_n_tasks,
                recv_timeout_ms,
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
    /// value, or if Julia has already been initialized. If the `JULIA_NUM_THREADS` environment
    /// variable is set to `auto`, it's assumed that three or more threads will be available. If
    /// the environment variable is not set, it's set to `auto`. See the [struct-level]
    /// documentation for more info about this method's arguments.
    ///
    /// Safety: this method can race with other crates that try to initialize Julia at the same
    /// time.
    ///
    /// [`PackageCompiler`]: https://julialang.github.io/PackageCompiler.jl/dev/
    pub async unsafe fn init_with_image_async<P, Q>(
        max_n_tasks: usize,
        channel_capacity: usize,
        recv_timeout_ms: u64,
        julia_bindir: P,
        image_path: Q,
    ) -> JlrsResult<(Self, AsyncStdHandle<JlrsResult<()>>)>
    where
        P: AsRef<Path> + Send + 'static,
        Q: AsRef<Path> + Send + 'static,
    {
        check_threads_var()?;

        let (sender, receiver) = if channel_capacity == 0 {
            unbounded()
        } else {
            bounded(channel_capacity)
        };
        let julia = AsyncJulia { sender };
        let handle = task::spawn_blocking(move || {
            run_async_with_image(
                max_n_tasks,
                recv_timeout_ms,
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
    pub async fn task<T, R>(&self, task: T, res_sender: R)
    where
        T: AsyncTask,
        R: ReturnChannel<T = T::Output>,
    {
        let sender = self.sender.clone();
        let msg = PendingTask::<_, _, Task>::new(task, res_sender);
        let boxed = Box::new(msg);
        self.sender
            .send(Message::Task(boxed, sender))
            .await
            .expect("Channel was closed");

        unsafe {
            wake_julia();
        }
    }

    /// Try to send a new task to the runtime, if there's no room in the channel an error is
    /// returned immediately. This method takes two arguments, the task and the sending half of a
    /// channel which is used to send the result back after the task has completed.
    pub fn try_task<T, R>(&self, task: T, res_sender: R) -> JlrsResult<()>
    where
        T: AsyncTask,
        R: ReturnChannel<T = T::Output>,
    {
        let sender = self.sender.clone();
        let msg = PendingTask::<_, _, Task>::new(task, res_sender);
        let boxed = Box::new(msg);
        self.sender
            .try_send(Message::Task(boxed, sender))
            .map(|v| {
                unsafe {
                    wake_julia();
                }
                v
            })
            .map_err(|e| match e {
                TrySendError::Full(Message::Task(t, _)) => {
                    Box::new(JlrsError::other(TrySendError::Full(t)))
                }
                TrySendError::Closed(Message::Task(t, _)) => {
                    Box::new(JlrsError::other(TrySendError::Closed(t)))
                }
                _ => unreachable!(),
            })
    }

    /// Send a new generator to the runtime, this method waits if there's no room in the channel.
    /// This method takes a single argument, the generator. It returns after
    /// [`GeneratorTask::init`] has been called, a [`GeneratorHandle`] to the generator is
    /// returned if this method completes successfully, and the error if it doesn't.
    pub async fn generator<T>(&self, task: T) -> JlrsResult<GeneratorHandle<T>>
    where
        T: GeneratorTask,
    {
        let rt_sender = self.sender.clone();
        let (init_sender, init_recv) = async_std::channel::bounded(1);
        let msg = PendingTask::<_, _, Generator>::new(task, init_sender);
        let boxed = Box::new(msg);

        self.sender
            .send(Message::Task(boxed, rt_sender))
            .await
            .expect("Channel was closed");

        unsafe {
            wake_julia();
        }

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
    pub fn try_generator<T>(&self, task: T) -> JlrsResult<GeneratorHandle<T>>
    where
        T: GeneratorTask,
    {
        let rt_sender = self.sender.clone();
        let (init_sender, init_recv) = crossbeam_channel::bounded(1);
        let msg = PendingTask::<_, _, Generator>::new(task, init_sender);
        let boxed = Box::new(msg);

        match self.sender.try_send(Message::Task(boxed, rt_sender)) {
            Ok(_) => unsafe { wake_julia() },
            Err(e) => match e {
                TrySendError::Full(Message::Task(t, _)) => {
                    Err(JlrsError::other(TrySendError::Full(t)))?;
                }
                TrySendError::Closed(Message::Task(t, _)) => {
                    Err(JlrsError::other(TrySendError::Closed(t)))?;
                }
                _ => unreachable!(),
            },
        }

        match init_recv.recv() {
            Ok(Ok(gh)) => Ok(gh),
            Ok(Err(e)) => Err(e)?,
            Err(e) => Err(JlrsError::other(e))?,
        }
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

        unsafe {
            wake_julia();
        }

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
                TrySendError::Full(Message::Include(t, _)) => {
                    Box::new(JlrsError::other(TrySendError::Full(t)))
                }
                TrySendError::Closed(Message::Include(t, _)) => {
                    Box::new(JlrsError::other(TrySendError::Closed(t)))
                }
                _ => unreachable!(),
            })
            .and_then(|_| {
                unsafe {
                    wake_julia();
                }

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
                TrySendError::Full(Message::TrySetCustomFns(_)) => {
                    Box::new(JlrsError::other(TrySendError::Full(())))
                }
                TrySendError::Closed(Message::TrySetCustomFns(_)) => {
                    Box::new(JlrsError::other(TrySendError::Closed(())))
                }
                _ => unreachable!(),
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
    Include(PathBuf, Arc<(AsyncStdMutex<Status>, AsyncStdCondvar)>),
    TryInclude(PathBuf, Arc<(Mutex<Status>, Condvar)>),
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
        let rtls = &mut *jlrs_current_task();
        stack.top[1].set(rtls.gcstack.cast());
        rtls.gcstack = stack.top[0..1].as_mut_ptr().cast();
    }
}

fn run_async(
    max_n_tasks: usize,
    recv_timeout_ms: u64,
    receiver: AsyncStdReceiver<Message>,
) -> JlrsResult<()> {
    task::block_on(async {
        unsafe {
            if jl_is_initialized() != 0 || INIT.swap(true, Ordering::SeqCst) {
                return Err(JlrsError::AlreadyInitialized.into());
            }

            jl_init();
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
            let mut func = null_mut();

            let mut n_running = 0;

            loop {
                let wait_time = if n_running > 0 {
                    recv_timeout_ms
                } else {
                    u64::MAX
                };
                match timeout(Duration::from_millis(wait_time), receiver.recv()).await {
                    Err(_) => {
                        if n_running > 0 {
                            debug_assert!(func != null_mut());
                            ASYNC_STATUS.store(IN_JULIA, Ordering::Release);
                            jl_call0(func);
                            ASYNC_STATUS.store(IN_RUST, Ordering::Release);
                        }
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
                    Ok(Ok(Message::SetCustomFns(completed))) => {
                        let stack = stacks[0].as_mut().expect("Async stack corrupted");
                        if let Some(wait_func) = set_custom_fns(stack, completed).await {
                            func = wait_func;
                        }
                    }
                    Ok(Ok(Message::TrySetCustomFns(completed))) => {
                        let stack = stacks[0].as_mut().expect("Async stack corrupted");
                        if let Some(wait_func) = try_set_custom_fns(stack, completed) {
                            func = wait_func;
                        }
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
    recv_timeout_ms: u64,
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
            let mut func = null_mut();

            loop {
                let wait_time = if n_running > 0 {
                    recv_timeout_ms
                } else {
                    u64::MAX
                };
                match timeout(Duration::from_millis(wait_time), receiver.recv()).await {
                    Err(_) => {
                        if n_running > 0 {
                            debug_assert!(func != null_mut());
                            ASYNC_STATUS.store(IN_JULIA, Ordering::Release);
                            jl_call0(func);
                            ASYNC_STATUS.store(IN_RUST, Ordering::Release);
                        }
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
                    Ok(Ok(Message::SetCustomFns(completed))) => {
                        let stack = stacks[0].as_mut().expect("Async stack corrupted");
                        if let Some(wait_func) = set_custom_fns(stack, completed).await {
                            func = wait_func;
                        }
                    }
                    Ok(Ok(Message::TrySetCustomFns(completed))) => {
                        let stack = stacks[0].as_mut().expect("Async stack corrupted");
                        if let Some(wait_func) = try_set_custom_fns(stack, completed) {
                            func = wait_func;
                        }
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

fn call_set_custom_fns(stack: &mut AsyncStackPage) -> JlrsResult<*mut jl_value_t> {
    let waiter = unsafe {
        let global = Global::new();
        let mode = Async(&stack.top[1]);
        let raw = stack.page.as_mut();
        let mut frame = GcFrame::new(raw, 2, mode);

        let waker = Value::new(&mut frame, julia_future::wake_task as *mut c_void)?;
        let jlrs_mod = Module::main(global)
            .submodule_ref("Jlrs")?
            .wrapper_unchecked();

        jlrs_mod
            .global_ref("wakerust")?
            .wrapper_unchecked()
            .set_nth_field_unchecked(0, waker);

        let dropper = Value::new(&mut frame, crate::droparray as *mut c_void)?;
        jlrs_mod
            .global_ref("droparray")?
            .wrapper_unchecked()
            .set_nth_field_unchecked(0, dropper);

        let async_cond_handle = jlrs_mod
            .global_ref("condition")?
            .value_unchecked()
            .get_raw_field_unchecked::<*mut c_void, _>("handle");

        ASYNC_CONDITION_HANDLE.store(async_cond_handle, Ordering::Release);

        jlrs_mod
            .global_ref("awaitcondition")?
            .value_unchecked()
            .data_ptr()
    };

    Ok(waiter.as_ptr().cast())
}

async fn set_custom_fns(
    stack: &mut AsyncStackPage,
    completed: Arc<(AsyncStdMutex<Status>, AsyncStdCondvar)>,
) -> Option<*mut jl_value_t> {
    let res = match call_set_custom_fns(stack) {
        Ok(v) => {
            let (lock, condvar) = &*completed;
            let mut completed = lock.lock().await;
            *completed = Status::Ok;
            condvar.notify_one();
            Some(v)
        }
        Err(e) => {
            let (lock, condvar) = &*completed;
            let mut completed = lock.lock().await;
            *completed = Status::Err(Some(e));
            condvar.notify_one();
            None
        }
    };

    res
}

fn try_set_custom_fns(
    stack: &mut AsyncStackPage,
    completed: Arc<(Mutex<Status>, Condvar)>,
) -> Option<*mut jl_value_t> {
    let res = match call_set_custom_fns(stack) {
        Ok(v) => {
            let (lock, condvar) = &*completed;
            let mut completed = lock.lock().expect("Cannot lock");
            *completed = Status::Ok;
            condvar.notify_one();
            Some(v)
        }
        Err(e) => {
            let (lock, condvar) = &*completed;
            let mut completed = lock.lock().expect("Cannot lock");
            *completed = Status::Err(Some(e));
            condvar.notify_one();
            None
        }
    };

    res
}

pub(crate) unsafe fn wake_julia() {
    if ASYNC_STATUS.load(Ordering::Acquire) == IN_JULIA {
        let handle = ASYNC_CONDITION_HANDLE.load(Ordering::Acquire);
        uv_async_send(handle.cast());
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
