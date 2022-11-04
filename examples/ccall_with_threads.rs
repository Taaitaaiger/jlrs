// FIXME TODO this is awful

use jlrs::prelude::*;
use std::{ffi::c_void, thread, time::Duration};
use thread::JoinHandle;

// This crate is called `ccall_with_threads`, so the library is called
// `libccall_with_threads`. The functions are  annotated with `no_mangle` to prevent name mangling
// and `extern "C"` to make them callable with the C ABI.

// A pointer of type T that always implements `Send`..
#[repr(transparent)]
pub struct SendablePtr<T>(*mut T);
unsafe impl<T> Send for SendablePtr<T> {}

#[no_mangle]
pub unsafe extern "C" fn multithreaded(
    out: SendablePtr<u32>,
    handle: SendablePtr<c_void>,
) -> *mut c_void {
    let handle = thread::spawn(move || {
        // Never call Julia from this thread!

        // Pretend we're doing something expensive
        thread::sleep(Duration::from_secs(1));
        // Write some result
        std::ptr::write(out.0, 127);
        // Notify Julia
        CCall::uv_async_send(handle.0);
    });

    // Box and return the JoinHandle as a pointer.
    // The handle must be dropped by calling `drop_handle`.
    let boxed = Box::new(handle);
    Box::leak(boxed) as *mut _ as *mut _
}

#[no_mangle]
pub unsafe extern "C" fn drop_handle(handle: *mut JoinHandle<()>) {
    Box::from_raw(handle).join().ok();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    thread_local! {
        pub static JULIA: RefCell<PendingJulia> = {
            RefCell::new(unsafe { RuntimeBuilder::new().start().unwrap() })
        };
    }

    #[test]
    fn call_multithreaded() {
        JULIA.with(|j| {
            let mut jlrs = j.borrow_mut();
            let mut frame = StackFrame::new();
            let mut jlrs = jlrs.instance(&mut frame);

            jlrs.scope(|mut frame| unsafe {
                let multithreaded_ptr = Value::new(&mut frame, multithreaded as *mut std::ffi::c_void);
                let drop_handle_ptr = Value::new(&mut frame, drop_handle as *mut std::ffi::c_void);

                let func = Value::eval_string(
                    &mut frame,
                    "function run(multithreaded_ptr::Ptr{Cvoid}, drop_handle_ptr::Ptr{Cvoid})
                        task = @async begin
                            condition = Base.AsyncCondition()
                            output::Ref{UInt32} = C_NULL
                            joinhandle = ccall(multithreaded_ptr, Ptr{Cvoid}, (Ref{UInt32}, Ptr{Cvoid}), output, condition.handle)
                            wait(condition)
                            ccall(drop_handle_ptr, Cvoid, (Ptr{Cvoid},), joinhandle)

                            output[]
                        end

                        task2 = @async begin
                            while !istaskdone(task)
                                sleep(0.1)
                            end

                            @assert fetch(task) == 127 \"Wrong result\"
                        end

                        wait(task)
                        wait(task2)
                    end"
                ).into_jlrs_result()?;

                let output = func.call2(&frame, multithreaded_ptr, drop_handle_ptr);
                assert!(output.is_ok());

                Ok(())
            }).unwrap();
        })
    }
}
