//! Support for values with the `Core.Expr` type.

use super::wrapper_ref::{ArrayRef, SymbolRef};
use super::Value;
use crate::convert::cast::Cast;
use crate::error::{JlrsError, JlrsResult};
use crate::{impl_julia_typecheck, impl_valid_layout};
use jl_sys::{jl_expr_t, jl_expr_type};
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    marker::PhantomData,
    ptr::NonNull,
};

/// A compound expression in Julia ASTs.
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct Expr<'frame>(NonNull<jl_expr_t>, PhantomData<&'frame ()>);

impl<'frame> Expr<'frame> {
    pub(crate) unsafe fn wrap(expr: *mut jl_expr_t) -> Self {
        debug_assert!(!expr.is_null());
        Expr(NonNull::new_unchecked(expr), PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn inner(self) -> NonNull<jl_expr_t> {
        self.0
    }

    /*
    for (a, b) in zip(fieldnames(Expr), fieldtypes(Expr))
        println(a, ": ", b)
    end
    head: Symbol
    args: Vector{Any}
    */

    /// Returns the head of the expression.
    pub fn head(self) -> SymbolRef<'frame> {
        unsafe { SymbolRef::wrap((&*self.inner().as_ptr()).head) }
    }

    /// Returns the arguments of the expression.
    pub fn args(self) -> ArrayRef<'frame, 'static> {
        unsafe { ArrayRef::wrap((&*self.inner().as_ptr()).args) }
    }

    /// Convert `self` to a `Value`.
    pub fn as_value(self) -> Value<'frame, 'static> {
        self.into()
    }
}

impl<'scope> Debug for Expr<'scope> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_tuple("Expr").finish()
    }
}

impl<'frame> Into<Value<'frame, 'static>> for Expr<'frame> {
    fn into(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap_non_null(self.inner().cast()) }
    }
}

unsafe impl<'frame, 'data> Cast<'frame, 'data> for Expr<'frame> {
    type Output = Self;
    fn cast(value: Value<'frame, 'data>) -> JlrsResult<Self::Output> {
        if value.is::<Self::Output>() {
            return unsafe { Ok(Self::cast_unchecked(value)) };
        }

        Err(JlrsError::NotAnExpr)?
    }

    unsafe fn cast_unchecked(value: Value<'frame, 'data>) -> Self::Output {
        Self::wrap(value.inner().as_ptr().cast())
    }
}

impl_julia_typecheck!(Expr<'frame>, jl_expr_type, 'frame);
impl_valid_layout!(Expr<'frame>, 'frame);
