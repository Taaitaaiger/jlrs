//! Frames ensure Julia's garbage collector is properly managed.
//!
//! Julia data is freed by the GC when it's not in use. You will need to use frames to do things
//! like calling Julia functions and creating new values, this ensures the values created with a
//! specific frame are protected from garbage collection until that frame goes out of scope.
//!
//! Four different kinds of frames exist; [`StaticFrame`], [`DynamicFrame`], [`NullFrame`], and
//! [`AsyncFrame`]. The first two of them can be nested and freely mixed. The main difference
//! between those two is that a [`StaticFrame`] is created with a definite capacity, while a
//! [`DynamicFrame`] will dynamically grow its capacity whenever a value is created or a function
//!  is called. A [`StaticFrame`] is more efficient, a [`DynamicFrame`] is easier to use. Creating
//! a nested frame takes no space in the current frame.
//!
//! The third type, [`NullFrame`] can only be used if you call Rust from Julia. They don't
//! allocate at all and can only be used to borrow array data.
//!
//! The final type, [`AsyncFrame`] is only available when you use the async runtime. Structs that
//! implement [`JuliaTask`] can use this kind of frame in the `run`-method. It's essentially a
//! [`DynamicFrame`] with the additional feature that it can be used to call
//! [`Value::call_async`].
//!
//! Frames have a lifetime, `'frame`. This lifetime ensures that a [`Value`] can only be used as
//! long as the frame that protects it has not been dropped.
//!
//! Most functionality that frames implement is defined by the [`Frame`] trait.
//!
//! [`StaticFrame`]: struct.StaticFrame.html
//! [`DynamicFrame`]: struct.DynamicFrame.html
//! [`NullFrame`]: struct.NullFrame.html
//! [`AsyncFrame`]: struct.AsyncFrame.html
//! [`Value`]: ../value/struct.Value.html
//! [`Value::call_async`]: ../value/struct.Value.html#method.call_async
//! [`Frame`]: ../traits/trait.Frame.html
//! [`JuliaTask`]: ../traits/multitask/trait.JuliaTask.html

use crate::error::JlrsResult;
#[cfg(feature = "async")]
use crate::mode::Async;
use crate::mode::Mode;
use crate::stack::{Dynamic, StackView, Static};
use crate::CCall;
use std::marker::PhantomData;

#[derive(Copy, Clone, Default)]
pub struct FrameIdx(pub(crate) usize);

/// A `StaticFrame` is a frame that has a definite number of slots on the GC stack. With some
/// exceptions, creating new `Value`s and calling them require one slot each. Rather than using
/// new slots on the GC stack when a slot is needed, a `StaticFrame` uses the slots it acquired on
/// creation. See the documentation in the [`value`] module for more information about the costs.
/// You get access to a `StaticFrame` by calling [`Julia::frame`] or [`Frame::frame`], most of
/// their functionality is defined in the [`Frame`] trait.
///
/// [`value`]: ../value/index.html
/// [`Julia::frame`]: ../struct.Julia.html#method.frame
/// [`Frame::frame`]: ../traits/trait.Frame.html#method.frame
/// [`Frame`]: ../traits/trait.Frame.html
pub struct StaticFrame<'frame, U>
where
    U: Mode,
{
    pub(crate) idx: FrameIdx,
    pub(crate) memory: StackView<'frame, U, Static>,
    pub(crate) capacity: usize,
    pub(crate) len: usize,
}

impl<'frame, M: Mode> StaticFrame<'frame, M> {
    pub(crate) unsafe fn with_capacity(
        idx: FrameIdx,
        capacity: usize,
        memory: StackView<'frame, M, Static>,
    ) -> StaticFrame<'frame, M> {
        StaticFrame {
            idx,
            memory,
            capacity,
            len: 0,
        }
    }

    pub(crate) unsafe fn nested_frame<'nested>(
        &'nested mut self,
        capacity: usize,
    ) -> JlrsResult<StaticFrame<'nested, M>> {
        let idx = self.memory.new_frame(capacity)?;
        Ok(StaticFrame {
            idx,
            memory: self.memory.nest_static(),
            capacity,
            len: 0,
        })
    }

    /// Returns the total number of slots.
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl<'frame, U> Drop for StaticFrame<'frame, U>
where
    U: Mode,
{
    fn drop(&mut self) {
        unsafe {
            self.memory.pop_frame(self.idx);
        }
    }
}

/// A `DynamicFrame` is a frame that has a dynamic number of slots on the GC stack. With some
/// exceptions, creating new `Value`s and calling them require one slot each. A `DynamicFrame`
/// acquires a new slot every time one is needed. See the documentation in the [`value`] module
/// for more information about the costs. You get access to a `DynamicFrame` by calling
/// [`Julia::dynamic_frame`] or [`Frame::dynamic_frame`], most of
/// their functionality is defined in the [`Frame`] trait.
///
/// [`value`]: ../value/index.html
/// [`Julia::dynamic_frame`]: ../struct.Julia.html#method.dynamic_frame
/// [`Frame::dynamic_frame`]: ../traits/trait.Frame.html#method.dynamic_frame
/// [`Frame`]: ../traits/trait.Frame.html
pub struct DynamicFrame<'frame, U>
where
    U: Mode,
{
    pub(crate) idx: FrameIdx,
    pub(crate) memory: StackView<'frame, U, Dynamic>,
    pub(crate) len: usize,
}

impl<'frame, M: Mode> DynamicFrame<'frame, M> {
    pub(crate) unsafe fn new(idx: FrameIdx, memory: StackView<'frame, M, Dynamic>) -> Self {
        DynamicFrame {
            idx,
            memory,
            len: 0,
        }
    }

    pub(crate) unsafe fn nested_frame<'nested>(
        &'nested mut self,
    ) -> JlrsResult<DynamicFrame<'nested, M>> {
        let idx = self.memory.new_frame()?;
        Ok(DynamicFrame {
            idx,
            memory: self.memory.nest_dynamic(),
            len: 0,
        })
    }
}

impl<'frame, U> Drop for DynamicFrame<'frame, U>
where
    U: Mode,
{
    fn drop(&mut self) {
        unsafe {
            self.memory.pop_frame(self.idx);
        }
    }
}

/// An `Output` is a slot of a frame that has been reserved for later use. It can be used to
/// extend the lifetime of the result of a function call to the `Output`'s lifetime. You can
/// create an output by calling [`Frame::output`].
///
/// [`Frame::output`]: ../traits/trait.Frame.html#method.output
pub struct Output<'frame> {
    pub(crate) offset: usize,
    _marker: PhantomData<&'frame ()>,
}

impl<'frame> Output<'frame> {
    pub(crate) unsafe fn new(offset: usize) -> Self {
        Output {
            offset,
            _marker: PhantomData,
        }
    }
}

/// A `NullFrame` can be used if you call Rust from Julia through `ccall` and want to borrow array
/// data but not perform any allocations. It can't be nested or be used for functions that
/// allocate (like creating new values or calling functions). Functions that depend on allocation
/// will return `JlrsError::NullFrame` if you call them with a `NullFrame`.
pub struct NullFrame<'frame>(PhantomData<&'frame ()>);

impl<'frame> NullFrame<'frame> {
    pub(crate) unsafe fn new(_: &'frame mut CCall) -> Self {
        NullFrame(PhantomData)
    }
}

/// An `AsyncFrame` is a special kind of `DynamicFrame` that's available when you implement
/// [`JuliaTask`]. In addition to the capabilities of a `DynamicFrame` it can be used to call
/// [`Value::call_async`] which lets you call a function in a new thread in Julia. This feature is
/// only available by using [`AsyncJulia`].
///
/// [`Value::call_async`]: ../value/struct.Value.html#method.call_async
/// [`JuliaTask`]: ../traits/multitask/trait.JuliaTask.html
/// [`AsyncJulia`]: ../multitask/struct.AsyncJulia.html
#[cfg(feature = "async")]
pub struct AsyncFrame<'frame> {
    pub(crate) idx: FrameIdx,
    pub(crate) memory: StackView<'frame, Async, Dynamic>,
    pub(crate) len: usize,
}

#[cfg(feature = "async")]
impl<'frame> AsyncFrame<'frame> {
    pub(crate) unsafe fn nested_frame<'nested>(
        &'nested mut self,
    ) -> JlrsResult<DynamicFrame<'nested, Async>> {
        let idx = self.memory.new_frame()?;
        Ok(DynamicFrame {
            idx,
            memory: self.memory.nest_dynamic(),
            len: 0,
        })
    }
}

#[cfg(feature = "async")]
impl<'frame> Drop for AsyncFrame<'frame> {
    fn drop(&mut self) {
        unsafe {
            self.memory.pop_frame(self.idx);
        }
    }
}
