//! A reusable target that uses a reserved slot in a frame.

use std::ptr::NonNull;

use crate::{memory::context::stack::Stack, private::Private, wrappers::ptr::Wrapper};

/// A reusable target that uses a reserved slot in a frame.
///
/// A `Temporary` can be allocated with [`GcFrame::temporary`]. It can be used in two ways, either
/// as a mutable reference or as a value. In the first case, the data remains rooted as long as
/// the `Temporary` remains borrowed, in the second it behaves like an [`Output`] and the data
/// remains rooted until the scope this target belongs to ends.
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
/// # let julia = julia.instance(&mut frame);
///
///   julia.scope(|mut frame| {
///       let mut temporary = frame.temporary();
///
///       let _v1 = Value::new(&mut temporary, 0usize);
///       /*
///       ...
///       */
///  
///       let _v2 = Value::new(&mut temporary, 0usize);
///       // _v1 can no longer be used
///
///       let _v = frame.scope(|_| {
///           // _v2 can no longer be used after this, the
///           // returned data is guaranteed to remain
///           // rooted until the parent scope ends
///           Ok(Value::new(temporary, 1u64))
///       }).unwrap();
///
///       Ok(())
///   }).unwrap();
/// # });
/// # }
/// ```
///
/// [`GcFrame::temporary`]: crate::memory::target::frame::GcFrame::temporary
/// [`Output`]: crate::memory::target::output::Output
pub struct Temporary<'scope> {
    pub(crate) stack: &'scope Stack,
    pub(crate) offset: usize,
}

impl<'scope> Temporary<'scope> {
    pub(crate) unsafe fn root<'target, 'data, T: Wrapper<'target, 'data>>(
        &'target mut self,
        ptr: NonNull<T::Wraps>,
    ) -> T {
        self.stack.set_root(self.offset, ptr.cast());
        T::wrap_non_null(ptr, Private)
    }

    pub(crate) unsafe fn consume<'data, T: Wrapper<'scope, 'data>>(
        self,
        ptr: NonNull<T::Wraps>,
    ) -> T {
        self.stack.set_root(self.offset, ptr.cast());
        T::wrap_non_null(ptr, Private)
    }
}
