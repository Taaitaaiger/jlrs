//! Functionality shared by the different frame types.

#[cfg(all(feature = "async", target_os = "linux"))]
use crate::memory::frame::AsyncGcFrame;
use crate::memory::frame::{GcFrame, NullFrame};
use crate::memory::global::Global;
use crate::value::{UnrootedCallResult, UnrootedValue, Value};
use crate::{
    error::{CallResult, JlrsError, JlrsResult},
    memory::output::Output,
};

use super::{mode::Mode, root::Root};

/// This trait provides the functionality shared by the different frame types. 
pub trait Frame<'frame>: private::Frame<'frame> {
    /// Returns a new `Global`, globals accessed with this token are valid until this frame is 
    /// dropped.
    fn global(&self) -> Global<'frame> {
        unsafe { Global::new() }
    }

    /// This method takes a mutable reference to a frame and returns it; this method can be used
    /// as an alternative to reborrowing a frame with `&mut *frame` when a [`Scope`] is needed.
    fn as_scope(&mut self) -> &mut Self {
        self
    }

    /// Reserve `additional` slots in the current frame. Returns `true` on success, or `false` if
    /// `self.n_slots() + additional > self.capacity()`.
    #[must_use]
    fn alloc_slots(&mut self, additional: usize) -> bool;

    /// Create a [`GcFrame`] and call the given closure with it. Returns the result of this
    /// closure.
    ///
    /// Example:
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # use jlrs::util::JULIA;
    /// # fn main() {
    /// # JULIA.with(|j| {
    /// # let mut julia = j.borrow_mut();
    ///   julia.frame(|global, frame| {
    ///       let sum = frame.frame(|frame| {
    ///           let v1 = Value::new(&mut *frame, 1usize)?;
    ///           let v2 = Value::new(&mut *frame, 2usize)?;
    ///
    ///           Module::base()
    ///               .function("+")?
    ///               .call2(&mut *frame, v1, v2)?
    ///               .unwrap()
    ///               .cast::<usize>()
    ///       })?;
    ///
    ///       assert_eq(sum, 3);
    ///       Ok(())
    ///   }).unwrap();
    /// # });
    /// # }
    /// ```
    fn frame<T, F>(&mut self, func: F) -> JlrsResult<T>
    where
        T: 'frame,
        for<'nested> F: FnOnce(&mut GcFrame<'nested, Self::Mode>) -> JlrsResult<T>;

    /// Create a [`GcFrame`] with `capacity` slots and call the given closure with it. Returns the
    /// result of this closure.
    ///
    /// Example:
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # use jlrs::util::JULIA;
    /// # fn main() {
    /// # JULIA.with(|j| {
    /// # let mut julia = j.borrow_mut();
    ///   julia.frame(|global, frame| {
    ///       let sum = frame.frame_with_slots(3, |frame| {
    ///           let v1 = Value::new(&mut *frame, 1usize)?;
    ///           let v2 = Value::new(&mut *frame, 2usize)?;
    ///
    ///           Module::base()
    ///               .function("+")?
    ///               .call2(&mut *frame, v1, v2)?
    ///               .unwrap()
    ///               .cast::<usize>()
    ///       })?;
    ///
    ///       assert_eq(sum, 3);
    ///       Ok(())
    ///   }).unwrap();
    /// # });
    /// # }
    /// ```
    fn frame_with_slots<T, F>(&mut self, capacity: usize, func: F) -> JlrsResult<T>
    where
        T: 'frame,
        for<'nested> F: FnOnce(&mut GcFrame<'nested, Self::Mode>) -> JlrsResult<T>;

    /// Create a new [`Output`] and [`GcFrame`] and call the given closure. The final result of this
    /// closure, an [`UnrootedValue`], is rooted in the current frame.
    ///
    /// This can be used to allocate one or more temporary values needed to create some value:
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # use jlrs::util::JULIA;
    /// # fn main() {
    /// # JULIA.with(|j| {
    /// # let mut julia = j.borrow_mut();
    ///   julia.frame(|global, frame| {
    ///       let nt = frame.value_frame(|output, frame| {
    ///           let v1 = Value::new(&mut *frame, 1usize)?;
    ///           let v2 = Value::new(&mut *frame, 2usize)?;
    ///
    ///           let output = output.into_scope(frame);
    ///           named_tuple!(output, "a" => v1, "b" => v2)
    ///       })?;
    ///
    ///       assert!(nt.is::<NamedTuple>());
    ///       Ok(())
    ///   }).unwrap();
    /// # });
    /// # }
    /// ```
    fn value_frame<'data, F>(&mut self, func: F) -> JlrsResult<Value<'frame, 'data>>
    where
        for<'nested, 'inner> F: FnOnce(
            Output<'frame>,
            &'inner mut GcFrame<'nested, Self::Mode>,
        ) -> JlrsResult<UnrootedValue<'frame, 'data, 'inner>>;

    /// Create a new [`Output`] and [`GcFrame`] with `capacity` slots and call the given closure. The
    /// final result of this closure, an [`UnrootedValue`], is rooted in the current frame.
    ///
    /// This can be used to allocate one or more temporary values needed to create some value:
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # use jlrs::value::datatype::NamedTuple;
    /// # use jlrs::util::JULIA;
    /// # fn main() {
    /// # JULIA.with(|j| {
    /// # let mut julia = j.borrow_mut();
    ///   julia.frame(|global, frame| {
    ///       let nt = frame.value_frame_with_slots(2, |output, frame| {
    ///           let v1 = Value::new(&mut *frame, 1usize)?;
    ///           let v2 = Value::new(&mut *frame, 2usize)?;
    ///
    ///           let output = output.into_scope(frame);
    ///           named_tuple!(output, "a" => v1, "b" => v2)
    ///       })?;
    ///
    ///       assert!(nt.is::<NamedTuple>());
    ///       Ok(())
    ///   }).unwrap();
    /// # });
    /// # }
    /// ```
    fn value_frame_with_slots<'data, F>(
        &mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<Value<'frame, 'data>>
    where
        for<'nested, 'inner> F: FnOnce(
            Output<'frame>,
            &'inner mut GcFrame<'nested, Self::Mode>,
        ) -> JlrsResult<UnrootedValue<'frame, 'data, 'inner>>;

    /// Create a new [`Output`] and [`GcFrame`] and call the given closure. The final result of this
    /// closure, an [`UnrootedCallResult`], is rooted in the current frame.
    ///
    /// This can be used to allocate one or more temporary values and call a function with them:
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # use jlrs::util::JULIA;
    /// # fn main() {
    /// # JULIA.with(|j| {
    /// # let mut julia = j.borrow_mut();
    ///   julia.frame(|global, frame| {
    ///       let _sum = frame.call_frame(|output, frame| {
    ///           let v1 = Value::new(&mut *frame, 1usize)?;
    ///           let v2 = Value::new(&mut *frame, 2usize)?;
    ///
    ///           let output = output.into_scope(frame);
    ///           Module::base(global)
    ///               .function("+")?
    ///               .call2(output, v1, v2)
    ///       })?.unwrap()
    ///           .cast::<usize>()?;
    ///       
    ///       assert_eq!(sum, 3);
    ///       Ok(())
    ///   }).unwrap();
    /// # });
    /// # }
    /// ```
    fn call_frame<'data, F>(&mut self, func: F) -> JlrsResult<CallResult<'frame, 'data>>
    where
        for<'nested, 'inner> F: FnOnce(
            Output<'frame>,
            &'inner mut GcFrame<'nested, Self::Mode>,
        )
            -> JlrsResult<UnrootedCallResult<'frame, 'data, 'inner>>;

    /// Create a new [`Output`] and [`GcFrame`] with `capacity` slots and call the given closure.
    /// The final result of this closure, an [`UnrootedCallResult`], is rooted in the current
    /// frame.
    ///
    /// This can be used to allocate one or more temporary values and call a function with them:
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # use jlrs::util::JULIA;
    /// # fn main() {
    /// # JULIA.with(|j| {
    /// # let mut julia = j.borrow_mut();
    ///   julia.frame(|global, frame| {
    ///       let sum = frame.call_frame_with_slots(2, |output, frame| {
    ///           let v1 = Value::new(&mut *frame, 1usize)?;
    ///           let v2 = Value::new(&mut *frame, 2usize)?;
    ///
    ///           let output = output.into_scope(frame);
    ///           Module::base(global)
    ///               .function("+")?
    ///               .call2(output, v1, v2)
    ///       })?.unwrap()
    ///           .cast::<usize>()?;
    ///       
    ///       assert_eq!(sum, 3);
    ///       Ok(())
    ///   }).unwrap();
    /// # });
    /// # }
    /// ```
    fn call_frame_with_slots<'data, F>(
        &mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<CallResult<'frame, 'data>>
    where
        for<'nested, 'inner> F: FnOnce(
            Output<'frame>,
            &'inner mut GcFrame<'nested, Self::Mode>,
        )
            -> JlrsResult<UnrootedCallResult<'frame, 'data, 'inner>>;

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

    fn frame<T, F>(&mut self, func: F) -> JlrsResult<T>
    where
        T: 'frame,
        for<'nested> F: FnOnce(&mut GcFrame<'nested, Self::Mode>) -> JlrsResult<T>,
    {
        let mut nested = self.nest(0);
        func(&mut nested)
    }

    fn frame_with_slots<T, F>(&mut self, capacity: usize, func: F) -> JlrsResult<T>
    where
        T: 'frame,
        for<'nested> F: FnOnce(&mut GcFrame<'nested, Self::Mode>) -> JlrsResult<T>,
    {
        let mut nested = self.nest(capacity);
        func(&mut nested)
    }

    fn value_frame<'data, F>(&mut self, func: F) -> JlrsResult<Value<'frame, 'data>>
    where
        for<'nested, 'inner> F: FnOnce(
            Output<'frame>,
            &'inner mut GcFrame<'nested, Self::Mode>,
        ) -> JlrsResult<UnrootedValue<'frame, 'data, 'inner>>,
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

    fn value_frame_with_slots<'data, F>(
        &mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<Value<'frame, 'data>>
    where
        for<'nested, 'inner> F: FnOnce(
            Output<'frame>,
            &'inner mut GcFrame<'nested, Self::Mode>,
        ) -> JlrsResult<UnrootedValue<'frame, 'data, 'inner>>,
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

    fn call_frame<'data, F>(&mut self, func: F) -> JlrsResult<CallResult<'frame, 'data>>
    where
        for<'nested, 'inner> F: FnOnce(
            Output<'frame>,
            &'inner mut GcFrame<'nested, Self::Mode>,
        )
            -> JlrsResult<UnrootedCallResult<'frame, 'data, 'inner>>,
    {
        unsafe {
            let v = {
                let mut nested = self.nest(0);
                let out = Output::new();
                func(out, &mut nested)?.into_pending()
            };

            CallResult::root(self, v)
        }
    }

    fn call_frame_with_slots<'data, F>(
        &mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<CallResult<'frame, 'data>>
    where
        for<'nested, 'inner> F: FnOnce(
            Output<'frame>,
            &'inner mut GcFrame<'nested, Self::Mode>,
        )
            -> JlrsResult<UnrootedCallResult<'frame, 'data, 'inner>>,
    {
        unsafe {
            let v = {
                let mut nested = self.nest(capacity);
                let out = Output::new();
                func(out, &mut nested)?.into_pending()
            };

            CallResult::root(self, v)
        }
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

    fn frame<T, F>(&mut self, func: F) -> JlrsResult<T>
    where
        T: 'frame,
        for<'nested> F: FnOnce(&mut GcFrame<'nested, Self::Mode>) -> JlrsResult<T>,
    {
        let mut nested = self.nest(0);
        func(&mut nested)
    }

    fn frame_with_slots<T, F>(&mut self, capacity: usize, func: F) -> JlrsResult<T>
    where
        T: 'frame,
        for<'nested> F: FnOnce(&mut GcFrame<'nested, Self::Mode>) -> JlrsResult<T>,
    {
        let mut nested = self.nest(capacity);
        func(&mut nested)
    }

    fn value_frame<'data, F>(&mut self, func: F) -> JlrsResult<Value<'frame, 'data>>
    where
        for<'nested, 'inner> F: FnOnce(
            Output<'frame>,
            &'inner mut GcFrame<'nested, Self::Mode>,
        ) -> JlrsResult<UnrootedValue<'frame, 'data, 'inner>>,
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

    fn value_frame_with_slots<'data, F>(
        &mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<Value<'frame, 'data>>
    where
        for<'nested, 'inner> F: FnOnce(
            Output<'frame>,
            &'inner mut GcFrame<'nested, Self::Mode>,
        ) -> JlrsResult<UnrootedValue<'frame, 'data, 'inner>>,
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

    fn call_frame<'data, F>(&mut self, func: F) -> JlrsResult<CallResult<'frame, 'data>>
    where
        for<'nested, 'inner> F: FnOnce(
            Output<'frame>,
            &'inner mut GcFrame<'nested, Self::Mode>,
        )
            -> JlrsResult<UnrootedCallResult<'frame, 'data, 'inner>>,
    {
        unsafe {
            let v = {
                let mut nested = self.nest(0);
                let out = Output::new();
                func(out, &mut nested)?.into_pending()
            };

            CallResult::root(self, v)
        }
    }

    fn call_frame_with_slots<'data, F>(
        &mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<CallResult<'frame, 'data>>
    where
        for<'nested, 'inner> F: FnOnce(
            Output<'frame>,
            &'inner mut GcFrame<'nested, Self::Mode>,
        )
            -> JlrsResult<UnrootedCallResult<'frame, 'data, 'inner>>,
    {
        unsafe {
            let v = {
                let mut nested = self.nest(capacity);
                let out = Output::new();
                func(out, &mut nested)?.into_pending()
            };

            CallResult::root(self, v)
        }
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

    fn frame<T, F>(&mut self, _func: F) -> JlrsResult<T>
    where
        T: 'frame,
        for<'nested> F: FnOnce(&mut GcFrame<'nested, Self::Mode>) -> JlrsResult<T>,
    {
        Err(JlrsError::NullFrame)?
    }

    fn frame_with_slots<T, F>(&mut self, _capacity: usize, _func: F) -> JlrsResult<T>
    where
        T: 'frame,
        for<'nested> F: FnOnce(&mut GcFrame<'nested, Self::Mode>) -> JlrsResult<T>,
    {
        Err(JlrsError::NullFrame)?
    }

    fn value_frame<'data, F>(&mut self, _func: F) -> JlrsResult<Value<'frame, 'data>>
    where
        for<'nested, 'inner> F: FnOnce(
            Output<'frame>,
            &'inner mut GcFrame<'nested, Self::Mode>,
        ) -> JlrsResult<UnrootedValue<'frame, 'data, 'inner>>,
    {
        Err(JlrsError::NullFrame)?
    }

    fn value_frame_with_slots<'data, F>(
        &mut self,
        _capacity: usize,
        _func: F,
    ) -> JlrsResult<Value<'frame, 'data>>
    where
        for<'nested, 'inner> F: FnOnce(
            Output<'frame>,
            &'inner mut GcFrame<'nested, Self::Mode>,
        ) -> JlrsResult<UnrootedValue<'frame, 'data, 'inner>>,
    {
        Err(JlrsError::NullFrame)?
    }

    fn call_frame<'data, F>(&mut self, _func: F) -> JlrsResult<CallResult<'frame, 'data>>
    where
        for<'nested, 'inner> F: FnOnce(
            Output<'frame>,
            &'inner mut GcFrame<'nested, Self::Mode>,
        )
            -> JlrsResult<UnrootedCallResult<'frame, 'data, 'inner>>,
    {
        Err(JlrsError::NullFrame)?
    }

    fn call_frame_with_slots<'data, F>(
        &mut self,
        _capacity: usize,
        _func: F,
    ) -> JlrsResult<CallResult<'frame, 'data>>
    where
        for<'nested, 'inner> F: FnOnce(
            Output<'frame>,
            &'inner mut GcFrame<'nested, Self::Mode>,
        )
            -> JlrsResult<UnrootedCallResult<'frame, 'data, 'inner>>,
    {
        Err(JlrsError::NullFrame)?
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
    use crate::{error::AllocError, value::traits::private::Internal};
    #[cfg(all(feature = "async", target_os = "linux"))]
    use crate::memory::frame::AsyncGcFrame;
    use crate::memory::frame::GcFrame;
    use crate::memory::frame::NullFrame;
    #[cfg(all(feature = "async", target_os = "linux"))]
    use crate::memory::mode::Async;
    use crate::memory::mode::Sync;
    use crate::memory::traits::mode::Mode;
    use crate::value::Value;
    use jl_sys::jl_value_t;

    pub trait Frame<'frame> {
        type Mode: Mode;
        // protect the value from being garbage collected while this frame is active.
        // safety: the value must be a valid pointer to a Julia value or a null pointer.
        unsafe fn push_root<'data>(
            &mut self,
            value: *mut jl_value_t,
            _: Internal,
        ) -> Result<Value<'frame, 'data>, AllocError>;

        fn nest<'nested>(
            &'nested mut self,
            capacity: usize,
            _: Internal,
        ) -> GcFrame<'nested, Self::Mode>;
    }

    impl<'frame, M: Mode> Frame<'frame> for GcFrame<'frame, M> {
        type Mode = M;

        unsafe fn push_root<'data>(
            &mut self,
            value: *mut jl_value_t,
            _: Internal,
        ) -> Result<Value<'frame, 'data>, AllocError> {
            let n_roots = self.n_roots();
            if n_roots == self.capacity() {
                return Err(AllocError::FrameOverflow(1, n_roots));
            }

            self.root(value);
            Ok(Value::wrap(value))
        }

        fn nest<'nested>(
            &'nested mut self,
            capacity: usize,
            _: Internal,
        ) -> GcFrame<'nested, Self::Mode> {
            self.nest(capacity)
        }
    }

    #[cfg(all(feature = "async", target_os = "linux"))]
    impl<'frame> Frame<'frame> for AsyncGcFrame<'frame> {
        type Mode = Async<'frame>;

        unsafe fn push_root<'data>(
            &mut self,
            value: *mut jl_value_t,
            _: Internal,
        ) -> Result<Value<'frame, 'data>, AllocError> {
            let n_roots = self.n_roots();
            if n_roots == self.capacity() {
                return Err(AllocError::FrameOverflow(1, n_roots));
            }

            self.root(value);
            Ok(Value::wrap(value))
        }

        fn nest<'nested>(
            &'nested mut self,
            capacity: usize,
            _: Internal,
        ) -> GcFrame<'nested, Self::Mode> {
            self.nest(capacity)
        }
    }

    impl<'frame> Frame<'frame> for NullFrame<'frame> {
        type Mode = Sync;

        unsafe fn push_root<'data>(
            &mut self,
            _value: *mut jl_value_t,
            _: Internal,
        ) -> Result<Value<'frame, 'data>, AllocError> {
            Err(AllocError::FrameOverflow(1, 0))
        }

        fn nest<'nested>(
            &'nested mut self,
            _capacity: usize,
            _: Internal,
        ) -> GcFrame<'nested, Self::Mode> {
            unreachable!()
        }
    }
}
