//! Wrapper for `Expr`.

use crate::{
    impl_debug, impl_julia_typecheck,
    memory::output::Output,
    private::Private,
    wrappers::ptr::{array::ArrayRef, private::WrapperPriv, symbol::SymbolRef, Ref},
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
        // Safety: the pointer points to valid data
        unsafe { SymbolRef::wrap(self.unwrap_non_null(Private).as_ref().head) }
    }

    /// Returns the arguments of the expression.
    pub fn args(self) -> ArrayRef<'scope, 'static> {
        // Safety: the pointer points to valid data
        unsafe { ArrayRef::wrap(self.unwrap_non_null(Private).as_ref().args) }
    }

    /// Use the `Output` to extend the lifetime of this data.
    pub fn root<'target>(self, output: Output<'target>) -> Expr<'target> {
        // Safety: the pointer points to valid data
        unsafe {
            let ptr = self.unwrap_non_null(Private);
            output.set_root::<Expr>(ptr);
            Expr::wrap_non_null(ptr, Private)
        }
    }
}

impl_julia_typecheck!(Expr<'scope>, jl_expr_type, 'scope);
impl_debug!(Expr<'_>);

impl<'scope> WrapperPriv<'scope, '_> for Expr<'scope> {
    type Wraps = jl_expr_t;
    const NAME: &'static str = "Expr";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

impl_root!(Expr, 1);

/// A reference to an [`Expr`] that has not been explicitly rooted.
pub type ExprRef<'scope> = Ref<'scope, 'static, Expr<'scope>>;
impl_valid_layout!(ExprRef, Expr);
impl_ref_root!(Expr, ExprRef, 1);
