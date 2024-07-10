use jlrs::{
    data::{
        managed::ccall_ref::{CCallRef, CCallRefRet},
        types::abstract_type::{AnyType, Number},
    },
    prelude::{Managed, Module, Value},
    weak_handle_unchecked,
};

pub fn takes_ref_usize(usize_ref: CCallRef<usize>) -> usize {
    usize_ref.as_ref().unwrap() + 1
}

pub fn takes_ref_module(module_ref: CCallRef<Module>) -> usize {
    let _module = module_ref.as_managed().unwrap();
    0
}

pub fn takes_ref_any(value_ref: CCallRef<AnyType>) -> usize {
    let _dt = value_ref.as_value_ref().datatype();
    0
}

pub fn takes_ref_number(value_ref: CCallRef<Number>) -> usize {
    let _dt = value_ref.as_value().unwrap().datatype();
    0
}

pub fn returns_ref_bool() -> CCallRefRet<bool> {
    let weak_handle = unsafe { weak_handle_unchecked!() };

    let v = Value::true_v(&weak_handle)
        .as_typed::<bool, _>(&weak_handle)
        .unwrap()
        .as_ref()
        .leak();

    CCallRefRet::new(v)
}
