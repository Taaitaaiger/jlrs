mod util;
#[cfg(all(feature = "local-rt", feature = "ccall"))]
mod tests {
    use jlrs::prelude::*;

    use super::util::JULIA;

    unsafe extern "C" fn doesnt_use_scope(array: TypedArray<f64>) -> bool {
        let tracked = array.track_shared().expect("Already borrowed");

        let borrowed = tracked.inline_data();

        if borrowed[1] == 1.0 {
            true
        } else {
            false
        }
    }

    unsafe extern "C" fn uses_scope(array: TypedArray<f64>) -> bool {
        let mut context_frame = StackFrame::new();
        let mut ccall = CCall::new(&mut context_frame);

        let out = ccall.scope(|mut frame| {
            let _ = Value::new(&mut frame, 0usize);
            let borrowed = array.inline_data();
            Ok(borrowed[1] == 1.0)
        });

        if let Ok(o) = out {
            o
        } else {
            false
        }
    }

    unsafe extern "C" fn uses_scope_with_realloced_slots(array: TypedArray<f64>) -> bool {
        let mut context_frame = StackFrame::new();
        let mut ccall = CCall::new(&mut context_frame);

        let out = ccall.scope(|mut frame| {
            let _ = Value::new(&mut frame, 0usize);
            let borrowed = array.inline_data();
            Ok(borrowed[1] == 1.0)
        });

        if let Ok(o) = out {
            o
        } else {
            false
        }
    }

    fn ccall_with_array() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let fn_ptr = Value::new(&mut frame, doesnt_use_scope as *mut std::ffi::c_void);
                    let mut arr_data = vec![0.0f64, 1.0f64];
                    let arr = TypedArray::<f64>::from_slice_unchecked(&mut frame, &mut arr_data, 2);
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .function(&frame, "callrustwitharr")?
                        .as_managed();

                    let out = func.call2(&mut frame, fn_ptr, arr.as_value()).unwrap();
                    let ok = out.unbox::<bool>()?.as_bool();
                    assert!(ok);
                    Ok(())
                })
                .unwrap();
        })
    }

    fn ccall_with_array_and_scope() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let fn_ptr = Value::new(&mut frame, uses_scope as *mut std::ffi::c_void);
                    let mut arr_data = vec![0.0f64, 1.0f64];
                    let arr = TypedArray::<f64>::from_slice_unchecked(&mut frame, &mut arr_data, 2);
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .function(&frame, "callrustwitharr")?
                        .as_managed();

                    let out = func.call2(&mut frame, fn_ptr, arr.as_value()).unwrap();
                    let ok = out.unbox::<bool>()?.as_bool();
                    assert!(ok);
                    Ok(())
                })
                .unwrap();
        })
    }

    fn ccall_with_array_and_reallocated_scope_with_slots() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .returning::<JlrsResult<_>>()
                .scope(|mut frame| unsafe {
                    let fn_ptr = Value::new(
                        &mut frame,
                        uses_scope_with_realloced_slots as *mut std::ffi::c_void,
                    );
                    let mut arr_data = vec![0.0f64, 1.0f64];
                    let arr = TypedArray::<f64>::from_slice_unchecked(&mut frame, &mut arr_data, 2);
                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .function(&frame, "callrustwitharr")?
                        .as_managed();

                    let out = func.call2(&mut frame, fn_ptr, arr.as_value()).unwrap();
                    let ok = out.unbox::<bool>()?.as_bool();
                    assert!(ok);
                    Ok(())
                })
                .unwrap();
        });
    }

    #[test]
    fn ccall_tests() {
        ccall_with_array();
        ccall_with_array_and_scope();
        ccall_with_array_and_reallocated_scope_with_slots();
    }
}
