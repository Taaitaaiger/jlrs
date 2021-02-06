//! Frames protect values from garbage collection.
//!
//! Julia's garbage collector essentially owns all Julia data and frees it when it's no longer in
//! use. Because it is unaware of values that are created through the C API, users of this API are
//! responsible for ensuring the garbage collector is made aware what values are in use.
//! Internally this works by maintaining a stack of garbage collector frames (frames). Each  
//! frame on this stack contains pointers to values; these values have been rooted in the  
//! frame. While a frame is on this stack, the values rooted in that frame (and the fields of
//! these values, and so on) will not be freed by the garbage collector.
//!
//! Unlike the raw C API, when a pointer to a value is returned jlrs ensures the value is rooted
//! before it can be used. Whenever a new value is produced which must be rooted, jlrs will
//! require you to provide something that implements [`Scope`]. Mutable references to the frame
//! types that jlrs provides are just that: the resulting value will be rooted in that frame and
//! can be used while that frame has not been popped from the stack. The main use case for stacking
//! frames is memory management. The more values that are rooted, the longer it will take the
//! garbage collector to run.
//!
//! Several kinds of frame exist in jlrs. The simplest one is [`NullFrame`], which is only used
//! when writing `ccall`able functions. It doesn't let you root any values or push another
//! frame, but can be used to (mutably) borrow array data. If you don't use the async runtime, the
//! only frame type you will use is [`GcFrame`]; this frame can be used to root a relatively
//! arbitrary number of values, and new frames can always be pushed on top of it. In the async
//! runtime the [`AsyncGcFrame`] is often used, this frame type offers the same functionalities
//! as the non-async version, as well as methods to stack a new async frames on top of the current
//! one. All of them implement the [`Frame`] trait.
//!
//! Frames that can be used to root values can preallocate a number of slots, each slot can root
//! one value. By preallocating the slots less work has to be done to root a value, more slots can
//! be allocated to the frame if necessary. The maximum number of slots that can be allocated to a
//! frame is its capacity. In general, the capacity of a frame that allocates no slots is at least
//! 32, while one that does allocates some slots guarantees a capacity of at least that number of
//! slots. When a new frame is pushed, it will try to use the current frame's remaining capacity.
//! If the remaining capacity is insufficient, more stack space is allocated.
//!
//! Frames are pushed to the stack when they're created, and popped when they're dropped. It's not
//! possible to create a frame directly, but the methods `frame`, `value_frame`, and `call_frame`
//! all take a closure which provides you with a mutable reference to a new frame, and in the
//! latter two cases an [`Output`] as well. This new frame is dropped after the closure has been
//! called. The first of these methods can return anything which lives at least as long as the
//! current frame. In order to create a value or call a Julia function in a new frame and root the
//! result in the current frame the latter two methods must be used. This allows you to allocate
//! temporary values, for example to create an instance of some complex type like a `NamedTuple`:
//!
//! ```
//! # use jlrs::prelude::*;
//! # use jlrs::util::JULIA;
//! # fn main() {
//! # JULIA.with(|j| {
//! # let mut julia = j.borrow_mut();
//!   julia.frame(|_global, parent_frame| {
//!       // `value_frame_with_slots` provides you with an output and a mutable reference to a new
//!       // frame. This new frame can be used to allocate temporary values, before converting the
//!       // output into a scope and using it to create a `NamedTuple` and rooting it in the
//!       // parent frame. Two slots are used in the child frame, one for each of the temporary
//!       // values. The `NamedTuple` will use a slot of the parent frame.
//!       let _nt = parent_frame.value_frame_with_slots(2, |output, child_frame| {
//!           let i = Value::new(&mut *child_frame, 1u64)?;
//!           let j = Value::new(&mut *child_frame, 2i32)?;
//!           let output_scope = output.into_scope(child_frame);
//!           named_tuple!(output_scope, "i" => i, "j" => j)
//!       })?;
//!
//!       Ok(())
//!   }).unwrap();
//! # });
//! # }
//! ```

#[cfg(all(feature = "async", target_os = "linux"))]
use super::mode::Async;
use super::traits::mode::Mode;
#[cfg(all(feature = "async", target_os = "linux"))]
use crate::{
    error::{AllocError, CallResult, JlrsError, JlrsResult},
    memory::output::Output,
    memory::traits::mode::private::Mode as _,
    value::{UnrootedCallResult, UnrootedValue, Value},
};
use crate::{value::traits::private::Internal, CCall};
use jl_sys::jl_value_t;
#[cfg(all(feature = "async", target_os = "linux"))]
use std::future::Future;
use std::{ffi::c_void, marker::PhantomData, mem, ptr::null_mut};

pub(crate) const PAGE_SIZE: usize = 4096 / mem::size_of::<usize>();
pub(crate) const MIN_FRAME_CAPACITY: usize = 32;

/// A frame that can root values.
pub struct GcFrame<'frame, M: Mode> {
    raw_frame: &'frame mut [*mut c_void],
    page: Option<Box<[*mut c_void]>>,
    len: usize,
    mode: M,
}

impl<'frame, M: Mode> GcFrame<'frame, M> {
    /// Returns the number of values currently rooted in this frame.
    pub fn n_roots(&self) -> usize {
        self.len
    }

    /// Returns the number of slots that are currently allocated to this frame.
    pub fn n_slots(&self) -> usize {
        self.raw_frame[0] as usize >> 1
    }

    /// Returns the maximum number of slots this frame can use.
    pub fn capacity(&self) -> usize {
        self.raw_frame.len() - 2
    }

    /// Try to allocate `additional` slots in the current frame. Returns `true` on success, or
    /// `false` if `self.n_slots() + additional > self.capacity()`.
    #[must_use]
    pub fn alloc_slots(&mut self, additional: usize) -> bool {
        let slots = self.n_slots();
        if additional + slots > self.capacity() {
            return false;
        }

        for idx in slots + 2..slots + additional + 2 {
            self.raw_frame[idx] = null_mut();
        }

        // The new number of slots does not exceed the capacity, and the new slots have been cleared
        unsafe { self.set_n_slots(slots + additional) }
        true
    }

    pub(crate) fn nest<'nested>(&'nested mut self, capacity: usize) -> GcFrame<'nested, M> {
        let used = self.n_slots() + 2;
        let needed = MIN_FRAME_CAPACITY.max(capacity) + 2;
        let raw_frame = if used + needed > self.raw_frame.len() {
            if self.page.is_none() || self.page.as_ref().unwrap().len() < needed {
                let raw = vec![null_mut::<c_void>(); PAGE_SIZE.max(needed)];
                let page = raw.into_boxed_slice();
                self.page = Some(page);
            }

            self.page.as_mut().unwrap().as_mut()
        } else {
            &mut self.raw_frame[used..]
        };

        GcFrame::new(raw_frame, capacity, self.mode)
    }

    pub(crate) fn new(raw_frame: &'frame mut [*mut c_void], capacity: usize, mode: M) -> Self {
        // Is popped when this frame is dropped
        unsafe {
            mode.push_frame(raw_frame, capacity, Internal);
        }

        GcFrame {
            raw_frame,
            page: None,
            len: 0,
            mode,
        }
    }

    // Safety: capacity >= n_slots
    pub(crate) unsafe fn set_n_slots(&mut self, n_slots: usize) {
        debug_assert!(self.capacity() >= n_slots);
        self.raw_frame[0] = (n_slots << 1) as _;
    }

    // Safety: capacity > n_roots
    pub(crate) unsafe fn root(&mut self, value: *mut jl_value_t) {
        debug_assert!(self.n_roots() < self.capacity());

        let n_roots = self.n_roots();
        self.raw_frame[n_roots + 2] = value.cast();
        if n_roots == self.n_slots() {
            self.set_n_slots(n_roots + 1);
        }
    }
}

impl<'frame, M: Mode> Drop for GcFrame<'frame, M> {
    fn drop(&mut self) {
        // The frame was pushed when the frame was created.
        unsafe { self.mode.pop_frame(self.raw_frame, Internal) }
    }
}

/// A frame that can root values, and can be used to dispatch function calls to a new thread and
/// await the result with [`Value::call_async`]. It provides the same public API as [`GcFrame`]
/// and async versions of `frame(_with_slots)`, `value_frame(_with_slots)`, and
/// `call_frame(_with_slots)`.
#[cfg(all(feature = "async", target_os = "linux"))]
pub struct AsyncGcFrame<'frame> {
    raw_frame: &'frame mut [*mut c_void],
    len: usize,
    page: Option<Box<[*mut c_void]>>,
    output: Option<&'frame mut *mut c_void>,
    mode: Async<'frame>,
}

#[cfg(all(feature = "async", target_os = "linux"))]
impl<'frame> AsyncGcFrame<'frame> {
    /// An async version of `value_frame`.
    pub async fn async_value_frame<'nested, 'data, F, G>(
        &'nested mut self,
        func: F,
    ) -> JlrsResult<Value<'frame, 'data>>
    where
        G: Future<Output = JlrsResult<UnrootedValue<'frame, 'data, 'nested>>>,
        F: FnOnce(Output<'frame>, &'nested mut AsyncGcFrame<'nested>) -> G,
    {
        unsafe {
            let mut nested = self.nest_async_with_output(0)?;
            let p_nested = &mut nested as *mut _;
            let r_nested = &mut *p_nested;
            let output = Output::new();
            let ptr = func(output, r_nested).await?.ptr();

            if let Some(output) = nested.output.take() {
                *output = ptr.cast();
            }

            Ok(Value::wrap(ptr))
        }
    }

    /// An async version of `value_frame_with_slots`.
    pub async fn async_value_frame_with_slots<'nested, 'data, F, G>(
        &'nested mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<Value<'frame, 'data>>
    where
        G: Future<Output = JlrsResult<UnrootedValue<'frame, 'data, 'nested>>>,
        F: FnOnce(Output<'frame>, &'nested mut AsyncGcFrame<'nested>) -> G,
    {
        unsafe {
            let mut nested = self.nest_async_with_output(capacity)?;
            let p_nested = &mut nested as *mut _;
            let r_nested = &mut *p_nested;
            let output = Output::new();
            let ptr = func(output, r_nested).await?.ptr();

            if let Some(output) = nested.output.take() {
                *output = ptr.cast();
            }

            Ok(Value::wrap(ptr))
        }
    }

    /// An async version of `call_frame`.
    pub async fn async_call_frame<'nested, 'data, F, G>(
        &'nested mut self,
        func: F,
    ) -> JlrsResult<CallResult<'frame, 'data>>
    where
        G: Future<Output = JlrsResult<UnrootedCallResult<'frame, 'data, 'nested>>>,
        F: FnOnce(Output<'frame>, &'nested mut AsyncGcFrame<'nested>) -> G,
    {
        unsafe {
            let mut nested = self.nest_async_with_output(0)?;
            let p_nested = &mut nested as *mut _;
            let r_nested = &mut *p_nested;
            let output = Output::new();
            let res = func(output, r_nested).await?;
            let is_exc = res.is_exception();
            let ptr = res.ptr();

            if let Some(output) = nested.output.take() {
                *output = ptr.cast();
            }

            if is_exc {
                Ok(CallResult::Ok(Value::wrap(ptr)))
            } else {
                Ok(CallResult::Err(Value::wrap(ptr)))
            }
        }
    }

    /// An async version of `call_frame_with_slots`.
    pub async fn async_call_frame_with_slots<'nested, 'data, F, G>(
        &'nested mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<CallResult<'frame, 'data>>
    where
        G: Future<Output = JlrsResult<UnrootedCallResult<'frame, 'data, 'nested>>>,
        F: FnOnce(Output<'frame>, &'nested mut AsyncGcFrame<'nested>) -> G,
    {
        unsafe {
            let mut nested = self.nest_async_with_output(capacity)?;
            let p_nested = &mut nested as *mut _;
            let r_nested = &mut *p_nested;
            let output = Output::new();
            let res = func(output, r_nested).await?;
            let is_exc = res.is_exception();
            let ptr = res.ptr();

            if let Some(output) = nested.output.take() {
                *output = ptr.cast();
            }

            if is_exc {
                Ok(CallResult::Ok(Value::wrap(ptr)))
            } else {
                Ok(CallResult::Err(Value::wrap(ptr)))
            }
        }
    }

    /// An async version of `frame`.
    pub async fn async_frame<'nested, T, F, G>(&'nested mut self, func: F) -> JlrsResult<T>
    where
        T: 'frame,
        G: Future<Output = JlrsResult<T>>,
        F: FnOnce(&'nested mut AsyncGcFrame<'nested>) -> G,
    {
        unsafe {
            let mut nested = self.nest_async(0);
            let p_nested = &mut nested as *mut _;
            let r_nested = &mut *p_nested;
            func(r_nested).await
        }
    }

    /// An async version of `frame_with_slots`.
    pub async fn async_frame_with_slots<'nested, T, F, G>(
        &'nested mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<T>
    where
        T: 'frame,
        G: Future<Output = JlrsResult<T>>,
        F: FnOnce(&'nested mut AsyncGcFrame<'nested>) -> G,
    {
        unsafe {
            let mut nested = self.nest_async(capacity);
            let p_nested = &mut nested as *mut _;
            let r_nested = &mut *p_nested;
            func(r_nested).await
        }
    }

    /// Returns the number of values currently rooted in this frame.
    pub fn n_roots(&self) -> usize {
        self.len
    }

    /// Returns the number of slots that are currently allocated to this frame.
    pub fn n_slots(&self) -> usize {
        self.raw_frame[0] as usize >> 1
    }

    /// Returns the maximum number of slots this frame can use.
    pub fn capacity(&self) -> usize {
        self.raw_frame.len() - 2
    }

    /// Try to allocate `additional` slots in the current frame. Returns `true` on success, or
    /// `false` if `self.n_slots() + additional > self.capacity()`.
    pub fn alloc_slots(&mut self, additional: usize) -> bool {
        let slots = self.n_slots();
        if additional + slots > self.capacity() {
            return false;
        }

        for idx in slots + 2..slots + additional + 2 {
            self.raw_frame[idx] = null_mut();
        }

        // The new number of slots does not exceed the capacity, and the new slots have been cleared
        unsafe { self.set_n_slots(slots + additional) }
        true
    }

    pub(crate) fn new(
        raw_frame: &'frame mut [*mut c_void],
        capacity: usize,
        mode: Async<'frame>,
    ) -> Self {
        // Is popped when this frame is dropped
        unsafe {
            mode.push_frame(raw_frame, capacity, Internal);
        }

        AsyncGcFrame {
            raw_frame,
            len: 0,
            page: None,
            output: None,
            mode,
        }
    }

    // Safety: capacity >= n_slots
    pub(crate) unsafe fn set_n_slots(&mut self, n_slots: usize) {
        debug_assert!(n_slots <= self.capacity());
        self.raw_frame[0] = (n_slots << 1) as _;
    }

    pub(crate) fn nest<'nested>(
        &'nested mut self,
        capacity: usize,
    ) -> GcFrame<'nested, Async<'frame>> {
        let used = self.n_slots() + 2;
        let needed = MIN_FRAME_CAPACITY.max(capacity) + 2;
        let raw_frame = if used + needed > self.raw_frame.len() {
            if self.page.is_none() || self.page.as_ref().unwrap().len() < needed {
                let raw = vec![null_mut::<c_void>(); PAGE_SIZE.max(needed)];
                let page = raw.into_boxed_slice();
                self.page = Some(page);
            }

            self.page.as_mut().unwrap().as_mut()
        } else {
            &mut self.raw_frame[used..]
        };

        GcFrame::new(raw_frame, capacity, self.mode)
    }

    pub(crate) fn nest_async<'nested>(&'nested mut self, capacity: usize) -> AsyncGcFrame<'nested> {
        let used = self.n_slots() + 2;
        let needed = MIN_FRAME_CAPACITY.max(capacity) + 2;
        let raw_frame = if used + needed > self.raw_frame.len() {
            if self.page.is_none() || self.page.as_ref().unwrap().len() < needed {
                let raw = vec![null_mut::<c_void>(); PAGE_SIZE.max(needed)];
                let page = raw.into_boxed_slice();
                self.page = Some(page);
            }

            self.page.as_mut().unwrap().as_mut()
        } else {
            &mut self.raw_frame[used..]
        };

        AsyncGcFrame::new(raw_frame, capacity, self.mode)
    }

    // Safety: n_roots < capacity
    pub(crate) unsafe fn root(&mut self, value: *mut jl_value_t) {
        println!("{} {} ", self.n_roots(), self.capacity());
        debug_assert!(self.n_roots() < self.capacity());

        let n_roots = self.n_roots();
        self.raw_frame[n_roots + 2] = value.cast();
        if n_roots == self.n_slots() {
            self.set_n_slots(n_roots + 1);
        }
    }

    pub(crate) fn nest_async_with_output<'nested>(
        &'nested mut self,
        capacity: usize,
    ) -> JlrsResult<AsyncGcFrame<'nested>> {
        if self.capacity() == self.n_slots() {
            Err(JlrsError::AllocError(AllocError::FrameOverflow(
                1,
                self.capacity(),
            )))?
        }

        let needed = MIN_FRAME_CAPACITY.max(capacity) + 2;
        let (output, raw_frame) = if let Some(output) = self.output.take() {
            let used = self.n_slots() + 2;

            if used + needed > self.raw_frame.len() {
                if self.page.is_none() || self.page.as_ref().unwrap().len() < needed {
                    let raw = vec![null_mut::<c_void>(); PAGE_SIZE.max(needed)];
                    let page = raw.into_boxed_slice();
                    self.page = Some(page);
                }

                (output, self.page.as_mut().unwrap().as_mut())
            } else {
                (output, &mut self.raw_frame[used..])
            }
        } else {
            let used = self.n_slots() + 3;

            if used + needed > self.raw_frame.len() {
                if self.page.is_none() || self.page.as_ref().unwrap().len() < needed {
                    let raw = vec![null_mut::<c_void>(); PAGE_SIZE.max(needed)];
                    let page = raw.into_boxed_slice();
                    self.page = Some(page);
                }

                (
                    &mut self.raw_frame[used],
                    self.page.as_mut().unwrap().as_mut(),
                )
            } else {
                self.raw_frame[used..].split_first_mut().unwrap()
            }
        };

        let mut frame = AsyncGcFrame::new(raw_frame, capacity, self.mode);
        frame.output = Some(output);
        Ok(frame)
    }
}

#[cfg(all(feature = "async", target_os = "linux"))]
impl<'frame> Drop for AsyncGcFrame<'frame> {
    fn drop(&mut self) {
        // The frame was pushed when the frame was created.
        unsafe { self.mode.pop_frame(self.raw_frame, Internal) }
    }
}

/// A `NullFrame` can be used if you call Rust from Julia through `ccall` and want to borrow array
/// data but not perform any allocations. It can't be stacked or used for functions that
/// allocate (like creating new values or calling functions). Functions that depend on allocation
/// will return `JlrsError::NullFrame` if you call them with a `NullFrame`.
pub struct NullFrame<'frame>(PhantomData<&'frame ()>);

impl<'frame> NullFrame<'frame> {
    pub(crate) unsafe fn new(_: &'frame mut CCall) -> Self {
        NullFrame(PhantomData)
    }
}
