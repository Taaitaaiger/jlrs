//! Generic `Tuple`s of different sizes.
//!
//! In this module generic tuple types from `Tuple0` up to and including `Tuple32` are available.
//! These types can be used to work with tuple values from Julia. A new tuple can be created with
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

use crate::wrappers::ptr::{
    datatype::DataType,
    value::{Value, MAX_SIZE},
};
use crate::wrappers::ptr::{private::Wrapper as _, Wrapper as _};
use crate::{
    convert::into_jlrs_result::IntoJlrsResult,
    error::JlrsResult,
    layout::typecheck::Typecheck,
    memory::{frame::Frame, scope::Scope},
    private::Private,
};
use jl_sys::jl_tuple_typename;

/// A typecheck that can be used in combination with `DataType::is`. This method returns true if
/// a value of this type is a tuple.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tuple;

impl Tuple {
    /// Create a new tuple from the contents of `values`.
    pub fn new<'target, 'current, 'value, 'borrow, V, S, F>(
        scope: S,
        mut values: V,
    ) -> JlrsResult<S::JuliaResult>
    where
        V: AsMut<[Value<'value, 'borrow>]>,
        S: Scope<'target, 'current, 'borrow, F>,
        F: Frame<'current>,
    {
        scope.result_scope(|output, frame| {
            let types: smallvec::SmallVec<[_; MAX_SIZE]> = values
                .as_mut()
                .iter()
                .copied()
                .map(|v| v.datatype().as_value())
                .collect();

            let tuple_ty = DataType::tuple_type(frame.global())
                .as_value()
                .apply_type(&mut *frame, types)?
                .into_jlrs_result()?
                .cast::<DataType>()?;

            let output = output.into_scope(frame);
            tuple_ty.instantiate(output, values)
        })
    }
}

unsafe impl Typecheck for Tuple {
    fn typecheck(t: DataType) -> bool {
        unsafe { t.unwrap_non_null(Private).as_ref().name == jl_tuple_typename }
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
        <$t>::valid_layout($fieldtypes[$n - 1 - count!($($x),+)].wrapper_unchecked()) && check!($fieldtypes, $n, $($x),+)
    };
    ($fieldtypes:expr, $n:expr, $t:ident) => {
        <$t>::valid_layout($fieldtypes[$n - 1].wrapper_unchecked())
    };
}

macro_rules! impl_tuple {
    ($name:ident, $($types:tt),+) => {
        #[repr(C)]
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $name<$($types: Clone + ::std::fmt::Debug),+>($(pub $types),+);

        impl<$($types),+> Copy for $name<$($types),+>
        where
            $($types: $crate::convert::into_julia::IntoJulia + ::std::fmt::Debug + Copy),+
        {}

        unsafe impl<$($types),+> $crate::convert::into_julia::IntoJulia for $name<$($types),+>
        where
            $($types: $crate::convert::into_julia::IntoJulia + ::std::fmt::Debug + Clone),+
        {
            fn julia_type<'scope>(
                global: $crate::memory::global::Global<'scope>
            ) -> $crate::wrappers::ptr::DataTypeRef<'scope> {
                let types = &mut [
                    $(<$types as $crate::convert::into_julia::IntoJulia>::julia_type(global)),+
                ];

                unsafe {
                    $crate::wrappers::ptr::DataTypeRef::wrap(
                        ::jl_sys::jl_apply_tuple_type_v(types.as_mut_ptr().cast(), types.len())
                    )
                }
            }
        }

        unsafe impl<$($types),+> $crate::layout::valid_layout::ValidLayout for $name<$($types),+>
        where
            $($types: $crate::layout::valid_layout::ValidLayout + Clone + ::std::fmt::Debug),+
        {
            fn valid_layout(v: $crate::wrappers::ptr::value::Value) -> bool {
                unsafe {
                    if let Ok(dt) = v.cast::<$crate::wrappers::ptr::datatype::DataType>() {
                        let fieldtypes = dt.field_types();
                        let n = count!($($types),+);
                        if fieldtypes.wrapper_unchecked().len() != n {
                            return false;
                        }

                        let types = fieldtypes.wrapper_unchecked().data();
                        if !check!(types, n, $($types),+) {
                            return false
                        }

                        return true
                    }

                    false
                }
            }
        }

        unsafe impl<$($types),+> $crate::convert::unbox::Unbox for $name<$($types),+>
        where
            $($types: $crate::layout::valid_layout::ValidLayout + Clone + ::std::fmt::Debug),+
        {
            type Output = Self;
        }

        unsafe impl<$($types),+> $crate::layout::typecheck::Typecheck for $name<$($types),+>
        where
            $($types: $crate::layout::valid_layout::ValidLayout + Clone + ::std::fmt::Debug),+
        {
            fn typecheck(t: $crate::wrappers::ptr::datatype::DataType) -> bool {
                <Self as $crate::layout::valid_layout::ValidLayout>::valid_layout(t.as_value())
            }
        }
    };
    ($name:ident) => {
        #[repr(C)]
        #[derive(Copy, Clone, Debug)]
        pub struct $name();

        unsafe impl $crate::convert::into_julia::IntoJulia for $name
        {
            fn julia_type<'scope>(
                global: $crate::memory::global::Global<'scope>
            ) -> $crate::wrappers::ptr::DataTypeRef<'scope> {
                $crate::wrappers::ptr::datatype::DataType::emptytuple_type(global).as_ref()
            }

            unsafe fn into_julia<'scope>(
                self,
                global: $crate::memory::global::Global<'scope>
            ) -> $crate::wrappers::ptr::ValueRef<'scope, 'static> {
                $crate::wrappers::ptr::value::Value::emptytuple(global).as_ref()
            }
        }

        unsafe impl $crate::layout::valid_layout::ValidLayout for $name {
            fn valid_layout(v: $crate::wrappers::ptr::value::Value) -> bool {
                if let Ok(dt) = v.cast::<$crate::wrappers::ptr::datatype::DataType>() {
                    let global = unsafe {$crate::memory::global::Global::new()};
                    return dt == $crate::wrappers::ptr::datatype::DataType::emptytuple_type(global)
                }

                false
            }
        }

        unsafe impl $crate::convert::unbox::Unbox for $name {
            type Output = Self;

            unsafe fn unbox(_: $crate::wrappers::ptr::value::Value) -> Self::Output {
                Tuple0()
            }
        }

        unsafe impl $crate::layout::typecheck::Typecheck for $name {
            fn typecheck(t: $crate::wrappers::ptr::datatype::DataType) -> bool {
                <Self as $crate::layout::valid_layout::ValidLayout>::valid_layout(t.as_value())
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
