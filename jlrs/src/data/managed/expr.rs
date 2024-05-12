//! Managed type for `Expr`.

use std::{marker::PhantomData, ptr::NonNull};

use jl_sys::{
    jl_expr_t, jl_expr_type, jlrs_expr_head, jlrs_expr_nargs, jlrs_exprarg, jlrs_exprargset,
};

use super::{
    value::{Value, ValueData},
    Managed,
};
use crate::{
    data::managed::{private::ManagedPriv, symbol::Symbol, Ref},
    impl_julia_typecheck,
    memory::target::{TargetResult, TargetType},
    prelude::Target,
    private::Private,
};

/// A compound expression in Julia ASTs.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Expr<'scope>(NonNull<jl_expr_t>, PhantomData<&'scope ()>);

impl<'scope> Expr<'scope> {
    /// Returns the head of the expression.
    pub fn head(self) -> Option<Symbol<'scope>> {
        // Safety: the pointer points to valid data
        unsafe {
            let head = jlrs_expr_head(self.unwrap(Private));
            let head = NonNull::new(head)?;
            Some(Symbol::wrap_non_null(head, Private))
        }
    }

    /// Returns the number of arguments of the expression.
    pub fn n_args(self) -> usize {
        unsafe { jlrs_expr_nargs(self.unwrap(Private)) }
    }

    /// Returns the argument at `index` of the expression.
    pub fn arg<'target, Tgt>(
        self,
        target: Tgt,
        index: usize,
    ) -> Option<ValueData<'target, 'static, Tgt>>
    where
        Tgt: Target<'target>,
    {
        // Safety: the pointer points to valid data
        unsafe {
            let n_args = self.n_args();
            if index >= n_args {
                return None;
            }

            let arg = jlrs_exprarg(self.unwrap(Private), index);
            let arg = NonNull::new(arg)?;
            Some(Value::wrap_non_null(arg, Private).root(target))
        }
    }

    /// Sets the argument at position `index` to `data`.
    pub unsafe fn set_arg(self, index: usize, data: Option<Value<'_, 'static>>) {
        unsafe { jlrs_exprargset(self.unwrap(Private), index, std::mem::transmute(data)) }
    }
}

impl_julia_typecheck!(Expr<'scope>, jl_expr_type, 'scope);
impl_debug!(Expr<'_>);

impl<'scope> ManagedPriv<'scope, '_> for Expr<'scope> {
    type Wraps = jl_expr_t;
    type WithLifetimes<'target, 'da> = Expr<'target>;
    const NAME: &'static str = "Expr";

    // Safety: `inner` must not have been freed yet, the result must never be
    // used after the GC might have freed it.
    #[inline]
    unsafe fn wrap_non_null(inner: NonNull<Self::Wraps>, _: Private) -> Self {
        Self(inner, PhantomData)
    }

    #[inline]
    fn unwrap_non_null(self, _: Private) -> NonNull<Self::Wraps> {
        self.0
    }
}

impl_construct_type_managed!(Expr, 1, jl_expr_type);

/// A reference to an [`Expr`] that has not been explicitly rooted.
pub type ExprRef<'scope> = Ref<'scope, 'static, Expr<'scope>>;

/// An [`ExprRef`] with static lifetimes. This is a useful shorthand for signatures of
/// `ccall`able functions that return a [`Expr`].
pub type ExprRet = Ref<'static, 'static, Expr<'static>>;

impl_valid_layout!(ExprRef, Expr, jl_expr_type);

/// `Expr` or `ExprRef`, depending on the target type `Tgt`.
pub type ExprData<'target, Tgt> = <Tgt as TargetType<'target>>::Data<'static, Expr<'target>>;

/// `JuliaResult<Expr>` or `JuliaResultRef<ExprRef>`, depending on the target type `Tgt`.
pub type ExprResult<'target, Tgt> = TargetResult<'target, 'static, Expr<'target>, Tgt>;

impl_ccall_arg_managed!(Expr, 1);
impl_into_typed!(Expr);
