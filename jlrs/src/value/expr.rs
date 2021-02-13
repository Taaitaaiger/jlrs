//! Support for values with the `Core.Expr` type.

use super::array::Array;
use super::symbol::Symbol;
use super::Value;
use crate::convert::cast::Cast;
use crate::error::{JlrsError, JlrsResult};
use crate::{impl_julia_type, impl_julia_typecheck, impl_valid_layout};
use jl_sys::{jl_expr_t, jl_expr_type};
use std::marker::PhantomData;

/// A compound expression in Julia ASTs.
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct Expr<'frame>(*mut jl_expr_t, PhantomData<&'frame ()>);

impl<'frame> Expr<'frame> {
    pub(crate) unsafe fn wrap(expr: *mut jl_expr_t) -> Self {
        Expr(expr, PhantomData)
    }

    #[doc(hidden)]
    pub unsafe fn ptr(self) -> *mut jl_expr_t {
        self.0
    }

    /// Returns the head of the expression.
    pub fn head(self) -> Symbol<'frame> {
        unsafe { Symbol::wrap((&*self.ptr()).head) }
    }

    /// Returns the arguments of the expression.
    pub fn args(self) -> Array<'frame, 'static> {
        unsafe { Array::wrap((&*self.ptr()).args) }
    }

    /// Convert `self` to a `Value`.
    pub fn as_value(self) -> Value<'frame, 'static> {
        self.into()
    }
}

impl<'frame> Into<Value<'frame, 'static>> for Expr<'frame> {
    fn into(self) -> Value<'frame, 'static> {
        unsafe { Value::wrap(self.ptr().cast()) }
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
        Self::wrap(value.ptr().cast())
    }
}

impl_julia_typecheck!(Expr<'frame>, jl_expr_type, 'frame);
impl_julia_type!(Expr<'frame>, jl_expr_type, 'frame);
impl_valid_layout!(Expr<'frame>, 'frame);
