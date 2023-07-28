use std::{
    ffi::c_void,
    sync::atomic::{AtomicU32, Ordering},
    thread,
    time::Duration,
};

use jlrs::prelude::*;
use thread::JoinHandle;

// This crate is called `ccall_with_threads`, so the library is called
// `libccall_with_threads`. The functions are annotated with `no_mangle` to prevent name mangling
// and `extern "C"` to make them callable with the C ABI.

/// The handle field of an `AsyncCondition`.
#[repr(transparent)]
pub struct UvHandle(*mut c_void);
unsafe impl Send for UvHandle {}
unsafe impl Sync for UvHandle {}

/// This function spawns a new thread, sleeps for a second, stores a result, and wakes Julia.
///
/// This function can be called with a reference to an `AtomicU32` and a `UvHandle`. This first
/// can be created as follows:
///
/// ```julia
/// mutable struct AtomicUInt32
///     @atomic v::UInt32
/// end
/// ```
///
/// The latter is the handle field of an `AsyncCondition`.
///
/// The reference to the atomic data has a static lifetime, what's required is that this data
/// lives while the spawned thread is active. This can be enforced with `GC.@preserve`.
///
/// The handle to the thread is boxed and leaked before it's returned. It must be cleaned up with
/// a call to `drop_handle` after the spawned thread has finished.
///
/// Putting all of this together:
///
/// ```julia
/// function run()
///     task = @async begin
///         condition = Base.AsyncCondition()
///         output = AtomicUInt32(0)
///
///         GC.@preserve output begin
///             joinhandle = ccall(("multithreaded", :libccall_with_threads), Ptr{Cvoid},
///                 (Any, Ptr{Cvoid}), output, condition.handle)
///             wait(condition)
///             ccall(("drop_handle", :libccall_with_threads), Cvoid, (Ptr{Cvoid},), joinhandle)
///
///             @atomic output.v
///         end
///      end
///
///      task2 = @async begin
///         @assert fetch(task) == 127 \"Wrong result\"
///      end
///
///      wait(task)
///      wait(task2)
/// end
/// ```
#[no_mangle]
pub unsafe extern "C" fn multithreaded(out: &'static AtomicU32, handle: UvHandle) -> *mut c_void {
    let handle = thread::spawn(move || {
        // Never call Julia from this thread!

        // Pretend we're doing something expensive
        thread::sleep(Duration::from_secs(1));
        // Write some result
        out.store(127, Ordering::SeqCst);
        // Notify Julia
        CCall::uv_async_send(handle.0);
    });

    // Box and return the JoinHandle as a pointer.
    let boxed = Box::new(handle);
    Box::leak(boxed) as *mut _ as *mut _
}

#[no_mangle]
pub unsafe extern "C" fn drop_handle(handle: *mut JoinHandle<()>) {
    Box::from_raw(handle).join().ok();
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

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

                // use Base.Threads.Atomic for lts compat
                let func = Value::eval_string(
                    &mut frame,
                    "function run(multithreaded_ptr::Ptr{Cvoid}, drop_handle_ptr::Ptr{Cvoid})
                        task = @async begin
                            condition = Base.AsyncCondition()
                            output = Base.Threads.Atomic{UInt32}(0)
                            GC.@preserve output begin
                                joinhandle = ccall(multithreaded_ptr, Ptr{Cvoid}, (Any, Ptr{Cvoid}), output, condition.handle)
                                wait(condition)
                                ccall(drop_handle_ptr, Cvoid, (Ptr{Cvoid},), joinhandle)

                                output[]
                            end
                        end

                        task2 = @async begin
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
