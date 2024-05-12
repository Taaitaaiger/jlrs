//! Generic `Tuple`s of different sizes.
//!
//! In this module generic tuple types from `Tuple0` up to and including `Tuple32` are available.
//! These types can be used to work with tuple values from Julia. A new tuple can be created with
//! `Value::new` if all fields implement the `IntoJulia` trait:
//!
//! ```
//! # use jlrs::prelude::*;
//! # fn main() {
//! # let mut julia = Builder::new().start_local().unwrap();
//! julia.local_scope::<_, 1>(|mut frame| {
//!     let tup = Tuple2(2i32, true);
//!     let val = Value::new(&mut frame, tup);
//!     assert!(val.is::<Tuple2<i32, bool>>());
//!     assert!(val.unbox::<Tuple2<i32, bool>>().is_ok());
//! });
//! # }
//! ```
//!
//! [`Tuple::new`] can be used to create a tuple from an arbitrary number of `Value`s.

use std::{marker::PhantomData, ptr::NonNull};

use jl_sys::{jl_apply_tuple_type_v, jlrs_tuple_of};

use crate::{
    catch::{catch_exceptions, unwrap_exc},
    data::{
        managed::{
            datatype::DataType,
            private::ManagedPriv as _,
            type_name::TypeName,
            value::{Value, ValueData, ValueResult},
        },
        types::{
            construct_type::{ConstructType, TypeVarEnv},
            typecheck::Typecheck,
        },
    },
    memory::target::{unrooted::Unrooted, Target},
    prelude::Managed,
    private::Private,
};

/// A tuple that has an arbitrary number of fields. This type can be used as a typecheck to check
/// if the data is a tuple type, and to create tuples of arbitrary sizes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tuple;

impl Tuple {
    /// Create a new tuple from the contents of `values`.
    pub fn new<'target, 'current, 'borrow, 'value, 'data, V, Tgt>(
        target: Tgt,
        values: V,
    ) -> ValueResult<'target, 'data, Tgt>
    where
        V: AsRef<[Value<'value, 'data>]>,
        Tgt: Target<'target>,
    {
        unsafe {
            let values = values.as_ref();
            let callback = || Self::new_unchecked(&target, values);

            match catch_exceptions(callback, unwrap_exc) {
                Ok(tup) => Ok(target.data_from_ptr(tup.ptr(), Private)),
                Err(err) => Err(target.data_from_ptr(err, Private)),
            }
        }
    }

    /// Create a new tuple from the contents of `values`.
    pub unsafe fn new_unchecked<'target, 'current, 'borrow, 'value, 'data, V, Tgt>(
        target: Tgt,
        values: V,
    ) -> ValueData<'target, 'data, Tgt>
    where
        V: AsRef<[Value<'value, 'data>]>,
        Tgt: Target<'target>,
    {
        let values = values.as_ref();
        let n = values.len();
        let values_ptr = values.as_ptr();
        let tuple: *mut jl_sys::jl_value_t = jlrs_tuple_of(values_ptr as *const _ as *mut _, n);

        target.data_from_ptr(NonNull::new_unchecked(tuple), Private)
    }
}

unsafe impl Typecheck for Tuple {
    #[inline]
    fn typecheck(t: DataType) -> bool {
        unsafe { t.type_name() == TypeName::of_tuple(&Unrooted::new()) }
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
    ($fieldtypes:expr, $unrooted:expr, $n:expr, $t:ident, $($x:ident),+) => {
        <$t>::valid_field($fieldtypes.get($unrooted, $n - 1 - count!($($x),+)).unwrap().as_managed()) && check!($fieldtypes, $unrooted, $n, $($x),+)
    };
    ($fieldtypes:expr, $unrooted:expr, $n:expr, $t:ident) => {
        <$t>::valid_field($fieldtypes.get($unrooted, $n - 1).unwrap().as_managed())
    };
}

macro_rules! impl_tuple {
    ($name:ident, $($types:tt),+) => {
        #[repr(C)]
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct $name<$($types),+>($(pub $types),+);

        impl<$($types),+> Copy for $name<$($types),+>
        where
            $($types: $crate::convert::into_julia::IntoJulia + ::std::fmt::Debug + Copy),+
        {}

        unsafe impl<$($types),+> $crate::data::layout::is_bits::IsBits for $name<$($types),+>
        where
            $($types: Clone + ::std::fmt::Debug + $crate::data::layout::is_bits::IsBits + $crate::data::layout::valid_layout::ValidField),+
        {}

        unsafe impl<$($types),+> $crate::convert::into_julia::IntoJulia for $name<$($types),+>
        where
            $($types: $crate::convert::into_julia::IntoJulia + $crate::data::types::construct_type::ConstructType + ::std::fmt::Debug + Clone),+
        {
            #[inline]
            fn julia_type<'scope, Tgt>(
                target: Tgt,
            ) -> $crate::data::managed::datatype::DataTypeData<'scope, Tgt>
            where
                Tgt: $crate::memory::target::Target<'scope>
            {
                unsafe {
                    <Self as $crate::data::types::construct_type::ConstructType>::construct_type(&target).as_value().cast_unchecked::<DataType>().root(target)
                }
            }
        }

        unsafe impl<$($types),+> $crate::data::layout::valid_layout::ValidLayout for $name<$($types),+>
        where
            $($types: $crate::data::layout::valid_layout::ValidField + Clone + ::std::fmt::Debug),+
        {
            fn valid_layout(v: $crate::data::managed::value::Value) -> bool {
                unsafe {
                    if v.is::<$crate::data::managed::datatype::DataType>() {
                        let unrooted = $crate::memory::target::unrooted::Unrooted::new();
                        let dt = v.cast_unchecked::<$crate::data::managed::datatype::DataType>();
                        let fieldtypes = dt.field_types();
                        let n = count!($($types),+);
                        if fieldtypes.len() != n {
                            return false;
                        }

                        let types = fieldtypes.data();
                        if !check!(types, unrooted, n, $($types),+) {
                            return false
                        }

                        return true
                    }

                    false
                }
            }

            #[inline]
            fn type_object<'target, Tgt>(
                _: &Tgt
            ) -> $crate::data::managed::value::Value<'target, 'static>
            where
                Tgt: $crate::memory::target::Target<'target>
            {
                unsafe {
                    <$crate::data::managed::value::Value as $crate::data::managed::private::ManagedPriv>::wrap_non_null(
                        std::ptr::NonNull::new_unchecked(jl_sys::jl_anytuple_type.cast()),
                        $crate::private::Private
                    )
                }
            }

            const IS_REF: bool = false;
        }

        unsafe impl<$($types),+> $crate::data::layout::valid_layout::ValidField for $name<$($types),+>
        where
            $($types: $crate::data::layout::valid_layout::ValidField + Clone + ::std::fmt::Debug),+
        {
            fn valid_field(v: $crate::data::managed::value::Value) -> bool {
                if v.is::<$crate::data::managed::datatype::DataType>() {
                    unsafe {
                        let unrooted = $crate::memory::target::unrooted::Unrooted::new();
                        let dt = v.cast_unchecked::<$crate::data::managed::datatype::DataType>();
                        let fieldtypes = dt.field_types();
                        let n = count!($($types),+);
                        if fieldtypes.len() != n {
                            return false;
                        }

                        let types = fieldtypes.data();
                        if !check!(types, unrooted, n, $($types),+) {
                            return false
                        }

                        return true
                    }
                }

                false
            }
        }

        unsafe impl<$($types),+> $crate::convert::unbox::Unbox for $name<$($types),+>
        where
            $($types: $crate::data::layout::valid_layout::ValidField + Clone + ::std::fmt::Debug),+
        {
            type Output = Self;
        }

        unsafe impl<$($types),+> $crate::data::types::typecheck::Typecheck for $name<$($types),+>
        where
            $($types: $crate::data::layout::valid_layout::ValidField + Clone + ::std::fmt::Debug),+
        {
            #[inline]
            fn typecheck(t: $crate::data::managed::datatype::DataType) -> bool {
                <Self as $crate::data::layout::valid_layout::ValidLayout>::valid_layout(t.as_value())
            }
        }

        unsafe impl<$($types),+> $crate::data::types::construct_type::ConstructType for $name<$($types),+>
        where
            $($types: $crate::data::types::construct_type::ConstructType),+
        {
            type Static = $name<$($types :: Static),*>;

            fn construct_type_uncached<'target, Tgt>(
                target: Tgt,
            ) -> $crate::data::managed::value::ValueData<'target, 'static, Tgt>
            where
                Tgt: $crate::memory::target::Target<'target>,
            {
                const N: usize = count!($($types),*);

                target.with_local_scope::<_, _, N>(|target, mut frame| {
                    let types = &mut [
                        $(<$types as $crate::data::types::construct_type::ConstructType>::construct_type(&mut frame)),+
                    ];

                    unsafe {
                        target.data_from_ptr(
                            ::std::ptr::NonNull::new_unchecked(::jl_sys::jl_apply_tuple_type_v(types.as_mut_ptr().cast(), types.len()).cast()),
                            $crate::private::Private,
                        )
                    }
                })
            }

            fn construct_type_with_env_uncached<'target, Tgt>(
                target: Tgt,
                env: &$crate::data::types::construct_type::TypeVarEnv,
            ) -> $crate::data::managed::value::ValueData<'target, 'static, Tgt>
            where
                Tgt: $crate::memory::target::Target<'target> {
                    const N: usize = count!($($types),*);

                    target.with_local_scope::<_, _, N>(|target, mut frame| {
                        let types = &mut [
                            $(<$types as $crate::data::types::construct_type::ConstructType>::construct_type_with_env(&mut frame, env)),+
                        ];

                        unsafe {
                            target.data_from_ptr(
                                ::std::ptr::NonNull::new_unchecked(::jl_sys::jl_apply_tuple_type_v(types.as_mut_ptr().cast(), types.len()).cast()),
                                $crate::private::Private,
                            )
                        }
                    })
            }

            #[inline]
            fn base_type<'target, Tgt>(target: &Tgt) -> Option<$crate::data::managed::value::Value<'target, 'static>>
            where
                Tgt: $crate::memory::target::Target<'target>,
            {
                Some($crate::data::managed::datatype::DataType::tuple_type(target).as_value())
            }
        }
    };
    ($name:ident) => {
        #[repr(C)]
        #[derive(Copy, Clone, Debug, PartialEq, Eq)]
        pub struct $name();

        unsafe impl $crate::data::layout::is_bits::IsBits for $name {}

        unsafe impl $crate::convert::into_julia::IntoJulia for $name
        {
            #[inline]
            fn julia_type<'scope, Tgt>(
                target: Tgt,
            ) -> $crate::data::managed::datatype::DataTypeData<'scope, Tgt>
            where
                Tgt: $crate::memory::target::Target<'scope>
            {
                unsafe {
                    let ptr = $crate::data::managed::datatype::DataType::emptytuple_type(&target).unwrap_non_null($crate::private::Private);
                    target.data_from_ptr(ptr, $crate::private::Private)
                }
            }

            #[inline]
            fn into_julia<'scope, Tgt>(self, target: Tgt) -> $crate::data::managed::value::ValueData<'scope, 'static, Tgt>
            where
                Tgt: $crate::memory::target::Target<'scope>,
            {
                unsafe {
                    let ptr = $crate::data::managed::value::Value::emptytuple(&target).unwrap_non_null($crate::private::Private);
                    target.data_from_ptr(ptr, $crate::private::Private)
                }
            }
        }

        unsafe impl $crate::data::layout::valid_layout::ValidLayout for $name {
            #[inline]
            fn valid_layout(v: $crate::data::managed::value::Value) -> bool {
                if v.is::<$crate::data::managed::datatype::DataType>() {
                    let dt = unsafe { v.cast_unchecked::<$crate::data::managed::datatype::DataType>() };
                    let global = unsafe {$crate::memory::target::unrooted::Unrooted::new()};
                    return dt == $crate::data::managed::datatype::DataType::emptytuple_type(&global)
                }

                false
            }

            #[inline]
            fn type_object<'target, Tgt>(
                _: &Tgt
            ) -> $crate::data::managed::value::Value<'target, 'static>
            where
                Tgt: $crate::memory::target::Target<'target>
            {
                unsafe {
                    <$crate::data::managed::value::Value as $crate::data::managed::private::ManagedPriv>::wrap_non_null(
                        std::ptr::NonNull::new_unchecked(jl_sys::jl_emptytuple_type.cast()),
                        $crate::private::Private
                    )
                }
            }

            const IS_REF: bool = false;
        }

        unsafe impl $crate::data::layout::valid_layout::ValidField for $name {
            #[inline]
            fn valid_field(v: $crate::data::managed::value::Value) -> bool {
                if v.is::<$crate::data::managed::datatype::DataType>() {
                    let dt = unsafe { v.cast_unchecked::<$crate::data::managed::datatype::DataType>() };
                    let global = unsafe {$crate::memory::target::unrooted::Unrooted::new()};
                    return dt == $crate::data::managed::datatype::DataType::emptytuple_type(&global)
                }

                false
            }
        }

        unsafe impl $crate::convert::unbox::Unbox for $name {
            type Output = Self;

            #[inline]
            unsafe fn unbox(_: $crate::data::managed::value::Value) -> Self::Output {
                Tuple0()
            }
        }

        unsafe impl $crate::data::types::typecheck::Typecheck for $name {
            #[inline]
            fn typecheck(t: $crate::data::managed::datatype::DataType) -> bool {
                <Self as $crate::data::layout::valid_layout::ValidLayout>::valid_layout(t.as_value())
            }
        }

        unsafe impl $crate::data::types::construct_type::ConstructType for $name {
            type Static = $name;

            const CACHEABLE: bool = false;

            #[inline]
            fn construct_type_uncached<'target, Tgt>(
                target: Tgt,
            ) -> $crate::data::managed::value::ValueData<'target, 'static, Tgt>
            where
                Tgt: $crate::memory::target::Target<'target>,
            {
                $crate::data::managed::datatype::DataType::emptytuple_type(&target).as_value().root(target)
            }

            fn construct_type_with_env_uncached<'target, Tgt>(
                target: Tgt,
                _env: &$crate::data::types::construct_type::TypeVarEnv,
            ) -> $crate::data::managed::value::ValueData<'target, 'static, Tgt>
            where
                Tgt: $crate::memory::target::Target<'target> {
                $crate::data::managed::datatype::DataType::emptytuple_type(&target).as_value().root(target)
            }

            #[inline]
            fn base_type<'target, Tgt>(target: &Tgt) -> Option<$crate::data::managed::value::Value<'target, 'static>>
            where
                Tgt: $crate::memory::target::Target<'target>,
            {
                Some($crate::data::managed::datatype::DataType::tuple_type(target).as_value())
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

pub struct NTuple<T, const N: usize> {
    _marker: PhantomData<[T; N]>,
}

unsafe impl<T: ConstructType, const N: usize> ConstructType for NTuple<T, N> {
    type Static = NTuple<T::Static, N>;

    fn construct_type_uncached<'target, Tgt>(target: Tgt) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            target.with_local_scope::<_, _, 1>(|target, mut frame| {
                let ty = T::construct_type(&mut frame);
                let types = [ty; N];
                let applied = jl_apply_tuple_type_v(&types as *const _ as *mut _, N);
                target.data_from_ptr(NonNull::new_unchecked(applied.cast()), Private)
            })
        }
    }

    fn construct_type_with_env_uncached<'target, Tgt>(
        target: Tgt,
        env: &TypeVarEnv,
    ) -> ValueData<'target, 'static, Tgt>
    where
        Tgt: Target<'target>,
    {
        unsafe {
            target.with_local_scope::<_, _, 1>(|target, mut frame| {
                let ty = T::construct_type_with_env(&mut frame, env);
                let types = [ty; N];
                let applied = jl_apply_tuple_type_v(&types as *const _ as *mut _, N);
                target.data_from_ptr(NonNull::new_unchecked(applied.cast()), Private)
            })
        }
    }

    fn base_type<'target, Tgt>(_target: &Tgt) -> Option<Value<'target, 'static>>
    where
        Tgt: Target<'target>,
    {
        None
    }
}
