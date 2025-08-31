use jlrs::{
    data::{
        managed::value::typed::{TypedValue, TypedValueRet},
        types::abstract_type::{AbstractArray, AbstractFloat},
    },
    prelude::*,
    tvar, tvars,
};

#[repr(C)]
#[derive(
    Clone,
    Debug,
    Unbox,
    ValidLayout,
    Typecheck,
    ValidField,
    IsBits,
    ConstructType,
    CCallArg,
    CCallReturn,
)]
#[jlrs(julia_type = "Main.JuliaModuleTest.FourGenericsI")]
pub struct FourGenericsI<A, B, C, D> {
    pub a: A,
    pub b: B,
    pub c: C,
    pub d: D,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ConstructType)]
#[jlrs(julia_type = "Main.JuliaModuleTest.FourGenericsM")]
pub struct FourGenericsM<A, B, C, D> {
    pub a: A,
    pub b: B,
    pub c: C,
    pub d: D,
}

pub fn takes_four_generics_m(
    v: TypedValue<'_, 'static, FourGenericsM<i32, i32, i32, i32>>,
) -> TypedValueRet<FourGenericsM<i32, i32, i32, i32>> {
    v.as_weak().leak()
}

pub fn takes_four_generics_m_trailing1(
    v: TypedValue<'_, 'static, FourGenericsM<i32, i32, i32, tvar!('D')>>,
) -> TypedValueRet<FourGenericsM<i32, i32, i32, tvar!('D')>> {
    v.as_weak().leak()
}

pub fn takes_four_generics_m_trailing2(
    v: TypedValue<'_, 'static, FourGenericsM<i32, i32, tvar!('C'), tvar!('D')>>,
) -> TypedValueRet<FourGenericsM<i32, i32, tvar!('C'), tvar!('D')>> {
    v.as_weak().leak()
}

pub fn takes_four_generics_m_middle(
    v: TypedValue<'_, 'static, FourGenericsM<i32, tvar!('B'), i32, tvar!('D')>>,
) -> TypedValueRet<FourGenericsM<i32, tvar!('B'), i32, tvar!('D')>> {
    v.as_weak().leak()
}

pub fn takes_four_generics_m_start1(
    v: TypedValue<'_, 'static, FourGenericsM<tvar!('A'), i32, i32, i32>>,
) -> TypedValueRet<FourGenericsM<tvar!('A'), i32, i32, i32>> {
    v.as_weak().leak()
}

pub fn takes_four_generics_m_start2(
    v: TypedValue<'_, 'static, FourGenericsM<tvar!('A'), tvar!('B'), i32, i32>>,
) -> TypedValueRet<FourGenericsM<tvar!('A'), tvar!('B'), i32, i32>> {
    v.as_weak().leak()
}

pub fn takes_four_generics_i_trailing1(
    v: TypedValue<'_, 'static, FourGenericsI<i32, i32, i32, tvar!('D')>>,
) -> TypedValueRet<FourGenericsI<i32, i32, i32, tvar!('D')>> {
    v.as_weak().leak()
}

pub type GenericEnv = tvars!(
    tvar!('T'; AbstractFloat),
    tvar!('N'),
    tvar!('A'; AbstractArray<tvar!('T'), tvar!('N')>)
);
pub fn takes_generics_from_env(_array: TypedValue<tvar!('A')>, _data: TypedValue<tvar!('T')>) {}

pub fn has_generic<T>(t: T) -> T {
    t
}

pub fn has_two_generics<T, U>(t: T, _u: U) -> T {
    t
}
