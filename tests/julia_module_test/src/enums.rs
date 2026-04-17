use jlrs::{
    data::managed::value::typed::{TypedValue, TypedValueRet},
    prelude::OpaqueType,
    weak_handle_unchecked,
};

#[derive(Clone, Debug, OpaqueType)]
pub enum OpaqueEnum {
    F64(f64),
    Usize(usize),
}

impl OpaqueEnum {
    pub fn new_usize(value: usize) -> TypedValueRet<OpaqueEnum> {
        let weak_handle = unsafe { weak_handle_unchecked!() };
        TypedValue::new(weak_handle, OpaqueEnum::Usize(value)).leak()
    }

    pub fn new_f64(value: f64) -> TypedValueRet<OpaqueEnum> {
        let weak_handle = unsafe { weak_handle_unchecked!() };
        TypedValue::new(weak_handle, OpaqueEnum::F64(value)).leak()
    }

    pub fn is_usize(&self) -> bool {
        match self {
            OpaqueEnum::F64(_) => false,
            OpaqueEnum::Usize(_) => true,
        }
    }

    pub fn is_f64(&self) -> bool {
        match self {
            OpaqueEnum::F64(_) => true,
            OpaqueEnum::Usize(_) => false,
        }
    }
}
