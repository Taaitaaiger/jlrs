use jlrs::{
    data::managed::value::typed::{TypedValue, TypedValueRet},
    prelude::{Managed, Value},
    weak_handle_unchecked,
};

pub unsafe fn takes_typed_value(a: TypedValue<usize>) -> usize {
    unsafe { a.unbox_unchecked::<usize>() }
}

pub fn returns_typed_value() -> TypedValueRet<bool> {
    let weak_handle = unsafe { weak_handle_unchecked!() };
    Value::true_v(&weak_handle)
        .as_typed::<bool, _>(&weak_handle)
        .unwrap()
        .leak()
}
