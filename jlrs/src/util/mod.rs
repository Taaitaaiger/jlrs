use std::ffi::{c_int, c_void};
#[cfg(not(any(windows, target_os = "windows", feature = "windows")))]
use std::{ffi::CStr, process::abort, ptr::null_mut, sync::atomic::AtomicPtr};

#[cfg(not(any(windows, target_os = "windows", feature = "windows")))]
use atomic::Ordering;
#[cfg(not(any(windows, target_os = "windows", feature = "windows")))]
use jl_sys::{jl_dlclose, jl_dlopen, jl_dlsym};

#[doc(hidden)]
#[cfg(feature = "local-rt")]
pub mod test;

pub trait RequireSendSync: 'static + Send + Sync {}

pub trait RequireSend: Send {}

pub(crate) type UvAsyncSendFn = unsafe extern "C" fn(handle: *mut c_void) -> c_int;

#[cfg(any(windows, target_os = "windows", feature = "windows"))]
#[cfg_attr(
    all(
        any(windows, target_os = "windows", feature = "windows"),
        any(target_env = "msvc", feature = "yggdrasil"),
        target_pointer_width = "64"
    ),
    link(name = "libuv-2", kind = "raw-dylib")
)]
extern "C" {
    pub fn uv_async_send(async_: *mut c_void) -> ::std::os::raw::c_int;
}

#[cfg(any(windows, target_os = "windows", feature = "windows"))]
pub(crate) unsafe fn uv_async_send_func() -> UvAsyncSendFn {
    uv_async_send
}

#[cfg(not(any(windows, target_os = "windows", feature = "windows")))]
pub(crate) unsafe fn uv_async_send_func() -> UvAsyncSendFn {
    static UV_ASYNC_SEND: AtomicPtr<c_void> = AtomicPtr::new(null_mut());
    let mut func = UV_ASYNC_SEND.load(Ordering::Relaxed);

    if func.is_null() {
        func = load_uv_async_send_func(&UV_ASYNC_SEND);
    }

    assert!(!func.is_null());

    std::mem::transmute::<_, UvAsyncSendFn>(func)
}

#[inline(never)]
#[cold]
#[cfg(not(any(windows, target_os = "windows", feature = "windows")))]
unsafe fn load_uv_async_send_func(tgt: &AtomicPtr<c_void>) -> *mut c_void {
    // The uv_async_send function normally requires linking libuv to use, which is kind of awkward
    // to do. Instead, we load it dynamically.
    unsafe {
        let handle = jl_dlopen(null_mut(), 8);
        let mut func: *mut c_void = null_mut();

        const SYM: &'static CStr = c"uv_async_send";
        let found = jl_dlsym(handle, SYM.as_ptr(), &mut func as *mut *mut c_void, 0) != 0;
        // Should be fine to close the handle immediately
        jl_dlclose(handle);

        if found {
            assert!(!func.is_null());
            tgt.store(func, Ordering::Relaxed);
            func
        } else {
            abort()
        }
    }
}
