mod util;
#[cfg(all(feature = "sync-rt", feature = "ccall"))]
mod tests {
    #[cfg(feature = "uv")]
    use std::{
        ffi::c_void,
        thread::{self, JoinHandle},
    };

    use jlrs::prelude::*;

    use super::util::JULIA;

    unsafe extern "C" fn doesnt_use_scope(array: TypedArray<f64>) -> bool {
        let tracked = array.track_shared().expect("Already borrowed");

        let borrowed = tracked.inline_data().expect("Not inline");

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
            let borrowed = array.inline_data()?;
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
            let borrowed = array.inline_data()?;
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
                .scope(|mut frame| unsafe {
                    let fn_ptr = Value::new(&mut frame, doesnt_use_scope as *mut std::ffi::c_void);
                    let mut arr_data = vec![0.0f64, 1.0f64];
                    let arr =
                        Array::from_slice_unchecked(frame.as_extended_target(), &mut arr_data, 2)?;
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
                .scope(|mut frame| unsafe {
                    let fn_ptr = Value::new(&mut frame, uses_scope as *mut std::ffi::c_void);
                    let mut arr_data = vec![0.0f64, 1.0f64];
                    let arr =
                        Array::from_slice_unchecked(frame.as_extended_target(), &mut arr_data, 2)?;
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
                .scope(|mut frame| unsafe {
                    let fn_ptr = Value::new(
                        &mut frame,
                        uses_scope_with_realloced_slots as *mut std::ffi::c_void,
                    );
                    let mut arr_data = vec![0.0f64, 1.0f64];
                    let arr =
                        Array::from_slice_unchecked(frame.as_extended_target(), &mut arr_data, 2)?;
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

    #[repr(transparent)]
    #[cfg(feature = "uv")]
    pub struct SendablePtr<T>(*mut T);
    #[cfg(feature = "uv")]
    unsafe impl<T> Send for SendablePtr<T> {}

    #[no_mangle]
    #[cfg(feature = "uv")]
    pub unsafe extern "C" fn multithreaded(
        out: SendablePtr<u32>,
        handle: SendablePtr<c_void>,
    ) -> *mut c_void {
        let handle = thread::spawn(move || {
            std::ptr::write(out.0, 127);
            CCall::uv_async_send(handle.0);
        });

        // Box and return the JoinHandle as a pointer.
        // The handle must be dropped by calling `drop_handle`.
        let boxed = Box::new(handle);
        Box::leak(boxed) as *mut _ as *mut _
    }

    #[no_mangle]
    #[cfg(feature = "uv")]
    pub unsafe extern "C" fn drop_handle(handle: *mut JoinHandle<()>) {
        Box::from_raw(handle).join().ok();
    }

    #[cfg(feature = "uv")]
    fn ccall_with_async_condition() {
        JULIA.with(|j| {
            let mut frame = StackFrame::new();
            let mut jlrs = j.borrow_mut();

            jlrs.instance(&mut frame)
                .scope(|mut frame| unsafe {
                    let fn_ptr = Value::new(&mut frame, multithreaded as *mut std::ffi::c_void);
                    let destroy_handle_fn_ptr =
                        Value::new(&mut frame, drop_handle as *mut std::ffi::c_void);

                    let func = Module::main(&frame)
                        .submodule(&frame, "JlrsTests")?
                        .as_managed()
                        .function(&frame, "callrustwithasynccond")?
                        .as_managed();

                    let out = func
                        .call2(&mut frame, fn_ptr, destroy_handle_fn_ptr)
                        .unwrap();
                    let ok = out.unbox::<u32>()?;
                    assert_eq!(ok, 127);
                    Ok(())
                })
                .unwrap();
        })
    }

    #[test]
    fn ccall_tests() {
        ccall_with_array();
        ccall_with_array_and_scope();
        ccall_with_array_and_reallocated_scope_with_slots();
        #[cfg(feature = "uv")]
        ccall_with_async_condition();
    }
}
