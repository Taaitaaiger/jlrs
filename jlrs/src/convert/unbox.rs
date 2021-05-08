#![allow(unused_imports)]
use std::{ffi::c_void, marker::PhantomData, ptr::NonNull};

use jl_sys::{
    jl_unbox_float32, jl_unbox_float64, jl_unbox_int16, jl_unbox_int32, jl_unbox_int64,
    jl_unbox_int8, jl_unbox_uint16, jl_unbox_uint32, jl_unbox_uint64, jl_unbox_uint8,
    jl_unbox_voidpointer,
};

use crate::{
    error::JlrsError,
    value::{
        datatype::{Immutable, Mutable},
        Value,
    },
};
use crate::{error::JlrsResult, private::Private};

use self::private::Unbox as _;

/// Converting a value from Julia to Rust is called unboxing. Types that can be converted from
/// Julia to Rust must implement this trait. By default, it's implemented for primitive types and
/// `String`. You should not implement it manually for custom types, but use `JlrsReflect.jl`
/// and `jlrs-derive` instead.
pub unsafe trait UnboxFn {
    type Output;
    #[doc(hidden)]
    unsafe fn call_unboxer(value: Value) -> Self::Output;
}

pub trait Unbox<T: UnboxFn>: private::Unbox<T> {}
impl<'scope, 'data, T: UnboxFn> Unbox<T> for Value<'scope, 'data> {}

mod private {
    use super::UnboxFn;
    use crate::private::Private;
    use crate::value::Value;

    pub unsafe trait Unbox<T: UnboxFn> {
        #[doc(hidden)]
        unsafe fn unbox(self, _: Private) -> T::Output;
    }

    unsafe impl<'scope, 'data, T> Unbox<T> for Value<'scope, 'data>
    where
        T: UnboxFn,
    {
        unsafe fn unbox(self, _: Private) -> T::Output {
            T::call_unboxer(self)
        }
    }
}

macro_rules! impl_unboxer {
    ($type:ty, $unboxer:expr) => {
        unsafe impl UnboxFn for $type {
            type Output = Self;
            unsafe fn call_unboxer(value: Value) -> $type {
                $unboxer(value.inner().as_ptr()) as $type
            }
        }
    };
}

impl_unboxer!(u8, jl_unbox_uint8);
impl_unboxer!(u16, jl_unbox_uint16);
impl_unboxer!(u32, jl_unbox_uint32);
impl_unboxer!(u64, jl_unbox_uint64);
impl_unboxer!(i8, jl_unbox_int8);
impl_unboxer!(i16, jl_unbox_int16);
impl_unboxer!(i32, jl_unbox_int32);
impl_unboxer!(i64, jl_unbox_int64);
impl_unboxer!(f32, jl_unbox_float32);
impl_unboxer!(f64, jl_unbox_float64);
impl_unboxer!(*mut c_void, jl_unbox_voidpointer);

#[cfg(not(target_pointer_width = "64"))]
impl_unboxer!(usize, jl_unbox_uint32);

#[cfg(not(target_pointer_width = "64"))]
impl_unboxer!(isize, jl_unbox_int32);

#[cfg(target_pointer_width = "64")]
impl_unboxer!(usize, jl_unbox_uint64);

#[cfg(target_pointer_width = "64")]
impl_unboxer!(isize, jl_unbox_int64);

unsafe impl UnboxFn for bool {
    type Output = Self;

    unsafe fn call_unboxer(value: Value) -> Self::Output {
        jl_unbox_int8(value.inner().as_ptr()) != 0
    }
}
