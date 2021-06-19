//! Wrapper for `Core.Expr`.

use super::private::Wrapper;
use crate::{impl_debug, impl_julia_typecheck, impl_valid_layout};
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
}

impl_julia_typecheck!(Expr<'scope>, jl_expr_type, 'scope);
impl_valid_layout!(Expr<'scope>, 'scope);
impl_debug!(Expr<'_>);

impl<'scope> Wrapper<'scope, '_> for Expr<'scope> {
    type Internal = jl_expr_t;
    const NAME: &'static str = "Expr";

    unsafe fn wrap_non_null(inner: NonNull<Self::Internal>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    unsafe fn unwrap_non_null(self, _: Private) -> NonNull<Self::Internal> {
        self.0
    }
}
