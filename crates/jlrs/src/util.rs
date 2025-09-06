use std::ffi::{c_int, c_void};
#[cfg(not(any(windows, target_os = "windows", feature = "windows")))]
use std::{ffi::CStr, process::abort, ptr::null_mut, sync::atomic::AtomicPtr};

#[cfg(not(any(windows, target_os = "windows", feature = "windows")))]
use atomic::Ordering;
#[cfg(not(any(windows, target_os = "windows", feature = "windows")))]
use jl_sys::{jl_dlclose, jl_dlopen, jl_dlsym};
use jl_sys::{jl_get_pgcstack, jl_value_t};

use crate::prelude::Target;

pub trait RequireSendSync: 'static + Send + Sync {}

pub trait RequireSend: Send {}

pub(crate) type UvAsyncSendFn = unsafe extern "C" fn(handle: *mut c_void) -> c_int;

#[cfg(any(windows, target_os = "windows", feature = "windows"))]
#[cfg_attr(
    all(
        any(windows, target_os = "windows", feature = "windows"),
        target_pointer_width = "64"
    ),
    link(name = "libuv-2", kind = "raw-dylib")
)]
unsafe extern "C" {
    pub fn uv_async_send(async_: *mut c_void) -> ::std::os::raw::c_int;
}

#[cfg(any(windows, target_os = "windows", feature = "windows"))]
pub(crate) unsafe fn uv_async_send_func() -> UvAsyncSendFn {
    uv_async_send
}

#[cfg(not(any(windows, target_os = "windows", feature = "windows")))]
pub(crate) unsafe fn uv_async_send_func() -> UvAsyncSendFn {
    unsafe {
        static UV_ASYNC_SEND: AtomicPtr<c_void> = AtomicPtr::new(null_mut());
        let mut func = UV_ASYNC_SEND.load(Ordering::Relaxed);

        if func.is_null() {
            func = load_uv_async_send_func(&UV_ASYNC_SEND);
        }

        assert!(!func.is_null());

        std::mem::transmute::<_, UvAsyncSendFn>(func)
    }
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

#[cfg(any(julia_1_10, julia_1_11))]
#[inline(always)]
pub(crate) fn kwcall_function<'target, Tgt>(_target: &Tgt) -> *mut jl_value_t
where
    Tgt: Target<'target>,
{
    unsafe { jl_sys::jl_kwcall_func }
}

#[cfg(not(any(julia_1_10, julia_1_11)))]
#[inline(always)]
pub(crate) fn kwcall_function<'target, Tgt>(target: &Tgt) -> *mut jl_value_t
where
    Tgt: Target<'target>,
{
    use crate::data::managed::private::ManagedPriv as _;

    crate::inline_static_ref!(KWCALL, crate::prelude::Value, "Core.kwcall", target)
        .unwrap(crate::private::Private)
}

#[doc(hidden)]
pub unsafe fn pgcstack() -> *mut *mut jl_sys::jl_gcframe_t {
    unsafe { jl_get_pgcstack() }
}
