use crate::error::JlrsResult;
use crate::prelude::*;
use crate::traits::{Cast, IntoJulia, JuliaType, JuliaTypecheck};
use crate::value::datatype::Concrete;
use jl_sys::{
    jl_any_type, jl_apply_tuple_type_v, jl_array_type, jl_array_typename, jl_datatype_t,
    jl_emptytuple, jl_emptytuple_type, jl_new_struct_uninit, jl_value_t,
};
use std::mem;

pub unsafe trait JuliaFieldtype {
    unsafe fn field_type() -> *mut jl_value_t;
}

unsafe impl<'frame, 'data> JuliaFieldtype for Array<'frame, 'data> {
    unsafe fn field_type() -> *mut jl_value_t {
        jl_array_typename.cast()
    }
}

unsafe impl<T: JuliaType> JuliaFieldtype for T {
    unsafe fn field_type() -> *mut jl_value_t {
        Self::julia_type().cast()
    }
}

macro_rules! impl_tuple {
    ($name:ident, $($types:tt),+) => {
        #[repr(C)]
        #[derive(Copy, Clone, Debug)]
        pub struct $name<$($types),+>($($types),+);

        unsafe impl<$($types),+> JuliaType for $name<$($types),+> where $($types: JuliaType),+
        {
            unsafe fn julia_type() -> *mut jl_datatype_t {
                let types = &mut [$($types::julia_type()),+];
                jl_apply_tuple_type_v(types.as_mut_ptr().cast(), types.len())
            }
        }

        unsafe impl<$($types),+> IntoJulia for $name<$($types),+>  where $($types: IntoJulia + JuliaType + Copy),+
        {
            unsafe fn into_julia(&self) -> *mut jl_value_t {
                let ty = Self::julia_type();
                let tuple = jl_new_struct_uninit(ty.cast());
                let data: *mut Self = tuple.cast();
                ::std::ptr::write(data, *self);

                tuple
            }
        }
    };
    ($name:ident) => {
        #[repr(C)]
        #[derive(Copy, Clone, Debug)]
        pub struct $name();

        unsafe impl JuliaType for $name
        {
            unsafe fn julia_type() -> *mut jl_datatype_t {
                jl_emptytuple_type
            }
        }

        unsafe impl IntoJulia for $name
        {
            unsafe fn into_julia(&self) -> *mut jl_value_t {
                jl_emptytuple
            }
        }
    };
}

impl_tuple!(Tuple0);
impl_tuple!(Tuple1, T1);
impl_tuple!(Tuple2, T1, T2);

unsafe impl<T1, T2> JuliaTypecheck for Tuple2<T1, T2>
where
    T1: JuliaFieldtype,
    T2: JuliaFieldtype,
{
    unsafe fn julia_typecheck(t: DataType) -> bool {
        /*        if !t.is::<Tuple>() {
                    return false;
                }

                if t.size() != mem::size_of::<Self>() as i32 {
                    return false;
                }

                if t.nfields() != 2 {
                    return false;
                }

                let types = t.field_types();

                if let Ok(dt) = types[0].cast::<DataType>() {
                    if dt.is::<Array>() && T1::field_type() != jl_array_typename {
                        return false
                    } else if T1::field_type().cast() != dt.ptr() {
                        return false;
                    }
                } else if types[0].is::<UnionAll>() {
                    if T1::julia_type().cast() != jl_any_type {
                        return false;
                    }
                } else if types[0].is::<UnionType>() {
                } else {
                }
        */
        true
    }
}

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
