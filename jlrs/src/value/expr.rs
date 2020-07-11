use super::Value;
use crate::error::{JlrsError, JlrsResult};
use crate::traits::Cast;
use crate::{impl_julia_type, impl_julia_typecheck};
use jl_sys::{jl_expr_t, jl_expr_type};
use std::marker::PhantomData;

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
