//! Wrapper for `Expr`.

use crate::{
    impl_julia_typecheck,
    memory::target::Target,
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

    /// Use the target to reroot this data.
    pub fn root<'target, T>(self, target: T) -> ExprData<'target, T>
    where
        T: Target<'target>,
    {
        // Safety: the data is valid.
        unsafe { target.data_from_ptr(self.unwrap_non_null(Private), Private) }
    }
}

impl_julia_typecheck!(Expr<'scope>, jl_expr_type, 'scope);
impl_debug!(Expr<'_>);

impl<'scope> WrapperPriv<'scope, '_> for Expr<'scope> {
    type Wraps = jl_expr_t;
    type TypeConstructorPriv<'target, 'da> = Expr<'target>;
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

use crate::memory::target::target_type::TargetType;
pub type ExprData<'target, T> = <T as TargetType<'target>>::Data<'static, Expr<'target>>;
pub type ExprResult<'target, T> = <T as TargetType<'target>>::Result<'static, Expr<'target>>;
