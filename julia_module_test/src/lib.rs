use std::ops::AddAssign;

use jlrs::{
    ccall::AsyncCallback,
    data::{
        managed::{
            array::{ArrayRet, TypedArrayUnbound},
            ccall_ref::CCallRef,
            value::{
                typed::{TypedValue, TypedValueRet},
                ValueRet,
            },
        },
        types::{
            abstract_types::{AnyType, Number},
            construct_type::ConstructType,
            foreign_type::{ForeignType, OpaqueType, ParametricBase, ParametricVariant},
        },
    },
    error::JlrsError,
    impl_type_parameters, impl_variant_parameters,
    memory::gc::{mark_queue_obj, write_barrier},
    prelude::*,
};

#[inline]
fn freestanding_func_trivial() {}

#[inline]
fn freestanding_func_noargs() -> usize {
    0
}

#[inline]
fn freestanding_func_bitsarg(a: usize) -> usize {
    a + 1
}

#[inline]
fn freestanding_func_ref_bitsarg(usize_ref: CCallRef<usize>) -> usize {
    usize_ref.as_ref().unwrap() + 1
}

#[inline]
fn freestanding_func_arrayarg(a: Array) -> usize {
    let elty = a.element_type();

    if let Ok(elty) = elty.cast::<DataType>() {
        elty.size().unwrap_or(std::mem::size_of::<usize>() as _) as _
    } else {
        std::mem::size_of::<usize>()
    }
}

#[inline]
fn freestanding_func_ref_mutarg(module_ref: CCallRef<Module>) -> usize {
    let _module = module_ref.as_managed().unwrap();
    //let target = module.unrooted_target();
    //module.set_global_unchecked("MyGlobal", Value::nothing(&target));
    0
}

#[inline]
fn freestanding_func_ref_any(value_ref: CCallRef<AnyType>) -> usize {
    let _dt = value_ref.as_value_ref().datatype();
    //println!("freestanding_func_ref_any {:?}", value.datatype());
    0
}

#[inline]
fn freestanding_func_ref_abstract(value_ref: CCallRef<Number>) -> usize {
    let _dt = value_ref.as_value().unwrap().datatype();
    //println!("freestanding_func_ref_abstract {:?}", value.datatype());
    0
}

#[inline]
unsafe fn freestanding_func_typevaluearg(a: TypedValue<usize>) -> usize {
    a.unbox_unchecked::<usize>()
}

#[inline]
unsafe fn freestanding_func_ret_array(dt: DataType) -> ArrayRet {
    CCall::stackless_invoke(|unrooted| {
        Array::new_for::<_, _>(unrooted, (2, 2), dt.as_value())
            .unwrap()
            .leak()
    })
}

#[inline]
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
    #[inline]
    fn new(value: i32) -> TypedValueRet<OpaqueInt> {
        unsafe {
            CCall::stackless_invoke(|unrooted| TypedValue::new(unrooted, OpaqueInt { a: value }).leak())
        }
    }

    #[inline]
    fn increment(&mut self) {
        self.a += 1;
    }

    #[inline]
    fn get(&self) -> i32 {
        self.a
    }

    #[inline]
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
    #[inline]
    fn new(value: T) -> TypedValueRet<Self> {
        unsafe {
            CCall::invoke(|mut frame| {
                let data = POpaque { value };
                TypedValue::new(&mut frame, data).leak()
            })
        }
    }

    #[inline]
    fn get(&self) -> T {
        self.value
    }

    #[inline]
    fn get_cloned(self) -> T {
        self.value
    }

    #[inline]
    fn set(&mut self, value: T) {
        self.value = value;
    }
}

#[derive(Clone)]
pub struct POpaqueTwo<T, U> {
    value: T,
    value2: U,
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

pub struct ForeignThing {
    a: ValueRef<'static, 'static>,
}
impl<T, U> POpaqueTwo<T, U>
where
    T: 'static + Send + ConstructType + AddAssign + Copy + jlrs::convert::into_julia::IntoJulia,
    U: 'static + Send + ConstructType + AddAssign + Copy + jlrs::convert::into_julia::IntoJulia,
{
    #[inline]
    fn new(value: T, value2: U) -> TypedValueRet<Self> {
        unsafe {
            CCall::stackless_invoke(|unrooted| {
                let data = POpaqueTwo { value, value2 };
                TypedValue::new(unrooted, data).leak()
            })
        }
    }

    #[inline]
    fn get_v1(&self) -> T {
        self.value
    }

    #[inline]
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

unsafe impl Send for ForeignThing {}

unsafe impl ForeignType for ForeignThing {
    #[inline]
    fn mark(ptls: jlrs::memory::PTls, data: &Self) -> usize {
        unsafe { mark_queue_obj(ptls, data.a) as usize }
    }
}

impl ForeignThing {
    #[inline]
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

    #[inline]
    fn get(&self) -> ValueRet {
        unsafe { self.a.assume_owned().leak() }
    }

    #[inline]
    fn set(&mut self, value: Value) {
        unsafe {
            self.a = value.assume_owned().leak();
            write_barrier(self, value);
        }
    }
}

struct UnexportedType;

impl UnexportedType {
    #[inline]
    fn assoc_func() -> isize {
        1
    }
}

#[inline]
fn async_callback(arr: TypedArrayUnbound<isize>) -> JlrsResult<impl AsyncCallback<isize>> {
    let arr = arr.track_shared_unbound()?;
    Ok(move || Ok(arr.as_slice().iter().sum()))
}

#[inline]
fn async_callback_init_err() -> JlrsResult<impl AsyncCallback<isize>> {
    Err(JlrsError::exception("Err"))?;
    Ok(move || Ok(0))
}

#[inline]
fn async_callback_callback_err() -> JlrsResult<impl AsyncCallback<isize>> {
    Ok(move || Err(JlrsError::exception("Err"))?)
}

#[inline]
fn generic_async_callback<T>(t: T) -> JlrsResult<impl AsyncCallback<T>>
where
    T: jlrs::convert::into_julia::IntoJulia
        + Send
        + jlrs::data::types::construct_type::ConstructType,
{
    Ok(move || Ok(t))
}

#[inline]
fn has_generic<T>(t: T) -> T {
    t
}

#[inline]
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

    #[doc = "    async_callback(array::Array{Int})::Int"]
    #[doc = ""]
    #[doc = "...docs for async_callback"]
    async fn async_callback(arr: TypedArrayUnbound<isize>) -> JlrsResult<impl AsyncCallback<isize>>;
    async fn async_callback_init_err() -> JlrsResult<impl AsyncCallback<isize>>;
    async fn async_callback_callback_err() -> JlrsResult<impl AsyncCallback<isize>>;

    for T in [f64, f32, f64] {
        fn has_generic(t: T) -> T;

        struct POpaque<T>;

        in POpaque<T> fn new(value: T) -> TypedValueRet<POpaque<T>> as POpaque;
        in POpaque<T> fn get(&self) -> T as popaque_get;
        in POpaque<T> fn get_cloned(self) -> T as popaque_get_cloned;
        in POpaque<T> fn set(&mut self, value: T) as popaque_set;

        #[doc = "    generic_async_callback{T}(t::T)::T"]
        #[doc = ""]
        #[doc = "...docs for generic_async_callback"]
        async fn generic_async_callback<T>(t: T) -> JlrsResult<impl AsyncCallback<T>>;

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
    in POpaque<f64> fn new(value: f64) -> TypedValueRet<POpaque<f64>> as POpaque64
}
