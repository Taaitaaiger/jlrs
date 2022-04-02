mod util;
#[cfg(all(feature = "sync-rt", feature = "ccall"))]
#[cfg(not(all(target_os = "windows", feature = "lts")))]
mod tests {
    #[cfg(feature = "uv")]
    use std::{
        ffi::c_void,
        thread::{self, JoinHandle},
    };

    use super::util::JULIA;
    use jlrs::prelude::*;

    unsafe extern "C" fn uses_null_scope(array: TypedArray<f64>) -> bool {
        let mut ccall = CCall::new();

        let out = ccall.null_scope(|frame| {
            let borrowed = array.inline_data(&mut *frame)?;
            Ok(borrowed[1] == 1.0)
        });

        if let Ok(o) = out {
            o
        } else {
            false
        }
    }

    unsafe extern "C" fn uses_scope(array: TypedArray<f64>) -> bool {
        let mut ccall = CCall::new();

        let out = ccall.scope(|_, frame| {
            let _ = Value::new(&mut *frame, 0usize)?;
            let borrowed = array.inline_data(&mut *frame)?;
            Ok(borrowed[1] == 1.0)
        });

        if let Ok(o) = out {
            o
        } else {
            false
        }
    }

    unsafe extern "C" fn uses_scope_with_slots(array: TypedArray<f64>) -> bool {
        let mut ccall = CCall::new();

        let out = ccall.scope_with_capacity(1, |_, frame| {
            let _ = Value::new(&mut *frame, 0usize)?;
            let borrowed = array.inline_data(&mut *frame)?;
            Ok(borrowed[1] == 1.0)
        });

        if let Ok(o) = out {
            o
        } else {
            false
        }
    }

    unsafe extern "C" fn uses_scope_with_realloced_slots(array: TypedArray<f64>) -> bool {
        let mut ccall = CCall::new();

        let out = ccall.scope_with_capacity(128, |_, frame| {
            let _ = Value::new(&mut *frame, 0usize)?;
            let borrowed = array.inline_data(&mut *frame)?;
            Ok(borrowed[1] == 1.0)
        });

        if let Ok(o) = out {
            o
        } else {
            false
        }
    }

    #[test]
    fn ccall_with_array() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, frame| unsafe {
                let fn_ptr = Value::new(&mut *frame, uses_null_scope as *mut std::ffi::c_void)?;
                let mut arr_data = vec![0.0f64, 1.0f64];
                let arr = Array::from_slice(&mut *frame, &mut arr_data, 2)?;
                let func = Module::main(global)
                    .submodule_ref("JlrsTests")?
                    .wrapper_unchecked()
                    .function_ref("callrustwitharr")?
                    .wrapper_unchecked();

                let out = func.call2(&mut *frame, fn_ptr, arr.as_value())?.unwrap();
                let ok = out.unbox::<bool>()?.as_bool();
                assert!(ok);
                Ok(())
            })
            .unwrap()
        })
    }

    #[test]
    fn ccall_with_array_and_scope() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, frame| unsafe {
                let fn_ptr = Value::new(&mut *frame, uses_scope as *mut std::ffi::c_void)?;
                let mut arr_data = vec![0.0f64, 1.0f64];
                let arr = Array::from_slice(&mut *frame, &mut arr_data, 2)?;
                let func = Module::main(global)
                    .submodule_ref("JlrsTests")?
                    .wrapper_unchecked()
                    .function_ref("callrustwitharr")?
                    .wrapper_unchecked();

                let out = func.call2(&mut *frame, fn_ptr, arr.as_value())?.unwrap();
                let ok = out.unbox::<bool>()?.as_bool();
                assert!(ok);
                Ok(())
            })
            .unwrap()
        })
    }

    #[test]
    fn ccall_with_array_and_scope_with_slots() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, frame| unsafe {
                let fn_ptr =
                    Value::new(&mut *frame, uses_scope_with_slots as *mut std::ffi::c_void)?;
                let mut arr_data = vec![0.0f64, 1.0f64];
                let arr = Array::from_slice(&mut *frame, &mut arr_data, 2)?;
                let func = Module::main(global)
                    .submodule_ref("JlrsTests")?
                    .wrapper_unchecked()
                    .function_ref("callrustwitharr")?
                    .wrapper_unchecked();

                let out = func.call2(&mut *frame, fn_ptr, arr.as_value())?.unwrap();
                let ok = out.unbox::<bool>()?.as_bool();
                assert!(ok);
                Ok(())
            })
            .unwrap()
        })
    }

    #[test]
    fn ccall_with_array_and_reallocated_scope_with_slots() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, frame| unsafe {
                let fn_ptr = Value::new(
                    &mut *frame,
                    uses_scope_with_realloced_slots as *mut std::ffi::c_void,
                )?;
                let mut arr_data = vec![0.0f64, 1.0f64];
                let arr = Array::from_slice(&mut *frame, &mut arr_data, 2)?;
                let func = Module::main(global)
                    .submodule_ref("JlrsTests")?
                    .wrapper_unchecked()
                    .function_ref("callrustwitharr")?
                    .wrapper_unchecked();

                let out = func.call2(&mut *frame, fn_ptr, arr.as_value())?.unwrap();
                let ok = out.unbox::<bool>()?.as_bool();
                assert!(ok);
                Ok(())
            })
            .unwrap()
        })
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

    #[test]
    #[cfg(feature = "uv")]
    fn ccall_with_async_condidtion() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();

            jlrs.scope(|global, frame| unsafe {
                let fn_ptr = Value::new(&mut *frame, multithreaded as *mut std::ffi::c_void)?;
                let destroy_handle_fn_ptr =
                    Value::new(&mut *frame, drop_handle as *mut std::ffi::c_void)?;

                let func = Module::main(global)
                    .submodule_ref("JlrsTests")?
                    .wrapper_unchecked()
                    .function_ref("callrustwithasynccond")?
                    .wrapper_unchecked();

                let out = func
                    .call2(&mut *frame, fn_ptr, destroy_handle_fn_ptr)?
                    .unwrap();
                let ok = out.unbox::<u32>()?;
                assert_eq!(ok, 127);
                Ok(())
            })
            .unwrap()
        })
    }
}
