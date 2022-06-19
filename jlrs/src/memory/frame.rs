//! Frames store GC roots.
//!
//! Several frame types exist in jlrs. They all implement the [`Frame`] trait, which provides
//! methods that return info about that frame, like its capacity and current number of roots,
//! methods to reserve a new [`Output`] or [`ReusableSlot`], and methods to create a nested
//! scope with its own frame. Only [`AsyncGcFrame`] provides additional public methods.
//!
//! See the [`memory`] module for more information.
//!
//! [`Scope`]: crate::memory::scope::Scope
//! [`PartialScope`]: crate::memory::scope::PartialScope
//! [`CallAsync`]: crate::call::CallAsync
//! [`memory`]: crate::memory

use self::private::FrameOwner;
use crate::{
    error::JlrsResult,
    memory::{
        mode::Mode,
        output::Output,
        reusable_slot::ReusableSlot,
        stack_page::{Slot, StackPage},
    },
    private::Private,
};
use jl_sys::jl_value_t;
use std::ptr::NonNull;

pub(crate) const MIN_FRAME_CAPACITY: usize = 16;

/// A frame that can be used to root Julia data.
///
/// Frames created with a capacity can store at least that number of roots. A frame's capacity is
/// at least 16.
pub struct GcFrame<'frame, M: Mode> {
    raw_frame: &'frame [Slot],
    page: Option<StackPage>,
    mode: M,
}

impl<'frame, M: Mode> GcFrame<'frame, M> {
    // Safety: frames must form a single nested hierarchy. A new frame must only be created when
    // entering a new scope.
    pub(crate) unsafe fn new(raw_frame: &'frame [Slot], mode: M) -> (Self, FrameOwner<'frame, M>) {
        mode.push_frame(raw_frame, Private);

        let owner = FrameOwner::new(raw_frame, mode);
        let frame = GcFrame {
            raw_frame,
            page: None,
            mode,
        };

        (frame, owner)
    }

    // Safety: capacity >= n_slots, the n_roots pointers the garbage collector
    // can see must point to valid Julia data or be null pointers.
    pub(crate) unsafe fn set_n_roots(&self, n_roots: usize) {
        debug_assert!(self.capacity() >= n_roots);
        self.raw_frame.get_unchecked(0).set((n_roots << 2) as _);
    }

    // Safety: capacity > n_roots, value must point to valid Julia data
    pub(crate) unsafe fn root(&self, value: NonNull<jl_value_t>) {
        debug_assert!(self.n_roots() < self.capacity());

        let n_roots = self.n_roots();
        self.raw_frame
            .get_unchecked(n_roots + 2)
            .set(value.cast().as_ptr());
        self.set_n_roots(n_roots + 1);
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "ccall")] {
        use crate::{ccall::CCall, error::AllocError};
        use std::marker::PhantomData;

        /// A frame that can't store any roots or be nested.
        ///
        /// A `NullFrame` can be used if you call Rust from Julia through `ccall` and want to
        /// borrow array data but not perform any allocations.
        pub struct NullFrame<'frame>(PhantomData<&'frame ()>);

        impl<'frame> NullFrame<'frame> {
            // Safety: frames must form a single nested hierarchy.
            pub(crate) unsafe fn new(_: &'frame CCall) -> Self {
                NullFrame(PhantomData)
            }
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "async")] {
        use super::mode::Async;
        use super::mode::private::ModePriv as _;
        use std::future::Future;

        /// A frame that can be used to root Julia data and call async methods.
        ///
        /// Frames created with a capacity can store at least that number of roots. A frame's
        /// capacity is at least 16.
        pub struct AsyncGcFrame<'frame> {
            raw_frame: &'frame [Slot],
            page: Option<StackPage>,
            mode: Async<'frame>,
        }

        impl<'frame> AsyncGcFrame<'frame> {
            /// An async version of [`Frame::scope`].
            ///
            /// The closure `func` must return an async block. Note that the returned value is
            /// required to live at least as long the current frame.
            #[inline(never)]
            pub async fn async_scope<'nested, T, F, G>(&'nested mut self, func: F) -> JlrsResult<T>
            where
                T: 'frame,
                G: Future<Output = JlrsResult<T>>,
                F: FnOnce(AsyncGcFrame<'nested>) -> G,
            {
                // Safety: the lifetime of the borrow is extended, but it's valid during the call
                // to func and data returned from func must live longer.
                let (nested, owner) = self.nest_async(0);
                let ret =  func(nested).await;
                std::mem::drop(owner);
                ret
            }

            /// An async version of [`Frame::scope_with_capacity`].
            ///
            /// The closure `func` must return an async block. Note that the returned value is
            /// required to live at least as long the current frame.
            #[inline(never)]
            pub async fn async_scope_with_capacity<'nested, T, F, G>(
                &'nested mut self,
                capacity: usize,
                func: F,
            ) -> JlrsResult<T>
            where
                T: 'frame,
                G: Future<Output = JlrsResult<T>>,
                F: FnOnce(AsyncGcFrame<'nested>) -> G,
            {
                // Safety: the lifetime of the borrow is extended, but it's valid during the call
                // to func and data returned from func must live longer.
                let (nested, owner) = self.nest_async(capacity);
                let ret =  func(nested).await;
                std::mem::drop(owner);
                ret
            }

            // Safety: frames must form a single nested hierarchy. A new frame must only be
            // created when entering a new scope.
            pub(crate) unsafe fn new(
                raw_frame: &'frame [Slot],
                mode: Async<'frame>,
            ) -> (Self, FrameOwner<'frame, Async<'frame>>) {
                // Is popped when this frame is dropped
                mode.push_frame(raw_frame, Private);

                let owner = FrameOwner::new(raw_frame, mode);
                let frame = AsyncGcFrame {
                    raw_frame,
                    page: None,
                    mode,
                };

                (frame, owner)
            }

            pub(crate) fn nest_async<'nested>(
                &'nested mut self,
                capacity: usize,
            ) -> (AsyncGcFrame<'nested>, FrameOwner<'nested, Async<'nested>>) {
                let used = self.n_roots() + 2;
                let new_frame_size = MIN_FRAME_CAPACITY.max(capacity) + 2;
                let raw_frame = if self.page.is_some() {
                    // Safety: page is some
                    unsafe {
                        if new_frame_size <= self.page.as_ref().unwrap_unchecked().size() {
                            self.page.as_ref().unwrap_unchecked().as_ref()
                        } else {
                            self.page = Some(StackPage::new(new_frame_size));
                            self.page.as_ref().unwrap_unchecked().as_ref()
                        }
                    }
                } else if used + new_frame_size <= self.raw_frame.len() {
                    &self.raw_frame[used..]
                } else {
                    self.page = Some(StackPage::new(new_frame_size));
                    // Safety: page is some
                    unsafe { self.page.as_ref().unwrap_unchecked().as_ref() }
                };

                // Safety: nested hierarchy is maintained
                unsafe { AsyncGcFrame::new(raw_frame, self.mode) }
            }

            // Safety: capacity >= n_slots, the n_roots pointers the garbage collector
            // can see must point to valid Julia data or be null pointers.
            pub(crate) unsafe fn set_n_roots(&self, n_slots: usize) {
                debug_assert!(n_slots <= self.capacity());
                self.raw_frame.get_unchecked(0).set((n_slots << 2) as _);
            }

            // Safety: capacity > n_roots, value must point to valid Julia data
            pub(crate) unsafe fn root(&self, value: NonNull<jl_value_t>) {
                debug_assert!(self.n_roots() < self.capacity());

                let n_roots = self.n_roots();
                self.raw_frame
                    .get_unchecked(n_roots + 2)
                    .set(value.cast().as_ptr());
                self.set_n_roots(n_roots + 1);
            }
        }

        impl<'frame> Frame<'frame> for AsyncGcFrame<'frame> {
            fn reusable_slot(&mut self) -> JlrsResult<ReusableSlot<'frame>> {
                // Safety: the slot can only be used while the frame exists.
                unsafe {
                    let slot = <Self as private::FramePriv>::reserve_slot(self, Private)?;
                    Ok(ReusableSlot::new(slot))
                }
            }

            fn n_roots(&self) -> usize {
                self.raw_frame[0].get() as usize >> 2
            }

            fn capacity(&self) -> usize {
                self.raw_frame.len() - 2
            }

            fn output(&mut self) -> JlrsResult<Output<'frame>> {
                // Safety: the slot can only be used while the frame exists.
                unsafe {
                    let slot = <Self as private::FramePriv>::reserve_slot(self, Private)?;
                    Ok(Output::new(slot))
                }
            }
        }
    }
}

/// Functionality shared by the different frame types.
pub trait Frame<'frame>: private::FramePriv<'frame> {
    /// Convert the frame to a scope.
    ///
    /// This method takes a mutable reference to a frame and returns it, it can be used as an
    /// alternative to borrowing a frame with when a [`Scope`] or [`PartialScope`] is needed.
    ///
    /// [`Scope`]: crate::memory::scope::Scope
    /// [`PartialScope`]: crate::memory::scope::PartialScope
    fn as_scope(&mut self) -> &mut Self {
        self
    }

    /// Reserve a new output in the current frame.
    ///
    /// Returns an error if the frame is full.
    fn output(&mut self) -> JlrsResult<Output<'frame>>;

    /// Reserve a new reusable slot in the current frame.
    ///
    /// Returns an error if the frame is full.
    fn reusable_slot(&mut self) -> JlrsResult<ReusableSlot<'frame>>;

    /// Returns the number of values currently rooted in this frame.
    fn n_roots(&self) -> usize;

    /// Returns the maximum number of values that can be rooted in this frame.
    fn capacity(&self) -> usize;

    /// Create a new scope and call func with that scope's frame.
    ///
    /// The frame can store at least 16 roots.
    #[inline(never)]
    fn scope<T, F>(&mut self, func: F) -> JlrsResult<T>
    where
        for<'inner> F: FnOnce(GcFrame<'inner, Self::Mode>) -> JlrsResult<T>,
    {
        let (nested, owner) = self.nest(0, Private);
        let ret = func(nested);
        std::mem::drop(owner);
        ret
    }

    /// Create a new scope and call func with that scope's frame.
    ///
    /// The frame can store at least `capacity` roots.
    #[inline(never)]
    fn scope_with_capacity<T, F>(&mut self, capacity: usize, func: F) -> JlrsResult<T>
    where
        for<'inner> F: FnOnce(GcFrame<'inner, Self::Mode>) -> JlrsResult<T>,
    {
        let (nested, owner) = self.nest(capacity, Private);
        let ret = func(nested);
        std::mem::drop(owner);
        ret
    }
}

impl<'frame, M: Mode> Frame<'frame> for GcFrame<'frame, M> {
    fn reusable_slot(&mut self) -> JlrsResult<ReusableSlot<'frame>> {
        // Safety: the slot can only be used while the frame exists.
        unsafe {
            let slot = <Self as private::FramePriv>::reserve_slot(self, Private)?;
            Ok(ReusableSlot::new(slot))
        }
    }

    fn n_roots(&self) -> usize {
        self.raw_frame[0].get() as usize >> 2
    }

    fn capacity(&self) -> usize {
        self.raw_frame.len() - 2
    }

    fn output(&mut self) -> JlrsResult<Output<'frame>> {
        // Safety: the slot can only be used while the frame exists.
        unsafe {
            let slot = <Self as private::FramePriv>::reserve_slot(self, Private)?;
            Ok(Output::new(slot))
        }
    }
}

#[cfg(feature = "ccall")]
impl<'frame> Frame<'frame> for NullFrame<'frame> {
    fn reusable_slot(&mut self) -> JlrsResult<ReusableSlot<'frame>> {
        Err(AllocError::NullFrame)?
    }

    fn n_roots(&self) -> usize {
        0
    }

    fn capacity(&self) -> usize {
        0
    }

    fn scope<T, F>(&mut self, _func: F) -> JlrsResult<T>
    where
        for<'inner> F: FnOnce(GcFrame<'inner, Self::Mode>) -> JlrsResult<T>,
    {
        Err(AllocError::NullFrame)?
    }

    fn scope_with_capacity<T, F>(&mut self, _capacity: usize, _func: F) -> JlrsResult<T>
    where
        for<'inner> F: FnOnce(GcFrame<'inner, Self::Mode>) -> JlrsResult<T>,
    {
        Err(AllocError::NullFrame)?
    }

    fn output(&mut self) -> JlrsResult<Output<'frame>> {
        Err(AllocError::NullFrame)?
    }
}

mod private {
    use std::ptr::{null_mut, NonNull};

    use super::{Frame as _, MIN_FRAME_CAPACITY};
    use crate::error::JlrsResult;
    use crate::memory::frame::GcFrame;
    use crate::memory::mode::Mode;
    use crate::memory::stack_page::Slot;
    use crate::memory::stack_page::StackPage;
    use crate::wrappers::ptr::private::WrapperPriv;
    use crate::{error::AllocError, private::Private};

    pub struct FrameOwner<'frame, M: Mode> {
        mode: M,
        raw_frame: &'frame [Slot],
    }

    impl<'frame, M: Mode> FrameOwner<'frame, M> {
        // Only one owner must be created for a frame.
        pub(crate) unsafe fn new(raw_frame: &'frame [Slot], mode: M) -> Self {
            FrameOwner { mode, raw_frame }
        }
    }

    #[cfg(feature = "async")]
    impl<'frame> FrameOwner<'frame, Async<'frame>> {
        pub(crate) unsafe fn reconstruct(&self) -> AsyncGcFrame<'frame> {
            AsyncGcFrame {
                raw_frame: self.raw_frame,
                page: None,
                mode: self.mode,
            }
        }
    }

    impl<M: Mode> Drop for FrameOwner<'_, M> {
        fn drop(&mut self) {
            unsafe { self.mode.pop_frame(self.raw_frame, Private) }
        }
    }

    pub trait FramePriv<'frame> {
        type Mode: Mode;
        // protect the value from being garbage collected while this frame is active.
        // safety: the value must be a valid pointer to a Julia value.
        unsafe fn push_root<'data, T: WrapperPriv<'frame, 'data>>(
            &mut self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Result<T, AllocError>;

        // safety: this slot must only be used while the frame exists.
        unsafe fn reserve_slot(&mut self, _: Private) -> JlrsResult<&'frame Slot>;

        fn nest<'nested>(
            &'nested mut self,
            capacity: usize,
            _: Private,
        ) -> (
            GcFrame<'nested, Self::Mode>,
            FrameOwner<'nested, Self::Mode>,
        );
    }

    impl<'frame, M: Mode> FramePriv<'frame> for GcFrame<'frame, M> {
        type Mode = M;

        unsafe fn push_root<'data, T: WrapperPriv<'frame, 'data>>(
            &mut self,
            value: NonNull<T::Wraps>,
            _: Private,
        ) -> Result<T, AllocError> {
            let n_roots = self.n_roots();
            if n_roots == self.capacity() {
                Err(AllocError::Full { cap: n_roots })?
            }

            self.root(value.cast());
            Ok(T::wrap_non_null(value, Private))
        }

        unsafe fn reserve_slot(&mut self, _: Private) -> JlrsResult<&'frame Slot> {
            let n_roots = self.n_roots();
            if n_roots == self.capacity() {
                Err(AllocError::Full { cap: n_roots })?
            }

            self.raw_frame.get_unchecked(n_roots + 2).set(null_mut());
            self.set_n_roots(n_roots + 1);

            Ok(self.raw_frame.get_unchecked(n_roots + 2))
        }

        fn nest<'nested>(
            &'nested mut self,
            capacity: usize,
            _: Private,
        ) -> (
            GcFrame<'nested, Self::Mode>,
            FrameOwner<'nested, Self::Mode>,
        ) {
            let used = self.n_roots() + 2;
            let new_frame_size = MIN_FRAME_CAPACITY.max(capacity) + 2;
            let raw_frame = if self.page.is_some() {
                // Safety: page is some
                unsafe {
                    if new_frame_size <= self.page.as_ref().unwrap_unchecked().size() {
                        self.page.as_ref().unwrap_unchecked().as_ref()
                    } else {
                        self.page = Some(StackPage::new(new_frame_size));
                        self.page.as_ref().unwrap_unchecked().as_ref()
                    }
                }
            } else if used + new_frame_size <= self.raw_frame.len() {
                &self.raw_frame[used..]
            } else {
                self.page = Some(StackPage::new(new_frame_size));
                // Safety: page is some
                unsafe { self.page.as_ref().unwrap_unchecked().as_ref() }
            };

            // Safety: nested hierarchy is maintained
            unsafe { GcFrame::new(raw_frame, self.mode) }
        }
    }

    cfg_if::cfg_if! {
        if #[cfg(feature = "ccall")] {
            use crate::memory::frame::NullFrame;
            use crate::memory::mode::Sync;

            impl<'frame> FramePriv<'frame> for NullFrame<'frame> {
                type Mode = Sync;

                unsafe fn push_root<'data, T: WrapperPriv<'frame, 'data>>(
                    &mut self,
                    _value: NonNull<T::Wraps>,
                    _: Private,
                ) -> Result<T, AllocError> {
                    Err(AllocError::NullFrame)?
                }

                unsafe fn reserve_slot(&mut self, _: Private) -> JlrsResult<&'frame Slot> {
                    Err(AllocError::NullFrame)?
                }

                fn nest<'nested>(
                    &'nested mut self,
                    _capacity: usize,
                    _: Private,
                ) -> (GcFrame<'nested, Self::Mode>, FrameOwner<'nested, Self::Mode>) {
                    unreachable!()
                }
            }
        }
    }

    cfg_if::cfg_if! {
        if #[cfg(feature = "async")] {
            use super::AsyncGcFrame;
            use super::super::mode::Async;

            impl<'frame> FramePriv<'frame> for AsyncGcFrame<'frame> {
                type Mode = Async<'frame>;

                unsafe fn push_root<'data, T: WrapperPriv<'frame, 'data>>(
                    &mut self,
                    value: NonNull<T::Wraps>,
                    _: Private,
                ) -> Result<T, AllocError> {
                    let n_roots = self.n_roots();
                    if n_roots == self.capacity() {
                        Err(AllocError::Full { cap: n_roots })?
                    }

                    self.root(value.cast());
                    Ok(T::wrap_non_null(value, Private))
                }

                unsafe fn reserve_slot(&mut self, _: Private) -> JlrsResult<&'frame Slot> {
                    let n_roots = self.n_roots();
                    if n_roots == self.capacity() {
                        Err(AllocError::Full { cap: n_roots })?
                    }

                    self.raw_frame
                        .get_unchecked(n_roots + 2)
                        .set(null_mut());

                    self.set_n_roots(n_roots + 1);

                    Ok(self.raw_frame.get_unchecked(n_roots + 2))
                }

                fn nest<'nested>(
                    &'nested mut self,
                    capacity: usize,
                    _: Private,
                ) -> (GcFrame<'nested, Self::Mode>, FrameOwner<'nested, Self::Mode>) {
                    let used = self.n_roots() + 2;
                    let new_frame_size = MIN_FRAME_CAPACITY.max(capacity) + 2;
                    let raw_frame = if self.page.is_some() {
                        // Safety: page is some
                        unsafe {
                            if new_frame_size <= self.page.as_ref().unwrap_unchecked().size() {
                                self.page.as_ref().unwrap_unchecked().as_ref()
                            } else {
                                self.page = Some(StackPage::new(new_frame_size));
                                self.page.as_ref().unwrap_unchecked().as_ref()
                            }
                        }
                    } else if used + new_frame_size <= self.raw_frame.len() {
                        &self.raw_frame[used..]
                    } else {
                        self.page = Some(StackPage::new(new_frame_size));
                        // Safety: page is some
                        unsafe { self.page.as_ref().unwrap_unchecked().as_ref() }
                    };

                    // Safety: nested hierarchy is maintained
                    unsafe { GcFrame::new(raw_frame, self.mode) }
                }
            }
        }
    }
}

#[cfg(test)]
#[cfg(feature = "sync-rt")]
mod tests {
    use super::private::FramePriv;
    use crate::{
        memory::{
            frame::{Frame as _, GcFrame},
            mode,
            stack_page::StackPage,
        },
        private::Private,
        util,
        wrappers::ptr::value::Value,
    };

    #[test]
    fn min_stack_pack_size() {
        let page = StackPage::new(0);
        assert_eq!(page.size(), 64);
    }

    #[test]
    fn create_base_frame() {
        util::JULIA.with(|julia| unsafe {
            let julia = julia.borrow_mut();
            let page = julia.get_page();
            let page_size = page.size();

            let frame = GcFrame::new(page.as_ref(), mode::Sync);
            assert_eq!(frame.0.capacity(), page_size - 2);
            assert_eq!(frame.0.n_roots(), 0);
        })
    }

    #[test]
    fn push_root() {
        util::JULIA.with(|julia| unsafe {
            let julia = julia.borrow_mut();
            let page = julia.get_page();
            let page_size = page.size();
            let mut frame = GcFrame::new(page.as_ref(), mode::Sync);
            let _value = Value::new(&mut frame.0, 1usize).unwrap();

            assert_eq!(frame.0.capacity(), page_size - 2);
            assert_eq!(frame.0.n_roots(), 1);
        })
    }

    #[test]
    fn push_too_many_roots() {
        util::JULIA.with(|julia| unsafe {
            let julia = julia.borrow_mut();
            let page = julia.get_page();
            let page_size = page.size();
            let mut frame = GcFrame::new(page.as_ref(), mode::Sync);

            for _ in 0..page_size - 2 {
                let _value = Value::new(&mut frame.0, 1usize).unwrap();
            }

            assert_eq!(frame.0.capacity(), page_size - 2);
            assert_eq!(frame.0.n_roots(), page_size - 2);

            assert!(Value::new(&mut frame.0, 1usize).is_err());
        })
    }

    #[test]
    fn push_new_frame() {
        util::JULIA.with(|julia| unsafe {
            let julia = julia.borrow_mut();
            let page = julia.get_page();
            let page_size = page.size();
            let mut frame = GcFrame::new(page.as_ref(), mode::Sync);

            {
                let nested = frame.0.nest(0, Private);
                let capacity = nested.0.capacity();
                assert_eq!(capacity, page_size - 4);
            }
        })
    }

    #[test]
    fn push_large_new_frame() {
        util::JULIA.with(|julia| unsafe {
            let julia = julia.borrow_mut();
            let page = julia.get_page();
            let page_size = page.size();
            let mut frame = GcFrame::new(page.as_ref(), mode::Sync);

            {
                let nested = frame.0.nest(2 * page_size, Private);
                let capacity = nested.0.capacity();
                let n_roots = nested.0.n_roots();
                assert_eq!(capacity, 2 * page_size);
                assert_eq!(n_roots, 0);
            }
        })
    }

    #[test]
    fn reuse_large_page() {
        util::JULIA.with(|julia| unsafe {
            let julia = julia.borrow_mut();
            let page = julia.get_page();
            let page_size = page.size();
            let mut frame = GcFrame::new(page.as_ref(), mode::Sync);

            {
                frame.0.nest(2 * page_size, Private);
            }

            {
                let nested = frame.0.nest(0, Private);
                let capacity = nested.0.capacity();
                let n_roots = nested.0.n_roots();
                assert_eq!(capacity, 2 * page_size);
                assert_eq!(n_roots, 0);
            }
        })
    }
}
