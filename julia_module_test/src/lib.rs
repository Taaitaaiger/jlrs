use jlrs::{
    data::{
        managed::{
            array::{ArrayRet, RankedArrayRet, TypedArrayRet, TypedRankedArrayRet},
            ccall_ref::{CCallRef, CCallRefRet},
            value::{
                typed::{TypedValue, TypedValueRet},
                ValueRet,
            },
        },
        types::{
            abstract_type::{AnyType, Number},
            construct_type::{ArrayTypeConstructor, ConstantIsize},
        },
    },
    prelude::*,
    tvar, tvars,
};

pub mod array;
pub mod constants;
pub mod exceptions;
pub mod foreign;
pub mod generics;
pub mod isbits;
pub mod ref_types;
pub mod typed_value;

use array::*;
use constants::*;
use exceptions::*;
use foreign::*;
use generics::*;
use isbits::*;
use ref_types::*;
use typed_value::*;

julia_module! {
    become julia_module_tests_init_fn;

    fn takes_no_args_returns_nothing();
    fn takes_no_args_returns_usize() -> usize;

    fn takes_usize_returns_usize(a: usize) -> usize;
    fn takes_array(a: Array) -> usize;
    fn takes_ranked_array(a: RankedArray<1>) -> usize;
    fn takes_typed_array(a: TypedArray<u32>) -> usize;
    fn takes_typed_ranked_array(a: TypedRankedArray<u32, 1>) -> usize;
    fn takes_ref_usize(usize_ref: CCallRef<usize>) -> usize;
    fn takes_ref_any(value_ref: CCallRef<AnyType>) -> usize;
    fn takes_ref_module(module_ref: CCallRef<Module>) -> usize;
    fn takes_ref_number(value_ref: CCallRef<Number>) -> usize;
    fn takes_typed_value(a: TypedValue<usize>) -> usize;
    fn returns_array(dt: DataType) -> ArrayRet;
    fn returns_rank0_array(dt: DataType) -> RankedArrayRet<0>;
    fn returns_rank1_array(dt: DataType) -> RankedArrayRet<1>;
    fn returns_rank2_array(dt: DataType) -> RankedArrayRet<2>;
    fn returns_rank3_array(dt: DataType) -> RankedArrayRet<3>;
    fn returns_typed_array() -> TypedArrayRet<f32>;
    fn returns_typed_rank2_array() -> TypedRankedArrayRet<f32, 2>;
    fn returns_jlrs_result(throw_err: Bool) -> JlrsResult<i32>;
    fn returns_ref_bool() -> CCallRefRet<bool>;
    fn returns_typed_value() -> TypedValueRet<bool>;
    fn takes_generics_from_env(array: TypedValue<tvar!('A')>, data: TypedValue<tvar!('T')>) use GenericEnv;
    fn takes_generic_typed_ranked_arrays_ctor(
        a: TypedValue<ArrayTypeConstructor<tvar!('T'), ConstantIsize<1>>>,
        _b: TypedValue<ArrayTypeConstructor<tvar!('T'), ConstantIsize<2>>>,
    ) -> ValueRet use TEnv;

    fn takes_generic_typed_arrays_ctor(
        a: TypedValue<ArrayTypeConstructor<tvar!('T'), tvar!('M')>>,
        _b: TypedValue<ArrayTypeConstructor<tvar!('T'), tvar!('N')>>,
    ) -> ValueRet use TMNEnv;

    fn takes_generic_typed_ranked_arrays(
        a: TypedRankedArray<tvar!('U'), 1>,
        _b: TypedRankedArray<tvar!('U'), 2>,
    ) -> ValueRet use UEnv;

    fn takes_generic_typed_arrays(
        a: TypedArray<tvar!('U')>,
        _b: TypedArray<tvar!('U')>,
    ) -> ValueRet use UEnv;

    fn takes_and_returns_generic_typed_ranked_array(
        a: TypedRankedArray<'_, 'static, tvar!('U'), 1>,
    ) -> TypedRankedArrayRet<tvar!('U'), 1> use UEnv;

    fn takes_restricted_generic_typed_arrays(
        a: TypedArray<tvar!('T')>,
    ) -> ValueRet use RestrictedEnv;

    struct OpaqueInt;
    in OpaqueInt fn new(value: i32) -> TypedValueRet<OpaqueInt> as OpaqueInt;
    in OpaqueInt fn increment(&mut self) as increment!;
    #[untracked_self]
    in OpaqueInt fn increment(&mut self) as increment_unchecked!;

    #[untracked_self]
    #[gc_safe]
    in OpaqueInt fn increment(&mut self) as increment_unchecked_nogc!;
    in OpaqueInt fn get(&self) -> i32 as unbox_opaque;

    #[untracked_self]
    in OpaqueInt fn get(&self) -> i32 as unbox_opaque_untracked;
    in OpaqueInt fn get_cloned(self) -> i32;

    struct ForeignThing;
    in ForeignThing fn new(value: Value<'_, 'static>) -> TypedValueRet<ForeignThing> as ForeignThing;

    in ForeignThing fn get(&self) -> ValueRet as extract_inner;
    in ForeignThing fn set(&mut self, value: Value) as set_inner!;

    in UnexportedType fn assoc_func() -> isize;

    for T in [f64, f32, f64] {
        fn has_generic(t: T) -> T;

        struct POpaque<T>;

        in POpaque<T> fn new(value: T) -> TypedValueRet<POpaque<T>> as POpaque;
        in POpaque<T> fn get(&self) -> T as popaque_get;
        in POpaque<T> fn get_cloned(self) -> T as popaque_get_cloned;
        in POpaque<T> fn set(&mut self, value: T) as popaque_set;

        for U in [T, i32] {
            struct POpaqueTwo<T, U>;

            in POpaqueTwo<T, U> fn new(value: T, value2: U) -> TypedValueRet<POpaqueTwo<T, U>> as POpaqueTwo;
            in POpaqueTwo<T, U> fn get_v1(&self) -> T as get_v1;
            in POpaqueTwo<T, U> fn get_v2(&self) -> U as get_v2;

            fn has_two_generics<T, U>(t: T, u: U) -> T;
        }
    };

    fn takes_four_generics_m(v: TypedValue<'_, 'static, FourGenericsM<i32, i32, i32, i32>>) -> TypedValueRet<FourGenericsM<i32, i32, i32, i32>>;
    fn takes_four_generics_m_trailing1(
        v: TypedValue<'_, 'static, FourGenericsM<i32, i32, i32, tvar!('D')>>,
    ) -> TypedValueRet<FourGenericsM<i32, i32, i32, tvar!('D')>> use tvars!(tvar!('D'));
    fn takes_four_generics_m_trailing2(
        v: TypedValue<'_, 'static, FourGenericsM<i32, i32, tvar!('C'), tvar!('D')>>,
    ) -> TypedValueRet<FourGenericsM<i32, i32, tvar!('C'), tvar!('D')>> use tvars!(tvar!('C'), tvar!('D'));
    fn takes_four_generics_m_middle(
        v: TypedValue<'_, 'static, FourGenericsM<i32, tvar!('B'), i32, tvar!('D')>>,
    ) -> TypedValueRet<FourGenericsM<i32, tvar!('B'), i32, tvar!('D')>> use tvars!(tvar!('B'), tvar!('D'));
    fn takes_four_generics_m_start1(
        v: TypedValue<'_, 'static, FourGenericsM<tvar!('A'), i32, i32, i32>>,
    ) -> TypedValueRet<FourGenericsM<tvar!('A'), i32, i32, i32>>  use tvars!(tvar!('A'));
    fn takes_four_generics_m_start2(
        v: TypedValue<'_, 'static, FourGenericsM<tvar!('A'), tvar!('B'), i32, i32>>,
    ) -> TypedValueRet<FourGenericsM<tvar!('A'), tvar!('B'), i32, i32>>  use tvars!(tvar!('A'), tvar!('B'));

    fn takes_four_generics_i_trailing1(
        v: TypedValue<'_, 'static, FourGenericsI<i32, i32, i32, tvar!('D')>>,
    ) -> TypedValueRet<FourGenericsI<i32, i32, i32, tvar!('D')>> use tvars!(tvar!('D'));

    const CONST_U8: u8;
    static CONST_U8: u8 as STATIC_CONST_U8;
    const STATIC_U8: u8 as CONST_STATIC_U8;
    static STATIC_U8: u8;

    type POpaque64 = POpaque<f64>;
    in POpaque<f64> fn new(value: f64) -> TypedValueRet<POpaque<f64>> as POpaque64;
}
