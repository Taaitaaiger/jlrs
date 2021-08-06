//! Frame that can be used with async functions.

use super::mode::Async;
use super::output_result_ext::OutputResultExt;
use crate::memory::{frame::Frame, root_pending::RootPending, stack_page::StackPage};
use crate::{
    error::{AllocError, JlrsError, JlrsResult, JuliaResult},
    memory::{
        frame::{GcFrame, MIN_FRAME_CAPACITY},
        mode::private::Mode,
        output::{Output, OutputResult, OutputValue},
    },
    private::Private,
    wrappers::ptr::value::Value,
};
use crate::{memory::frame::private, wrappers::ptr::private::Wrapper as _};
use futures::Future;
use jl_sys::jl_value_t;
use std::{
    ffi::c_void,
    ptr::{null_mut, NonNull},
};

/// A frame that can be used to root values and dispatch Julia function calls to another thread
/// with [`CallAsync::call_async`]. An `AsyncGcFrame` is available by implementing the `AsyncTask`
/// trait, this struct provides create a nested async scope.
///
/// Roots are stored in slots, each slot can contain one root. Frames created with slots will
/// preallocate that number of slots. Frames created without slots will dynamically create new
/// slots as needed. A frame is able to create at least 16 slots. If there is sufficient capacity
/// available, a new frame will use this remaining capacity. If the capacity is insufficient, more
/// stack space is allocated.
///
/// [`CallAsync::call_async`]: crate::extensions::multitask::call_async::CallAsync
pub struct AsyncGcFrame<'frame> {
    raw_frame: &'frame mut [*mut c_void],
    n_roots: usize,
    page: Option<StackPage>,
    output: Option<&'frame mut *mut c_void>,
    mode: Async<'frame>,
}

impl<'frame> AsyncGcFrame<'frame> {
    /// An async version of [`Scope::value_scope`]. Rather than a closure, it takes an async
    /// closure that provides a new `AsyncGcFrame`.
    ///
    /// [`Scope::value_scope`]: crate::memory::scope::Scope::value_scope
    pub async fn async_value_scope<'nested, 'data, F, G>(
        &'nested mut self,
        func: F,
    ) -> JlrsResult<Value<'frame, 'data>>
    where
        G: Future<Output = JlrsResult<OutputValue<'frame, 'data, 'nested>>>,
        F: FnOnce(Output<'frame>, &'nested mut AsyncGcFrame<'nested>) -> G,
    {
        unsafe {
            let mut nested = self.nest_async_with_output(0)?;
            let p_nested = &mut nested as *mut _;
            let r_nested = &mut *p_nested;
            let output = Output::new();
            let ptr = func(output, r_nested).await?.unwrap_non_null();

            if let Some(output) = nested.output.take() {
                *output = ptr.cast().as_ptr();
            }

            Ok(Value::wrap_non_null(ptr, Private))
        }
    }

    /// An async version of [`Scope::value_scope_with_slots`]. Rather than a closure, it takes an
    /// async closure that provides a new `AsyncGcFrame`.
    ///
    /// [`Scope::value_scope_with_slots`]: crate::memory::scope::Scope::value_scope_with_slots
    pub async fn async_value_scope_with_slots<'nested, 'data, F, G>(
        &'nested mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<Value<'frame, 'data>>
    where
        G: Future<Output = JlrsResult<OutputValue<'frame, 'data, 'nested>>>,
        F: FnOnce(Output<'frame>, &'nested mut AsyncGcFrame<'nested>) -> G,
    {
        unsafe {
            let mut nested = self.nest_async_with_output(capacity)?;
            let p_nested = &mut nested as *mut _;
            let r_nested = &mut *p_nested;
            let output = Output::new();
            let ptr = func(output, r_nested).await?.unwrap_non_null();

            if let Some(output) = nested.output.take() {
                *output = ptr.cast().as_ptr();
            }

            Ok(Value::wrap_non_null(ptr, Private))
        }
    }

    /// An async version of [`Scope::result_scope`]. Rather than a closure, it takes an async
    /// closure that provides a new `AsyncGcFrame`.
    ///
    /// [`Scope::result_scope`]: crate::memory::scope::Scope::result_scope
    pub async fn async_result_scope<'nested, 'data, F, G>(
        &'nested mut self,
        func: F,
    ) -> JlrsResult<JuliaResult<'frame, 'data>>
    where
        G: Future<Output = JlrsResult<OutputResult<'frame, 'data, 'nested>>>,
        F: FnOnce(Output<'frame>, &'nested mut AsyncGcFrame<'nested>) -> G,
    {
        unsafe {
            let mut nested = self.nest_async_with_output(0)?;
            let p_nested = &mut nested as *mut _;
            let r_nested = &mut *p_nested;
            let output = Output::new();
            let res = func(output, r_nested).await?;
            let is_exc = res.is_exception();
            let ptr = res.unwrap_non_null();

            if let Some(output) = nested.output.take() {
                *output = ptr.cast().as_ptr();
            }

            if is_exc {
                Ok(JuliaResult::Err(Value::wrap_non_null(ptr, Private)))
            } else {
                Ok(JuliaResult::Ok(Value::wrap_non_null(ptr, Private)))
            }
        }
    }

    /// An async version of [`Scope::result_scope_with_slots`]. Rather than a closure, it takes an
    /// async closure that provides a new `AsyncGcFrame`.
    ///
    /// [`Scope::result_scope_with_slots`]: crate::memory::scope::Scope::result_scope_with_slots
    pub async fn async_result_scope_with_slots<'nested, 'data, F, G>(
        &'nested mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<JuliaResult<'frame, 'data>>
    where
        G: Future<Output = JlrsResult<OutputResult<'frame, 'data, 'nested>>>,
        F: FnOnce(Output<'frame>, &'nested mut AsyncGcFrame<'nested>) -> G,
    {
        unsafe {
            let mut nested = self.nest_async_with_output(capacity)?;
            let p_nested = &mut nested as *mut _;
            let r_nested = &mut *p_nested;
            let output = Output::new();
            let res = func(output, r_nested).await?;
            let is_exc = res.is_exception();
            let ptr = res.unwrap_non_null();

            if let Some(output) = nested.output.take() {
                *output = ptr.cast().as_ptr();
            }

            if is_exc {
                Ok(JuliaResult::Err(Value::wrap_non_null(ptr, Private)))
            } else {
                Ok(JuliaResult::Ok(Value::wrap_non_null(ptr, Private)))
            }
        }
    }

    /// An async version of [`ScopeExt::scope`]. Rather than a closure, it takes an async closure
    /// that provides a new `AsyncGcFrame`.
    ///
    /// [`ScopeExt::scope`]: crate::memory::scope::ScopeExt::scope
    pub async fn async_scope<'nested, T, F, G>(&'nested mut self, func: F) -> JlrsResult<T>
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

    /// An async version of [`ScopeExt::scope_with_slots`]. Rather than a closure, it takes an
    /// async closure that provides a new `AsyncGcFrame`.
    ///
    /// [`ScopeExt::scope_with_slots`]: crate::memory::scope::ScopeExt::scope_with_slots
    pub async fn async_scope_with_slots<'nested, T, F, G>(
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
        self.n_roots
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

        // The new number of slots doesn't  exceed the capacity, and the new slots have been cleared
        unsafe { self.set_n_slots(slots + additional) }
        true
    }

    // Safety: must be dropped
    pub(crate) unsafe fn new(
        raw_frame: &'frame mut [*mut c_void],
        capacity: usize,
        mode: Async<'frame>,
    ) -> Self {
        // Is popped when this frame is dropped
        mode.push_frame(raw_frame, capacity, Private);

        AsyncGcFrame {
            raw_frame,
            n_roots: 0,
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
            if self.page.is_none() || self.page.as_ref().unwrap().size() < needed {
                self.page = Some(StackPage::new(needed));
            }

            self.page.as_mut().unwrap().as_mut()
        } else {
            &mut self.raw_frame[used..]
        };

        GcFrame::new(raw_frame, capacity, self.mode)
    }

    // Safety: frame must be dropped
    pub(crate) unsafe fn nest_async<'nested>(
        &'nested mut self,
        capacity: usize,
    ) -> AsyncGcFrame<'nested> {
        let used = self.n_slots() + 2;
        let needed = MIN_FRAME_CAPACITY.max(capacity) + 2;
        let raw_frame = if used + needed > self.raw_frame.len() {
            if self.page.is_none() || self.page.as_ref().unwrap().size() < needed {
                self.page = Some(StackPage::new(needed));
            }

            self.page.as_mut().unwrap().as_mut()
        } else {
            &mut self.raw_frame[used..]
        };

        AsyncGcFrame::new(raw_frame, capacity, self.mode)
    }

    // Safety: n_roots < capacity
    pub(crate) unsafe fn root(&mut self, value: NonNull<jl_value_t>) {
        debug_assert!(self.n_roots() < self.capacity());

        let n_roots = self.n_roots();
        self.raw_frame[n_roots + 2] = value.cast().as_ptr();
        if n_roots == self.n_slots() {
            self.set_n_slots(n_roots + 1);
        }
    }

    // Safety: frame must be dropped
    pub(crate) unsafe fn nest_async_with_output<'nested>(
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
                if self.page.is_none() || self.page.as_ref().unwrap().size() < needed {
                    self.page = Some(StackPage::new(needed));
                }

                (output, self.page.as_mut().unwrap().as_mut())
            } else {
                (output, &mut self.raw_frame[used..])
            }
        } else {
            let used = self.n_slots() + 3;

            if used + needed > self.raw_frame.len() {
                if self.page.is_none() || self.page.as_ref().unwrap().size() < needed {
                    self.page = Some(StackPage::new(needed));
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

impl<'frame> Drop for AsyncGcFrame<'frame> {
    fn drop(&mut self) {
        // The frame was pushed when the frame was created.
        unsafe { self.mode.pop_frame(self.raw_frame, Private) }
    }
}

impl<'frame> Frame<'frame> for AsyncGcFrame<'frame> {
    #[must_use]
    fn alloc_slots(&mut self, additional: usize) -> bool {
        self.alloc_slots(additional)
    }

    fn n_roots(&self) -> usize {
        self.n_roots()
    }

    fn n_slots(&self) -> usize {
        self.n_slots()
    }

    fn capacity(&self) -> usize {
        self.capacity()
    }
}

impl<'frame> private::Frame<'frame> for AsyncGcFrame<'frame> {
    type Mode = Async<'frame>;

    unsafe fn push_root<'data>(
        &mut self,
        value: NonNull<jl_value_t>,
        _: Private,
    ) -> Result<Value<'frame, 'data>, AllocError> {
        let n_roots = self.n_roots();
        if n_roots == self.capacity() {
            return Err(AllocError::FrameOverflow(1, n_roots));
        }

        self.root(value);
        Ok(Value::wrap_non_null(value, Private))
    }

    fn nest<'nested>(
        &'nested mut self,
        capacity: usize,
        _: Private,
    ) -> GcFrame<'nested, Self::Mode> {
        self.nest(capacity)
    }

    fn value_scope<'data, F>(&mut self, func: F, _: Private) -> JlrsResult<Value<'frame, 'data>>
    where
        for<'nested, 'inner> F: FnOnce(
            Output<'frame>,
            &'inner mut GcFrame<'nested, Self::Mode>,
        ) -> JlrsResult<OutputValue<'frame, 'data, 'inner>>,
    {
        unsafe {
            let v = {
                let mut nested = self.nest(0);
                let out = Output::new();
                func(out, &mut nested)?.into_pending()
            };

            Value::root_pending(self, v)
        }
    }

    fn value_scope_with_slots<'data, F>(
        &mut self,
        capacity: usize,
        func: F,
        _: Private,
    ) -> JlrsResult<Value<'frame, 'data>>
    where
        for<'nested, 'inner> F: FnOnce(
            Output<'frame>,
            &'inner mut GcFrame<'nested, Self::Mode>,
        ) -> JlrsResult<OutputValue<'frame, 'data, 'inner>>,
    {
        unsafe {
            let v = {
                let mut nested = self.nest(capacity);
                let out = Output::new();
                func(out, &mut nested)?.into_pending()
            };

            Value::root_pending(self, v)
        }
    }

    fn result_scope<'data, F>(
        &mut self,
        func: F,
        _: Private,
    ) -> JlrsResult<JuliaResult<'frame, 'data>>
    where
        for<'nested, 'inner> F: FnOnce(
            Output<'frame>,
            &'inner mut GcFrame<'nested, Self::Mode>,
        ) -> JlrsResult<OutputResult<'frame, 'data, 'inner>>,
    {
        unsafe {
            let v = {
                let mut nested = self.nest(0);
                let out = Output::new();
                func(out, &mut nested)?.into_pending()
            };

            JuliaResult::root_pending(self, v)
        }
    }

    fn result_scope_with_slots<'data, F>(
        &mut self,
        capacity: usize,
        func: F,
        _: Private,
    ) -> JlrsResult<JuliaResult<'frame, 'data>>
    where
        for<'nested, 'inner> F: FnOnce(
            Output<'frame>,
            &'inner mut GcFrame<'nested, Self::Mode>,
        ) -> JlrsResult<OutputResult<'frame, 'data, 'inner>>,
    {
        unsafe {
            let v = {
                let mut nested = self.nest(capacity);
                let out = Output::new();
                func(out, &mut nested)?.into_pending()
            };

            JuliaResult::root_pending(self, v)
        }
    }

    fn scope<T, F>(&mut self, func: F, _: Private) -> JlrsResult<T>
    where
        for<'inner> F: FnOnce(&mut GcFrame<'inner, Self::Mode>) -> JlrsResult<T>,
    {
        let mut nested = self.nest(0);
        func(&mut nested)
    }

    fn scope_with_slots<T, F>(&mut self, capacity: usize, func: F, _: Private) -> JlrsResult<T>
    where
        for<'inner> F: FnOnce(&mut GcFrame<'inner, Self::Mode>) -> JlrsResult<T>,
    {
        let mut nested = self.nest(capacity);
        func(&mut nested)
    }
}
