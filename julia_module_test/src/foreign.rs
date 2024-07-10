use std::ops::AddAssign;

use jlrs::{
    data::{
        managed::value::{
            typed::{TypedValue, TypedValueRet},
            ValueRet,
        },
        types::{
            construct_type::ConstructType,
            foreign_type::{ForeignType, OpaqueType, ParametricBase, ParametricVariant},
        },
    },
    impl_type_parameters, impl_variant_parameters,
    memory::gc::{mark_queue_obj, write_barrier},
    prelude::{Managed, Value, ValueRef},
    weak_handle_unchecked,
};

#[derive(Clone, Debug)]
pub struct OpaqueInt {
    a: i32,
}

unsafe impl OpaqueType for OpaqueInt {}

impl OpaqueInt {
    pub fn new(value: i32) -> TypedValueRet<OpaqueInt> {
        let weak_handle = unsafe { weak_handle_unchecked!() };
        TypedValue::new(weak_handle, OpaqueInt { a: value }).leak()
    }

    pub fn increment(&mut self) {
        self.a += 1;
    }

    pub fn get(&self) -> i32 {
        self.a
    }

    pub fn get_cloned(self) -> i32 {
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
    pub fn new(value: T) -> TypedValueRet<Self> {
        let weak_handle = unsafe { weak_handle_unchecked!() };
        let data = POpaque { value };
        TypedValue::new(weak_handle, data).leak()
    }

    pub fn get(&self) -> T {
        self.value
    }

    pub fn get_cloned(self) -> T {
        self.value
    }

    pub fn set(&mut self, value: T) {
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
    pub fn new(value: T, value2: U) -> TypedValueRet<Self> {
        let weak_handle = unsafe { weak_handle_unchecked!() };
        let data = POpaqueTwo { value, value2 };
        TypedValue::new(weak_handle, data).leak()
    }

    pub fn get_v1(&self) -> T {
        self.value
    }

    pub fn get_v2(&self) -> U {
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
    pub fn new(value: Value<'_, 'static>) -> TypedValueRet<ForeignThing> {
        let weak_handle = unsafe { weak_handle_unchecked!() };
        TypedValue::new(weak_handle, ForeignThing { a: value.leak() }).leak()
    }

    pub fn get(&self) -> ValueRet {
        unsafe { self.a.assume_owned().leak() }
    }

    pub fn set(&mut self, value: Value) {
        unsafe {
            self.a = value.assume_owned().leak();
            write_barrier(self, value);
        }
    }
}

pub struct UnexportedType;

impl UnexportedType {
    pub fn assoc_func() -> isize {
        1
    }
}
