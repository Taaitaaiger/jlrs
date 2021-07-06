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

use super::{mode::Mode, stack_page::StackPage};
use crate::{private::Private, CCall};
use jl_sys::jl_value_t;
use std::{
    ffi::c_void,
    marker::PhantomData,
    ptr::{null_mut, NonNull},
};

pub(crate) const MIN_FRAME_CAPACITY: usize = 16;

/// A frame that can be used to root values.
///
/// Roots are stored in slots, each slot can contain one root. Frames created with slots will
/// preallocate that number of slots. Frames created without slots will dynamically create new
/// slots as needed. A frame's capacity is at least 16.
pub struct GcFrame<'frame, M: Mode> {
    raw_frame: &'frame mut [*mut c_void],
    page: Option<StackPage>,
    n_roots: usize,
    mode: M,
}

impl<'frame, M: Mode> GcFrame<'frame, M> {
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
    #[must_use]
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

    // Safety: this frame must be dropped in the same scope it has been created.
    pub(crate) fn nest<'nested>(&'nested mut self, capacity: usize) -> GcFrame<'nested, M> {
        let used = self.n_slots() + 2;
        let new_frame_size = MIN_FRAME_CAPACITY.max(capacity) + 2;
        let raw_frame = if used + new_frame_size > self.raw_frame.len() {
            if self.page.is_none() || self.page.as_ref().unwrap().size() < new_frame_size {
                self.page = Some(StackPage::new(new_frame_size));
            }

            self.page.as_mut().unwrap().as_mut()
        } else {
            &mut self.raw_frame[used..]
        };

        GcFrame::new(raw_frame, capacity, self.mode)
    }

    // Safety: this frame must be dropped in the same scope it has been created.
    pub(crate) fn new(raw_frame: &'frame mut [*mut c_void], slots: usize, mode: M) -> Self {
        unsafe {
            mode.push_frame(raw_frame, slots, Private);

            GcFrame {
                raw_frame,
                page: None,
                n_roots: 0,
                mode,
            }
        }
    }

    // Safety: capacity >= n_slots
    pub(crate) unsafe fn set_n_slots(&mut self, n_slots: usize) {
        debug_assert!(self.capacity() >= n_slots);
        self.raw_frame[0] = (n_slots << 1) as _;
    }

    // Safety: capacity > n_roots
    pub(crate) unsafe fn root(&mut self, value: NonNull<jl_value_t>) {
        debug_assert!(self.n_roots() < self.capacity());

        let n_roots = self.n_roots();
        self.raw_frame[n_roots + 2] = value.cast().as_ptr();
        if n_roots == self.n_slots() {
            self.set_n_slots(n_roots + 1);
        }
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

    /// Reserve `additional` slots in the current frame. Returns `true` on success, or `false` if
    /// `self.n_slots() + additional > self.capacity()`.
    #[must_use]
    fn alloc_slots(&mut self, additional: usize) -> bool;

    /// Returns the number of values currently rooted in this frame.
    fn n_roots(&self) -> usize;

    /// Returns the number of slots that are currently allocated to this frame.
    fn n_slots(&self) -> usize;

    /// Returns the maximum number of slots this frame can use.
    fn capacity(&self) -> usize;
}

impl<'frame, M: Mode> Frame<'frame> for GcFrame<'frame, M> {
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

impl<'frame> Frame<'frame> for NullFrame<'frame> {
    #[must_use]
    fn alloc_slots(&mut self, _additional: usize) -> bool {
        false
    }

    fn n_roots(&self) -> usize {
        0
    }

    fn n_slots(&self) -> usize {
        0
    }

    fn capacity(&self) -> usize {
        0
    }
}

pub(crate) mod private {
    use std::ptr::NonNull;

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

        fn nest<'nested>(
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
            )
                -> JlrsResult<OutputValue<'frame, 'data, 'inner>>,
        {
            let v = {
                let mut nested = self.nest(0);
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
                let mut nested = self.nest(capacity);
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
                let mut nested = self.nest(0);
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
                let mut nested = self.nest(capacity);
                let out = Output::new();
                func(out, &mut nested)?.into_pending()
            };

            unsafe { JuliaResult::root_pending(self, v) }
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

    impl<'frame> Frame<'frame> for NullFrame<'frame> {
        type Mode = Sync;

        unsafe fn push_root<'data>(
            &mut self,
            _value: NonNull<jl_value_t>,
            _: Private,
        ) -> Result<Value<'frame, 'data>, AllocError> {
            Err(AllocError::FrameOverflow(1, 0))
        }

        fn nest<'nested>(
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
