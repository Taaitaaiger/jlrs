//! A target that uses a reserved slot in a frame.

use std::ptr::NonNull;

use crate::{data::managed::Managed, memory::context::stack::Stack, private::Private};

/// A target that uses a reserved slot in a frame.
///
/// An `Output` can be allocated with [`GcFrame::output`]. When it's used as a target, the
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
///         let output = frame.output();
///
///         let _v = frame.scope(|_| {
///             // The output has been allocated in the parent
///             // scope's frame, so by using it as a target the
///             // result can be returned from this child scope.
///             Ok(Value::new(output, 1u64))
///         })?;
///
///         Ok(())
///     })
///     .unwrap();
/// # });
/// # }
/// ```
///
/// An output can also be used to temporarily root data by using a mutable reference to an
/// `Output` as a target:
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
///         let mut output = frame.output();
///
///         let _v = frame.scope(|_| {
///             // _v1 can be used until the output is used again.
///             let _v1 = Value::new(&mut output, 2u64);
///
///             Ok(Value::new(output, 1u64))
///         })?;
///
///         Ok(())
///     })
///     .unwrap();
/// # });
/// # }
/// ```
///
/// [`GcFrame::output`]: crate::memory::target::frame::GcFrame::output
pub struct Output<'target> {
    pub(crate) stack: &'target Stack,
    pub(crate) offset: usize,
}

impl<'scope> Output<'scope> {
    pub(crate) unsafe fn consume<'data, T: Managed<'scope, 'data>>(
        self,
        ptr: NonNull<T::Wraps>,
    ) -> T {
        self.stack.set_root(self.offset, ptr.cast());
        T::wrap_non_null(ptr, Private)
    }

    pub(crate) unsafe fn temporary<'target, 'data, T: Managed<'target, 'data>>(
        &'target mut self,
        ptr: NonNull<T::Wraps>,
    ) -> T {
        self.stack.set_root(self.offset, ptr.cast());
        T::wrap_non_null(ptr, Private)
    }

    pub(crate) fn restrict<'target>(&'target mut self) -> Output<'target> {
        Output {
            stack: self.stack,
            offset: self.offset,
        }
    }
}
