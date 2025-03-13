//! This example shows how to throw a Julia exception from a `ccall`ed function.

use jlrs::{prelude::*, runtime::handle::ccall::throw_exception, weak_handle};

// This function returns `nothing` if a < b, throws an `AssertionError` otherwise.
#[no_mangle]
pub unsafe extern "C" fn assert_less_than(a: i32, b: i32) {
    match weak_handle!() {
        Ok(handle) => {
            let res = handle.local_scope::<2>(|mut frame| {
                if a >= b {
                    let msg = JuliaString::new(&mut frame, "a is larger than b").as_value();

                    let leaked = Module::core(&frame)
                        .global(&frame, "AssertionError")
                        .expect("AssertionError does not exist in Core")
                        .as_value()
                        .cast::<DataType>()
                        .expect("AssertionError is not a DataType")
                        .instantiate_unchecked(&mut frame, [msg])
                        .leak();

                    return Err(leaked);
                }

                Ok(())
            });

            // Safe: there are no pendings drops.
            if let Err(exc) = res {
                throw_exception(exc)
            }
        }
        Err(_) => panic!("Not called from Julia"),
    }
}

#[cfg(test)]
mod tests {
    use jlrs::memory::scope::LocalReturning;

    use super::*;

    #[test]
    fn call_assert_less_than() {
        let mut jlrs = Builder::new().start_local().unwrap();

        jlrs.returning::<JlrsResult<_>>()
            .local_scope::<3>(|mut frame|{
                let assert_less_than_ptr =
                    Value::new(&mut frame, assert_less_than as *mut std::ffi::c_void);

                    unsafe {
                        let func = Value::eval_string(
                            &mut frame,
                            "throwing_func(fnptr::Ptr{Cvoid}) = ccall(fnptr, Cvoid, (Int32, Int32), 2, 1)",
                        )
                        .into_jlrs_result()?;

                    let output = func.call1(&mut frame, assert_less_than_ptr);
                    assert!(output.is_err());
                }

                Ok(())
            })
            .unwrap();
    }
}
