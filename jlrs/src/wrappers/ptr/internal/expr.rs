//! Wrapper for `Expr`.

use crate::{
    impl_julia_typecheck,
    prelude::Symbol,
    private::Private,
    wrappers::ptr::{array::ArrayRef, private::WrapperPriv, Ref},
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
    pub fn head(self) -> Option<Symbol<'scope>> {
        // Safety: the pointer points to valid data
        unsafe {
            let head = self.unwrap_non_null(Private).as_ref().head;
            let head = NonNull::new(head)?;
            Some(Symbol::wrap_non_null(head, Private))
        }
    }

    /// Returns the arguments of the expression.
    pub fn args(self) -> Option<ArrayRef<'scope, 'static>> {
        // Safety: the pointer points to valid data
        unsafe {
            let args = self.unwrap_non_null(Private).as_ref().args;
            let args = NonNull::new(args)?;
            Some(ArrayRef::wrap(args))
        }
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

/// A reference to an [`Expr`] that has not been explicitly rooted.
pub type ExprRef<'scope> = Ref<'scope, 'static, Expr<'scope>>;
impl_valid_layout!(ExprRef, Expr);
impl_ref_root!(Expr, ExprRef, 1);

use crate::memory::target::target_type::TargetType;

/// `Expr` or `ExprRef`, depending on the target type `T`.
pub type ExprData<'target, T> = <T as TargetType<'target>>::Data<'static, Expr<'target>>;

/// `JuliaResult<Expr>` or `JuliaResultRef<ExprRef>`, depending on the target type `T`.
pub type ExprResult<'target, T> = <T as TargetType<'target>>::Result<'static, Expr<'target>>;
