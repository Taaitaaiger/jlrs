use crate::error::{AllocError, JlrsError, JlrsResult};
use crate::frame::{DynamicFrame, FrameIdx};
use crate::global::Global;
use crate::stack::{Dynamic, StackView};
use crate::sync::Async;
use crate::value::module::Module;
use crate::value::{CallResult, Value};
use async_std::future::timeout;
use async_std::sync::{
    channel, Receiver as AsyncStdReceiver, RecvError, Sender as AsyncStdSender, TrySendError,
};
use async_std::task::{self, JoinHandle};
use async_trait::async_trait;
use crossbeam_channel::Sender as CrossbeamSender;
use futures::task::{Context, Poll, Waker};
use futures::Future;
use jl_sys::{jl_atexit_hook, jl_gc_safepoint, jl_get_ptls_states};
use std::collections::VecDeque;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

const MAX_SIZE: usize = 8;

/// The `JuliaTask` trait is used to define tasks that can be sent to and executed by
/// `AsyncJulia`.
#[async_trait(?Send)]
pub trait JuliaTask: Send + Sync + 'static {
    /// The type of the result of this task. Must be the same across all implementations.
    type T: 'static + Send + Sync;

    /// The type of the sender that is used to send the result of this task back to the original
    /// caller. Must be the same across all implementations.
    type R: ReturnChannel<T = Self::T>;

    /// The entrypoint of your task. You can use the `Global` and `AsyncFrame` to call arbitrary
    /// functions from Julia. Additionally, `Values::call_async` can be used to
    async fn run<'base>(
        &mut self,
        global: Global<'base>,
        frame: &mut AsyncFrame<'base>,
    ) -> JlrsResult<Self::T>;

    fn return_channel(&self) -> Option<&Self::R> {
        None
    }
}

#[async_trait]
pub trait ReturnChannel {
    type T: Send + Sync + 'static;
    async fn send(&self, response: JlrsResult<Self::T>);
}

#[async_trait]
impl<T: Send + Sync + 'static> ReturnChannel for AsyncStdSender<JlrsResult<T>> {
    type T = T;
    async fn send(&self, response: JlrsResult<Self::T>) {
        self.send(response).await
    }
}

#[async_trait]
impl<T: Send + Sync + 'static> ReturnChannel for CrossbeamSender<JlrsResult<T>> {
    type T = T;
    async fn send(&self, response: JlrsResult<Self::T>) {
        self.send(response).ok();
    }
}

struct TaskState<'frame, 'data> {
    completed: bool,
    waker: Option<Waker>,
    task: Option<crate::value::task::Task<'frame>>,
    _marker: PhantomData<&'data ()>,
}

pub struct JuliaFuture<'frame, 'data> {
    shared_state: Arc<Mutex<TaskState<'frame, 'data>>>,
}

unsafe extern "C" fn wake_task(state: *mut Mutex<TaskState>) {
    // the strong count is 1.
    let state = std::mem::ManuallyDrop::new(Arc::from_raw(state));
    let shared_state = state.lock();
    match shared_state {
        Ok(mut state) => {
            state.completed = true;
            state.waker.take().map(|waker| waker.wake());
        }
        Err(_) => (),
    }
}

impl<'frame, 'data> JuliaFuture<'frame, 'data> {
    pub fn new<'value, V>(
        frame: &mut AsyncFrame<'frame>,
        func: Value,
        mut values: V,
    ) -> JlrsResult<Self>
    where
        V: AsMut<[Value<'value, 'data>]>,
    {
        let shared_state = Arc::new(Mutex::new(TaskState {
            completed: false,
            waker: None,
            task: None,
            _marker: PhantomData,
        }));

        unsafe {
            let values = values.as_mut();
            let ptr = Arc::as_ptr(&shared_state) as *mut c_void;
            let ptr_boxed = Value::new(frame, ptr)?;

            let mut vals: smallvec::SmallVec<[Value; MAX_SIZE]> =
                smallvec::SmallVec::with_capacity(2 + values.len());

            vals.push(func);
            vals.push(ptr_boxed);
            vals.extend_from_slice(values);

            let global = Global::new();
            let task = Module::main(global)
                .submodule("Jlrs")?
                .function("asynccall")?
                .call(frame, vals)?
                .or(crate::error::exception(
                    "asynccall threw an exception".into(),
                ))?
                .cast::<crate::value::task::Task>()?;

            {
                let locked = shared_state.lock();
                match locked {
                    Ok(mut data) => data.task = Some(task),
                    _ => crate::error::exception("Cannot set task".into())?,
                }
            }

            Ok(JuliaFuture { shared_state })
        }
    }
}

impl<'frame, 'data> Future for JuliaFuture<'frame, 'data> {
    type Output = CallResult<'frame, 'data>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared_state = self.shared_state.lock().unwrap();
        if shared_state.completed {
            if let Some(task) = shared_state.task {
                if let Some(result) = task.result() {
                    Poll::Ready(Ok(result))
                } else if let Some(exc) = task.exception() {
                    Poll::Ready(Err(exc))
                } else {
                    unsafe { Poll::Ready(Err(Value::wrap(jl_sys::jl_nothing))) }
                }
            } else {
                // JuliaFuture is not created if task cannot be set
                unreachable!()
            }
        } else {
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

struct Node<T> {
    value: T,
    next: Option<Box<Node<T>>>,
}

struct LinkedList<T> {
    head: Option<Node<T>>,
}

impl LinkedList<usize> {
    pub fn new_free_list(n_tasks: usize) -> Self {
        let mut current = Node {
            value: n_tasks - 1,
            next: None,
        };

        for i in (0..n_tasks - 1).rev() {
            let new = Node {
                value: i,
                next: Some(Box::new(current)),
            };

            current = new;
        }

        LinkedList {
            head: Some(current),
        }
    }
}

impl<T> LinkedList<T> {
    pub fn pop(&mut self) -> Option<T> {
        let Node { value, next } = self.head.take()?;
        self.head = next.map(|x| *x);
        Some(value)
    }

    pub fn push(&mut self, value: T) {
        let head = Node {
            value,
            next: self.head.take().map(Box::new),
        };
        self.head = Some(head);
    }
}

struct TaskStack {
    raw: Box<[*mut std::ffi::c_void]>,
}

impl TaskStack {
    pub(crate) unsafe fn new(stack_size: usize) -> Self {
        let v = vec![std::ptr::null_mut(); stack_size];

        Self {
            raw: v.into_boxed_slice(),
        }
    }

    pub(crate) unsafe fn init(&mut self) -> JlrsResult<()> {
        if self.raw.len() < 3 {
            Err(JlrsError::AllocError(AllocError::StackOverflow(
                3,
                self.raw.len(),
            )))?;
        }

        let rtls = &mut *jl_get_ptls_states();

        self.raw[0] = 3 as _;
        self.raw[2] = rtls.pgcstack as _;

        rtls.pgcstack = self.raw[1..].as_mut_ptr().cast();
        Ok(())
    }

    #[allow(dead_code)]
    pub fn print_memory(&self) {
        println!("{:?}", self.raw.as_ref());
    }
}

struct MultitaskStack<T, R> {
    raw: Box<[Option<TaskStack>]>,
    queue: VecDeque<Box<dyn JuliaTask<T = T, R = R>>>,
    free_list: LinkedList<usize>,
    running: Box<[Option<JoinHandle<()>>]>,
    n: usize,
}

impl<T, R> MultitaskStack<T, R> {
    pub unsafe fn new(n_tasks: usize, stack_size: usize) -> Self {
        let mut raw = Vec::new();

        for _ in 0..n_tasks + 1 {
            raw.push(Some(TaskStack::new(stack_size)));
        }

        let running = raw
            .iter()
            .map(|_| None)
            .collect::<Vec<_>>()
            .into_boxed_slice();

        for s in raw.iter_mut() {
            match s {
                Some(ref mut s) => s.init().unwrap(),
                _ => unreachable!(),
            }
        }

        MultitaskStack {
            raw: raw.into_boxed_slice(),
            queue: VecDeque::new(),
            free_list: LinkedList::new_free_list(n_tasks),
            running,
            n: 0,
        }
    }

    pub fn acquire_task_frame(&mut self) -> Option<(usize, TaskStack)> {
        let idx = self.free_list.pop()?;
        let ts = self.raw[idx]
            .take()
            .expect("Memory was corrupted: Task stack is None.");
        Some((idx, ts))
    }

    pub fn return_task_frame(&mut self, frame: usize, ts: TaskStack) {
        self.free_list.push(frame);
        self.raw[frame] = Some(ts);
    }

    pub fn add_pending(&mut self, jl_task: Box<dyn JuliaTask<T = T, R = R>>) {
        self.queue.push_back(jl_task);
    }

    pub fn pop_pending(&mut self) -> Option<Box<dyn JuliaTask<T = T, R = R>>> {
        self.queue.pop_front()
    }

    #[allow(dead_code)]
    pub fn print_memory(&self) {
        println!("[");
        for stack in self.raw.iter() {
            stack.as_ref().map(|f| f.print_memory());
        }
        println!("]");
    }
}

pub enum Message<T, R> {
    Task(
        Box<dyn JuliaTask<T = T, R = R>>,
        AsyncStdSender<Message<T, R>>,
    ),
    Include(PathBuf, Arc<(Mutex<bool>, Condvar)>),
    Complete(Wrapper, AsyncStdSender<Message<T, R>>),
    SetWakeFn(Arc<(Mutex<bool>, Condvar)>),
}

pub struct Wrapper(usize, TaskStack);
unsafe impl Sync for Wrapper {}
unsafe impl Send for Wrapper {}

pub struct AsyncFrame<'frame> {
    pub(crate) idx: FrameIdx,
    pub(crate) memory: StackView<'frame, Async, Dynamic>,
    pub(crate) len: usize,
}

impl<'frame> AsyncFrame<'frame> {
    pub(crate) unsafe fn nested_frame<'nested>(
        &'nested mut self,
    ) -> JlrsResult<DynamicFrame<'nested, Async>> {
        let idx = self.memory.new_frame()?;
        Ok(DynamicFrame {
            idx,
            memory: self.memory.nest_dynamic(),
            len: 0,
        })
    }
}

impl<'frame> Drop for AsyncFrame<'frame> {
    fn drop(&mut self) {
        unsafe {
            self.memory.pop_frame(self.idx);
        }
    }
}

fn run_task<T: Send + Sync + 'static, R>(
    mut jl_task: Box<dyn JuliaTask<T = T, R = R>>,
    task_idx: usize,
    mut task_stack: TaskStack,
    rt_sender: AsyncStdSender<Message<T, R>>,
) -> async_std::task::JoinHandle<()>
where
    R: ReturnChannel<T = T> + 'static,
{
    unsafe {
        task::spawn_local(async move {
            let global = Global::new();
            let mut tv = StackView::<Async, Dynamic>::new(&mut task_stack.raw);

            match tv.new_frame() {
                Ok(frame_idx) => {
                    let mut frame = AsyncFrame {
                        idx: frame_idx,
                        memory: tv,
                        len: 0,
                    };
                    let res = jl_task.run(global, &mut frame).await;

                    if let Some(s) = jl_task.return_channel() {
                        s.send(res).await;
                    }
                }
                Err(e) => {
                    if let Some(s) = jl_task.return_channel() {
                        s.send(Err(e)).await;
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
pub struct AsyncJulia<T, R> {
    _t: PhantomData<T>,
    _r: PhantomData<R>,
}

#[derive(Clone)]
pub struct TaskSender<T, R> {
    sender: AsyncStdSender<Message<T, R>>,
}

impl<T, R> TaskSender<T, R>
where
    T: Send + Sync + 'static,
    R: ReturnChannel<T = T>,
{
    pub async fn send_task<D: JuliaTask<T = T, R = R>>(&self, task: D) {
        let sender = self.sender.clone();
        self.sender
            .send(Message::Task(Box::new(task), sender))
            .await
    }

    pub fn try_send_task<D: JuliaTask<T = T, R = R>>(
        &self,
        task: D,
    ) -> Result<(), TrySendError<Box<dyn JuliaTask<T = T, R = R>>>> {
        let sender = self.sender.clone();
        self.sender
            .try_send(Message::Task(Box::new(task), sender))
            .map_err(|e| match e {
                TrySendError::Full(Message::Task(t, _)) => TrySendError::Full(t),
                TrySendError::Disconnected(Message::Task(t, _)) => TrySendError::Disconnected(t),
                _ => unreachable!(),
            })
    }

    pub async fn include<P: AsRef<Path>>(&self, path: P) {
        let completed = Arc::new((Mutex::new(false), Condvar::new()));
        let s = self
            .sender
            .send(Message::Include(
                path.as_ref().to_path_buf(),
                completed.clone(),
            ))
            .await;

        let (lock, cvar) = &*completed;
        let mut completed = lock.lock().unwrap();
        while !*completed {
            completed = cvar.wait(completed).unwrap();
        }

        s
    }

    pub fn try_include<P: AsRef<Path>>(&self, path: P) -> Result<(), TrySendError<PathBuf>> {
        let completed = Arc::new((Mutex::new(false), Condvar::new()));
        self.sender
            .try_send(Message::Include(
                path.as_ref().to_path_buf(),
                completed.clone(),
            ))
            .map_err(|e| match e {
                TrySendError::Full(Message::Include(t, _)) => TrySendError::Full(t),
                TrySendError::Disconnected(Message::Include(t, _)) => TrySendError::Disconnected(t),
                _ => unreachable!(),
            })
            .and_then(|f| {
                let (lock, cvar) = &*completed;
                let mut completed = lock.lock().unwrap();
                while !*completed {
                    completed = cvar.wait(completed).unwrap();
                }

                Ok(f)
            })
    }

    pub fn capacity(&self) -> usize {
        self.sender.capacity()
    }

    pub fn len(&self) -> usize {
        self.sender.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sender.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.sender.is_full()
    }

    fn set_wake_fn(&self) -> Result<(), TrySendError<()>> {
        let completed = Arc::new((Mutex::new(false), Condvar::new()));
        self.sender
            .try_send(Message::SetWakeFn(completed.clone()))
            .map_err(|e| match e {
                TrySendError::Full(_) => TrySendError::Full(()),
                TrySendError::Disconnected(_) => TrySendError::Disconnected(()),
            })
            .and_then(|f| {
                let (lock, cvar) = &*completed;
                let mut completed = lock.lock().unwrap();
                while !*completed {
                    completed = cvar.wait(completed).unwrap();
                }

                Ok(f)
            })
    }
}

impl<T, R> AsyncJulia<T, R>
where
    T: Send + Sync + 'static,
    R: ReturnChannel<T = T> + 'static,
{
    pub fn init<P: AsRef<Path>>(
        channel_capacity: usize,
        n_threads: usize,
        stack_size: usize,
        safepoint_ms: u64,
        jlrs_path: P,
    ) -> JlrsResult<(TaskSender<T, R>, std::thread::JoinHandle<()>)> {
        let (s, r) = channel(channel_capacity);
        let ts = TaskSender { sender: s };

        let handle = Self::run(n_threads, stack_size, safepoint_ms, r);

        ts.try_include(jlrs_path).expect("Unable to include Jlrs");

        ts.set_wake_fn().expect("Unable to set wake function");

        Ok((ts, handle))
    }

    fn run(
        n_threads: usize,
        stack_size: usize,
        safepoint_ms: u64,
        receiver: AsyncStdReceiver<Message<T, R>>,
    ) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || {
            let handle = task::spawn_local(async move {
                let mut t: MultitaskStack<T, R> = unsafe {
                    jl_sys::jl_init();
                    MultitaskStack::new(n_threads, stack_size)
                };

                loop {
                    match timeout(Duration::from_millis(safepoint_ms), receiver.recv()).await {
                        Ok(Ok(Message::Task(jl_task, sender))) => {
                            if let Some((task_idx, task_stack)) = t.acquire_task_frame() {
                                t.n += 1;
                                t.running[task_idx] =
                                    Some(run_task(jl_task, task_idx, task_stack, sender));
                            } else {
                                t.add_pending(jl_task);
                            }
                        }
                        Ok(Ok(Message::Complete(Wrapper(task_idx, task_stack), sender))) => {
                            if let Some(jl_task) = t.pop_pending() {
                                t.running[task_idx] =
                                    Some(run_task(jl_task, task_idx, task_stack, sender));
                            } else {
                                t.n -= 1;
                                t.running[task_idx] = None;
                                t.return_task_frame(task_idx, task_stack);
                            }
                        }
                        Ok(Ok(Message::Include(path, completed))) => {
                            let idx = t.raw.len() - 1;
                            let mut stack = t.raw[idx].take().expect("GC stack is corrupted.");

                            unsafe {
                                let global = Global::new();
                                let mut view = StackView::<Async, Dynamic>::new(&mut stack.raw);
                                let idx = view.new_frame().expect("GC stack is too small");

                                let mut frame = AsyncFrame {
                                    idx,
                                    len: 0,
                                    memory: view,
                                };

                                let path = Value::new(
                                    &mut frame,
                                    path.to_str().expect("The include path is invalid"),
                                )
                                .expect("GC stack is too small");

                                Module::main(global)
                                    .function("include")
                                    .expect("Cannot find the include function in the Main module")
                                    .call1(&mut frame, path)
                                    .expect("GC stack is too small")
                                    .ok();
                            }

                            t.raw[idx] = Some(stack);

                            {
                                let (lock, condvar) = &*completed;
                                let mut completed = lock.lock().expect("Cannot lock");
                                *completed = true;
                                condvar.notify_one();
                            }
                        }
                        Ok(Ok(Message::SetWakeFn(completed))) => {
                            let idx = t.raw.len() - 1;
                            let mut stack = t.raw[idx].take().expect("GC stack is corrupted.");

                            unsafe {
                                let global = Global::new();
                                let mut view = StackView::<Async, Dynamic>::new(&mut stack.raw);
                                let idx = view.new_frame().expect("GC stack is too small");

                                let mut frame = AsyncFrame {
                                    idx,
                                    len: 0,
                                    memory: view,
                                };

                                let waker = Value::new(&mut frame, wake_task as *mut c_void)
                                    .expect("GC stack is too small");

                                Module::main(global)
                                    .submodule("Jlrs")
                                    .expect("Cannot find the Jlrs module")
                                    .set_const("wakerust", waker)
                                    .expect("Unable to set wake function");
                            }

                            t.raw[idx] = Some(stack);

                            {
                                let (lock, condvar) = &*completed;
                                let mut completed = lock.lock().expect("Cannot lock");
                                *completed = true;
                                condvar.notify_one();
                            }
                        }
                        Ok(Err(RecvError)) => {
                            break;
                        }
                        Err(_) => unsafe {
                            if t.n > 0 {
                                // periodically insert a safepoint so the GC can run
                                jl_gc_safepoint();
                            }
                        },
                    }
                }

                // Wait for tasks to finish
                // Move to drop impl
                for pending in t.running.iter_mut() {
                    if let Some(handle) = pending.take() {
                        handle.await;
                    }
                }

                unsafe {
                    jl_atexit_hook(0);
                }
            });

            async_std::task::block_on(handle);
        })
    }
}
