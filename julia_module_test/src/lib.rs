use std::ops::AddAssign;

use jlrs::{
    data::{
        managed::{
            array::ArrayRet,
            ccall_ref::{CCallRef, CCallRefRet},
            value::{
                typed::{TypedValue, TypedValueRet},
                ValueRet,
            },
        },
        types::{
            abstract_type::{AbstractArray, AbstractFloat, AnyType, Number},
            construct_type::ConstructType,
            foreign_type::{ForeignType, OpaqueType, ParametricBase, ParametricVariant},
        },
    },
    impl_type_parameters, impl_variant_parameters,
    memory::gc::{mark_queue_obj, write_barrier},
    prelude::*,
    tvar, tvars,
};

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
        Array::new_for::<_, _>(unrooted, dt.as_value(), (2, 2))
            .unwrap()
            .leak()
    })
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

const CONST_U8: u8 = 1;
static STATIC_U8: u8 = 2;

julia_module! {
    become julia_module_tests_init_fn;

    fn freestanding_func_trivial();
    fn freestanding_func_noargs() -> usize;

    fn freestanding_func_bitsarg(a: usize) -> usize;
    fn freestanding_func_arrayarg(a: Array) -> usize;
    fn freestanding_func_ref_bitsarg(usize_ref: CCallRef<usize>) -> usize;
    fn freestanding_func_ref_any(value_ref: CCallRef<AnyType>) -> usize;
    fn freestanding_func_ref_mutarg(module_ref: CCallRef<Module>) -> usize;
    fn freestanding_func_ref_abstract(value_ref: CCallRef<Number>) -> usize;
    fn freestanding_func_typevaluearg(a: TypedValue<usize>) -> usize;
    fn freestanding_func_ret_array(dt: DataType) -> ArrayRet;
    fn freestanding_func_ret_rust_result(throw_err: Bool) -> JlrsResult<i32>;
    fn freestanding_func_ccall_ref_ret() -> CCallRefRet<bool>;
    fn freestanding_func_typed_value_ret() -> TypedValueRet<bool>;
    fn takes_generics_from_env(array: TypedValue<tvar!('A')>, data: TypedValue<tvar!('T')>) use GenericEnv;

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

    const CONST_U8: u8;
    static CONST_U8: u8 as STATIC_CONST_U8;
    const STATIC_U8: u8 as CONST_STATIC_U8;
    static STATIC_U8: u8;

    type POpaque64 = POpaque<f64>;
    in POpaque<f64> fn new(value: f64) -> TypedValueRet<POpaque<f64>> as POpaque64;
}
