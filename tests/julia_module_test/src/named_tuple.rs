use jlrs::{
    data::managed::{
        ccall_ref::CCallRef,
        named_tuple::{NamedTuple, NamedTupleRet},
    },
    named_tuple,
    prelude::{JuliaString, LocalScope, Managed, Value},
    weak_handle_unchecked,
};

pub fn returns_named_tuple() -> NamedTupleRet {
    let weak_handle = unsafe { weak_handle_unchecked!() };

    weak_handle.local_scope::<_, 2>(|mut frame| {
        let a = Value::new(&mut frame, 1usize);
        let b = JuliaString::new(&mut frame, "foo").as_value();

        named_tuple!(&frame, "a" => a, "b" => b).unwrap().leak()
    })
}

pub fn take_return_named_tuple(nt: CCallRef<NamedTuple<'_, 'static>>) -> NamedTupleRet {
    nt.as_managed().unwrap().leak()
}
