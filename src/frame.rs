//! Frames ensure Julia's garbage collector (GC) is properly managed.
//!
//! Julia data is freed by its GC when it's not in use. You will need to use a frame to do things
//! like calling Julia functions, creating new values, and accessing modules; this ensures the
//! values created in a specific frame are protected from garbage collection until that frame goes
//! out of scope.
//!
//! Frames can be nested, the two frame types that currently exist can be freely mixed. The main
//! difference between the two is that a [`StaticFrame`] is created with a definite capacity,
//! while a [`DynamicFrame`] will dynamically grow its capacity whenever a value is created or a
//! function is called. A `StaticFrame` is more efficient, a `DynamicFrame` is easier to use.
//! Creating a nested frame takes no space in the current frame.
//!
//! Frames have two lifetimes, `'base` and `'frame`. The former is used to allow global values,
//! like modules and functions defined in them, to be freely used across frames; the only
//! restriction is that you can't return them from the base frame that was created through
//! [`Julia::frame`] or [`Julia::dynamic_frame`]. The latter is used by data that is only
//! valid until its frame goes out of scope, as a result values can only be used when they're
//! guaranteed to be protected from garbage collection.
//!
//! Most functionality that frames implement is defined in the [`Frame`] trait.
//!
//! [`StaticFrame`]: struct.StaticFrame.html
//! [`DynamicFrame`]: struct.DynamicFrame.html
//! [`Julia::frame`]: ../struct.Julia.html#method.frame
//! [`Julia::dynamic_frame`]: ../struct.Julia.html#method.dynamic_frame
//! [`Frame`]: ../traits/trait.Frame.html

use crate::error::JlrsResult;
use crate::stack::{Dynamic, FrameIdx, StackView, Static};
use std::marker::PhantomData;

pub(crate) struct Scope;

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
pub struct StaticFrame<'base: 'frame, 'frame> {
    pub(crate) idx: FrameIdx,
    pub(crate) memory: StackView<'frame, Static>,
    pub(crate) capacity: usize,
    pub(crate) len: usize,
    pub(crate) _guard: PhantomData<&'base ()>,
}

impl<'base: 'frame, 'frame> StaticFrame<'base, 'frame> {
    pub(crate) unsafe fn with_capacity(
        idx: FrameIdx,
        capacity: usize,
        memory: StackView<'frame, Static>,
        _: &'base mut Scope,
    ) -> StaticFrame<'base, 'frame> {
        StaticFrame {
            idx,
            memory,
            capacity,
            len: 0,
            _guard: PhantomData,
        }
    }

    pub(crate) unsafe fn nested_frame<'nested>(
        &'nested mut self,
        capacity: usize,
        scope: &'nested mut Scope,
    ) -> JlrsResult<StaticFrame<'base, 'nested>> {
        let idx = self.memory.new_frame(capacity)?;
        Ok(StaticFrame {
            idx,
            memory: self.memory.nest_static(scope),
            capacity,
            len: 0,
            _guard: PhantomData,
        })
    }

    /// Returns the total number of slots.
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl<'base: 'frame, 'frame> Drop for StaticFrame<'base, 'frame> {
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
pub struct DynamicFrame<'base: 'frame, 'frame> {
    pub(crate) idx: FrameIdx,
    pub(crate) memory: StackView<'frame, Dynamic>,
    pub(crate) len: usize,
    pub(crate) _guard: PhantomData<&'base ()>,
}

impl<'base: 'frame, 'frame> DynamicFrame<'base, 'frame> {
    pub(crate) unsafe fn new(
        idx: FrameIdx,
        memory: StackView<'frame, Dynamic>,
        _: &'base mut Scope,
    ) -> Self {
        DynamicFrame {
            idx,
            memory,
            len: 0,
            _guard: PhantomData,
        }
    }

    pub(crate) unsafe fn nested_frame<'nested>(
        &'nested mut self,
        scope: &'nested mut Scope,
    ) -> JlrsResult<DynamicFrame<'base, 'nested>> {
        let idx = self.memory.new_frame()?;
        Ok(DynamicFrame {
            idx,
            memory: self.memory.nest_dynamic(scope),
            len: 0,
            _guard: PhantomData,
        })
    }
}

impl<'base: 'frame, 'frame> Drop for DynamicFrame<'base, 'frame> {
    fn drop(&mut self) {
        unsafe {
            self.memory.pop_frame(self.idx);
        }
    }
}

/// An `Output` is a slot on the GC stack in the frame that was used to create it. It can be used
/// to extend the lifetime of the result of a function call to the `Output`'s lifetime. You can
/// create an output by calling [`Frame::output`].
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
