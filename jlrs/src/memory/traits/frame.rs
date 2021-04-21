//! Functionality shared by the different frame types.
//!
//! More information about frames and their capabilities can be found in the modules
//! [`jlrs::memory`] and [`jlrs::memory::frame`].
//!
//! [`jlrs::memory`]: crate::memory
//! [`jlrs::memory::frame`]: crate::memory::frame

use super::mode::Mode;
#[cfg(all(feature = "async", target_os = "linux"))]
use crate::memory::frame::AsyncGcFrame;
use crate::memory::frame::{GcFrame, NullFrame};

/// This trait provides the functionality shared by the different frame types.
pub trait Frame<'frame>: private::Frame<'frame> {
    /// This method takes a mutable reference to a frame and returns it; this method can be used
    /// as an alternative to reborrowing a frame with `&mut *frame` when a [`Scope`] is needed.
    ///
    /// [`Scope`]: crate::memory::traits::scope::Scope
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

#[cfg(all(feature = "async", target_os = "linux"))]
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
    #[cfg(all(feature = "async", target_os = "linux"))]
    use crate::memory::frame::AsyncGcFrame;
    use crate::memory::frame::GcFrame;
    use crate::memory::frame::NullFrame;
    #[cfg(all(feature = "async", target_os = "linux"))]
    use crate::memory::mode::Async;
    use crate::memory::mode::Sync;
    use crate::memory::traits::{mode::Mode, root::Root};
    use crate::value::{UnrootedResult, UnrootedValue, Value};
    use crate::{error::AllocError, private::Private};
    use crate::{
        error::{JlrsError, JlrsResult, JuliaResult},
        memory::output::Output,
    };
    use jl_sys::jl_value_t;

    pub trait Frame<'frame> {
        type Mode: Mode;
        // protect the value from being garbage collected while this frame is active.
        // safety: the value must be a valid pointer to a Julia value or a null pointer.
        unsafe fn push_root<'data>(
            &mut self,
            value: *mut jl_value_t,
            _: Private,
        ) -> Result<Value<'frame, 'data>, AllocError>;

        // safety: frame must be dropped
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
                -> JlrsResult<UnrootedValue<'frame, 'data, 'inner>>;

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
                -> JlrsResult<UnrootedValue<'frame, 'data, 'inner>>;

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
                -> JlrsResult<UnrootedResult<'frame, 'data, 'inner>>;

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
                -> JlrsResult<UnrootedResult<'frame, 'data, 'inner>>;

        fn scope<'outer, T, F>(&'outer mut self, func: F, _: Private) -> JlrsResult<T>
        where
            T: 'outer,
            for<'inner> F: FnOnce(&mut GcFrame<'inner, Self::Mode>) -> JlrsResult<T>;

        fn scope_with_slots<'outer, T, F>(
            &'outer mut self,
            capacity: usize,
            func: F,
            _: Private,
        ) -> JlrsResult<T>
        where
            T: 'outer,
            for<'inner> F: FnOnce(&mut GcFrame<'inner, Self::Mode>) -> JlrsResult<T>;
    }

    impl<'frame, M: Mode> Frame<'frame> for GcFrame<'frame, M> {
        type Mode = M;

        unsafe fn push_root<'data>(
            &mut self,
            value: *mut jl_value_t,
            _: Private,
        ) -> Result<Value<'frame, 'data>, AllocError> {
            let n_roots = self.n_roots();
            if n_roots == self.capacity() {
                return Err(AllocError::FrameOverflow(1, n_roots));
            }

            self.root(value);
            Ok(Value::wrap(value))
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
                -> JlrsResult<UnrootedValue<'frame, 'data, 'inner>>,
        {
            unsafe {
                let v = {
                    let mut nested = self.nest(0);
                    let out = Output::new();
                    func(out, &mut nested)?.into_pending()
                };

                Value::root(self, v)
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
            )
                -> JlrsResult<UnrootedValue<'frame, 'data, 'inner>>,
        {
            unsafe {
                let v = {
                    let mut nested = self.nest(capacity);
                    let out = Output::new();
                    func(out, &mut nested)?.into_pending()
                };

                Value::root(self, v)
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
            )
                -> JlrsResult<UnrootedResult<'frame, 'data, 'inner>>,
        {
            unsafe {
                let v = {
                    let mut nested = self.nest(0);
                    let out = Output::new();
                    func(out, &mut nested)?.into_pending()
                };

                JuliaResult::root(self, v)
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
            )
                -> JlrsResult<UnrootedResult<'frame, 'data, 'inner>>,
        {
            unsafe {
                let v = {
                    let mut nested = self.nest(capacity);
                    let out = Output::new();
                    func(out, &mut nested)?.into_pending()
                };

                JuliaResult::root(self, v)
            }
        }

        fn scope<'outer, T, F>(&'outer mut self, func: F, _: Private) -> JlrsResult<T>
        where
            T: 'outer,
            for<'inner> F: FnOnce(&mut GcFrame<'inner, Self::Mode>) -> JlrsResult<T>,
        {
            // safe: frame is dropped
            let mut nested = unsafe { self.nest(0) };
            func(&mut nested)
        }

        fn scope_with_slots<'outer, T, F>(
            &'outer mut self,
            capacity: usize,
            func: F,
            _: Private,
        ) -> JlrsResult<T>
        where
            T: 'outer,
            for<'inner> F: FnOnce(&mut GcFrame<'inner, Self::Mode>) -> JlrsResult<T>,
        {
            // safe: frame is dropped
            let mut nested = unsafe { self.nest(capacity) };
            func(&mut nested)
        }
    }

    #[cfg(all(feature = "async", target_os = "linux"))]
    impl<'frame> Frame<'frame> for AsyncGcFrame<'frame> {
        type Mode = Async<'frame>;

        unsafe fn push_root<'data>(
            &mut self,
            value: *mut jl_value_t,
            _: Private,
        ) -> Result<Value<'frame, 'data>, AllocError> {
            let n_roots = self.n_roots();
            if n_roots == self.capacity() {
                return Err(AllocError::FrameOverflow(1, n_roots));
            }

            self.root(value);
            Ok(Value::wrap(value))
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
                -> JlrsResult<UnrootedValue<'frame, 'data, 'inner>>,
        {
            unsafe {
                let v = {
                    let mut nested = self.nest(0);
                    let out = Output::new();
                    func(out, &mut nested)?.into_pending()
                };

                Value::root(self, v)
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
            )
                -> JlrsResult<UnrootedValue<'frame, 'data, 'inner>>,
        {
            unsafe {
                let v = {
                    let mut nested = self.nest(capacity);
                    let out = Output::new();
                    func(out, &mut nested)?.into_pending()
                };

                Value::root(self, v)
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
            )
                -> JlrsResult<UnrootedResult<'frame, 'data, 'inner>>,
        {
            unsafe {
                let v = {
                    let mut nested = self.nest(0);
                    let out = Output::new();
                    func(out, &mut nested)?.into_pending()
                };

                JuliaResult::root(self, v)
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
            )
                -> JlrsResult<UnrootedResult<'frame, 'data, 'inner>>,
        {
            unsafe {
                let v = {
                    let mut nested = self.nest(capacity);
                    let out = Output::new();
                    func(out, &mut nested)?.into_pending()
                };

                JuliaResult::root(self, v)
            }
        }

        fn scope<'outer, T, F>(&'outer mut self, func: F, _: Private) -> JlrsResult<T>
        where
            T: 'outer,
            for<'inner> F: FnOnce(&mut GcFrame<'inner, Self::Mode>) -> JlrsResult<T>,
        {
            // safe: frame is dropped
            let mut nested = unsafe { self.nest(0) };
            func(&mut nested)
        }

        fn scope_with_slots<'outer, T, F>(
            &'outer mut self,
            capacity: usize,
            func: F,
            _: Private,
        ) -> JlrsResult<T>
        where
            T: 'outer,
            for<'inner> F: FnOnce(&mut GcFrame<'inner, Self::Mode>) -> JlrsResult<T>,
        {
            // safe: frame is dropped
            let mut nested = unsafe { self.nest(capacity) };
            func(&mut nested)
        }
    }

    impl<'frame> Frame<'frame> for NullFrame<'frame> {
        type Mode = Sync;

        unsafe fn push_root<'data>(
            &mut self,
            _value: *mut jl_value_t,
            _: Private,
        ) -> Result<Value<'frame, 'data>, AllocError> {
            Err(AllocError::FrameOverflow(1, 0))
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
                -> JlrsResult<UnrootedValue<'frame, 'data, 'inner>>,
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
                -> JlrsResult<UnrootedValue<'frame, 'data, 'inner>>,
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
                -> JlrsResult<UnrootedResult<'frame, 'data, 'inner>>,
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
                -> JlrsResult<UnrootedResult<'frame, 'data, 'inner>>,
        {
            Err(JlrsError::NullFrame)?
        }

        fn scope<'outer, T, F>(&'outer mut self, _func: F, _: Private) -> JlrsResult<T>
        where
            T: 'outer,
            for<'inner> F: FnOnce(&mut GcFrame<'inner, Self::Mode>) -> JlrsResult<T>,
        {
            Err(JlrsError::NullFrame)?
        }

        fn scope_with_slots<'outer, T, F>(
            &'outer mut self,
            _capacity: usize,
            _func: F,
            _: Private,
        ) -> JlrsResult<T>
        where
            T: 'outer,
            for<'inner> F: FnOnce(&mut GcFrame<'inner, Self::Mode>) -> JlrsResult<T>,
        {
            Err(JlrsError::NullFrame)?
        }
    }
}
