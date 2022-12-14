//! A target that uses a reserved slot in a frame.

use std::ptr::NonNull;

use super::output::Output;
use crate::{
    data::managed::{Managed, Ref},
    memory::context::stack::Stack,
    private::Private,
};

/// A target that uses a reserved slot in a frame.
///
/// An `ReusableSlot` can be allocated with [`GcFrame::reusable_slot`]. When it's used as a target, the
/// returned data remains rooted until the scope this target belongs to ends.
///
/// Example:
///
/// ```
/// # use jlrs::prelude::*;
/// # use jlrs::util::test::JULIA;
/// # fn main() {
/// # JULIA.with(|j| {
/// # let mut julia = j.borrow_mut();
/// # let mut frame = StackFrame::new();
/// # let mut julia = julia.instance(&mut frame);
///
/// julia
///     .scope(|mut frame| {
///         let reusable_slot = frame.reusable_slot();
///
///         let _v = frame.scope(|_| {
///             // The reusableslot has been allocated in the parent
///             // scope's frame, so by using it as a target the
///             // result can be returned from this child scope.
///             Ok(Value::new(reusable_slot, 1u64))
///         })?;
///
///         Ok(())
///     })
///     .unwrap();
/// # });
/// # }
/// ```
///
/// A reusable slot can also be used to temporarily root data by using a mutable reference to a
/// `ReusableSlot` as a target. It's returned as a `Ref` because the lifetime is not tied to the
/// mutable borrow:
///
/// ```
/// # use jlrs::prelude::*;
/// # use jlrs::util::test::JULIA;
/// # fn main() {
/// # JULIA.with(|j| {
/// # let mut julia = j.borrow_mut();
/// # let mut frame = StackFrame::new();
/// # let mut julia = julia.instance(&mut frame);
///
/// julia
///     .scope(|mut frame| {
///         let mut reusable_slot = frame.reusable_slot();
///
///         let _v = frame.scope(|_| {
///             // _v1 can be used even after the slot has been used again, it's
///             // your responsibility that you don't use this data after the slot
///             // has been reused.
///             let _v1 = Value::new(&mut reusable_slot, 2u64);
///
///             Ok(Value::new(reusable_slot, 1u64))
///         })?;
///
///         Ok(())
///     })
///     .unwrap();
/// # });
/// # }
/// ```
///
/// [`GcFrame::reusable_slot`]: crate::memory::target::frame::GcFrame::reusable_slot
pub struct ReusableSlot<'target> {
    pub(crate) stack: &'target Stack,
    pub(crate) offset: usize,
}

impl<'scope> ReusableSlot<'scope> {
    pub(crate) unsafe fn consume<'data, T: Managed<'scope, 'data>>(
        self,
        ptr: NonNull<T::Wraps>,
    ) -> T {
        self.stack.set_root(self.offset, ptr.cast());
        T::wrap_non_null(ptr, Private)
    }

    pub(crate) unsafe fn temporary<'data, T: Managed<'scope, 'data>>(
        &mut self,
        ptr: NonNull<T::Wraps>,
    ) -> Ref<'scope, 'data, T> {
        self.stack.set_root(self.offset, ptr.cast());
        Ref::<T>::wrap(ptr)
    }

    pub(crate) fn into_output(self) -> Output<'scope> {
        Output {
            stack: self.stack,
            offset: self.offset,
        }
    }
}
