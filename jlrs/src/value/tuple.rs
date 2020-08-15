//! Generic `Tuple`s of different sizes.
//!
//! In this module generic tuple types from `Tuple0` up to and including `Tuple32` are available.
//! These types can be use to work with tuple values from Julia. A new tuple can be created with
//! `Value::new` if all fields implement the `IntoJulia` trait:
//!
//! ```
//! # use jlrs::prelude::*;
//! # use jlrs::util::JULIA;
//! # fn main() {
//! # JULIA.with(|j| {
//! # let mut julia = j.borrow_mut();
//! julia.frame(2, |global, frame| {
//!     let tup = Tuple2(2i32, true);
//!     let val = Value::new(frame, tup)?;
//!     assert!(val.is::<Tuple2<i32, bool>>());
//!     assert!(val.cast::<Tuple2<i32, bool>>().is_ok());
//!     Ok(())
//! }).unwrap();
//! # });
//! # }
//! ```

use super::datatype::DataType;
use crate::traits::JuliaTypecheck;
use jl_sys::jl_tuple_typename;

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a tuple.
pub struct Tuple;

unsafe impl JuliaTypecheck for Tuple {
    unsafe fn julia_typecheck(t: DataType) -> bool {
        (&*t.ptr()).name == jl_tuple_typename
    }
}

macro_rules! count {
    ($t:ident, $($x:ident),+) => {
        1 + count!($($x),+)
    };
    ($t:ident) => {
        1
    };
}

macro_rules! check {
    ($fieldtypes:expr, $n:expr, $t:ident, $($x:ident),+) => {
        <$t>::valid_layout($fieldtypes[$n - 1 - count!($($x),+)]) && check!($fieldtypes, $n, $($x),+)
    };
    ($fieldtypes:expr, $n:expr, $t:ident) => {
        <$t>::valid_layout($fieldtypes[$n - 1])
    };
}

macro_rules! impl_tuple {
    ($name:ident, $($types:tt),+) => {
        #[repr(C)]
        #[derive(Copy, Clone, Debug, PartialEq)]
        pub struct $name<$($types),+>($(pub $types),+);

        unsafe impl<$($types),+> $crate::traits::JuliaType for $name<$($types),+> where $($types: $crate::traits::JuliaType),+
        {
            unsafe fn julia_type() -> *mut $crate::jl_sys_export::jl_datatype_t {
                let types = &mut [$(<$types as $crate::traits::JuliaType>::julia_type()),+];
                $crate::jl_sys_export::jl_apply_tuple_type_v(types.as_mut_ptr().cast(), types.len())
            }
        }

        unsafe impl<$($types),+> $crate::traits::IntoJulia for $name<$($types),+>  where $($types: $crate::traits::IntoJulia + $crate::traits::JuliaType + Copy),+
        {
            unsafe fn into_julia(&self) -> *mut $crate::jl_sys_export::jl_value_t {
                let ty = <Self as $crate::traits::JuliaType>::julia_type();
                let tuple = $crate::jl_sys_export::jl_new_struct_uninit(ty.cast());
                let data: *mut Self = tuple.cast();
                ::std::ptr::write(data, *self);

                tuple
            }
        }

        unsafe impl<$($types),+> $crate::traits::ValidLayout for $name<$($types),+>  where $($types: $crate::traits::ValidLayout + Copy),+ {
            unsafe fn valid_layout(v: $crate::value::Value) -> bool {
                if let Ok(dt) = v.cast::<$crate::value::datatype::DataType>() {
                    let fieldtypes = dt.field_types();
                    let n = count!($($types),+);
                    if fieldtypes.len() != n {
                        return false;
                    }

                    if !check!(fieldtypes, n, $($types),+) {
                        return false
                    }
                }

                true
            }
        }

        unsafe impl<'frame, 'data, $($types),+> $crate::traits::Cast<'frame, 'data> for $name<$($types),+>  where $($types: $crate::traits::ValidLayout + Copy),+ {
            type Output = Self;

            fn cast(value: $crate::value::Value) -> $crate::error::JlrsResult<Self::Output> {
                if value.is_nothing() {
                    Err($crate::error::JlrsError::Nothing)?;
                }

                unsafe {
                    if <Self::Output as $crate::traits::ValidLayout>::valid_layout(value.datatype().unwrap().into()) {
                        Ok(Self::cast_unchecked(value))
                    } else {
                        Err($crate::error::JlrsError::WrongType)?
                    }
                }
            }

            unsafe fn cast_unchecked(value: $crate::value::Value) -> Self::Output {
                *(value.ptr() as *mut Self::Output)
            }
        }

        unsafe impl<$($types),+> $crate::traits::JuliaTypecheck for $name<$($types),+> where $($types: $crate::traits::JuliaType),+ {
            unsafe fn julia_typecheck(t: $crate::value::datatype::DataType) -> bool {
                t.ptr() == <Self as $crate::traits::JuliaType>::julia_type()
            }
        }
    };
    ($name:ident) => {
        #[repr(C)]
        #[derive(Copy, Clone, Debug)]
        pub struct $name();

        unsafe impl $crate::value::JuliaType for $name
        {
            unsafe fn julia_type() -> *mut $crate::jl_sys_export::jl_datatype_t {
                $crate::jl_sys_export::jl_emptytuple_type
            }
        }

        unsafe impl $crate::value::IntoJulia for $name
        {
            unsafe fn into_julia(&self) -> *mut $crate::jl_sys_export::jl_value_t {
                $crate::jl_sys_export::jl_emptytuple
            }
        }

        unsafe impl $crate::traits::ValidLayout for $name {
            unsafe fn valid_layout(v: $crate::value::Value) -> bool {
                if let Ok(dt) = v.cast::<$crate::value::datatype::DataType>() {
                    if dt.is::<Self>() {
                        return true;
                    }
                }

                true
            }
        }

        unsafe impl<'frame, 'data> $crate::traits::Cast<'frame, 'data> for $name {
            type Output = Self;

            fn cast(value: $crate::value::Value) -> $crate::error::JlrsResult<Self::Output> {
                if value.is_nothing() {
                    Err($crate::error::JlrsError::Nothing)?;
                }

                unsafe {
                    if <Self::Output as $crate::traits::ValidLayout>::valid_layout(value.datatype().unwrap().into()) {
                        Ok(Self::cast_unchecked(value))
                    } else {
                        Err($crate::error::JlrsError::WrongType)?
                    }
                }
            }

            unsafe fn cast_unchecked(value: $crate::value::Value) -> Self::Output {
                *(value.ptr() as *mut Self::Output)
            }
        }

        unsafe impl $crate::traits::JuliaTypecheck for $name {
            unsafe fn julia_typecheck(t: $crate::value::datatype::DataType) -> bool {
                t.ptr() == <Self as $crate::traits::JuliaType>::julia_type()
            }
        }
    };
}

impl_tuple!(Tuple0);
impl_tuple!(Tuple1, T1);
impl_tuple!(Tuple2, T1, T2);
impl_tuple!(Tuple3, T1, T2, T3);
impl_tuple!(Tuple4, T1, T2, T3, T4);
impl_tuple!(Tuple5, T1, T2, T3, T4, T5);
impl_tuple!(Tuple6, T1, T2, T3, T4, T5, T6);
impl_tuple!(Tuple7, T1, T2, T3, T4, T5, T6, T7);
impl_tuple!(Tuple8, T1, T2, T3, T4, T5, T6, T7, T8);
impl_tuple!(Tuple9, T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_tuple!(Tuple10, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_tuple!(Tuple11, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_tuple!(Tuple12, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_tuple!(Tuple13, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_tuple!(Tuple14, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_tuple!(Tuple15, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
impl_tuple!(Tuple16, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);
impl_tuple!(Tuple17, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17);
impl_tuple!(
    Tuple18, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18
);
impl_tuple!(
    Tuple19, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19
);
impl_tuple!(
    Tuple20, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19,
    T20
);
impl_tuple!(
    Tuple21, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19,
    T20, T21
);
impl_tuple!(
    Tuple22, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19,
    T20, T21, T22
);
impl_tuple!(
    Tuple23, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19,
    T20, T21, T22, T23
);
impl_tuple!(
    Tuple24, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19,
    T20, T21, T22, T23, T24
);
impl_tuple!(
    Tuple25, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19,
    T20, T21, T22, T23, T24, T25
);
impl_tuple!(
    Tuple26, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19,
    T20, T21, T22, T23, T24, T25, T26
);
impl_tuple!(
    Tuple27, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19,
    T20, T21, T22, T23, T24, T25, T26, T27
);
impl_tuple!(
    Tuple28, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19,
    T20, T21, T22, T23, T24, T25, T26, T27, T28
);
impl_tuple!(
    Tuple29, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19,
    T20, T21, T22, T23, T24, T25, T26, T27, T28, T29
);
impl_tuple!(
    Tuple30, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19,
    T20, T21, T22, T23, T24, T25, T26, T27, T28, T29, T30
);
impl_tuple!(
    Tuple31, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19,
    T20, T21, T22, T23, T24, T25, T26, T27, T28, T29, T30, T31
);
impl_tuple!(
    Tuple32, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19,
    T20, T21, T22, T23, T24, T25, T26, T27, T28, T29, T30, T31, T32
);
