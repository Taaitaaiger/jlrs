mod util;
#[cfg(all(feature = "local-rt", feature = "ccall"))]
mod tests {
    use jlrs::{
        data::managed::{ccall_ref::CCallRef, named_tuple::NamedTuple},
        prelude::*,
        weak_handle,
    };

    use super::util::JULIA;

    unsafe extern "C" fn doesnt_use_scope(array: TypedArray<f64>) -> bool {
        let tracked = array.track_shared().expect("Already borrowed");
        let borrowed = tracked.inline_data();
        borrowed[1] == 1.0
    }

    unsafe extern "C" fn uses_scope(array: TypedArray<f64>) -> bool {
        unsafe {
            match weak_handle!() {
                Ok(handle) => handle.local_scope::<_, 1>(|mut frame| {
                    let _ = Value::new(&mut frame, 0usize);
                    let borrowed = array.inline_data();
                    borrowed[1] == 1.0
                }),
                Err(_) => panic!("Not called from Julia"),
            }
        }
    }

    fn ccall_with_array() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let fn_ptr = Value::new(&mut frame, doesnt_use_scope as *mut std::ffi::c_void);
                    let mut arr_data = vec![0.0f64, 1.0f64];
                    let arr = TypedArray::<f64>::from_slice_unchecked(&mut frame, &mut arr_data, 2);
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "callrustwitharr")
                        .unwrap()
                        .as_managed();

                    let out = func.call(&mut frame, [fn_ptr, arr.as_value()]).unwrap();
                    let ok = out.unbox::<bool>().unwrap().as_bool();
                    assert!(ok);
                })
            })
        })
    }

    fn ccall_with_array_and_scope() {
        JULIA.with(|handle| {
            handle.borrow_mut().with_stack(|mut stack| {
                stack.scope(|mut frame| unsafe {
                    let fn_ptr = Value::new(&mut frame, uses_scope as *mut std::ffi::c_void);
                    let mut arr_data = vec![0.0f64, 1.0f64];
                    let arr = TypedArray::<f64>::from_slice_unchecked(&mut frame, &mut arr_data, 2);
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")
                        .unwrap()
                        .as_managed()
                        .global(&frame, "callrustwitharr")
                        .unwrap()
                        .as_managed();

                    let out = func.call(&mut frame, [fn_ptr, arr.as_value()]).unwrap();
                    let ok = out.unbox::<bool>().unwrap().as_bool();
                    assert!(ok);
                })
            })
        })
    }

    fn ccall_ref_named_tuple() {
        JULIA.with(|handle| {
            handle.borrow().local_scope::<_, 2>(|mut frame| {
                let a = Value::new(&mut frame, 2usize);
                let kw = named_tuple!(&mut frame, "a" => a).unwrap();
                let ccall_ref =
                    unsafe { std::mem::transmute::<NamedTuple, CCallRef<NamedTuple>>(kw) };

                ccall_ref.as_managed().unwrap();
            })
        })
    }

    #[test]
    fn ccall_tests() {
        ccall_with_array();
        ccall_with_array_and_scope();
        ccall_ref_named_tuple();
    }
}
