//! Provide a handle with a dynamically-sized stack.

use super::IsActive;
use crate::{
    memory::{context::stack::Stack, target::frame::GcFrame},
    prelude::{LocalScope, Scope, Value},
    weak_handle_unchecked,
};

/// Provide a handle with a stack that can be used to create dynamically-sized scopes.
pub trait WithStack: IsActive {
    /// Allocate a dynamic stack and call `func`.
    fn with_stack<T, F>(&mut self, func: F) -> T
    where
        for<'ctx> F: FnOnce(StackHandle<'ctx>) -> T,
    {
        unsafe {
            weak_handle_unchecked!().local_scope::<_, 1>(|mut frame| {
                let stack = Value::new(&mut frame, Stack::default());
                func(StackHandle {
                    stack: stack.data_ptr().cast().as_ref(),
                })
            })
        }
    }
}

impl<H: IsActive> WithStack for H {}

/// A handle that can create dynamically-sized scopes.
///
/// `StackHandle` is the only implementor of [`Scope`].
#[derive(Debug)]
pub struct StackHandle<'ctx> {
    stack: &'ctx Stack,
}

impl<'ctx> IsActive for StackHandle<'ctx> {}

unsafe impl<'ctx> Scope for StackHandle<'ctx> {
    #[inline]
    fn scope<T>(&mut self, func: impl for<'scope> FnOnce(GcFrame<'scope>) -> T) -> T {
        unsafe {
            let frame = GcFrame::base(&self.stack);
            let ret = func(frame);
            self.stack.pop_roots(0);
            ret
        }
    }
}
