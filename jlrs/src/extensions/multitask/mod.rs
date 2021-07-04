//! Run Julia in a separate thread and execute tasks in parallel.
//!
//! While access to Julia with the C API is entirely single-threaded, it's possible to offload a
//! function call to another thread by using `Threads.@spawn`. The experimental async runtime
//! offered by jlrs combines this feature with Rust's async/.await syntax.
//!
//! In order to use the async runtime, Julia must be started with three or more threads by setting
//! the `JULIA_NUM_THREADS` environment variable. In order to create tasks that can be executed
//! you must implement the [`AsyncTask`] trait.
//!
//! Examples that show how to use the async runtime and implement async tasks can be found in the
//! [`examples`] directory of the repository.
//!
//! [`examples`]: https://github.com/Taaitaaiger/jlrs/tree/master/examples

pub mod async_frame;
pub mod async_task;
pub mod call_async;
pub(crate) mod julia_future;
pub mod mode;
pub(crate) mod output_result_ext;

use crate::memory::global::Global;
use crate::wrappers::ptr::module::Module;
use crate::wrappers::ptr::string::JuliaString;
use crate::wrappers::ptr::value::Value;
use crate::{
    error::{JlrsError, JlrsResult},
    memory::stack_page::StackPage,
};
use crate::{memory::frame::GcFrame, wrappers::ptr::call::Call};
use crate::{INIT, JLRS_JL};
use async_std::channel::{
    bounded, Receiver as AsyncStdReceiver, RecvError, Sender as AsyncStdSender, TrySendError,
};
use async_std::future::timeout;
use async_std::sync::{Condvar as AsyncStdCondvar, Mutex as AsyncStdMutex};
use async_std::task::{self, JoinHandle as AsyncStdHandle};
use async_task::{AsyncTask, ReturnChannel};
use jl_sys::{
    jl_atexit_hook, jl_eval_string, jl_init, jl_init_with_image, jl_is_initialized,
    jlrs_current_task,
};
use mode::Async;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle as ThreadHandle};
use std::time::Duration;
use std::{
    cell::Cell,
    io::{Error as IOError, ErrorKind},
};
use std::{
    collections::VecDeque,
    env,
    ffi::{c_void, CString},
    ptr::null_mut,
};

use self::async_frame::AsyncGcFrame;

/// A handle to the async runtime. It can be used to include files and send new tasks. The
/// runtime shuts down when the last handle is dropped. The two generic type parameters `T`
/// and `R` are the return type and return channel type respectively, which must be the same across
/// all different implementations of [`AsyncTask`] that you use.
///
/// The easiest way to get started is to use `T = Box<dyn Any + Send + Sync>`, the return channel
/// must implement the [`ReturnChannel`] trait. This trait is implemented for `Sender` from
/// `async_std` and `crossbeam_channel`.
///
/// All initialization methods share two arguments:
///
///  - `channel_capacity`: the capacity of the channel used to communicate with the runtime.
///  - `process_events_ms`: to ensure the garbage collector can run and tasks that have yielded in
///    Julia are rescheduled, events must be processed periodically when at least one task is
///    running.
#[derive(Clone)]
pub struct AsyncJulia<T, R>
where
    T: Send + Sync + 'static,
    R: ReturnChannel<T = T>,
{
    sender: AsyncStdSender<Message<T, R>>,
}

// Ensure AsyncJulia can be sent to other threads.
trait RequireSendSync: Send + Sync {}
impl<T, R> RequireSendSync for AsyncJulia<T, R>
where
    T: Send + Sync + 'static,
    R: ReturnChannel<T = T>,
{
}

impl<T, R> AsyncJulia<T, R>
where
    T: Send + Sync + 'static,
    R: ReturnChannel<T = T>,
{
    /// Initialize Julia in a new thread.
    ///
    /// This function will return an error if the  `JULIA_NUM_THREADS` environment variable is not
    /// set or set to a value smaller than 3, or if Julia has already been initialized. It is
    /// unsafe because this crate provides you with a way to execute arbitrary Julia code which
    /// can't be checked for correctness.
    pub unsafe fn init(
        channel_capacity: usize,
        process_events_ms: u64,
    ) -> JlrsResult<(Self, ThreadHandle<JlrsResult<()>>)> {
        let n_threads = env::var("JULIA_NUM_THREADS")
            .map_err(JlrsError::other)?
            .parse::<usize>()
            .map_err(JlrsError::other)?;

        if n_threads <= 2 {
            Err(JlrsError::MoreThreadsRequired)?;
        }

        let (sender, receiver) = bounded(channel_capacity);
        let julia = AsyncJulia { sender };
        let handle = thread::spawn(move || run_async(n_threads - 1, process_events_ms, receiver));
        julia.try_set_wake_fn()?;

        Ok((julia, handle))
    }

    /// Initialize Julia as a blocking task.
    ///
    /// This function will return an error if the  `JULIA_NUM_THREADS` environment variable is not
    /// set or set to a value smaller than 3, or if Julia has already been initialized. It is
    /// unsafe because this crate provides you with a way to execute arbitrary Julia code which
    /// can't be checked for correctness.
    pub async unsafe fn init_async(
        channel_capacity: usize,
        process_events_ms: u64,
    ) -> JlrsResult<(Self, AsyncStdHandle<JlrsResult<()>>)> {
        let n_threads = env::var("JULIA_NUM_THREADS")
            .map_err(JlrsError::other)?
            .parse::<usize>()
            .map_err(JlrsError::other)?;

        if n_threads <= 2 {
            Err(JlrsError::MoreThreadsRequired)?;
        }

        let (sender, receiver) = bounded(channel_capacity);
        let julia = AsyncJulia { sender };
        let handle =
            task::spawn_blocking(move || run_async(n_threads - 1, process_events_ms, receiver));
        julia.set_wake_fn().await?;

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
    /// This function will return an error if either of the two paths doesn't  exist, if the
    /// `JULIA_NUM_THREADS` environment variable is not set or set to a value smaller than 3, or
    /// if Julia has already been initialized. It is unsafe because this crate provides you with
    /// a way to execute arbitrary Julia code which can't be checked for correctness.
    ///
    /// [`PackageCompiler`]: https://julialang.github.io/PackageCompiler.jl/dev/
    pub unsafe fn init_with_image<P, Q>(
        channel_capacity: usize,
        process_events_ms: u64,
        julia_bindir: P,
        image_path: Q,
    ) -> JlrsResult<(Self, ThreadHandle<JlrsResult<()>>)>
    where
        P: AsRef<Path> + Send + 'static,
        Q: AsRef<Path> + Send + 'static,
    {
        let n_threads = env::var("JULIA_NUM_THREADS")
            .map_err(JlrsError::other)?
            .parse::<usize>()
            .map_err(JlrsError::other)?;

        if n_threads <= 2 {
            Err(JlrsError::MoreThreadsRequired)?;
        }

        let (sender, receiver) = bounded(channel_capacity);
        let julia = AsyncJulia { sender };
        let handle = thread::spawn(move || {
            run_async_with_image(
                n_threads - 1,
                process_events_ms,
                receiver,
                julia_bindir,
                image_path,
            )
        });
        julia.try_set_wake_fn()?;

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
    /// This function will return an error if either of the two paths doesn't  exist, if the
    /// `JULIA_NUM_THREADS` environment variable is not set or set to a value smaller than 3, or
    /// if Julia has already been initialized. It is unsafe because this crate provides you with
    /// a way to execute arbitrary Julia code which can't be checked for correctness.
    ///
    /// [`PackageCompiler`]: https://julialang.github.io/PackageCompiler.jl/dev/
    pub async unsafe fn init_with_image_async<P, Q>(
        channel_capacity: usize,
        process_events_ms: u64,
        julia_bindir: P,
        image_path: Q,
    ) -> JlrsResult<(Self, AsyncStdHandle<JlrsResult<()>>)>
    where
        P: AsRef<Path> + Send + 'static,
        Q: AsRef<Path> + Send + 'static,
    {
        let n_threads = env::var("JULIA_NUM_THREADS")
            .map_err(JlrsError::other)?
            .parse::<usize>()
            .map_err(JlrsError::other)?;

        if n_threads <= 2 {
            Err(JlrsError::MoreThreadsRequired)?;
        }

        let (sender, receiver) = bounded(channel_capacity);
        let julia = AsyncJulia { sender };
        let handle = task::spawn_blocking(move || {
            run_async_with_image(
                n_threads - 1,
                process_events_ms,
                receiver,
                julia_bindir,
                image_path,
            )
        });
        julia.set_wake_fn().await?;

        Ok((julia, handle))
    }

    /// Send a new task to the runtime, this method waits until there's room in the channel.
    pub async fn task<D: AsyncTask<T = T, R = R>>(&self, task: D) {
        let sender = self.sender.clone();
        self.sender
            .send(Message::Task(Box::new(task), sender))
            .await
            .expect("Channel was closed");
    }

    /// Try to send a new task to the runtime, if there's no room in the channel an error is
    /// returned immediately.
    pub fn try_task<D: AsyncTask<T = T, R = R>>(&self, task: D) -> JlrsResult<()> {
        let sender = self.sender.clone();
        self.sender
            .try_send(Message::Task(Box::new(task), sender))
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

    /// Include a Julia file. This method waits until the call `Main.include` in Julia has been
    /// completed. It returns an error if the path doesn't  exist or the call to `Main.include`
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
                TrySendError::Full(Message::Include(t, _)) => {
                    Box::new(JlrsError::other(TrySendError::Full(t)))
                }
                TrySendError::Closed(Message::Include(t, _)) => {
                    Box::new(JlrsError::other(TrySendError::Closed(t)))
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

    fn try_set_wake_fn(&self) -> JlrsResult<()> {
        let completed = Arc::new((Mutex::new(Status::Pending), Condvar::new()));
        self.sender
            .try_send(Message::TrySetWakeFn(completed.clone()))
            .map_err(|e| match e {
                TrySendError::Full(Message::TrySetWakeFn(_)) => {
                    Box::new(JlrsError::other(TrySendError::Full(())))
                }
                TrySendError::Closed(Message::TrySetWakeFn(_)) => {
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

    async fn set_wake_fn(&self) -> JlrsResult<()> {
        let completed = Arc::new((AsyncStdMutex::new(Status::Pending), AsyncStdCondvar::new()));
        self.sender
            .send(Message::SetWakeFn(completed.clone()))
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

enum Status {
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

enum Message<T, R> {
    Task(
        Box<dyn AsyncTask<T = T, R = R>>,
        AsyncStdSender<Message<T, R>>,
    ),
    Include(PathBuf, Arc<(AsyncStdMutex<Status>, AsyncStdCondvar)>),
    TryInclude(PathBuf, Arc<(Mutex<Status>, Condvar)>),
    Complete(usize, Box<AsyncStackPage>),
    SetWakeFn(Arc<(AsyncStdMutex<Status>, AsyncStdCondvar)>),
    TrySetWakeFn(Arc<(Mutex<Status>, Condvar)>),
}

#[derive(Debug)]
struct AsyncStackPage {
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

fn run_async<T, R>(
    n_threads: usize,
    process_events_ms: u64,
    receiver: AsyncStdReceiver<Message<T, R>>,
) -> JlrsResult<()>
where
    T: Send + Sync + 'static,
    R: ReturnChannel<T = T> + 'static,
{
    task::block_on(async {
        unsafe {
            if jl_is_initialized() != 0 || INIT.swap(true, Ordering::SeqCst) {
                return Err(JlrsError::AlreadyInitialized.into());
            }

            jl_init();
            let jlrs_jl = CString::new(JLRS_JL).expect("Invalid Jlrs module");
            jl_eval_string(jlrs_jl.as_ptr());

            let mut free_stacks = VecDeque::with_capacity(n_threads);
            for i in 1..n_threads {
                free_stacks.push_back(i);
            }

            let mut stacks = Vec::with_capacity(n_threads);
            for _ in 0..n_threads {
                stacks.push(Some(AsyncStackPage::new()));
            }
            link_stacks(&mut stacks);
            let mut stacks = stacks.into_boxed_slice();

            let mut running_tasks = Vec::with_capacity(n_threads);
            for _ in 0..n_threads {
                running_tasks.push(None);
            }
            let mut running_tasks = running_tasks.into_boxed_slice();
            let mut pending_tasks = VecDeque::new();

            let mut n_running = 0usize;

            loop {
                match timeout(Duration::from_millis(process_events_ms), receiver.recv()).await {
                    Err(_) => {
                        // periodically insert a safepoint so the GC can run when nothing is happening on
                        // the main thread but tasks are active
                        if n_running > 0 {
                            // jl_process_events inserts a safepoint
                            jl_sys::jl_process_events();
                        }
                    }
                    Ok(Ok(Message::Task(jl_task, sender))) => {
                        if let Some(idx) = free_stacks.pop_front() {
                            n_running += 1;
                            let stack = stacks[idx].take().expect("Async stack corrupted");
                            let task = run_task(jl_task, idx, stack, sender);
                            running_tasks[idx] = Some(task);
                        } else {
                            pending_tasks.push_back((jl_task, sender));
                        }
                    }
                    Ok(Ok(Message::Complete(idx, stack))) => {
                        if let Some((jl_task, sender)) = pending_tasks.pop_front() {
                            let task = run_task(jl_task, idx, stack, sender);
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
                    Ok(Ok(Message::SetWakeFn(completed))) => {
                        let stack = stacks[0].as_mut().expect("Async stack corrupted");
                        set_wake_fn(stack, completed).await
                    }
                    Ok(Ok(Message::TrySetWakeFn(completed))) => {
                        let stack = stacks[0].as_mut().expect("Async stack corrupted");
                        try_set_wake_fn(stack, completed)
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

fn run_async_with_image<T, R, P, Q>(
    n_threads: usize,
    process_events_ms: u64,
    receiver: AsyncStdReceiver<Message<T, R>>,
    julia_bindir: P,
    image_path: Q,
) -> JlrsResult<()>
where
    T: Send + Sync + 'static,
    R: ReturnChannel<T = T> + 'static,
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

            let mut free_stacks = VecDeque::with_capacity(n_threads);
            for i in 1..n_threads {
                free_stacks.push_back(i);
            }

            let mut stacks = Vec::with_capacity(n_threads);
            for _ in 0..n_threads {
                stacks.push(Some(AsyncStackPage::new()));
            }
            link_stacks(&mut stacks);
            let mut stacks = stacks.into_boxed_slice();

            let mut running_tasks = Vec::with_capacity(n_threads);
            for _ in 0..n_threads {
                running_tasks.push(None);
            }
            let mut running_tasks = running_tasks.into_boxed_slice();
            let mut pending_tasks = VecDeque::new();

            let mut n_running = 0usize;

            loop {
                match timeout(Duration::from_millis(process_events_ms), receiver.recv()).await {
                    Err(_) => {
                        // periodically insert a safepoint so the GC can run when nothing is happening on
                        // the main thread but tasks are active
                        if n_running > 0 {
                            // jl_process_events inserts a safepoint
                            jl_sys::jl_process_events();
                        }
                    }
                    Ok(Ok(Message::Task(jl_task, sender))) => {
                        if let Some(idx) = free_stacks.pop_front() {
                            n_running += 1;
                            let stack = stacks[idx].take().expect("Async stack corrupted");
                            let task = run_task(jl_task, idx, stack, sender);
                            running_tasks[idx] = Some(task);
                        } else {
                            pending_tasks.push_back((jl_task, sender));
                        }
                    }
                    Ok(Ok(Message::Complete(idx, stack))) => {
                        if let Some((jl_task, sender)) = pending_tasks.pop_front() {
                            let task = run_task(jl_task, idx, stack, sender);
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
                    Ok(Ok(Message::SetWakeFn(completed))) => {
                        let stack = stacks[0].as_mut().expect("Async stack corrupted");
                        set_wake_fn(stack, completed).await
                    }
                    Ok(Ok(Message::TrySetWakeFn(completed))) => {
                        let stack = stacks[0].as_mut().expect("Async stack corrupted");
                        try_set_wake_fn(stack, completed)
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

fn run_task<T: Send + Sync + 'static, R>(
    mut jl_task: Box<dyn AsyncTask<T = T, R = R>>,
    task_idx: usize,
    mut stack: Box<AsyncStackPage>,
    rt_sender: AsyncStdSender<Message<T, R>>,
) -> AsyncStdHandle<()>
where
    R: ReturnChannel<T = T> + 'static,
{
    unsafe {
        task::spawn_local(async move {
            let res = {
                let mode = Async(&stack.top[1]);
                let raw = stack.page.as_mut();
                let mut frame = AsyncGcFrame::new(raw, 0, mode);
                let global = Global::new();
                jl_task.run(global, &mut frame).await
            };

            if let Some(sender) = jl_task.return_channel() {
                sender.send(res).await;
            }

            rt_sender
                .send(Message::Complete(task_idx, stack))
                .await
                .expect("Channel was closed");
        })
    }
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

fn call_set_wake_fn(stack: &mut AsyncStackPage) -> JlrsResult<()> {
    unsafe {
        let global = Global::new();
        let mode = Async(&stack.top[1]);
        let raw = stack.page.as_mut();
        let mut frame = GcFrame::new(raw, 2, mode);

        let waker = Value::new(&mut frame, julia_future::wake_task as *mut c_void)?;
        Module::main(global)
            .submodule_ref("Jlrs")?
            .wrapper_unchecked()
            .global_ref("wakerust")?
            .wrapper_unchecked()
            .set_nth_field_unchecked(0, waker);

        let dropper = Value::new(&mut frame, crate::droparray as *mut c_void)?;
        Module::main(global)
            .submodule_ref("Jlrs")?
            .wrapper_unchecked()
            .global_ref("droparray")?
            .wrapper_unchecked()
            .set_nth_field_unchecked(0, dropper);
    }

    Ok(())
}

async fn set_wake_fn(
    stack: &mut AsyncStackPage,
    completed: Arc<(AsyncStdMutex<Status>, AsyncStdCondvar)>,
) {
    let res = call_set_wake_fn(stack);

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

fn try_set_wake_fn(stack: &mut AsyncStackPage, completed: Arc<(Mutex<Status>, Condvar)>) {
    let res = call_set_wake_fn(stack);

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
