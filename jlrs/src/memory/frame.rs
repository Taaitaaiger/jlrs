//! Frames protect values from garbage collection.
//!
//! The garbage collector owns of all Julia data and is not automatically aware of references to
//! this data existing outside of Julia. In order to prevent data from being freed by the garbage
//! collector while it's used from Rust, this data must be rooted in a frame (or at least
//! reachable from such a root). Frames and scopes are strongly connected, each scope creates a
//! single frame. All frames implement the [`Frame`] trait, all mutable references to them
//! implement [`Scope`] and [`ScopeExt`]. The [`scope`] module contains much information about
//! how frames can be used.
//!
//! Several kinds of frame exist in jlrs. The simplest one is [`NullFrame`], which is only used
//! when writing `ccall`able functions. It doesn't let you root any values or create a nested
//! scope, but can be used to (mutably) borrow array data. If you neither use the async runtime
//! nor write Rust functions that Julia will call, the only frame type you will use is
//! [`GcFrame`]; this frame can be used to root a relatively arbitrary number of values.
//!
//! A [`GcFrame`] can be created with a number of preallocated slots by using one of the
//! `Scope[Ext]::*scope_with_slots` methods or without them by using the
//! `Scope[Ext]::*scope` methods, each slot can root one value. By preallocating the slots less
//! work has to be done to root a value, more slots are allocated if this is necessary. The
//! maximum number of slots that can be allocated is the frame's capacity, which is at least 16.
//! When a new frame is created, the current frame's remaining capacity can be used to store this
//! new frame if it can hold the requested number of slots and provide capacity for 16 slots.  If
//! the remaining capacity is insufficient more stack space is allocated, this doesn't  affect the
//! existing frames.
//!
//! [`scope`]: crate::memory::scope
//! [`Scope`]: crate::memory::scope::Scope
//! [`ScopeExt`]: crate::memory::scope::ScopeExt
//! [`Scope::value_scope`]: crate::memory::scope::Scope::value_scope
//! [`Scope::result_scope`]: crate::memory::scope::Scope::result_scope
//! [`ScopeExt::scope`]: crate::memory::scope::ScopeExt::scope

use super::{mode::Mode, reusable_slot::ReusableSlot, stack_page::StackPage};
use crate::{
    error::{JlrsError, JlrsResult},
    private::Private,
    CCall,
};
use jl_sys::{jl_value_t, jlrs_current_task};
use std::{cell::Cell, ffi::c_void, marker::PhantomData, ptr::NonNull};

pub(crate) const MIN_FRAME_CAPACITY: usize = 16;

/// A frame that can be used to root values.
///
/// Roots are stored in slots, each slot can contain one root. Frames created with slots will
/// preallocate that number of slots. Frames created without slots will dynamically create new
/// slots as needed. A frame's capacity is at least 16.
pub struct GcFrame<'frame, M: Mode> {
    pub(crate) raw_frame: &'frame mut [Cell<*mut c_void>],
    pub(crate) page: Option<StackPage>,
    pub(crate) mode: M,
}

impl<'frame, M: Mode> GcFrame<'frame, M> {
    /// Returns the number of values currently rooted in this frame.
    pub fn n_roots(&self) -> usize {
        self.raw_frame[0].get() as usize >> 1
    }

    pub unsafe fn print_stack(&self) {
        let last = jlrs_current_task();
        jl_sys::jlrs_print_stack(NonNull::new_unchecked(last).as_ref().gcstack);
    }

    /// Returns the maximum number of slots this frame can use.
    pub fn capacity(&self) -> usize {
        self.raw_frame.len() - 2
    }

    // Safety: this frame must be dropped in the same scope it has been created.
    pub(crate) unsafe fn nest<'nested>(&'nested mut self, capacity: usize) -> GcFrame<'nested, M> {
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

    // Safety: this frame must be dropped in the same scope it has been created and raw_frame must
    // have 2 + slots capacity available.
    pub(crate) unsafe fn new(raw_frame: &'frame mut [Cell<*mut c_void>], mode: M) -> Self {
        mode.push_frame(raw_frame, Private);

        GcFrame {
            raw_frame,
            page: None,
            mode,
        }
    }

    // Safety: capacity >= n_slots
    pub(crate) unsafe fn set_n_roots(&mut self, n_roots: usize) {
        debug_assert!(self.capacity() >= n_roots);
        self.raw_frame.get_unchecked_mut(0).set((n_roots << 1) as _);
    }

    // Safety: capacity > n_roots
    pub(crate) unsafe fn root(&mut self, value: NonNull<jl_value_t>) {
        debug_assert!(self.n_roots() < self.capacity());

        let n_roots = self.n_roots();
        self.raw_frame
            .get_unchecked_mut(n_roots + 2)
            .set(value.cast().as_ptr());
        self.set_n_roots(n_roots + 1);
    }
}

impl<'frame, M: Mode> Drop for GcFrame<'frame, M> {
    fn drop(&mut self) {
        // The frame was pushed when the frame was created.
        unsafe { self.mode.pop_frame(self.raw_frame, Private) }
    }
}

/// A `NullFrame` can be used if you call Rust from Julia through `ccall` and want to borrow array
/// data but not perform any allocations. It can't be used to created a nested scope or root a
/// newly allocated value. If you try to do so a `JlrsError::NullFrame` is returned.
pub struct NullFrame<'frame>(PhantomData<&'frame ()>);

impl<'frame> NullFrame<'frame> {
    pub(crate) unsafe fn new(_: &'frame mut CCall) -> Self {
        NullFrame(PhantomData)
    }
}

/// This trait provides the functionality shared by the different frame types.
pub trait Frame<'frame>: private::Frame<'frame> {
    /// This method takes a mutable reference to a frame and returns it; this method can be used
    /// as an alternative to reborrowing a frame with `&mut *frame` when a [`Scope`] is needed.
    ///
    /// [`Scope`]: crate::memory::scope::Scope
    fn as_scope(&mut self) -> &mut Self {
        self
    }

    fn reusable_slot(&mut self) -> JlrsResult<ReusableSlot<'frame>>;

    /// Returns the number of values currently rooted in this frame.
    fn n_roots(&self) -> usize;

    /// Returns the maximum number of slots this frame can use.
    fn capacity(&self) -> usize;
}

impl<'frame, M: Mode> Frame<'frame> for GcFrame<'frame, M> {
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

impl<'frame> Frame<'frame> for NullFrame<'frame> {
    fn reusable_slot(&mut self) -> JlrsResult<ReusableSlot<'frame>> {
        Err(JlrsError::NullFrame)?
    }

    fn n_roots(&self) -> usize {
        0
    }

    fn capacity(&self) -> usize {
        0
    }
}

pub(crate) mod private {
    use std::{
        ffi::c_void,
        ptr::{null_mut, NonNull},
    };

    use crate::memory::frame::GcFrame;
    use crate::memory::frame::NullFrame;
    use crate::memory::mode::{Mode, Sync};
    use crate::memory::root_pending::RootPending;
    use crate::wrappers::ptr::private::Wrapper;
    use crate::wrappers::ptr::value::Value;
    use crate::{error::AllocError, private::Private};
    use crate::{
        error::{JlrsError, JlrsResult, JuliaResult},
        memory::output::{Output, OutputResult, OutputValue},
    };
    use jl_sys::jl_value_t;

    pub trait Frame<'frame> {
        type Mode: Mode;
        // protect the value from being garbage collected while this frame is active.
        // safety: the value must be a valid pointer to a Julia value.
        unsafe fn push_root<'data>(
            &mut self,
            value: NonNull<jl_value_t>,
            _: Private,
        ) -> Result<Value<'frame, 'data>, AllocError>;

        unsafe fn reserve_slot(&mut self, _: Private) -> JlrsResult<*mut *mut c_void>;

        unsafe fn nest<'nested>(
            &'nested mut self,
            capacity: usize,
            _: Private,
        ) -> GcFrame<'nested, Self::Mode>;

        fn value_scope<'data, F>(
            &mut self,
            func: F,
            _: Private,
        ) -> JlrsResult<Value<'frame, 'data>>
        where
            for<'nested, 'inner> F: FnOnce(
                Output<'frame>,
                &'inner mut GcFrame<'nested, Self::Mode>,
            )
                -> JlrsResult<OutputValue<'frame, 'data, 'inner>>;

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
            )
                -> JlrsResult<OutputValue<'frame, 'data, 'inner>>;

        fn result_scope<'data, F>(
            &mut self,
            func: F,
            _: Private,
        ) -> JlrsResult<JuliaResult<'frame, 'data>>
        where
            for<'nested, 'inner> F: FnOnce(
                Output<'frame>,
                &'inner mut GcFrame<'nested, Self::Mode>,
            )
                -> JlrsResult<OutputResult<'frame, 'data, 'inner>>;

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
            )
                -> JlrsResult<OutputResult<'frame, 'data, 'inner>>;

        fn scope<T, F>(&mut self, func: F, _: Private) -> JlrsResult<T>
        where
            for<'inner> F: FnOnce(&mut GcFrame<'inner, Self::Mode>) -> JlrsResult<T>;

        fn scope_with_slots<T, F>(&mut self, capacity: usize, func: F, _: Private) -> JlrsResult<T>
        where
            for<'inner> F: FnOnce(&mut GcFrame<'inner, Self::Mode>) -> JlrsResult<T>;
    }

    impl<'frame, M: Mode> Frame<'frame> for GcFrame<'frame, M> {
        type Mode = M;

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
            )
                -> JlrsResult<OutputValue<'frame, 'data, 'inner>>,
        {
            let v = {
                let mut nested = unsafe { self.nest(0) };
                let out = Output::new();
                func(out, &mut nested)?.into_pending()
            };

            unsafe { Value::root_pending(self, v) }
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
            )
                -> JlrsResult<OutputValue<'frame, 'data, 'inner>>,
        {
            let v = {
                let mut nested = unsafe { self.nest(capacity) };
                let out = Output::new();
                func(out, &mut nested)?.into_pending()
            };

            unsafe { Value::root_pending(self, v) }
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
            )
                -> JlrsResult<OutputResult<'frame, 'data, 'inner>>,
        {
            let v = {
                let mut nested = unsafe { self.nest(0) };
                let out = Output::new();
                func(out, &mut nested)?.into_pending()
            };

            unsafe { JuliaResult::root_pending(self, v) }
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
            )
                -> JlrsResult<OutputResult<'frame, 'data, 'inner>>,
        {
            let v = {
                let mut nested = unsafe { self.nest(capacity) };
                let out = Output::new();
                func(out, &mut nested)?.into_pending()
            };

            unsafe { JuliaResult::root_pending(self, v) }
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

    impl<'frame> Frame<'frame> for NullFrame<'frame> {
        type Mode = Sync;

        unsafe fn push_root<'data>(
            &mut self,
            _value: NonNull<jl_value_t>,
            _: Private,
        ) -> Result<Value<'frame, 'data>, AllocError> {
            Err(AllocError::FrameOverflow(1, 0))
        }

        unsafe fn reserve_slot(&mut self, _: Private) -> JlrsResult<*mut *mut c_void> {
            Err(JlrsError::NullFrame)?
        }

        unsafe fn nest<'nested>(
            &'nested mut self,
            _capacity: usize,
            _: Private,
        ) -> GcFrame<'nested, Self::Mode> {
            unreachable!()
        }

        fn value_scope<'data, F>(
            &mut self,
            _func: F,
            _: Private,
        ) -> JlrsResult<Value<'frame, 'data>>
        where
            for<'nested, 'inner> F: FnOnce(
                Output<'frame>,
                &'inner mut GcFrame<'nested, Self::Mode>,
            )
                -> JlrsResult<OutputValue<'frame, 'data, 'inner>>,
        {
            Err(JlrsError::NullFrame)?
        }

        fn value_scope_with_slots<'data, F>(
            &mut self,
            _capacity: usize,
            _func: F,
            _: Private,
        ) -> JlrsResult<Value<'frame, 'data>>
        where
            for<'nested, 'inner> F: FnOnce(
                Output<'frame>,
                &'inner mut GcFrame<'nested, Self::Mode>,
            )
                -> JlrsResult<OutputValue<'frame, 'data, 'inner>>,
        {
            Err(JlrsError::NullFrame)?
        }

        fn result_scope<'data, F>(
            &mut self,
            _func: F,
            _: Private,
        ) -> JlrsResult<JuliaResult<'frame, 'data>>
        where
            for<'nested, 'inner> F: FnOnce(
                Output<'frame>,
                &'inner mut GcFrame<'nested, Self::Mode>,
            )
                -> JlrsResult<OutputResult<'frame, 'data, 'inner>>,
        {
            Err(JlrsError::NullFrame)?
        }

        fn result_scope_with_slots<'data, F>(
            &mut self,
            _capacity: usize,
            _func: F,
            _: Private,
        ) -> JlrsResult<JuliaResult<'frame, 'data>>
        where
            for<'nested, 'inner> F: FnOnce(
                Output<'frame>,
                &'inner mut GcFrame<'nested, Self::Mode>,
            )
                -> JlrsResult<OutputResult<'frame, 'data, 'inner>>,
        {
            Err(JlrsError::NullFrame)?
        }

        fn scope<T, F>(&mut self, _func: F, _: Private) -> JlrsResult<T>
        where
            for<'inner> F: FnOnce(&mut GcFrame<'inner, Self::Mode>) -> JlrsResult<T>,
        {
            Err(JlrsError::NullFrame)?
        }

        fn scope_with_slots<T, F>(
            &mut self,
            _capacity: usize,
            _func: F,
            _: Private,
        ) -> JlrsResult<T>
        where
            for<'inner> F: FnOnce(&mut GcFrame<'inner, Self::Mode>) -> JlrsResult<T>,
        {
            Err(JlrsError::NullFrame)?
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        memory::{frame::GcFrame, mode, stack_page::StackPage},
        util,
        wrappers::ptr::value::Value,
    };

    #[test]
    fn min_stack_pack_size() {
        let mut page = StackPage::new(0);
        assert_eq!(page.as_mut().len(), 64);
    }

    #[test]
    fn create_base_frame() {
        util::JULIA.with(|julia| unsafe {
            let mut julia = julia.borrow_mut();
            let page = julia.get_page();
            let page_size = page.size();

            let frame = GcFrame::new(page.as_mut(), mode::Sync);
            assert_eq!(frame.capacity(), page_size - 2);
            assert_eq!(frame.n_roots(), 0);
        })
    }

    #[test]
    fn push_root() {
        util::JULIA.with(|julia| unsafe {
            let mut julia = julia.borrow_mut();
            let page = julia.get_page();
            let page_size = page.size();
            let mut frame = GcFrame::new(page.as_mut(), mode::Sync);
            let _value = Value::new(&mut frame, 1usize).unwrap();

            assert_eq!(frame.capacity(), page_size - 2);
            assert_eq!(frame.n_roots(), 1);
        })
    }

    #[test]
    fn push_too_many_roots() {
        util::JULIA.with(|julia| unsafe {
            let mut julia = julia.borrow_mut();
            let page = julia.get_page();
            let page_size = page.size();
            let mut frame = GcFrame::new(page.as_mut(), mode::Sync);

            for _ in 0..page_size - 2 {
                let _value = Value::new(&mut frame, 1usize).unwrap();
            }

            assert_eq!(frame.capacity(), page_size - 2);
            assert_eq!(frame.n_roots(), page_size - 2);

            assert!(Value::new(&mut frame, 1usize).is_err());
        })
    }

    #[test]
    fn push_new_frame() {
        util::JULIA.with(|julia| unsafe {
            let mut julia = julia.borrow_mut();
            let page = julia.get_page();
            let page_size = page.size();
            let mut frame = GcFrame::new(page.as_mut(), mode::Sync);

            {
                let nested = frame.nest(0);
                let capacity = nested.capacity();
                assert_eq!(capacity, page_size - 4);
            }
        })
    }

    #[test]
    fn push_large_new_frame() {
        util::JULIA.with(|julia| unsafe {
            let mut julia = julia.borrow_mut();
            let page = julia.get_page();
            let page_size = page.size();
            let mut frame = GcFrame::new(page.as_mut(), mode::Sync);

            {
                let nested = frame.nest(2 * page_size);
                let capacity = nested.capacity();
                let n_roots = nested.n_roots();
                assert_eq!(capacity, 2 * page_size);
                assert_eq!(n_roots, 0);
            }
        })
    }

    #[test]
    fn reuse_large_page() {
        util::JULIA.with(|julia| unsafe {
            let mut julia = julia.borrow_mut();
            let page = julia.get_page();
            let page_size = page.size();
            let mut frame = GcFrame::new(page.as_mut(), mode::Sync);

            {
                frame.nest(2 * page_size);
            }

            {
                let nested = frame.nest(0);
                let capacity = nested.capacity();
                let n_roots = nested.n_roots();
                assert_eq!(capacity, 2 * page_size);
                assert_eq!(n_roots, 0);
            }
        })
    }
}
