use jlrs::prelude::*;
use std::{ffi::c_void, thread, time::Duration};
use thread::JoinHandle;

// NB: This crate is called `ccall_with_threads`, so the library is called
// `libccall_with_threads`. The functions are  annotated with `no_mangle` to prevent name mangling
// and `extern "C"` to make them callable with the C ABI.

// A pointer of type T that always implements `Send`. This is highly unsafe.
#[repr(transparent)]
pub struct SendablePtr<T>(*mut T);
unsafe impl<T> Send for SendablePtr<T> {}

#[no_mangle]
pub unsafe extern "C" fn multithreaded(
    out: SendablePtr<u32>,
    handle: SendablePtr<c_void>,
) -> *mut c_void {
    let handle = thread::spawn(move || {
        // NB: do NOT call Julia from this thread

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
