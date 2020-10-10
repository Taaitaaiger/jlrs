// In order to prevent the GC from freeing things that are in use, the Julia C API offers a few
// macros that should be used. These macros allocate some space on the stack with alloca and use
// it to construct a struct of type jl_gcframe_t. The first two fields contain the number of
// protected values (times two) and a pointer to the previous frame, then pointers to the values
// that should not be freed.
//
// Rust doesn't really like dynamically sized types, and as far as I'm aware something like alloca
// is unavailable. As a workaround jlrs creates a boxed array to contain these frames. A new frame
// is pushed when a frame is created, and is popped when the frame is dropped.
//
// Compared to the possibilities of the macros, jlrs is a bit more flexible. For example, the
// DynamicFrame dynamically grows its associated GC frame which is not possible in the C API and
// Outputs allow you to protect the result of a function call until the output's frame is dropped.
// My driving assumption is that when the GC runs, it can't make any assumptions about the
// contents of the GC stack based on earlier runs. Dynamically growing the frame does not make
// sense in C because alloca is used but there's no technical reason preventing such a feature
// from existing in Rust. Similarly, thanks to lifetimes we can enforce that a value can't live
// longer than its frame while C can offer no such guarantees.

use crate::error::{AllocError, JlrsResult};
use crate::frame::{FrameIdx, Output};
use crate::mode::Mode;
use crate::value::{Value, Values};
use std::ffi::c_void;
use std::marker::PhantomData;
use std::ptr::null_mut;

pub(crate) enum Static {}
pub(crate) enum Dynamic {}

pub(crate) struct RawStack(Box<[*mut c_void]>);

impl RawStack {
    pub(crate) unsafe fn new(stack_size: usize) -> Self {
        debug_assert!(stack_size > 0);
        let mut raw = vec![null_mut(); stack_size];
        raw[0] = 1 as _;
        let boxed = raw.into_boxed_slice();
        RawStack(boxed)
    }

    pub(crate) fn as_mut<'original: 'scope, 'scope>(
        &'original mut self,
    ) -> &'scope mut [*mut c_void] {
        &mut self.0
    }

    pub(crate) fn size(&self) -> usize {
        self.0.len()
    }
}

pub(crate) struct StackView<'stack, U: Mode, V> {
    stack: &'stack mut [*mut c_void],
    _u: PhantomData<U>,
    _v: PhantomData<V>,
}

impl<'stack, M: Mode, V> StackView<'stack, M, V> {
    pub(crate) fn size(&self) -> usize {
        self.stack[0] as _
    }

    pub(crate) fn print_memory(&self) {
        println!("{:?}", &self.stack);
    }

    pub(crate) unsafe fn pop_frame(&mut self, idx: FrameIdx) {
        M::pop_frame(&mut self.stack, idx)
    }

    pub(crate) unsafe fn nest_static<'nested>(&'nested mut self) -> StackView<'nested, M, Static> {
        StackView {
            stack: self.stack,
            _u: PhantomData,
            _v: PhantomData,
        }
    }

    pub(crate) unsafe fn nest_dynamic<'nested>(
        &'nested mut self,
    ) -> StackView<'nested, M, Dynamic> {
        StackView {
            stack: self.stack,
            _u: PhantomData,
            _v: PhantomData,
        }
    }

    pub(crate) unsafe fn as_values<'output>(
        &mut self,
        idx: FrameIdx,
        offset: usize,
        n: usize,
    ) -> Values<'output> {
        let ptr = self.stack[idx.0 + offset..].as_mut_ptr();
        Values::wrap(ptr.cast(), n)
    }
}

impl<'stack, M> StackView<'stack, M, Dynamic>
where
    M: Mode,
{
    pub(crate) unsafe fn new(stack: &'stack mut [*mut c_void]) -> Self {
        StackView {
            stack,
            _u: PhantomData,
            _v: PhantomData,
        }
    }

    pub(crate) unsafe fn new_frame(&mut self) -> JlrsResult<FrameIdx> {
        if self.size() + 2 >= self.stack.len() {
            return Err(Box::new(
                AllocError::StackOverflow(2, self.stack.len()).into(),
            ));
        }

        let size = self.size();
        Ok(M::new_dynamic_frame(&mut self.stack, size))
    }

    pub(crate) unsafe fn new_output<'output>(
        &mut self,
        idx: FrameIdx,
    ) -> JlrsResult<Output<'output>> {
        if self.size() >= self.stack.len() {
            return Err(Box::new(
                AllocError::StackOverflow(1, self.stack.len()).into(),
            ));
        }

        let sz = self.size();
        self.stack[sz] = null_mut();
        self.stack[idx.0 - 2] = (self.stack[idx.0 - 2] as usize + 2) as _;
        self.stack[0] = (self.size() + 1) as _;
        Ok(Output::new(sz))
    }

    pub(crate) unsafe fn protect<'output>(
        &mut self,
        idx: FrameIdx,
        value: *mut c_void,
    ) -> Result<Value<'output, 'static>, AllocError> {
        if self.size() == self.stack.len() {
            return Err(AllocError::StackOverflow(1, self.stack.len()));
        }

        self.stack[self.size()] = value.cast::<_>();
        self.stack[idx.0 - 2] = (self.stack[idx.0 - 2] as usize + 2) as _;
        self.stack[0] = (self.size() + 1) as _;

        Ok(Value::wrap(value.cast::<_>()))
    }

    pub(crate) unsafe fn protect_output<'output>(
        &mut self,
        output: Output,
        value: *mut c_void,
    ) -> Value<'output, 'static> {
        self.stack[output.offset] = value.cast::<_>();
        Value::wrap(value.cast::<_>())
    }
}

impl<'stack, M> StackView<'stack, M, Static>
where
    M: Mode,
{
    pub(crate) unsafe fn new(stack: &'stack mut [*mut c_void]) -> Self {
        StackView {
            stack,
            _u: PhantomData,
            _v: PhantomData,
        }
    }

    pub(crate) unsafe fn new_frame(&mut self, capacity: usize) -> JlrsResult<FrameIdx> {
        let size = self.size();
        if size + capacity + 2 >= self.stack.len() {
            return Err(Box::new(
                AllocError::StackOverflow(capacity + 2, self.stack.len()).into(),
            ));
        }

        Ok(M::new_frame(&mut self.stack, size, capacity))
    }

    pub(crate) unsafe fn new_output<'output>(
        &mut self,
        idx: FrameIdx,
        offset: usize,
    ) -> Output<'output> {
        Output::new(idx.0 + offset)
    }

    pub(crate) unsafe fn protect<'output>(
        &mut self,
        idx: FrameIdx,
        offset: usize,
        value: *mut c_void,
    ) -> Value<'output, 'static> {
        self.stack[idx.0 + offset] = value;
        Value::wrap(value.cast())
    }
}

#[cfg(all(feature = "async", target_os = "linux"))]
pub(crate) mod multitask {
    use crate::error::{AllocError, JlrsError, JlrsResult};
    use crate::traits::multitask::JuliaTask;
    use async_std::task::JoinHandle;
    use jl_sys::jl_get_ptls_states;
    use std::collections::VecDeque;
    use std::ffi::c_void;
    use std::ptr::null_mut;

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

    pub(crate) struct TaskStack {
        pub(crate) raw: Box<[*mut c_void]>,
    }

    impl TaskStack {
        pub(crate) unsafe fn new(stack_size: usize) -> Self {
            let v = vec![null_mut(); stack_size];

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

    pub(crate) struct MultitaskStack<T, R> {
        pub(crate) raw: Box<[Option<TaskStack>]>,
        queue: VecDeque<Box<dyn JuliaTask<T = T, R = R>>>,
        free_list: LinkedList<usize>,
        pub(crate) running: Box<[Option<JoinHandle<()>>]>,
        pub(crate) n: usize,
    }

    impl<T, R> MultitaskStack<T, R> {
        pub(crate) unsafe fn new(n_tasks: usize, stack_size: usize) -> Self {
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

        pub(crate) fn acquire_task_frame(&mut self) -> Option<(usize, TaskStack)> {
            let idx = self.free_list.pop()?;
            let ts = self.raw[idx]
                .take()
                .expect("Memory was corrupted: Task stack is None.");
            Some((idx, ts))
        }

        pub(crate) fn return_task_frame(&mut self, frame: usize, ts: TaskStack) {
            self.free_list.push(frame);
            self.raw[frame] = Some(ts);
        }

        pub(crate) fn add_pending(&mut self, jl_task: Box<dyn JuliaTask<T = T, R = R>>) {
            self.queue.push_back(jl_task);
        }

        pub(crate) fn pop_pending(&mut self) -> Option<Box<dyn JuliaTask<T = T, R = R>>> {
            self.queue.pop_front()
        }

        // keep this around for debugging purposes
        #[allow(dead_code)]
        pub(crate) fn print_memory(&self) {
            println!("[");
            for stack in self.raw.iter() {
                stack.as_ref().map(|f| f.print_memory());
            }
            println!("]");
        }
    }
}
