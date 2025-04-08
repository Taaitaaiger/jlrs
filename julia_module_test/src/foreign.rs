use std::{collections::HashMap, ops::AddAssign};

use jlrs::{
    data::{
        managed::value::{
            typed::{TypedValue, TypedValueRet},
            ValueRet,
        },
        types::{construct_type::ConstructType, foreign_type::mark::Mark},
    },
    memory::gc::{write_barrier},
    prelude::{ForeignType, Managed, OpaqueType, Value, WeakValue},
    weak_handle_unchecked,
};

#[derive(Clone, Debug, OpaqueType)]
pub struct OpaqueInt {
    a: i32,
}

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

#[derive(Clone, OpaqueType)]
pub struct POpaque<T> {
    value: T,
}

impl<T> POpaque<T>
where
    T: 'static
        + Send
        + Sync
        + ConstructType
        + AddAssign
        + Copy
        + jlrs::convert::into_julia::IntoJulia,
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

#[derive(Clone, OpaqueType)]
pub struct POpaqueTwo<T, U> {
    value: T,
    value2: U,
}

impl<T, U> POpaqueTwo<T, U>
where
    T: 'static
        + Send
        + Sync
        + ConstructType
        + AddAssign
        + Copy
        + jlrs::convert::into_julia::IntoJulia,
    U: 'static
        + Send
        + Sync
        + ConstructType
        + AddAssign
        + Copy
        + jlrs::convert::into_julia::IntoJulia,
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

unsafe fn mark_map<M: Mark, P: jlrs::data::types::foreign_type::ForeignType>(
    data: &HashMap<(), M>,
    ptls: jlrs::memory::PTls,
    parent: &P,
) -> usize {
    data.values().map(|v| unsafe { v.mark(ptls, parent) }).sum()
}

#[derive(ForeignType)]
pub struct ForeignThing {
    #[jlrs(mark)]
    a: WeakValue<'static, 'static>,
    #[jlrs(mark_with = mark_map)]
    b: HashMap<(), WeakValue<'static, 'static>>,
}

unsafe impl Send for ForeignThing {}
unsafe impl Sync for ForeignThing {}

impl ForeignThing {
    pub fn new(value: Value<'_, 'static>) -> TypedValueRet<ForeignThing> {
        let weak_handle = unsafe { weak_handle_unchecked!() };
        TypedValue::new(
            weak_handle,
            ForeignThing {
                a: value.leak(),
                b: HashMap::default(),
            },
        )
        .leak()
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
