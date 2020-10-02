//! Run Julia in a separate thread and execute tasks in parallel.
//!
//! While access to Julia with the C API is entirely single-threaded, it's possible to offload a
//! function call to another thread by using `Base.Threads.@spawn`. The experimental async runtime
//! offered by jlrs combines this feature with Rust's async/.await syntax.
//!
//! In order to use the async runtime, Julia must be started with more than one thread by setting
//! the `JULIA_NUM_THREADS` environment variable. In order to create tasks that can be executed
//! you must implement the [`JuliaTask`] trait.
//!
//! [`JuliaTask`]: ../traits/multitask/trait.JuliaTask.html

use crate::error::other_err;
use crate::error::{JlrsError, JlrsResult};
use crate::frame::AsyncFrame;
use crate::global::Global;
use crate::mode::Async;
use crate::stack::multitask::{MultitaskStack, TaskStack};
use crate::stack::{Dynamic, StackView};
use crate::traits::multitask::{JuliaTask, ReturnChannel};
use crate::value::module::Module;
use crate::value::Value;
use crate::INIT;
use async_std::future::timeout;
use async_std::sync::{
    channel, Condvar as AsyncStdCondvar, Mutex as AsyncStdMutex, Receiver as AsyncStdReceiver,
    RecvError, Sender as AsyncStdSender, TrySendError,
};
use async_std::task::{self, JoinHandle as AsyncStdHandle};
use jl_sys::{jl_atexit_hook, jl_gc_safepoint, jl_init_with_image__threading, jl_is_initialized};
use std::ffi::c_void;
use std::io::{Error as IOError, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle as ThreadHandle};
use std::time::Duration;

/// A handle to the async runtime. It can be used to include files and create new tasks. The
/// runtime shuts down when the last handle has been dropped. The two generic type parameters `T`
/// and `R` are the return type and return channel type respectively, which must be the same across
/// all different implementations of [`JuliaTask`] that you use.
///
/// The easiest way to get started is to use `T = Box<dyn Any + Send + Sync>`, the return channel
/// must implement the [`ReturnChannel`] trait. This trait is implemented for `Sender` from
/// `async_std` and `crossbeam_channel`.
///
/// The initialization methods share several arguments:
///
///  - `channel_capacity`: the capacity of the channel used to communicate with the runtime.
///  - `n_threads`: the number of threads that can be used to run tasks at the same time, it must
///    be less than the number of threads set with the `JULIA_NUM_THREADS` environment variable
///    (which defaults to 1).
///  - `stack_size`: the size of a stack that is created for each of the tasks threads and the
///    main thread (so `n_thread + 1` stacks with `stack_size` slots are created).
///  - `process_events_ms`: to ensure the garbage collector can run and tasks that have yielded in
///    Julia are rescheduled, events must be processed periodically when at least one task is
///    running.
///
/// [`JuliaTask`]: ../traits/multitask/trait.JuliaTask.html
/// [`ReturnChannel`]: ../traits/multitask/trait.ReturnChannel.html
#[derive(Clone)]
pub struct AsyncJulia<T, R>
where
    T: Send + Sync + 'static,
    R: ReturnChannel<T = T> + 'static,
{
    sender: AsyncStdSender<Message<T, R>>,
}

impl<T, R> AsyncJulia<T, R>
where
    T: Send + Sync + 'static,
    R: ReturnChannel<T = T> + 'static,
{
    /// Initializes Julia in a new thread, this function can only be called once. If Julia has
    /// already been initialized this will return an error, otherwise it will return a handle to
    /// the runtime and a handle to the thread. The runtime is shut down after the final handle to
    /// it has been dropped.
    ///
    /// In addition to the common arguments to initialize the async runtime, you need to provide
    /// `jlrs_path`. This is the path to `jlrs.jl`, this file is required for `AsyncJulia` to work
    /// correctly.
    ///
    /// This function is unsafe because this crate provides you with a way to execute arbitrary
    /// Julia code which can't be checked for correctness.
    pub unsafe fn init<P: AsRef<Path>>(
        channel_capacity: usize,
        n_threads: usize,
        stack_size: usize,
        process_events_ms: u64,
        jlrs_path: P,
    ) -> JlrsResult<(Self, ThreadHandle<JlrsResult<()>>)> {
        let (sender, receiver) = channel(channel_capacity);
        let julia = AsyncJulia { sender };
        let handle =
            thread::spawn(move || run_async(n_threads, stack_size, process_events_ms, receiver));
        julia.try_include(jlrs_path).map_err(other_err)?;
        julia.try_set_wake_fn().map_err(other_err)?;

        Ok((julia, handle))
    }

    /// Initializes Julia as a blocking task, this function can only be called once. If Julia was
    /// already initialized this will return an error, otherwise it will return a handle to the
    /// runtime and a handle to the task. The runtime is shut down after the final handle to it
    /// has been dropped.
    ///
    /// In addition to the common arguments to initialize the async runtime, you need to provide
    /// `jlrs_path`. This is the path to `jlrs.jl`, this file is required for `AsyncJulia` to work
    /// correctly.
    ///
    /// This function is unsafe because this crate provides you with a way to execute arbitrary
    /// Julia code which can't be checked for correctness.
    pub async unsafe fn init_async<P: AsRef<Path>>(
        channel_capacity: usize,
        n_threads: usize,
        stack_size: usize,
        process_events_ms: u64,
        jlrs_path: P,
    ) -> JlrsResult<(Self, AsyncStdHandle<JlrsResult<()>>)> {
        let (sender, receiver) = channel(channel_capacity);
        let julia = AsyncJulia { sender };
        let handle = task::spawn_blocking(move || {
            run_async(n_threads, stack_size, process_events_ms, receiver)
        });
        julia.include(jlrs_path).await?;
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
    /// absolute or a relative path to a system image. Note that `jlrs.jl` must be available in
    /// this custom image in the `Main` module, ie `Main.Jlrs` must exist.
    ///
    /// This function will return an error if either of the two paths does not exist, if Julia has
    /// already been initialized, or if `Main.Jlrs` doesn't exist. This function is unsafe because
    /// this crate provides you with a way to execute arbitrary Julia code which can't be checked
    /// for correctness.
    ///
    /// [`AsyncJulia::init`]: struct.Julia.html#init
    /// [`PackageCompiler`]: https://julialang.github.io/PackageCompiler.jl/dev/
    pub unsafe fn init_with_image<P, Q>(
        channel_capacity: usize,
        n_threads: usize,
        stack_size: usize,
        process_events_ms: u64,
        julia_bindir: P,
        image_path: Q,
    ) -> JlrsResult<(Self, ThreadHandle<JlrsResult<()>>)>
    where
        P: AsRef<Path> + Send + 'static,
        Q: AsRef<Path> + Send + 'static,
    {
        let (sender, receiver) = channel(channel_capacity);
        let julia = AsyncJulia { sender };
        let handle = thread::spawn(move || {
            run_async_with_image(
                n_threads,
                stack_size,
                process_events_ms,
                receiver,
                julia_bindir,
                image_path,
            )
        });
        julia.try_set_wake_fn().map_err(other_err)?;

        Ok((julia, handle))
    }

    /// Initializes Julia with a custom system image as as a blocking task. A custom image can be
    /// generated with the [`PackageCompiler`] package for Julia. The main advantage of using a
    /// custom image over the default one is that it allows you to avoid much of the compilation
    /// overhead often associated with Julia.
    ///
    /// In addition to the common arguments to initialize the async runtime, you need to provide
    /// `julia_bindir` and `image_path`. The first must be the absolute path to a directory that
    /// contains a compatible Julia binary (eg `${JULIA_DIR}/bin`), the second must be either an
    /// absolute or a relative path to a system image. Note that `jlrs.jl` must be available in
    /// this custom image in the `Main` module, ie `Main.Jlrs` must exist.
    ///
    /// This function will return an error if either of the two paths does not exist, if Julia has
    /// already been initialized, or if `Main.Jlrs` doesn't exist. This function is unsafe because
    /// this crate provides you with a way to execute arbitrary Julia code which can't be checked
    /// for correctness.
    ///
    /// [`AsyncJulia::init`]: struct.Julia.html#init
    /// [`PackageCompiler`]: https://julialang.github.io/PackageCompiler.jl/dev/
    pub async unsafe fn init_with_image_async<P, Q>(
        channel_capacity: usize,
        n_threads: usize,
        stack_size: usize,
        process_events_ms: u64,
        julia_bindir: P,
        image_path: Q,
    ) -> JlrsResult<(Self, AsyncStdHandle<JlrsResult<()>>)>
    where
        P: AsRef<Path> + Send + 'static,
        Q: AsRef<Path> + Send + 'static,
    {
        let (sender, receiver) = channel(channel_capacity);
        let julia = AsyncJulia { sender };
        let handle = task::spawn_blocking(move || {
            run_async_with_image(
                n_threads,
                stack_size,
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
    pub async fn new_task<D: JuliaTask<T = T, R = R>>(&self, task: D) {
        let sender = self.sender.clone();
        self.sender
            .send(Message::Task(Box::new(task), sender))
            .await
    }

    /// Try to send a new task to the runtime, if there's no room in the channel an error is
    /// returned immediately.
    pub fn try_new_task<D: JuliaTask<T = T, R = R>>(&self, task: D) -> JlrsResult<()> {
        let sender = self.sender.clone();
        self.sender
            .try_send(Message::Task(Box::new(task), sender))
            .map_err(|e| match e {
                TrySendError::Full(Message::Task(t, _)) => {
                    Box::new(other_err(TrySendError::Full(t)))
                }
                TrySendError::Disconnected(Message::Task(t, _)) => {
                    Box::new(other_err(TrySendError::Disconnected(t)))
                }
                _ => unreachable!(),
            })
    }

    /// Include a Julia file. This method waits until the call `Main.include` in Julia has been
    /// completed. It returns an error if the path does not exist or the call to `Main.include`
    /// throws an exception.
    pub async fn include<P: AsRef<Path>>(&self, path: P) -> JlrsResult<()> {
        if !path.as_ref().exists() {
            return Err(JlrsError::IncludeNotFound(path.as_ref().to_string_lossy().into()).into());
        }

        let completed = Arc::new((AsyncStdMutex::new(Status::Pending), AsyncStdCondvar::new()));
        self.sender
            .send(Message::Include(
                path.as_ref().to_path_buf(),
                completed.clone(),
            ))
            .await;

        let (lock, cvar) = &*completed;
        let mut completed = lock.lock().await;
        while (&*completed).is_pending() {
            completed = cvar.wait(completed).await;
        }

        (&mut *completed).as_jlrs_result()
    }

    /// Include a Julia file. This method waits until the call `Main.include` in Julia has been
    /// completed. It returns an error if the path does not exist, the channel is full, or the
    /// call to `Main.include` throws an exception.
    pub fn try_include<P: AsRef<Path>>(&self, path: P) -> JlrsResult<()> {
        if !path.as_ref().exists() {
            return Err(JlrsError::IncludeNotFound(path.as_ref().to_string_lossy().into()).into());
        }

        let completed = Arc::new((Mutex::new(Status::Pending), Condvar::new()));
        self.sender
            .try_send(Message::TryInclude(
                path.as_ref().to_path_buf(),
                completed.clone(),
            ))
            .map_err(|e| match e {
                TrySendError::Full(Message::Include(t, _)) => {
                    Box::new(other_err(TrySendError::Full(t)))
                }
                TrySendError::Disconnected(Message::Include(t, _)) => {
                    Box::new(other_err(TrySendError::Disconnected(t)))
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
        self.sender.capacity()
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
                    Box::new(other_err(TrySendError::Full(())))
                }
                TrySendError::Disconnected(Message::TrySetWakeFn(_)) => {
                    Box::new(other_err(TrySendError::Disconnected(())))
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
            .await;

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
        Box<dyn JuliaTask<T = T, R = R>>,
        AsyncStdSender<Message<T, R>>,
    ),
    Include(PathBuf, Arc<(AsyncStdMutex<Status>, AsyncStdCondvar)>),
    TryInclude(PathBuf, Arc<(Mutex<Status>, Condvar)>),
    Complete(Wrapper, AsyncStdSender<Message<T, R>>),
    SetWakeFn(Arc<(AsyncStdMutex<Status>, AsyncStdCondvar)>),
    TrySetWakeFn(Arc<(Mutex<Status>, Condvar)>),
}

// NB: I'm not sure if this is sound, but the TaskStack is never sent to (or used from) another
// thread.
struct Wrapper(usize, TaskStack);
unsafe impl Send for Wrapper {}

fn run_task<T: Send + Sync + 'static, R>(
    mut jl_task: Box<dyn JuliaTask<T = T, R = R>>,
    task_idx: usize,
    mut task_stack: TaskStack,
    rt_sender: AsyncStdSender<Message<T, R>>,
) -> AsyncStdHandle<()>
where
    R: ReturnChannel<T = T> + 'static,
{
    unsafe {
        task::spawn_local(async move {
            let mut tv = StackView::<Async, Dynamic>::new(&mut task_stack.raw);

            match tv.new_frame() {
                Ok(frame_idx) => {
                    let global = Global::new();
                    let mut frame = AsyncFrame {
                        idx: frame_idx,
                        memory: tv,
                        len: 0,
                    };
                    let res = jl_task.run(global, &mut frame).await;

                    if let Some(sender) = jl_task.return_channel() {
                        sender.send(res).await;
                    }
                }
                Err(e) => {
                    if let Some(sender) = jl_task.return_channel() {
                        sender.send(Err(e)).await;
                    }
                }
            }

            let rt_c = rt_sender.clone();
            rt_sender
                .send(Message::Complete(Wrapper(task_idx, task_stack), rt_c))
                .await;
        })
    }
}

fn run_async<T, R>(
    n_threads: usize,
    stack_size: usize,
    process_events_ms: u64,
    receiver: AsyncStdReceiver<Message<T, R>>,
) -> JlrsResult<()>
where
    T: Send + Sync + 'static,
    R: ReturnChannel<T = T> + 'static,
{
    task::block_on(async {
        let mut mt_stack: MultitaskStack<T, R> = unsafe {
            if jl_is_initialized() != 0 || INIT.swap(true, Ordering::SeqCst) {
                return Err(JlrsError::AlreadyInitialized.into());
            }

            jl_sys::jl_init();
            MultitaskStack::new(n_threads, stack_size)
        };

        loop {
            match timeout(Duration::from_millis(process_events_ms), receiver.recv()).await {
                Err(_) => unsafe {
                    // periodically insert a safepoint so the GC can run when nothing is happening on
                    // the main thread but tasks are active
                    if mt_stack.n > 0 {
                        // jl_process_events inserts a safepoint
                        jl_sys::jl_process_events();
                    }
                },
                Ok(Ok(Message::Task(jl_task, sender))) => {
                    if let Some((task_idx, task_stack)) = mt_stack.acquire_task_frame() {
                        mt_stack.n += 1;
                        mt_stack.running[task_idx] =
                            Some(run_task(jl_task, task_idx, task_stack, sender));
                    } else {
                        mt_stack.add_pending(jl_task);
                    }
                }
                Ok(Ok(Message::Complete(Wrapper(task_idx, task_stack), sender))) => {
                    if let Some(jl_task) = mt_stack.pop_pending() {
                        mt_stack.running[task_idx] =
                            Some(run_task(jl_task, task_idx, task_stack, sender));
                    } else {
                        mt_stack.n -= 1;
                        mt_stack.running[task_idx] = None;
                        mt_stack.return_task_frame(task_idx, task_stack);
                    }
                }
                Ok(Ok(Message::Include(path, completed))) => {
                    include(&mut mt_stack.raw, path, completed).await
                }
                Ok(Ok(Message::TryInclude(path, completed))) => {
                    try_include(&mut mt_stack.raw, path, completed)
                }
                Ok(Ok(Message::SetWakeFn(completed))) => {
                    set_wake_fn(&mut mt_stack.raw, completed).await
                }
                Ok(Ok(Message::TrySetWakeFn(completed))) => {
                    try_set_wake_fn(&mut mt_stack.raw, completed)
                }
                Ok(Err(RecvError)) => break,
            }
        }

        // Wait for tasks to finish
        for running in mt_stack.running.iter_mut() {
            if let Some(handle) = running.take() {
                handle.await;
            }
        }

        unsafe {
            jl_atexit_hook(0);
        }

        Ok(())
    })
}

fn run_async_with_image<T, R, P, Q>(
    n_threads: usize,
    stack_size: usize,
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
        let mut mt_stack: MultitaskStack<T, R> = unsafe {
            if jl_is_initialized() != 0 || INIT.swap(true, Ordering::SeqCst) {
                return Err(JlrsError::AlreadyInitialized.into());
            }

            let julia_bindir_str = julia_bindir.as_ref().to_string_lossy().to_string();
            let image_path_str = image_path.as_ref().to_string_lossy().to_string();

            if !julia_bindir.as_ref().exists() {
                let io_err = IOError::new(ErrorKind::NotFound, julia_bindir_str);
                return Err(other_err(io_err))?;
            }

            if !image_path.as_ref().exists() {
                let io_err = IOError::new(ErrorKind::NotFound, image_path_str);
                return Err(other_err(io_err))?;
            }

            let bindir = std::ffi::CString::new(julia_bindir_str).unwrap();
            let im_rel_path = std::ffi::CString::new(image_path_str).unwrap();

            jl_init_with_image__threading(bindir.as_ptr(), im_rel_path.as_ptr());
            MultitaskStack::new(n_threads, stack_size)
        };

        loop {
            match timeout(Duration::from_millis(process_events_ms), receiver.recv()).await {
                Err(_) => unsafe {
                    // periodically insert a safepoint so the GC can run when nothing is happening on
                    // the main thread but tasks are active
                    if mt_stack.n > 0 {
                        jl_gc_safepoint();
                    }
                },
                Ok(Ok(Message::Task(jl_task, sender))) => {
                    if let Some((task_idx, task_stack)) = mt_stack.acquire_task_frame() {
                        mt_stack.n += 1;
                        mt_stack.running[task_idx] =
                            Some(run_task(jl_task, task_idx, task_stack, sender));
                    } else {
                        mt_stack.add_pending(jl_task);
                    }
                }
                Ok(Ok(Message::Complete(Wrapper(task_idx, task_stack), sender))) => {
                    if let Some(jl_task) = mt_stack.pop_pending() {
                        mt_stack.running[task_idx] =
                            Some(run_task(jl_task, task_idx, task_stack, sender));
                    } else {
                        mt_stack.n -= 1;
                        mt_stack.running[task_idx] = None;
                        mt_stack.return_task_frame(task_idx, task_stack);
                    }
                }
                Ok(Ok(Message::Include(path, completed))) => {
                    include(&mut mt_stack.raw, path, completed).await
                }
                Ok(Ok(Message::TryInclude(path, completed))) => {
                    try_include(&mut mt_stack.raw, path, completed)
                }
                Ok(Ok(Message::SetWakeFn(completed))) => {
                    set_wake_fn(&mut mt_stack.raw, completed).await
                }
                Ok(Ok(Message::TrySetWakeFn(completed))) => {
                    try_set_wake_fn(&mut mt_stack.raw, completed)
                }
                Ok(Err(RecvError)) => break,
            }
        }

        // Wait for tasks to finish
        for pending in mt_stack.running.iter_mut() {
            if let Some(handle) = pending.take() {
                handle.await;
            }
        }

        unsafe {
            jl_atexit_hook(0);
        }

        Ok(())
    })
}

fn call_set_wake_fn(stack: &mut [*mut c_void]) -> JlrsResult<()> {
    unsafe {
        let global = Global::new();
        let mut view = StackView::<Async, Dynamic>::new(stack);
        let idx = view.new_frame()?;

        let mut frame = AsyncFrame {
            idx,
            len: 0,
            memory: view,
        };

        let waker = Value::new(&mut frame, crate::julia_future::wake_task as *mut c_void)?;
        Module::main(global)
            .submodule("Jlrs")?
            .global("wakerust")?
            .set_nth_field(0, waker)?;
    }

    Ok(())
}

async fn set_wake_fn(
    stacks: &mut [Option<TaskStack>],
    completed: Arc<(AsyncStdMutex<Status>, AsyncStdCondvar)>,
) {
    let idx = stacks.len() - 1;
    let mut stack = stacks[idx].take().expect("GC stack is corrupted.");

    let set_wake_result = call_set_wake_fn(&mut stack.raw);

    stacks[idx] = Some(stack);

    {
        let (lock, condvar) = &*completed;
        let mut completed = lock.lock().await;
        if set_wake_result.is_ok() {
            *completed = Status::Ok;
        } else {
            *completed = Status::Err(Some(set_wake_result.unwrap_err()));
        }
        condvar.notify_one();
    }
}

fn try_set_wake_fn(stacks: &mut [Option<TaskStack>], completed: Arc<(Mutex<Status>, Condvar)>) {
    let idx = stacks.len() - 1;
    let mut stack = stacks[idx].take().expect("GC stack is corrupted.");

    let set_wake_result = call_set_wake_fn(&mut stack.raw);

    stacks[idx] = Some(stack);

    {
        let (lock, condvar) = &*completed;
        let mut completed = lock.lock().expect("Cannot lock");
        if set_wake_result.is_ok() {
            *completed = Status::Ok;
        } else {
            *completed = Status::Err(Some(set_wake_result.unwrap_err()));
        }
        condvar.notify_one();
    }
}

fn call_include(stack: &mut [*mut c_void], path: PathBuf) -> JlrsResult<()> {
    unsafe {
        let global = Global::new();
        let mut view = StackView::<Async, Dynamic>::new(stack);
        let idx = view.new_frame()?;

        let mut frame = AsyncFrame {
            idx,
            len: 0,
            memory: view,
        };

        match path.to_str() {
            Some(path) => {
                let path = Value::new(&mut frame, path)?;
                Module::main(global)
                    .function("include")?
                    .call1(&mut frame, path)?
                    .map_err(|_e| {
                        crate::error::exception::<Value>("Include error".into()).unwrap_err()
                    })?;
            }
            None => {}
        }

        Ok(())
    }
}

async fn include(
    stacks: &mut [Option<TaskStack>],
    path: PathBuf,
    completed: Arc<(AsyncStdMutex<Status>, AsyncStdCondvar)>,
) {
    let idx = stacks.len() - 1;
    let include_result = {
        let mut stack = stacks[idx].take().expect("GC stack is corrupted.");
        let res = call_include(&mut stack.raw, path);
        stacks[idx] = Some(stack);
        res
    };

    {
        let (lock, condvar) = &*completed;
        let mut completed = lock.lock().await;
        if include_result.is_ok() {
            *completed = Status::Ok;
        } else {
            *completed = Status::Err(Some(include_result.unwrap_err()));
        }

        condvar.notify_one();
    }
}

fn try_include(
    stacks: &mut [Option<TaskStack>],
    path: PathBuf,
    completed: Arc<(Mutex<Status>, Condvar)>,
) {
    let idx = stacks.len() - 1;
    let include_result = {
        let mut stack = stacks[idx].take().expect("GC stack is corrupted.");
        let res = call_include(&mut stack.raw, path);
        stacks[idx] = Some(stack);
        res
    };

    {
        let (lock, condvar) = &*completed;
        let mut completed = lock.lock().expect("Cannot lock");
        if include_result.is_ok() {
            *completed = Status::Ok;
        } else {
            *completed = Status::Err(Some(include_result.unwrap_err()));
        }

        condvar.notify_one();
    }
}
