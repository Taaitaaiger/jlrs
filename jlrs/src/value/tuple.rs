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
//! julia.scope(|global, frame| {
//!     let tup = Tuple2(2i32, true);
//!     let val = Value::new(frame, tup)?;
//!     assert!(val.is::<Tuple2<i32, bool>>());
//!     assert!(val.unbox::<Tuple2<i32, bool>>().is_ok());
//!     Ok(())
//! }).unwrap();
//! # });
//! # }
//! ```

use super::datatype::DataType;
use crate::layout::julia_typecheck::JuliaTypecheck;
use jl_sys::jl_tuple_typename;

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a tuple.
pub struct Tuple;

unsafe impl JuliaTypecheck for Tuple {
    unsafe fn julia_typecheck(t: DataType) -> bool {
        (&*t.inner().as_ptr()).name == jl_tuple_typename
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
        <$t>::valid_layout($fieldtypes[$n - 1 - count!($($x),+)].assume_reachable_unchecked()) && check!($fieldtypes, $n, $($x),+)
    };
    ($fieldtypes:expr, $n:expr, $t:ident) => {
        <$t>::valid_layout($fieldtypes[$n - 1].assume_reachable_unchecked())
    };
}

macro_rules! impl_tuple {
    ($name:ident, $($types:tt),+) => {
        #[repr(C)]
        #[derive(Copy, Clone, Debug, PartialEq)]
        pub struct $name<$($types),+>($(pub $types),+);

        unsafe impl<$($types),+> $crate::convert::into_julia::IntoJulia for $name<$($types),+>  where $($types: $crate::convert::into_julia::IntoJulia + Copy),+
        {
            unsafe fn julia_type() -> *mut $crate::jl_sys_export::jl_datatype_t {
                let types = &mut [$(<$types as $crate::convert::into_julia::IntoJulia>::julia_type()),+];
                $crate::jl_sys_export::jl_apply_tuple_type_v(types.as_mut_ptr().cast(), types.len())
            }
        }

        unsafe impl<$($types),+> $crate::layout::valid_layout::ValidLayout for $name<$($types),+>  where $($types: $crate::layout::valid_layout::ValidLayout + Copy),+ {
            unsafe fn valid_layout(v: $crate::value::Value) -> bool {
                if let Ok(dt) = v.cast::<$crate::value::datatype::DataType>() {
                    let fieldtypes = dt.field_types();
                    let n = count!($($types),+);
                    if fieldtypes.len() != n {
                        return false;
                    }

                    if !check!(fieldtypes.data(), n, $($types),+) {
                        return false
                    }

                    return true
                }

                false
            }
        }

        unsafe impl<$($types),+> $crate::convert::unbox::UnboxFn for $name<$($types),+>  where $($types: $crate::layout::valid_layout::ValidLayout + Copy),+ {
            type Output = Self;

            unsafe fn call_unboxer(value: $crate::value::Value) -> Self::Output {
                value.inner().as_ptr().cast::<Self::Output>().read()
            }
        }

        unsafe impl<$($types),+> $crate::layout::julia_typecheck::JuliaTypecheck for $name<$($types),+> where $($types: $crate::layout::valid_layout::ValidLayout + Copy),+ {
            unsafe fn julia_typecheck(t: $crate::value::datatype::DataType) -> bool {
                <Self as crate::layout::valid_layout::ValidLayout>::valid_layout(t.as_value())
            }
        }
    };
    ($name:ident) => {
        #[repr(C)]
        #[derive(Copy, Clone, Debug)]
        pub struct $name();

        unsafe impl $crate::convert::into_julia::IntoJulia for $name
        {
            unsafe fn julia_type() -> *mut $crate::jl_sys_export::jl_datatype_t {
                $crate::jl_sys_export::jl_emptytuple_type
            }

            unsafe fn into_julia(self) -> *mut $crate::jl_sys_export::jl_value_t {
                $crate::jl_sys_export::jl_emptytuple
            }
        }

        unsafe impl $crate::layout::valid_layout::ValidLayout for $name {
            unsafe fn valid_layout(v: $crate::value::Value) -> bool {
                if let Ok(dt) = v.cast::<$crate::value::datatype::DataType>() {
                    return dt.inner().as_ptr() == ::jl_sys::jl_emptytuple_type
                }

                false
            }
        }

        unsafe impl<'frame, 'data> $crate::convert::cast::Cast<'frame, 'data> for $name {
            type Output = Self;

            fn cast(value: $crate::value::Value) -> $crate::error::JlrsResult<Self::Output> {
                unsafe {
                    if <Self::Output as $crate::layout::valid_layout::ValidLayout>::valid_layout(value.datatype().as_value()) {
                        Ok(Self::cast_unchecked(value))
                    } else {
                        Err($crate::error::JlrsError::WrongType)?
                    }
                }
            }

            unsafe fn cast_unchecked(value: $crate::value::Value) -> Self::Output {
                *(value.inner().as_ptr() as *mut Self::Output)
            }
        }

        unsafe impl $crate::layout::julia_typecheck::JuliaTypecheck for $name {
            unsafe fn julia_typecheck(t: $crate::value::datatype::DataType) -> bool {
                <Self as crate::layout::valid_layout::ValidLayout>::valid_layout(t.as_value())
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
