use jlrs::{
    data::managed::value::typed::{TypedValue, TypedValueRet},
    prelude::*, tvar,
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

pub fn take_four_generics_m(
    v: TypedValue<'_, 'static, FourGenericsM<i32, i32, i32, i32>>,
) -> TypedValueRet<FourGenericsM<i32, i32, i32, i32>> {
    v.as_ref().leak()
}

pub fn take_four_generics_m_trailing1(
    v: TypedValue<'_, 'static, FourGenericsM<i32, i32, i32, tvar!('D')>>,
) -> TypedValueRet<FourGenericsM<i32, i32, i32, tvar!('D')>> {
    v.as_ref().leak()
}

pub fn take_four_generics_m_trailing2(
    v: TypedValue<'_, 'static, FourGenericsM<i32, i32, tvar!('C'), tvar!('D')>>,
) -> TypedValueRet<FourGenericsM<i32, i32, tvar!('C'), tvar!('D')>> {
    v.as_ref().leak()
}

pub fn take_four_generics_m_middle(
    v: TypedValue<'_, 'static, FourGenericsM<i32, tvar!('B'), i32, tvar!('D')>>,
) -> TypedValueRet<FourGenericsM<i32, tvar!('B'), i32, tvar!('D')>> {
    v.as_ref().leak()
}

pub fn take_four_generics_m_start1(
    v: TypedValue<'_, 'static, FourGenericsM<tvar!('A'), i32, i32, i32>>,
) -> TypedValueRet<FourGenericsM<tvar!('A'), i32, i32, i32>> {
    v.as_ref().leak()
}

pub fn take_four_generics_m_start2(
    v: TypedValue<'_, 'static, FourGenericsM<tvar!('A'), tvar!('B'), i32, i32>>,
) -> TypedValueRet<FourGenericsM<tvar!('A'), tvar!('B'), i32, i32>> {
    v.as_ref().leak()
}

pub fn take_four_generics_i_trailing1(
    v: TypedValue<'_, 'static, FourGenericsI<i32, i32, i32, tvar!('D')>>,
) -> TypedValueRet<FourGenericsI<i32, i32, i32, tvar!('D')>> {
    v.as_ref().leak()
}