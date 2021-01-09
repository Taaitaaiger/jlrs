//! Functionality shared by the different frame types.

use crate::error::{CallResult, JlrsError, JlrsResult};
#[cfg(all(feature = "async", target_os = "linux"))]
use crate::frame::DynamicAsyncFrame;
use crate::frame::{
    DynamicFrame, NullFrame, Output, StaticFrame, UnrootedCallResult, UnrootedValue,
};
use crate::global::Global;
use crate::mode::Sync;
use crate::traits::mode::Mode;
use crate::traits::root::Root;
use crate::value::Value;

/// Functionality shared by all frame types. Frames are used to protect data from garbage
/// collection. The lifetime of a frame is assigned to the values and outputs
/// that are created using that frame. After a frame is dropped, these items are no longer
/// protected and cannot be used.
pub trait Frame<'frame>: private::Frame<'frame> {
    /// This method takes a mutable reference to a frame and returns it; this method can be used
    /// as an alternative to reborrowing a frame with `&mut *` when a [`Scope`] is needed.
    fn as_scope(&mut self) -> &mut Self {
        self
    }

    /// Create a `StaticFrame` that can hold `capacity` values, and call the given closure.
    /// Returns the result of this closure.
    fn frame<T, F: for<'nested> FnOnce(&mut StaticFrame<'nested, Self::Mode>) -> JlrsResult<T>>(
        &mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<T>;

    /// Create a `DynamicFrame` and call the given closure. Returns the result of this closure. A
    /// dynamic frame can contain at least 64 values, but is not guaranteed to be able to contain
    /// more than that.
    fn dynamic_frame<
        T,
        F: for<'nested> FnOnce(&mut DynamicFrame<'nested, Self::Mode>) -> JlrsResult<T>,
    >(
        &mut self,
        func: F,
    ) -> JlrsResult<T>;

    /// Create a new `StaticFrame` that can be used to root `capacity` values, an `Output` for the
    /// current scope, and use them to call the inner closure. The final result is not rooted in
    /// this newly created frame, but the current frame. The final result must not be the result
    /// of a function call, use [`Frame::call_frame`] for that purpose instead. If the current
    /// scope is a mutable reference to a frame, calling this method will require one slot of the
    /// current frame.
    ///
    /// Example:
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # use jlrs::util::JULIA;
    /// # fn main() {
    /// # JULIA.with(|j| {
    /// # let mut julia = j.borrow_mut();
    ///   julia.frame(1, |global, frame| {
    ///       let _nt = frame.value_frame(2, |output, frame| {
    ///           let v1 = Value::new(&mut *frame, 1usize)?;
    ///           let v2 = Value::new(&mut *frame, 2usize)?;
    ///
    ///           let output = output.into_scope(frame);
    ///           named_tuple!(output, "a" => v1, "b" => v2)
    ///       })?;
    ///
    ///       Ok(())
    ///   }).unwrap();
    /// # });
    /// # }
    /// ```
    fn value_frame<'data, F>(
        &mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<Value<'frame, 'data>>
    where
        F: for<'nested, 'inner> FnOnce(
            Output<'frame>,
            &'inner mut StaticFrame<'nested, Self::Mode>,
        ) -> JlrsResult<UnrootedValue<'frame, 'data, 'inner>>;

    /// Create a new `StaticFrame` that can be used to root `capacity` values, an `Output` for the
    /// current scope, and use them to call the inner closure. The final result is not rooted in
    /// this newly created frame, but the current frame. The final result must be the result of a
    /// function call, if you want to create a new value use [`Frame::value_frame`] instead. If
    /// the current scope is a mutable reference to a frame, calling this method will require one
    /// slot of the current frame.
    ///
    /// Example:
    ///
    /// ```
    /// # use jlrs::prelude::*;
    /// # use jlrs::util::JULIA;
    /// # fn main() {
    /// # JULIA.with(|j| {
    /// # let mut julia = j.borrow_mut();
    ///   julia.frame(1, |global, frame| {
    ///       let sum = frame.call_frame(2, |output, frame| {
    ///           let v1 = Value::new(&mut *frame, 1usize)?;
    ///           let v2 = Value::new(&mut *frame, 2usize)?;
    ///
    ///           let add = Module::base(global).function("+")?;
    ///
    ///           let output = output.into_scope(frame);
    ///           add.call2(output, v1, v2)
    ///       })?.unwrap().cast::<usize>()?;
    ///
    ///       assert_eq!(sum, 3);
    ///       Ok(())
    ///   }).unwrap();
    /// # });
    /// # }
    /// ```
    fn call_frame<'data, F>(
        &mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<CallResult<'frame, 'data>>
    where
        F: for<'nested, 'inner> FnOnce(
            Output<'frame>,
            &'inner mut StaticFrame<'nested, Self::Mode>,
        )
            -> JlrsResult<UnrootedCallResult<'frame, 'data, 'inner>>;

    /// Returns the number of values belonging to this frame.
    fn size(&self) -> usize;

    /// Create a new `Global` which can be used to access a global in Julia. These globals are
    /// limited to the frame's lifetime rather than the base lifetime.
    fn global(&self) -> Global<'frame> {
        unsafe { Global::new() }
    }
}

impl<'frame, M: Mode> Frame<'frame> for StaticFrame<'frame, M> {
    fn frame<T, F: for<'nested> FnOnce(&mut StaticFrame<'nested, M>) -> JlrsResult<T>>(
        &mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<T> {
        unsafe {
            let mut frame = self.nested_frame(capacity);
            func(&mut frame)
        }
    }

    fn dynamic_frame<T, F: for<'nested> FnOnce(&mut DynamicFrame<'nested, M>) -> JlrsResult<T>>(
        &mut self,
        func: F,
    ) -> JlrsResult<T> {
        unsafe {
            let mut frame = self.nested_dynamic_frame();
            func(&mut frame)
        }
    }

    fn value_frame<'data, F>(
        &mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<Value<'frame, 'data>>
    where
        F: for<'nested, 'inner> FnOnce(
            Output<'frame>,
            &'inner mut StaticFrame<'nested, Self::Mode>,
        ) -> JlrsResult<UnrootedValue<'frame, 'data, 'inner>>,
    {
        unsafe {
            let pr = {
                let mut frame = self.nested_frame(capacity);
                let out = Output::new();
                func(out, &mut frame)?.done()
            };

            Value::root(self, pr)
        }
    }

    fn call_frame<'data, F>(
        &mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<CallResult<'frame, 'data>>
    where
        F: for<'nested, 'inner> FnOnce(
            Output<'frame>,
            &'inner mut StaticFrame<'nested, Self::Mode>,
        )
            -> JlrsResult<UnrootedCallResult<'frame, 'data, 'inner>>,
    {
        unsafe {
            let pr = {
                let mut frame = self.nested_frame(capacity);
                let out = Output::new();
                func(out, &mut frame)?.done()
            };

            CallResult::root(self, pr)
        }
    }

    fn size(&self) -> usize {
        self.size()
    }
}

impl<'frame, M: Mode> Frame<'frame> for DynamicFrame<'frame, M> {
    fn frame<T, F: for<'nested> FnOnce(&mut StaticFrame<'nested, M>) -> JlrsResult<T>>(
        &mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<T> {
        unsafe {
            let mut frame = self.nested_frame(capacity);
            func(&mut frame)
        }
    }

    fn dynamic_frame<T, F: for<'nested> FnOnce(&mut DynamicFrame<'nested, M>) -> JlrsResult<T>>(
        &mut self,
        func: F,
    ) -> JlrsResult<T> {
        let mut frame = unsafe { self.nested_dynamic_frame() };
        func(&mut frame)
    }

    fn value_frame<'data, F>(
        &mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<Value<'frame, 'data>>
    where
        F: for<'nested, 'inner> FnOnce(
            Output<'frame>,
            &'inner mut StaticFrame<'nested, Self::Mode>,
        ) -> JlrsResult<UnrootedValue<'frame, 'data, 'inner>>,
    {
        unsafe {
            let pr = {
                let mut frame = self.nested_frame(capacity);
                let out = Output::new();
                func(out, &mut frame)?.done()
            };

            Value::root(self, pr)
        }
    }

    fn call_frame<'data, F>(
        &mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<CallResult<'frame, 'data>>
    where
        F: for<'nested, 'inner> FnOnce(
            Output<'frame>,
            &'inner mut StaticFrame<'nested, Self::Mode>,
        )
            -> JlrsResult<UnrootedCallResult<'frame, 'data, 'inner>>,
    {
        unsafe {
            let pr = {
                let mut frame = self.nested_frame(capacity);
                let out = Output::new();
                func(out, &mut frame)?.done()
            };

            CallResult::root(self, pr)
        }
    }

    fn size(&self) -> usize {
        self.size()
    }
}

impl<'frame> Frame<'frame> for NullFrame<'frame> {
    fn frame<T, F: for<'nested> FnOnce(&mut StaticFrame<'nested, Sync>) -> JlrsResult<T>>(
        &mut self,
        _: usize,
        _: F,
    ) -> JlrsResult<T> {
        Err(JlrsError::NullFrame)?
    }

    fn dynamic_frame<
        T,
        F: for<'nested> FnOnce(&mut DynamicFrame<'nested, Sync>) -> JlrsResult<T>,
    >(
        &mut self,
        _: F,
    ) -> JlrsResult<T> {
        Err(JlrsError::NullFrame)?
    }

    fn value_frame<'data, F>(
        &mut self,
        _capacity: usize,
        _func: F,
    ) -> JlrsResult<Value<'frame, 'data>>
    where
        F: for<'nested, 'inner> FnOnce(
            Output<'frame>,
            &'inner mut StaticFrame<'nested, Sync>,
        ) -> JlrsResult<UnrootedValue<'frame, 'data, 'inner>>,
    {
        Err(JlrsError::NullFrame)?
    }

    fn call_frame<'data, F>(
        &mut self,
        _capacity: usize,
        _func: F,
    ) -> JlrsResult<CallResult<'frame, 'data>>
    where
        F: for<'nested, 'inner> FnOnce(
            Output<'frame>,
            &'inner mut StaticFrame<'nested, Sync>,
        )
            -> JlrsResult<UnrootedCallResult<'frame, 'data, 'inner>>,
    {
        Err(JlrsError::NullFrame)?
    }

    fn size(&self) -> usize {
        0
    }
}

#[cfg(all(feature = "async", target_os = "linux"))]
impl<'frame> Frame<'frame> for DynamicAsyncFrame<'frame> {
    fn frame<T, F: for<'nested> FnOnce(&mut StaticFrame<'nested, Self::Mode>) -> JlrsResult<T>>(
        &mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<T> {
        unsafe {
            let mut frame = self.nested_frame(capacity);
            func(&mut frame)
        }
    }

    fn dynamic_frame<
        T,
        F: for<'nested> FnOnce(&mut DynamicFrame<'nested, Self::Mode>) -> JlrsResult<T>,
    >(
        &mut self,
        func: F,
    ) -> JlrsResult<T> {
        let mut frame = unsafe { self.nested_dynamic_frame() };
        func(&mut frame)
    }

    fn value_frame<'data, F>(
        &mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<Value<'frame, 'data>>
    where
        F: for<'nested, 'inner> FnOnce(
            Output<'frame>,
            &'inner mut StaticFrame<'nested, Self::Mode>,
        ) -> JlrsResult<UnrootedValue<'frame, 'data, 'inner>>,
    {
        unsafe {
            let pr = {
                let mut frame = self.nested_frame(capacity);
                let out = Output::new();
                func(out, &mut frame)?.done()
            };

            Value::root(self, pr)
        }
    }

    fn call_frame<'data, F>(
        &mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<CallResult<'frame, 'data>>
    where
        F: for<'nested, 'inner> FnOnce(
            Output<'frame>,
            &'inner mut StaticFrame<'nested, Self::Mode>,
        )
            -> JlrsResult<UnrootedCallResult<'frame, 'data, 'inner>>,
    {
        unsafe {
            let pr = {
                let mut frame = self.nested_frame(capacity);
                let out = Output::new();
                func(out, &mut frame)?.done()
            };

            CallResult::root(self, pr)
        }
    }

    fn size(&self) -> usize {
        self.len()
    }
}

pub(crate) mod private {
    use super::super::private::Internal;
    use crate::error::AllocError;
    #[cfg(all(feature = "async", target_os = "linux"))]
    use crate::frame::DynamicAsyncFrame;
    use crate::frame::{DynamicFrame, NullFrame, StaticFrame};
    #[cfg(all(feature = "async", target_os = "linux"))]
    use crate::mode::Async;
    use crate::mode::Sync;
    use crate::traits::mode::Mode;
    use crate::value::Value;
    use jl_sys::jl_value_t;

    pub trait Frame<'frame> {
        type Mode: Mode;
        // protect the value from being garbage collected while this frame is active.
        // safety: the value must be a valid Julia value
        unsafe fn root(
            &mut self,
            value: *mut jl_value_t,
            _: Internal,
        ) -> Result<Value<'frame, 'static>, AllocError>;

        unsafe fn nested_frame<'nested>(
            &'nested mut self,
            capacity: usize,
            _: Internal,
        ) -> StaticFrame<'nested, Self::Mode>;

        unsafe fn nested_dynamic_frame<'nested>(
            &'nested mut self,
            _: Internal,
        ) -> DynamicFrame<'nested, Self::Mode>;
    }

    impl<'frame, M: Mode> Frame<'frame> for StaticFrame<'frame, M> {
        type Mode = M;
        unsafe fn root(
            &mut self,
            value: *mut jl_value_t,
            _: Internal,
        ) -> Result<Value<'frame, 'static>, AllocError> {
            if self.capacity() == self.size() {
                return Err(AllocError::FrameOverflow(1, self.size()));
            }

            let idx = self.size() + 2;
            let encoded_len = self.raw_frame[0] as usize + 2;
            self.raw_frame[0] = encoded_len as _;
            self.raw_frame[idx] = value.cast();

            Ok(Value::wrap(value))
        }

        unsafe fn nested_frame<'nested>(
            &'nested mut self,
            capacity: usize,
            _: Internal,
        ) -> StaticFrame<'nested, M> {
            self.nested_frame(capacity)
        }

        unsafe fn nested_dynamic_frame<'nested>(
            &'nested mut self,
            _: Internal,
        ) -> DynamicFrame<'nested, M> {
            self.nested_dynamic_frame()
        }
    }

    impl<'frame, M: Mode> Frame<'frame> for DynamicFrame<'frame, M> {
        type Mode = M;
        unsafe fn root(
            &mut self,
            value: *mut jl_value_t,
            _: Internal,
        ) -> Result<Value<'frame, 'static>, AllocError> {
            if self.size() + 2 == self.raw_frame.len() {
                return Err(AllocError::FrameOverflow(1, self.size()));
            }

            let idx = self.size() + 2;
            let encoded_len = self.raw_frame[0] as usize + 2;
            self.raw_frame[0] = encoded_len as _;
            self.raw_frame[idx] = value.cast();

            Ok(Value::wrap(value))
        }

        unsafe fn nested_frame<'nested>(
            &'nested mut self,
            capacity: usize,
            _: Internal,
        ) -> StaticFrame<'nested, M> {
            self.nested_frame(capacity)
        }

        unsafe fn nested_dynamic_frame<'nested>(
            &'nested mut self,
            _: Internal,
        ) -> DynamicFrame<'nested, M> {
            self.nested_dynamic_frame()
        }
    }

    impl<'frame> Frame<'frame> for NullFrame<'frame> {
        type Mode = Sync;
        unsafe fn root(
            &mut self,
            _: *mut jl_value_t,
            _: Internal,
        ) -> Result<Value<'frame, 'static>, AllocError> {
            Err(AllocError::FrameOverflow(1, 0))
        }

        unsafe fn nested_frame<'nested>(
            &'nested mut self,
            _capacity: usize,
            _: Internal,
        ) -> StaticFrame<'nested, Sync> {
            unreachable!()
        }

        unsafe fn nested_dynamic_frame<'nested>(
            &'nested mut self,
            _: Internal,
        ) -> DynamicFrame<'nested, Sync> {
            unreachable!()
        }
    }

    #[cfg(all(feature = "async", target_os = "linux"))]
    impl<'frame> Frame<'frame> for DynamicAsyncFrame<'frame> {
        type Mode = Async<'frame>;

        unsafe fn root(
            &mut self,
            value: *mut jl_value_t,
            _: Internal,
        ) -> Result<Value<'frame, 'static>, AllocError> {
            if self.len() + 2 == self.raw_frame.len() {
                return Err(AllocError::FrameOverflow(1, self.len()));
            }

            let idx = self.len() + 2;
            let encoded_len = self.raw_frame[0] as usize + 2;
            self.raw_frame[0] = encoded_len as _;
            self.raw_frame[idx] = value.cast();

            Ok(Value::wrap(value))
        }

        unsafe fn nested_frame<'nested>(
            &'nested mut self,
            capacity: usize,
            _: Internal,
        ) -> StaticFrame<'nested, Self::Mode> {
            todo!() // self.nested_frame(capacity)
        }

        unsafe fn nested_dynamic_frame<'nested>(
            &'nested mut self,
            _: Internal,
        ) -> DynamicFrame<'nested, Self::Mode> {
            todo!() // self.nested_dynamic_frame()
        }
    }
}
