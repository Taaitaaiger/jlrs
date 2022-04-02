//! Wrapper for `Expr`.

use super::super::private::Wrapper;
use crate::{impl_debug, impl_julia_typecheck, impl_valid_layout, memory::output::Output};
use crate::{
    private::Private,
    wrappers::ptr::{ArrayRef, SymbolRef},
};
use jl_sys::{jl_expr_t, jl_expr_type};
use std::{marker::PhantomData, ptr::NonNull};

/// A compound expression in Julia ASTs.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Expr<'scope>(NonNull<jl_expr_t>, PhantomData<&'scope ()>);

impl<'scope> Expr<'scope> {
    /*
    for (a, b) in zip(fieldnames(Expr), fieldtypes(Expr))
        println(a, ": ", b)
    end
    head: Symbol
    args: Vector{Any}
    */

    /// Returns the head of the expression.
    pub fn head(self) -> SymbolRef<'scope> {
        unsafe { SymbolRef::wrap(self.unwrap_non_null(Private).as_ref().head) }
    }

    /// Returns the arguments of the expression.
    pub fn args(self) -> ArrayRef<'scope, 'static> {
        unsafe { ArrayRef::wrap(self.unwrap_non_null(Private).as_ref().args) }
    }

    /// Use the `Output` to extend the lifetime of this data.
    pub fn root<'target>(self, output: Output<'target>) -> Expr<'target> {
        unsafe {
            let ptr = self.unwrap_non_null(Private);
            output.set_root::<Expr>(ptr);
            Expr::wrap_non_null(ptr, Private)
        }
    }
}

impl_julia_typecheck!(Expr<'scope>, jl_expr_type, 'scope);
impl_valid_layout!(Expr<'scope>, 'scope);
impl_debug!(Expr<'_>);

impl<'scope> Wrapper<'scope, '_> for Expr<'scope> {
    type Wraps = jl_expr_t;
    const NAME: &'static str = "Expr";

    #[inline(always)]
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    #[inline(always)]
    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

impl_root!(Expr, 1);
