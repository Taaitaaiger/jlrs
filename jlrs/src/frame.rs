//! Frames ensure Julia's garbage collector is properly managed.
//!
//! Julia data is freed by the GC when it's not in use. You will need to use frames to do things
//! like calling Julia functions and creating new values; this ensures the values created with a
//! specific frame are protected from garbage collection until that frame goes out of scope.
//!
//! Frames can be nested, the two frame types that currently exist can be freely mixed. The main
//! difference between the two is that a [`StaticFrame`] is created with a definite capacity,
//! while a [`DynamicFrame`] will dynamically grow its capacity whenever a value is created or a
//! function is called. A `StaticFrame` is more efficient, a `DynamicFrame` is easier to use.
//! Creating a nested frame takes no space in the current frame.
//!
//! Frames have a lifetime, `'frame`. This lifetime ensures that a [`Value`] can only be used as
//! long as the frame that protects it has not been dropped.
//!
//! Most functionality that frames implement is defined in the [`Frame`] trait.
//!
//! [`StaticFrame`]: struct.StaticFrame.html
//! [`DynamicFrame`]: struct.DynamicFrame.html
//! [`Julia::frame`]: ../struct.Julia.html#method.frame
//! [`Julia::dynamic_frame`]: ../struct.Julia.html#method.dynamic_frame
//! [`Value`]: ../value/struct.Value.html
//! [`Frame`]: ../traits/trait.Frame.html

use crate::error::JlrsResult;
use crate::stack::{Dynamic, FrameIdx, StackView, Static};
use std::marker::PhantomData;

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
pub struct StaticFrame<'frame> {
    pub(crate) idx: FrameIdx,
    pub(crate) memory: StackView<'frame, Static>,
    pub(crate) capacity: usize,
    pub(crate) len: usize,
}

impl<'frame> StaticFrame<'frame> {
    pub(crate) unsafe fn with_capacity(
        idx: FrameIdx,
        capacity: usize,
        memory: StackView<'frame, Static>,
    ) -> StaticFrame<'frame> {
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
    ) -> JlrsResult<StaticFrame<'nested>> {
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

impl<'frame> Drop for StaticFrame<'frame> {
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
pub struct DynamicFrame<'frame> {
    pub(crate) idx: FrameIdx,
    pub(crate) memory: StackView<'frame, Dynamic>,
    pub(crate) len: usize,
}

impl<'frame> DynamicFrame<'frame> {
    pub(crate) unsafe fn new(idx: FrameIdx, memory: StackView<'frame, Dynamic>) -> Self {
        DynamicFrame {
            idx,
            memory,
            len: 0,
        }
    }

    pub(crate) unsafe fn nested_frame<'nested>(
        &'nested mut self,
    ) -> JlrsResult<DynamicFrame<'nested>> {
        let idx = self.memory.new_frame()?;
        Ok(DynamicFrame {
            idx,
            memory: self.memory.nest_dynamic(),
            len: 0,
        })
    }
}

impl<'frame> Drop for DynamicFrame<'frame> {
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
