use crate::error::{AllocError, JlrsError, JlrsResult};
#[cfg(all(feature = "async", target_os = "linux"))]
use crate::frame::AsyncFrame;
use crate::frame::{DynamicFrame, NullFrame, Output, StaticFrame};
use crate::global::Global;
#[cfg(all(feature = "async", target_os = "linux"))]
use crate::mode::Async;
use crate::mode::{Mode, Sync};

/// Functionality shared by [`StaticFrame`] and [`DynamicFrame`]. These structs let you protect
/// data from garbage collection. The lifetime of a frame is assigned to the values and outputs
/// that are created using that frame. After a frame is dropped, these items are no longer
/// protected and cannot be used.
///
/// If you need the result of a function call to be valid outside the frame where it is called,
/// you can call `Frame::output` to create an [`Output`] and use [`Value::with_output`] to use the
/// output to protect the value rather than the current frame. The result will share the output's
/// lifetime so it can be used until the output's frame goes out of scope.
///
/// [`StaticFrame`]: ../frame/struct.StaticFrame.html
/// [`DynamicFrame`]: ../frame/struct.DynamicFrame.html
/// [`Module`]: ../module/struct.Module.html
/// [`Julia::frame`]: ../struct.Julia.html#method.frame
/// [`Julia::dynamic_frame`]: ../struct.Julia.html#method.dynamic_frame
/// [`Output`]: ../frame/struct.Output.html
/// [`Value::with_output`]: ../value/struct.Value.html#method.with_output
pub trait Frame<'frame>: private::Frame<'frame> {
    /// Create a `StaticFrame` that can hold `capacity` values, and call the given closure.
    /// Returns the result of this closure, or an error if the new frame can't be created
    /// because there's not enough space on the GC stack. The number of required slots on the
    /// stack is `capacity + 2`.
    ///
    /// Returns an error if there is not enough space on the stack.
    fn frame<'nested, T, F: FnOnce(&mut StaticFrame<'nested, Self::U>) -> JlrsResult<T>>(
        &'nested mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<T>;

    /// Create a `DynamicFrame` and call the given closure.  Returns the result of this closure,
    /// or an error if the new frame can't be created because the stack is too small. The number
    /// of required slots on the stack is `2`.
    ///
    /// Returns an error if there is not enough space on the stack.
    fn dynamic_frame<'nested, T, F: FnOnce(&mut DynamicFrame<'nested, Self::U>) -> JlrsResult<T>>(
        &'nested mut self,
        func: F,
    ) -> JlrsResult<T>;

    /// Returns a new `Output`, this takes one slot on the GC stack. A function that uses this
    /// output will not use a slot on the GC stack, but the one associated with this output. This
    /// extends the lifetime of that value to be valid until the frame that created the output
    /// goes out of scope.
    ///
    /// Returns an error if there is not enough space on the stack.
    fn output(&mut self) -> JlrsResult<Output<'frame>>;

    /// Returns the number of values belonging to this frame.
    fn size(&self) -> usize;

    #[doc(hidden)]
    // Exists for debugging purposes, prints the contents of the GC stack.
    fn print_memory(&self);

    /// Create a new `Global` which can be used to access a global in Julia. These globals are 
    /// limited to the frame's lifetime rather than the base lifetime. 
    fn global(&self) -> Global<'frame> {
        unsafe {
            Global::new()
        }
    }
}

impl<'frame, M: Mode> Frame<'frame> for StaticFrame<'frame, M> {
    fn frame<'nested, T, F: FnOnce(&mut StaticFrame<'nested, M>) -> JlrsResult<T>>(
        &'nested mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<T> {
        let mut frame = unsafe { self.nested_frame(capacity).unwrap() };
        func(&mut frame)
    }

    fn dynamic_frame<'nested, T, F: FnOnce(&mut DynamicFrame<'nested, M>) -> JlrsResult<T>>(
        &'nested mut self,
        func: F,
    ) -> JlrsResult<T> {
        unsafe {
            let mut view = self.memory.nest_dynamic();
            let idx = view.new_frame()?;
            let mut frame = DynamicFrame {
                idx,
                len: 0,
                memory: view,
            };

            func(&mut frame)
        }
    }

    fn output(&mut self) -> JlrsResult<Output<'frame>> {
        if self.capacity == self.len {
            return Err(AllocError::FrameOverflow(1, self.len).into());
        }

        let out = unsafe {
            let out = self.memory.new_output(self.idx, self.len);
            self.len += 1;
            out
        };

        Ok(out)
    }

    fn size(&self) -> usize {
        self.len
    }

    fn print_memory(&self) {
        self.memory.print_memory()
    }
}

impl<'frame, M: Mode> Frame<'frame> for DynamicFrame<'frame, M> {
    fn dynamic_frame<'nested, T, F: FnOnce(&mut DynamicFrame<'nested, M>) -> JlrsResult<T>>(
        &'nested mut self,
        func: F,
    ) -> JlrsResult<T> {
        let mut frame = unsafe { self.nested_frame().unwrap() };
        func(&mut frame)
    }

    fn frame<'nested, T, F: FnOnce(&mut StaticFrame<'nested, M>) -> JlrsResult<T>>(
        &'nested mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<T> {
        unsafe {
            let mut view = self.memory.nest_static();
            let idx = view.new_frame(capacity)?;
            let mut frame = StaticFrame {
                idx,
                capacity,
                len: 0,
                memory: view,
            };

            func(&mut frame)
        }
    }

    fn output(&mut self) -> JlrsResult<Output<'frame>> {
        unsafe {
            let out = self.memory.new_output(self.idx)?;
            self.len += 1;
            Ok(out)
        }
    }

    fn size(&self) -> usize {
        self.len
    }

    fn print_memory(&self) {
        self.memory.print_memory()
    }
}

impl<'frame> Frame<'frame> for NullFrame<'frame> {
    fn frame<'nested, T, F: FnOnce(&mut StaticFrame<'nested, Sync>) -> JlrsResult<T>>(
        &'nested mut self,
        _: usize,
        _: F,
    ) -> JlrsResult<T> {
        Err(JlrsError::NullFrame)?
    }

    fn dynamic_frame<'nested, T, F: FnOnce(&mut DynamicFrame<'nested, Sync>) -> JlrsResult<T>>(
        &'nested mut self,
        _: F,
    ) -> JlrsResult<T> {
        Err(JlrsError::NullFrame)?
    }

    fn output(&mut self) -> JlrsResult<Output<'frame>> {
        Err(JlrsError::NullFrame)?
    }

    fn size(&self) -> usize {
        0
    }

    fn print_memory(&self) {}
}

#[cfg(all(feature = "async", target_os = "linux"))]
impl<'frame> Frame<'frame> for AsyncFrame<'frame> {
    fn frame<'nested, T, F: FnOnce(&mut StaticFrame<'nested, Async>) -> JlrsResult<T>>(
        &'nested mut self,
        capacity: usize,
        func: F,
    ) -> JlrsResult<T> {
        unsafe {
            let mut view = self.memory.nest_static();
            let idx = view.new_frame(capacity)?;
            let mut frame = StaticFrame {
                idx,
                capacity,
                len: 0,
                memory: view,
            };

            func(&mut frame)
        }
    }

    fn dynamic_frame<'nested, T, F: FnOnce(&mut DynamicFrame<'nested, Async>) -> JlrsResult<T>>(
        &'nested mut self,
        func: F,
    ) -> JlrsResult<T> {
        let mut frame = unsafe { self.nested_frame().unwrap() };
        func(&mut frame)
    }

    fn output(&mut self) -> JlrsResult<Output<'frame>> {
        unsafe {
            let out = self.memory.new_output(self.idx)?;
            self.len += 1;
            Ok(out)
        }
    }

    fn size(&self) -> usize {
        self.len
    }

    fn print_memory(&self) {
        self.memory.print_memory()
    }
}

pub(crate) mod private {
    use super::super::{private::Internal, IntoJulia};
    use crate::error::AllocError;
    #[cfg(all(feature = "async", target_os = "linux"))]
    use crate::frame::AsyncFrame;
    use crate::frame::{DynamicFrame, FrameIdx, NullFrame, Output, StaticFrame};
    #[cfg(all(feature = "async", target_os = "linux"))]
    use crate::mode::Async;
    use crate::mode::{Mode, Sync};
    use crate::value::{Value, Values};
    use jl_sys::jl_value_t;

    pub trait Frame<'frame> {
        type U: Mode;
        // protect the value from being garbage collected while this frame is active.
        // safety: the value must be a valid Julia value
        unsafe fn protect(
            &mut self,
            value: *mut jl_value_t,
            _: Internal,
        ) -> Result<Value<'frame, 'static>, AllocError>;

        // Create and protect multiple values from being garbage collected while this frame is active.
        fn create_many<P: IntoJulia>(
            &mut self,
            values: &[P],
            _: Internal,
        ) -> Result<Values<'frame>, AllocError>;

        // Create and protect multiple values from being garbage collected while this frame is active.
        fn create_many_dyn(
            &mut self,
            values: &[&dyn IntoJulia],
            _: Internal,
        ) -> Result<Values<'frame>, AllocError>;

        // Protect a value from being garbage collected while the output's frame is active.
        fn assign_output<'output>(
            &mut self,
            output: Output<'output>,
            value: *mut jl_value_t,
            _: Internal,
        ) -> Value<'output, 'static>;
    }

    impl<'frame, M: Mode> Frame<'frame> for StaticFrame<'frame, M> {
        type U = M;
        unsafe fn protect(
            &mut self,
            value: *mut jl_value_t,
            _: Internal,
        ) -> Result<Value<'frame, 'static>, AllocError> {
            if self.capacity == self.len {
                return Err(AllocError::FrameOverflow(1, self.len));
            }

            let out = {
                let out = self.memory.protect(self.idx, self.len, value.cast());
                self.len += 1;
                out
            };

            Ok(out)
        }

        fn create_many<P: IntoJulia>(
            &mut self,
            values: &[P],
            _: Internal,
        ) -> Result<Values<'frame>, AllocError> {
            unsafe {
                if self.capacity < self.len + values.len() {
                    return Err(AllocError::FrameOverflow(values.len(), self.capacity()));
                }

                let offset = self.len;
                for value in values {
                    self.memory
                        .protect(self.idx, self.len, value.into_julia().cast());
                    self.len += 1;
                }

                Ok(self.memory.as_values(self.idx, offset, values.len()))
            }
        }

        fn create_many_dyn(
            &mut self,
            values: &[&dyn IntoJulia],
            _: Internal,
        ) -> Result<Values<'frame>, AllocError> {
            unsafe {
                if self.capacity < self.len + values.len() {
                    return Err(AllocError::FrameOverflow(values.len(), self.capacity()));
                }

                let offset = self.len;
                for value in values {
                    self.memory
                        .protect(self.idx, self.len, value.into_julia().cast());
                    self.len += 1;
                }

                Ok(self.memory.as_values(self.idx, offset, values.len()))
            }
        }

        fn assign_output<'output>(
            &mut self,
            output: Output<'output>,
            value: *mut jl_value_t,
            _: Internal,
        ) -> Value<'output, 'static> {
            unsafe {
                self.memory
                    .protect(FrameIdx::default(), output.offset, value.cast())
            }
        }
    }

    impl<'frame, M: Mode> Frame<'frame> for DynamicFrame<'frame, M> {
        type U = M;
        unsafe fn protect(
            &mut self,
            value: *mut jl_value_t,
            _: Internal,
        ) -> Result<Value<'frame, 'static>, AllocError> {
            let out = self.memory.protect(self.idx, value.cast())?;
            self.len += 1;
            Ok(out)
        }

        fn create_many<P: IntoJulia>(
            &mut self,
            values: &[P],
            _: Internal,
        ) -> Result<Values<'frame>, AllocError> {
            unsafe {
                let offset = self.len;
                // TODO: check capacity

                for value in values {
                    match self.memory.protect(self.idx, value.into_julia().cast()) {
                        Ok(_) => (),
                        Err(AllocError::StackOverflow(_, n)) => {
                            return Err(AllocError::StackOverflow(values.len(), n))
                        }
                        _ => unreachable!(),
                    }
                    self.len += 1;
                }

                Ok(self.memory.as_values(self.idx, offset, values.len()))
            }
        }

        fn create_many_dyn(
            &mut self,
            values: &[&dyn IntoJulia],
            _: Internal,
        ) -> Result<Values<'frame>, AllocError> {
            unsafe {
                let offset = self.len;
                // TODO: check capacity in advance

                for value in values {
                    self.memory.protect(self.idx, value.into_julia().cast())?;
                    self.len += 1;
                }

                Ok(self.memory.as_values(self.idx, offset, values.len()))
            }
        }

        fn assign_output<'output>(
            &mut self,
            output: Output<'output>,
            value: *mut jl_value_t,
            _: Internal,
        ) -> Value<'output, 'static> {
            unsafe { self.memory.protect_output(output, value.cast()) }
        }
    }

    impl<'frame> Frame<'frame> for NullFrame<'frame> {
        type U = Sync;
        unsafe fn protect(
            &mut self,
            _: *mut jl_value_t,
            _: Internal,
        ) -> Result<Value<'frame, 'static>, AllocError> {
            Err(AllocError::FrameOverflow(1, 0))
        }

        fn create_many<P: IntoJulia>(
            &mut self,
            values: &[P],
            _: Internal,
        ) -> Result<Values<'frame>, AllocError> {
            Err(AllocError::FrameOverflow(values.len(), 0))
        }

        fn create_many_dyn(
            &mut self,
            values: &[&dyn IntoJulia],
            _: Internal,
        ) -> Result<Values<'frame>, AllocError> {
            Err(AllocError::FrameOverflow(values.len(), 0))
        }

        fn assign_output<'output>(
            &mut self,
            _: Output<'output>,
            _: *mut jl_value_t,
            _: Internal,
        ) -> Value<'output, 'static> {
            unreachable!()
        }
    }

    #[cfg(all(feature = "async", target_os = "linux"))]
    impl<'frame> Frame<'frame> for AsyncFrame<'frame> {
        type U = Async;

        unsafe fn protect(
            &mut self,
            value: *mut jl_value_t,
            _: Internal,
        ) -> Result<Value<'frame, 'static>, AllocError> {
            let out = self.memory.protect(self.idx, value.cast())?;
            self.len += 1;
            Ok(out)
        }

        fn create_many<P: IntoJulia>(
            &mut self,
            values: &[P],
            _: Internal,
        ) -> Result<Values<'frame>, AllocError> {
            unsafe {
                let offset = self.len;
                // TODO: check capacity

                for value in values {
                    match self.memory.protect(self.idx, value.into_julia().cast()) {
                        Ok(_) => (),
                        Err(AllocError::StackOverflow(_, n)) => {
                            return Err(AllocError::StackOverflow(values.len(), n))
                        }
                        _ => unreachable!(),
                    }
                    self.len += 1;
                }

                Ok(self.memory.as_values(self.idx, offset, values.len()))
            }
        }

        fn create_many_dyn(
            &mut self,
            values: &[&dyn IntoJulia],
            _: Internal,
        ) -> Result<Values<'frame>, AllocError> {
            unsafe {
                let offset = self.len;
                // TODO: check capacity in advance

                for value in values {
                    self.memory.protect(self.idx, value.into_julia().cast())?;
                    self.len += 1;
                }

                Ok(self.memory.as_values(self.idx, offset, values.len()))
            }
        }

        fn assign_output<'output>(
            &mut self,
            output: Output<'output>,
            value: *mut jl_value_t,
            _: Internal,
        ) -> Value<'output, 'static> {
            unsafe { self.memory.protect_output(output, value.cast()) }
        }
    }
}
