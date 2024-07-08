use std::ops::AddAssign;

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
            abstract_type::{AbstractArray, AbstractFloat, AnyType, Number},
            construct_type::{ArrayTypeConstructor, ConstantIsize, ConstructType},
            foreign_type::{ForeignType, OpaqueType, ParametricBase, ParametricVariant},
        },
    },
    impl_type_parameters, impl_variant_parameters,
    memory::gc::{mark_queue_obj, write_barrier},
    prelude::*,
    tvar, tvars,
};

pub mod generics;
use generics::*;

fn freestanding_func_trivial() {}

fn freestanding_func_noargs() -> usize {
    0
}

fn freestanding_func_bitsarg(a: usize) -> usize {
    a + 1
}

fn freestanding_func_ref_bitsarg(usize_ref: CCallRef<usize>) -> usize {
    usize_ref.as_ref().unwrap() + 1
}

fn freestanding_func_arrayarg(a: Array) -> usize {
    let elty = a.element_type();

    if let Ok(elty) = elty.cast::<DataType>() {
        elty.size().unwrap_or(std::mem::size_of::<usize>() as _) as _
    } else {
        std::mem::size_of::<usize>()
    }
}

fn freestanding_func_ranked_arrayarg(a: RankedArray<1>) -> usize {
    let elty = a.element_type();

    if let Ok(elty) = elty.cast::<DataType>() {
        elty.size().unwrap_or(std::mem::size_of::<usize>() as _) as _
    } else {
        std::mem::size_of::<usize>()
    }
}

fn freestanding_func_typed_arrayarg(a: TypedArray<u32>) -> usize {
    let elty = a.element_type();

    if let Ok(elty) = elty.cast::<DataType>() {
        elty.size().unwrap_or(std::mem::size_of::<usize>() as _) as _
    } else {
        std::mem::size_of::<usize>()
    }
}

fn freestanding_func_ref_mutarg(module_ref: CCallRef<Module>) -> usize {
    let _module = module_ref.as_managed().unwrap();
    0
}

fn freestanding_func_ref_any(value_ref: CCallRef<AnyType>) -> usize {
    let _dt = value_ref.as_value_ref().datatype();
    0
}

fn freestanding_func_ref_abstract(value_ref: CCallRef<Number>) -> usize {
    let _dt = value_ref.as_value().unwrap().datatype();
    0
}

unsafe fn freestanding_func_typevaluearg(a: TypedValue<usize>) -> usize {
    a.unbox_unchecked::<usize>()
}

unsafe fn freestanding_func_ret_array(dt: DataType) -> ArrayRet {
    CCall::stackless_invoke(|unrooted| {
        Array::new_for(unrooted, dt.as_value(), (2, 2))
            .unwrap()
            .leak()
    })
}

unsafe fn freestanding_func_ret_ranked_array0(dt: DataType) -> RankedArrayRet<0> {
    CCall::stackless_invoke(|unrooted| {
        RankedArray::<0>::new_for(unrooted, dt.as_value(), ())
            .unwrap()
            .leak()
    })
}

unsafe fn freestanding_func_ret_ranked_array1(dt: DataType) -> RankedArrayRet<1> {
    CCall::stackless_invoke(|unrooted| {
        RankedArray::<1>::new_for(unrooted, dt.as_value(), (2,))
            .unwrap()
            .leak()
    })
}

unsafe fn freestanding_func_ret_ranked_array2(dt: DataType) -> RankedArrayRet<2> {
    CCall::stackless_invoke(|unrooted| {
        RankedArray::<2>::new_for(unrooted, dt.as_value(), (2, 2))
            .unwrap()
            .leak()
    })
}

unsafe fn freestanding_func_ret_ranked_array3(dt: DataType) -> RankedArrayRet<3> {
    CCall::stackless_invoke(|unrooted| {
        RankedArray::<3>::new_for(unrooted, dt.as_value(), (2, 2, 2))
            .unwrap()
            .leak()
    })
}

unsafe fn freestanding_func_ret_typed_array() -> TypedArrayRet<f32> {
    CCall::stackless_invoke(|unrooted| TypedArray::<f32>::new(unrooted, (2, 2)).unwrap().leak())
}

unsafe fn freestanding_func_ccall_ref_ret() -> CCallRefRet<bool> {
    CCall::stackless_invoke(|unrooted| {
        let v = Value::true_v(&unrooted)
            .as_typed::<bool, _>(&unrooted)
            .unwrap()
            .leak();
        CCallRefRet::new(v)
    })
}

unsafe fn freestanding_func_typed_value_ret() -> TypedValueRet<bool> {
    CCall::stackless_invoke(|unrooted| {
        Value::true_v(&unrooted)
            .as_typed::<bool, _>(&unrooted)
            .unwrap()
            .leak()
    })
}

fn freestanding_func_ret_rust_result(throw_err: Bool) -> JlrsResult<i32> {
    if throw_err.as_bool() {
        Err(jlrs::error::JlrsError::exception("Error"))?
    } else {
        Ok(3)
    }
}

#[derive(Clone, Debug)]
struct OpaqueInt {
    a: i32,
}

unsafe impl OpaqueType for OpaqueInt {}

impl OpaqueInt {
    fn new(value: i32) -> TypedValueRet<OpaqueInt> {
        unsafe {
            CCall::stackless_invoke(|unrooted| {
                TypedValue::new(unrooted, OpaqueInt { a: value }).leak()
            })
        }
    }

    fn increment(&mut self) {
        self.a += 1;
    }

    fn get(&self) -> i32 {
        self.a
    }

    fn get_cloned(self) -> i32 {
        self.a
    }
}

#[derive(Clone)]
pub struct POpaque<T> {
    value: T,
}

impl<T> POpaque<T>
where
    T: 'static + Send + ConstructType + AddAssign + Copy + jlrs::convert::into_julia::IntoJulia,
{
    fn new(value: T) -> TypedValueRet<Self> {
        unsafe {
            CCall::invoke(|mut frame| {
                let data = POpaque { value };
                TypedValue::new(&mut frame, data).leak()
            })
        }
    }

    fn get(&self) -> T {
        self.value
    }

    fn get_cloned(self) -> T {
        self.value
    }

    fn set(&mut self, value: T) {
        self.value = value;
    }
}

unsafe impl<T> ParametricBase for POpaque<T>
where
    T: 'static + Send + ConstructType,
{
    type Key = POpaque<()>;
    impl_type_parameters!('T');
}

unsafe impl<T: 'static + Send + ConstructType> ParametricVariant for POpaque<T> {
    impl_variant_parameters!(T);
}

#[derive(Clone)]
pub struct POpaqueTwo<T, U> {
    value: T,
    value2: U,
}

impl<T, U> POpaqueTwo<T, U>
where
    T: 'static + Send + ConstructType + AddAssign + Copy + jlrs::convert::into_julia::IntoJulia,
    U: 'static + Send + ConstructType + AddAssign + Copy + jlrs::convert::into_julia::IntoJulia,
{
    fn new(value: T, value2: U) -> TypedValueRet<Self> {
        unsafe {
            CCall::stackless_invoke(|unrooted| {
                let data = POpaqueTwo { value, value2 };
                TypedValue::new(unrooted, data).leak()
            })
        }
    }

    fn get_v1(&self) -> T {
        self.value
    }

    fn get_v2(&self) -> U {
        self.value2
    }
}

unsafe impl<T, U> ParametricBase for POpaqueTwo<T, U>
where
    T: 'static + Send + ConstructType,
    U: 'static + Send + ConstructType,
{
    type Key = POpaqueTwo<(), ()>;
    impl_type_parameters!('T', 'U');
}

unsafe impl<T, U> ParametricVariant for POpaqueTwo<T, U>
where
    T: 'static + Send + ConstructType,
    U: 'static + Send + ConstructType,
{
    impl_variant_parameters!(T, U);
}

pub struct ForeignThing {
    a: ValueRef<'static, 'static>,
}

unsafe impl Send for ForeignThing {}

unsafe impl ForeignType for ForeignThing {
    fn mark(ptls: jlrs::memory::PTls, data: &Self) -> usize {
        unsafe { mark_queue_obj(ptls, data.a) as usize }
    }
}

impl ForeignThing {
    fn new(value: Value) -> TypedValueRet<ForeignThing> {
        unsafe {
            CCall::stackless_invoke(|unrooted| {
                TypedValue::new(
                    &unrooted,
                    ForeignThing {
                        a: value.assume_owned().leak(),
                    },
                )
                .leak()
            })
        }
    }

    fn get(&self) -> ValueRet {
        unsafe { self.a.assume_owned().leak() }
    }

    fn set(&mut self, value: Value) {
        unsafe {
            self.a = value.assume_owned().leak();
            write_barrier(self, value);
        }
    }
}

struct UnexportedType;

impl UnexportedType {
    fn assoc_func() -> isize {
        1
    }
}

type GenericEnv = tvars!(
    tvar!('T'; AbstractFloat),
    tvar!('N'),
    tvar!('A'; AbstractArray<tvar!('T'), tvar!('N')>)
);
fn takes_generics_from_env(_array: TypedValue<tvar!('A')>, _data: TypedValue<tvar!('T')>) {}

fn has_generic<T>(t: T) -> T {
    t
}

fn has_two_generics<T, U>(t: T, _u: U) -> T {
    t
}

type TEnv = tvars!(tvar!('T'));
fn generic_arrays(
    a: TypedValue<ArrayTypeConstructor<tvar!('T'), ConstantIsize<1>>>,
    _b: TypedValue<ArrayTypeConstructor<tvar!('T'), ConstantIsize<2>>>,
) -> ValueRet {
    let a = a.as_value().cast::<RankedArray<1>>().expect("Wrong rank");
    a.element_type().as_ref().leak()
}

type UEnv = tvars!(tvar!('U'));
fn generic_ranked_arrays(
    a: TypedRankedArray<tvar!('U'), 1>,
    _b: TypedRankedArray<tvar!('U'), 2>,
) -> ValueRet {
    a.element_type().as_ref().leak()
}

fn generic_typed_arrays(
    a: TypedArray<tvar!('U')>,
    _b: TypedArray<tvar!('U')>,
) -> ValueRet {
    a.element_type().as_ref().leak()
}

fn generic_ranked_array_ret_self(
    a: TypedRankedArray<'_, 'static, tvar!('U'), 1>,
) -> TypedRankedArrayRet<tvar!('U'), 1> {
    a.as_ref().leak()
}

const CONST_U8: u8 = 1;
static STATIC_U8: u8 = 2;

julia_module! {
    become julia_module_tests_init_fn;

    fn freestanding_func_trivial();
    fn freestanding_func_noargs() -> usize;

    fn freestanding_func_bitsarg(a: usize) -> usize;
    fn freestanding_func_arrayarg(a: Array) -> usize;
    fn freestanding_func_ranked_arrayarg(a: RankedArray<1>) -> usize;
    fn freestanding_func_typed_arrayarg(a: TypedArray<u32>) -> usize;
    fn freestanding_func_ref_bitsarg(usize_ref: CCallRef<usize>) -> usize;
    fn freestanding_func_ref_any(value_ref: CCallRef<AnyType>) -> usize;
    fn freestanding_func_ref_mutarg(module_ref: CCallRef<Module>) -> usize;
    fn freestanding_func_ref_abstract(value_ref: CCallRef<Number>) -> usize;
    fn freestanding_func_typevaluearg(a: TypedValue<usize>) -> usize;
    fn freestanding_func_ret_array(dt: DataType) -> ArrayRet;
    fn freestanding_func_ret_ranked_array0(dt: DataType) -> RankedArrayRet<0>;
    fn freestanding_func_ret_ranked_array1(dt: DataType) -> RankedArrayRet<1>;
    fn freestanding_func_ret_ranked_array2(dt: DataType) -> RankedArrayRet<2>;
    fn freestanding_func_ret_ranked_array3(dt: DataType) -> RankedArrayRet<3>;
    fn freestanding_func_ret_typed_array() -> TypedArrayRet<f32>;
    fn freestanding_func_ret_rust_result(throw_err: Bool) -> JlrsResult<i32>;
    fn freestanding_func_ccall_ref_ret() -> CCallRefRet<bool>;
    fn freestanding_func_typed_value_ret() -> TypedValueRet<bool>;
    fn takes_generics_from_env(array: TypedValue<tvar!('A')>, data: TypedValue<tvar!('T')>) use GenericEnv;
    fn generic_arrays(
        a: TypedValue<ArrayTypeConstructor<tvar!('T'), ConstantIsize<1>>>,
        _b: TypedValue<ArrayTypeConstructor<tvar!('T'), ConstantIsize<2>>>,
    ) -> ValueRet use TEnv;

    fn generic_ranked_arrays(
        a: TypedRankedArray<tvar!('U'), 1>,
        _b: TypedRankedArray<tvar!('U'), 2>,
    ) -> ValueRet use UEnv;

    fn generic_typed_arrays(
        a: TypedArray<tvar!('U')>,
        _b: TypedArray<tvar!('U')>,
    ) -> ValueRet use UEnv;

    fn generic_ranked_array_ret_self(
        a: TypedRankedArray<'_, 'static, tvar!('U'), 1>,
    ) -> TypedRankedArrayRet<tvar!('U'), 1> use UEnv;

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
    in ForeignThing fn new(value: Value) -> TypedValueRet<ForeignThing> as ForeignThing;

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

    fn take_four_generics_m(v: TypedValue<'_, 'static, FourGenericsM<i32, i32, i32, i32>>) -> TypedValueRet<FourGenericsM<i32, i32, i32, i32>>;
    fn take_four_generics_m_trailing1(
        v: TypedValue<'_, 'static, FourGenericsM<i32, i32, i32, tvar!('D')>>,
    ) -> TypedValueRet<FourGenericsM<i32, i32, i32, tvar!('D')>> use tvars!(tvar!('D'));
    fn take_four_generics_m_trailing2(
        v: TypedValue<'_, 'static, FourGenericsM<i32, i32, tvar!('C'), tvar!('D')>>,
    ) -> TypedValueRet<FourGenericsM<i32, i32, tvar!('C'), tvar!('D')>> use tvars!(tvar!('C'), tvar!('D'));
    fn take_four_generics_m_middle(
        v: TypedValue<'_, 'static, FourGenericsM<i32, tvar!('B'), i32, tvar!('D')>>,
    ) -> TypedValueRet<FourGenericsM<i32, tvar!('B'), i32, tvar!('D')>> use tvars!(tvar!('B'), tvar!('D'));
    fn take_four_generics_m_start1(
        v: TypedValue<'_, 'static, FourGenericsM<tvar!('A'), i32, i32, i32>>,
    ) -> TypedValueRet<FourGenericsM<tvar!('A'), i32, i32, i32>>  use tvars!(tvar!('A'));
    fn take_four_generics_m_start2(
        v: TypedValue<'_, 'static, FourGenericsM<tvar!('A'), tvar!('B'), i32, i32>>,
    ) -> TypedValueRet<FourGenericsM<tvar!('A'), tvar!('B'), i32, i32>>  use tvars!(tvar!('A'), tvar!('B'));

    fn take_four_generics_i_trailing1(
        v: TypedValue<'_, 'static, FourGenericsI<i32, i32, i32, tvar!('D')>>,
    ) -> TypedValueRet<FourGenericsI<i32, i32, i32, tvar!('D')>> use tvars!(tvar!('D'));

    const CONST_U8: u8;
    static CONST_U8: u8 as STATIC_CONST_U8;
    const STATIC_U8: u8 as CONST_STATIC_U8;
    static STATIC_U8: u8;

    type POpaque64 = POpaque<f64>;
    in POpaque<f64> fn new(value: f64) -> TypedValueRet<POpaque<f64>> as POpaque64;
}
