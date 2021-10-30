//! Frame that can be used with async functions.

use super::mode::Async;
use super::output_result_ext::OutputResultExt;
use crate::memory::{
    frame::Frame, reusable_slot::ReusableSlot, root_pending::RootPending, stack_page::StackPage,
};
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
    cell::Cell,
    ffi::c_void,
    ptr::{null_mut, NonNull},
};

/// A frame is used to root Julia data, which guarantees the garbage collector doesn't free the
/// data while the frame has not been dropped. More information about this topic can be found in
/// the [`memory`] module.
///
/// An `AsyncGcFrame` offers the same functionality as a [`GcFrame`], and some additional async
/// methods that can be used to create nested async scopes. It can also be used to call the trait
/// methods of [`CallAsync`].
///
/// [`CallAsync`]: crate::extensions::multitask::call_async::CallAsync
/// [`memory`]: crate::memory
pub struct AsyncGcFrame<'frame> {
    raw_frame: &'frame mut [Cell<*mut c_void>],
    page: Option<StackPage>,
    output: Option<&'frame mut Cell<*mut c_void>>,
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
                output.set(ptr.cast().as_ptr());
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
                output.set(ptr.cast().as_ptr());
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
                output.set(ptr.cast().as_ptr());
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
                output.set(ptr.cast().as_ptr());
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
        self.raw_frame[0].get() as usize >> 2
    }

    /// Returns the maximum number of slots this frame can use.
    pub fn capacity(&self) -> usize {
        self.raw_frame.len() - 2
    }

    // Safety: this frame must be dropped in the same scope it has been created and raw_frame must
    // have 2 + slots capacity available.
    pub(crate) unsafe fn new(
        raw_frame: &'frame mut [Cell<*mut c_void>],
        mode: Async<'frame>,
    ) -> Self {
        // Is popped when this frame is dropped
        mode.push_frame(raw_frame, Private);

        AsyncGcFrame {
            raw_frame,
            page: None,
            output: None,
            mode,
        }
    }

    // Safety: capacity >= n_slots
    pub(crate) unsafe fn set_n_roots(&mut self, n_slots: usize) {
        debug_assert!(n_slots <= self.capacity());
        self.raw_frame.get_unchecked_mut(0).set((n_slots << 2) as _);
    }

    // Safety: this frame must be dropped in the same scope it has been created.
    pub(crate) unsafe fn nest<'nested>(
        &'nested mut self,
        capacity: usize,
    ) -> GcFrame<'nested, Async<'frame>> {
        let used = self.n_roots() + 2;
        let new_frame_size = MIN_FRAME_CAPACITY.max(capacity) + 2;
        let raw_frame = if self.page.is_some() {
            if new_frame_size <= self.page.as_ref().unwrap().size() {
                self.page.as_mut().unwrap().as_mut()
            } else {
                self.page = Some(StackPage::new(new_frame_size));
                self.page.as_mut().unwrap().as_mut()
            }
        } else if used + new_frame_size <= self.raw_frame.len() {
            &mut self.raw_frame[used..]
        } else {
            self.page = Some(StackPage::new(new_frame_size));
            self.page.as_mut().unwrap().as_mut()
        };

        GcFrame::new(raw_frame, self.mode)
    }

    // Safety: this frame must be dropped in the same scope it has been created.
    pub(crate) unsafe fn nest_async<'nested>(
        &'nested mut self,
        capacity: usize,
    ) -> AsyncGcFrame<'nested> {
        let used = self.n_roots() + 2;
        let new_frame_size = MIN_FRAME_CAPACITY.max(capacity) + 2;
        let raw_frame = if self.page.is_some() {
            if new_frame_size <= self.page.as_ref().unwrap().size() {
                self.page.as_mut().unwrap().as_mut()
            } else {
                self.page = Some(StackPage::new(new_frame_size));
                self.page.as_mut().unwrap().as_mut()
            }
        } else if used + new_frame_size <= self.raw_frame.len() {
            &mut self.raw_frame[used..]
        } else {
            self.page = Some(StackPage::new(new_frame_size));
            self.page.as_mut().unwrap().as_mut()
        };

        AsyncGcFrame::new(raw_frame, self.mode)
    }

    // Safety: n_roots < capacity
    pub(crate) unsafe fn root(&mut self, value: NonNull<jl_value_t>) {
        debug_assert!(self.n_roots() < self.capacity());

        let n_roots = self.n_roots();
        self.raw_frame
            .get_unchecked_mut(n_roots + 2)
            .set(value.cast().as_ptr());
        self.set_n_roots(n_roots + 1);
    }

    // Safety: this frame must be dropped in the same scope it has been created.
    pub(crate) unsafe fn nest_async_with_output<'nested>(
        &'nested mut self,
        capacity: usize,
    ) -> JlrsResult<AsyncGcFrame<'nested>> {
        if self.capacity() == self.n_roots() {
            Err(JlrsError::AllocError(AllocError::FrameOverflow(
                1,
                self.capacity(),
            )))?
        }

        let new_frame_size = MIN_FRAME_CAPACITY.max(capacity) + 2;
        let (output, raw_frame) = if let Some(output) = self.output.take() {
            let used = self.n_roots() + 2;

            let new_frame_size = MIN_FRAME_CAPACITY.max(capacity) + 2;
            let raw_frame = if self.page.is_some() {
                if new_frame_size <= self.page.as_ref().unwrap().size() {
                    self.page.as_mut().unwrap().as_mut()
                } else {
                    self.page = Some(StackPage::new(new_frame_size));
                    self.page.as_mut().unwrap().as_mut()
                }
            } else if used + new_frame_size <= self.raw_frame.len() {
                &mut self.raw_frame[used..]
            } else {
                self.page = Some(StackPage::new(new_frame_size));
                self.page.as_mut().unwrap().as_mut()
            };

            (output, raw_frame)
        } else {
            let used = self.n_roots() + 3;
            if self.page.is_some() {
                if new_frame_size > self.page.as_ref().unwrap().size() {
                    self.page = Some(StackPage::new(new_frame_size));
                }

                (
                    &mut self.raw_frame[used],
                    self.page.as_mut().unwrap().as_mut(),
                )
            } else if used + new_frame_size <= self.raw_frame.len() {
                self.raw_frame[used..].split_first_mut().unwrap()
            } else {
                self.page = Some(StackPage::new(new_frame_size));
                (
                    &mut self.raw_frame[used],
                    self.page.as_mut().unwrap().as_mut(),
                )
            }
        };

        let mut frame = AsyncGcFrame::new(raw_frame, self.mode);
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
    fn reusable_slot(&mut self) -> JlrsResult<ReusableSlot<'frame>> {
        ReusableSlot::new(self)
    }

    fn n_roots(&self) -> usize {
        self.n_roots()
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

    unsafe fn reserve_slot(&mut self, _: Private) -> JlrsResult<*mut *mut c_void> {
        let n_roots = self.n_roots();
        if n_roots == self.capacity() {
            Err(JlrsError::alloc_error(AllocError::FrameOverflow(
                1, n_roots,
            )))?;
        }

        self.raw_frame
            .get_unchecked_mut(n_roots + 2)
            .set(null_mut());
        self.set_n_roots(n_roots + 1);

        Ok(self.raw_frame.get_unchecked_mut(n_roots + 2).as_ptr())
    }
    unsafe fn nest<'nested>(
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
        let mut nested = unsafe { self.nest(0) };
        func(&mut nested)
    }

    fn scope_with_slots<T, F>(&mut self, capacity: usize, func: F, _: Private) -> JlrsResult<T>
    where
        for<'inner> F: FnOnce(&mut GcFrame<'inner, Self::Mode>) -> JlrsResult<T>,
    {
        let mut nested = unsafe { self.nest(capacity) };
        func(&mut nested)
    }
}
