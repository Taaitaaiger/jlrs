//! Different frame types that prevent data from being freed by the garbage collector while it's
//! in use.
//!
//! The garbage collector owns all Julia data and is not automatically aware of references to this
//! data existing outside of Julia. In order to prevent data from being freed by the garbage
//! collector while it's used from Rust this data must be rooted, or stored, in a frame (or at
//! least be reachable from such a root). A new frame is created whenever a new scope is created
//! and dropped when its scope ends.
//!
//! Several frame types exist in jlrs. They all implement the [`Frame`] trait, mutable references
//! to them implement [`Scope`] and [`PartialScope`]. The `Frame` trait provides methods that
//! return info about that frame, like its capacity and current number of roots, and methods to
//! reserve a new [`Output`] or [`ReusableSlot`], and create a nested scope with its own frame.
//! Only [`AsyncGcFrame`] provides additional public methods, these methods create a nested async
//! scope with its own async frame.
//!
//! [`Scope`]: crate::memory::scope::Scope
//! [`PartialScope`]: crate::memory::scope::PartialScope
//! [`CallAsync`]: crate::call::CallAsync

use super::{mode::Mode, output::Output, reusable_slot::ReusableSlot, stack_page::StackPage};
use crate::{error::JlrsResult, private::Private};
use jl_sys::jl_value_t;
use std::{cell::Cell, ffi::c_void, ptr::NonNull};

pub(crate) const MIN_FRAME_CAPACITY: usize = 16;

/// A frame that can be used to root Julia data.
///
/// Frames created with a capacity can store at least that number of roots. A frame's capacity is
/// at least 16.
pub struct GcFrame<'frame, M: Mode> {
    raw_frame: &'frame mut [Cell<*mut c_void>],
    page: Option<StackPage>,
    mode: M,
}

impl<'frame, M: Mode> GcFrame<'frame, M> {
    #[inline]
    pub(crate) fn n_roots(&self) -> usize {
        self.raw_frame[0].get() as usize >> 2
    }

    #[inline]
    pub(crate) fn capacity(&self) -> usize {
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

    // Safety: this frame must be dropped in the same scope it has been created.
    pub(crate) unsafe fn new(raw_frame: &'frame mut [Cell<*mut c_void>], mode: M) -> Self {
        mode.push_frame(raw_frame, Private);

        GcFrame {
            raw_frame,
            page: None,
            mode,
        }
    }

    // Safety: capacity >= n_slots, the n_roots pointers the garbage collector
    // can see must all be null or point to valid Julia data.
    pub(crate) unsafe fn set_n_roots(&mut self, n_roots: usize) {
        debug_assert!(self.capacity() >= n_roots);
        self.raw_frame.get_unchecked_mut(0).set((n_roots << 2) as _);
    }

    // Safety: capacity > n_roots, and the pointer must point to valid Julia data
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

cfg_if::cfg_if! {
    if #[cfg(feature = "ccall")] {
        use crate::{ccall::CCall, error::JlrsError};
        use std::marker::PhantomData;

        /// A `NullFrame` can be used if you call Rust from Julia through `ccall` and want to borrow array
        /// data but not perform any allocations. It can't be used to created a new scope or root Julia
        /// data. If you try to do so `JlrsError::NullFrame` is returned.
        pub struct NullFrame<'frame>(PhantomData<&'frame ()>);

        impl<'frame> NullFrame<'frame> {
            pub(crate) unsafe fn new(_: &'frame mut CCall) -> Self {
                NullFrame(PhantomData)
            }
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "async")] {
        use super::mode::Async;
        use super::mode::private::Mode as _;
        use std::future::Future;

        /// A frame is used to root Julia data, which guarantees the garbage collector doesn't free the
        /// data while the frame has not been dropped. More information about this topic can be found in
        /// the [`memory`] module.
        ///
        /// An `AsyncGcFrame` offers the same functionality as a [`GcFrame`], and some additional async
        /// methods that can be used to create nested async scopes. It can also be used to call the trait
        /// methods of [`CallAsync`].
        ///
        /// [`CallAsync`]: crate::call::CallAsync
        /// [`memory`]: crate::memory
        pub struct AsyncGcFrame<'frame> {
            raw_frame: &'frame mut [Cell<*mut c_void>],
            page: Option<StackPage>,
            mode: Async<'frame>,
        }

        impl<'frame> AsyncGcFrame<'frame> {
            /// An async version of [`Frame::scope`]. Rather than a closure, it takes an async closure
            /// that provides a new `AsyncGcFrame`.
            #[inline(never)]
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

            /// An async version of [`Frame::scope_with_capacity`]. Rather than a closure, it takes an
            /// async closure that provides a new `AsyncGcFrame`.
            #[inline(never)]
            pub async fn async_scope_with_capacity<'nested, T, F, G>(
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
            #[inline]
            pub(crate) fn n_roots(&self) -> usize {
                self.raw_frame[0].get() as usize >> 2
            }

            /// Returns the maximum number of slots this frame can use.
            #[inline]
            pub(crate) fn capacity(&self) -> usize {
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
        }

        impl<'frame> Drop for AsyncGcFrame<'frame> {
            fn drop(&mut self) {
                // The frame was pushed when the frame was created.
                unsafe { self.mode.pop_frame(self.raw_frame, Private) }
            }
        }

        impl<'frame> Frame<'frame> for AsyncGcFrame<'frame> {
            fn reusable_slot(&mut self) -> JlrsResult<ReusableSlot<'frame>> {
                unsafe {
                    let slot = <Self as private::Frame>::reserve_slot(self, Private)?;
                    Ok(ReusableSlot::new(self, slot))
                }
            }

            fn n_roots(&self) -> usize {
                self.n_roots()
            }

            fn capacity(&self) -> usize {
                self.capacity()
            }

            fn reserve_output(&mut self) -> JlrsResult<Output<'frame>> {
                unsafe {
                    let slot = <Self as private::Frame>::reserve_slot(self, Private)?;
                    Ok(Output::new(self, slot))
                }
            }
        }
    }
}

/// Functionality shared by the different frame types.
pub trait Frame<'frame>: private::Frame<'frame> {
    /// This method takes a mutable reference to a frame and returns it; this method can be used
    /// as an alternative to reborrowing a frame with `&mut *frame` when a [`Scope`] or
    /// [`PartialScope`] is needed.
    ///
    /// [`Scope`]: crate::memory::scope::Scope
    /// [`PartialScope`]: crate::memory::scope::PartialScope
    fn as_scope(&mut self) -> &mut Self {
        self
    }

    /// Reserve a new output in the current frame. Returns an error if the frame is full.
    fn reserve_output(&mut self) -> JlrsResult<Output<'frame>>;

    /// Create a new reusable slot in the current frame. Returns an error if the frame is full.
    fn reusable_slot(&mut self) -> JlrsResult<ReusableSlot<'frame>>;

    /// Returns the number of values currently rooted in this frame.
    fn n_roots(&self) -> usize;

    /// Returns the maximum number of values that can be rooted in this frame.
    fn capacity(&self) -> usize;

    /// Creates a new `GcFrame` and calls `func` with it. The new frame is popped from the GC stack
    /// stack after `func` returns.
    #[inline(never)]
    fn scope<T, F>(&mut self, func: F) -> JlrsResult<T>
    where
        for<'inner> F: FnOnce(&mut GcFrame<'inner, Self::Mode>) -> JlrsResult<T>,
    {
        unsafe {
            let mut nested = self.nest(0, Private);
            func(&mut nested)
        }
    }

    /// Creates a frame that can store at least `capacity` roots and calls `func` with this new
    /// frame. The new frame is popped from the GC stack after `func` returns.
    #[inline(never)]
    fn scope_with_capacity<T, F>(&mut self, capacity: usize, func: F) -> JlrsResult<T>
    where
        for<'inner> F: FnOnce(&mut GcFrame<'inner, Self::Mode>) -> JlrsResult<T>,
    {
        unsafe {
            let mut nested = self.nest(capacity, Private);
            func(&mut nested)
        }
    }
}

impl<'frame, M: Mode> Frame<'frame> for GcFrame<'frame, M> {
    fn reusable_slot(&mut self) -> JlrsResult<ReusableSlot<'frame>> {
        unsafe {
            let slot = <Self as private::Frame>::reserve_slot(self, Private)?;
            Ok(ReusableSlot::new(self, slot))
        }
    }

    fn n_roots(&self) -> usize {
        self.n_roots()
    }

    fn capacity(&self) -> usize {
        self.capacity()
    }

    fn reserve_output(&mut self) -> JlrsResult<Output<'frame>> {
        unsafe {
            let slot = <Self as private::Frame>::reserve_slot(self, Private)?;
            Ok(Output::new(self, slot))
        }
    }
}

#[cfg(feature = "ccall")]
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

    #[inline(never)]
    fn scope<T, F>(&mut self, _func: F) -> JlrsResult<T>
    where
        for<'inner> F: FnOnce(&mut GcFrame<'inner, Self::Mode>) -> JlrsResult<T>,
    {
        Err(JlrsError::NullFrame)?
    }

    #[inline(never)]
    fn scope_with_capacity<T, F>(&mut self, _capacity: usize, _func: F) -> JlrsResult<T>
    where
        for<'inner> F: FnOnce(&mut GcFrame<'inner, Self::Mode>) -> JlrsResult<T>,
    {
        Err(JlrsError::NullFrame)?
    }

    fn reserve_output(&mut self) -> JlrsResult<Output<'frame>> {
        Err(JlrsError::NullFrame)?
    }
}

pub(crate) mod private {
    use std::{
        cell::Cell,
        ffi::c_void,
        ptr::{null_mut, NonNull},
    };

    use crate::error::{JlrsError, JlrsResult};
    use crate::memory::frame::GcFrame;
    #[cfg(feature = "ccall")]
    use crate::memory::frame::NullFrame;
    use crate::memory::mode::Mode;
    #[cfg(feature = "ccall")]
    use crate::memory::mode::Sync;
    use crate::wrappers::ptr::private::Wrapper;
    use crate::{error::AllocError, private::Private};

    pub trait Frame<'frame> {
        type Mode: Mode;
        // protect the value from being garbage collected while this frame is active.
        // safety: the value must be a valid pointer to a Julia value.
        unsafe fn push_root<'data, T: Wrapper<'frame, 'data>>(
            &mut self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Result<T, AllocError>;

        // safety: this pointer must only be used while the frame exists.
        unsafe fn reserve_slot(&mut self, _: Private) -> JlrsResult<*const Cell<*mut c_void>>;

        // safety: the nested frame must be dropped in the same scope as it has been created in.
        unsafe fn nest<'nested>(
            &'nested mut self,
            capacity: usize,
            _: Private,
        ) -> GcFrame<'nested, Self::Mode>;
    }

    impl<'frame, M: Mode> Frame<'frame> for GcFrame<'frame, M> {
        type Mode = M;

        unsafe fn push_root<'data, T: Wrapper<'frame, 'data>>(
            &mut self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Result<T, AllocError> {
            let n_roots = self.n_roots();
            if n_roots == self.capacity() {
                return Err(AllocError::FrameOverflow(n_roots));
            }

            self.root(value.cast());
            Ok(T::wrap_non_null(value, Private))
        }

        unsafe fn reserve_slot(&mut self, _: Private) -> JlrsResult<*const Cell<*mut c_void>> {
            let n_roots = self.n_roots();
            if n_roots == self.capacity() {
                Err(JlrsError::alloc_error(AllocError::FrameOverflow(n_roots)))?;
            }

            self.raw_frame
                .get_unchecked_mut(n_roots + 2)
                .set(null_mut());
            self.set_n_roots(n_roots + 1);

            Ok(self.raw_frame.get_unchecked_mut(n_roots + 2))
        }

        unsafe fn nest<'nested>(
            &'nested mut self,
            capacity: usize,
            _: Private,
        ) -> GcFrame<'nested, Self::Mode> {
            self.nest(capacity)
        }
    }

    #[cfg(feature = "ccall")]
    impl<'frame> Frame<'frame> for NullFrame<'frame> {
        type Mode = Sync;

        unsafe fn push_root<'data, T: Wrapper<'frame, 'data>>(
            &mut self,
            _value: NonNull<T::Wraps>,
            _: Private,
        ) -> Result<T, AllocError> {
            Err(AllocError::FrameOverflow(0))
        }

        unsafe fn reserve_slot(&mut self, _: Private) -> JlrsResult<*const Cell<*mut c_void>> {
            Err(JlrsError::NullFrame)?
        }

        unsafe fn nest<'nested>(
            &'nested mut self,
            _capacity: usize,
            _: Private,
        ) -> GcFrame<'nested, Self::Mode> {
            unreachable!()
        }
    }

    cfg_if::cfg_if! {
        if #[cfg(feature = "async")] {
            use super::AsyncGcFrame;
            use super::super::mode::Async;

            impl<'frame> Frame<'frame> for AsyncGcFrame<'frame> {
                type Mode = Async<'frame>;

                unsafe fn push_root<'data, T: Wrapper<'frame, 'data>>(
                    &mut self,
                    value: NonNull<T::Wraps>,
                    _: Private,
                ) -> Result<T, AllocError> {
                    let n_roots = self.n_roots();
                    if n_roots == self.capacity() {
                        return Err(AllocError::FrameOverflow(n_roots));
                    }

                    self.root(value.cast());
                    Ok(T::wrap_non_null(value, Private))
                }

                unsafe fn reserve_slot(&mut self, _: Private) -> JlrsResult<*const Cell<*mut c_void>> {
                    let n_roots = self.n_roots();
                    if n_roots == self.capacity() {
                        Err(JlrsError::alloc_error(AllocError::FrameOverflow(n_roots)))?;
                    }

                    self.raw_frame
                        .get_unchecked_mut(n_roots + 2)
                        .set(null_mut());
                    self.set_n_roots(n_roots + 1);

                    Ok(self.raw_frame.get_unchecked_mut(n_roots + 2))
                }

                unsafe fn nest<'nested>(
                    &'nested mut self,
                    capacity: usize,
                    _: Private,
                ) -> GcFrame<'nested, Self::Mode> {
                    self.nest(capacity)
                }
            }
        }
    }
}

#[cfg(test)]
#[cfg(feature = "sync-rt")]
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
