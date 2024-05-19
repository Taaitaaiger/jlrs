//! System and Julia version information.

use std::{ffi::CStr, ptr::NonNull};

use jl_sys::{
    jl_cpu_threads, jl_get_UNAME, jl_is_debugbuild, jl_n_threads, jl_ver_is_release, jl_ver_major,
    jl_ver_minor, jl_ver_patch, jl_ver_string,
};
use jlrs_macros::julia_version;

use crate::{
    data::managed::{private::ManagedPriv, symbol::Symbol},
    private::Private,
};

/// Global Julia information.
pub struct Info;

impl Info {
    /// Number of threads the CPU supports.
    #[inline]
    pub fn n_cpu_threads() -> usize {
        unsafe { jl_cpu_threads() as usize }
    }

    #[julia_version(until = "1.8")]
    #[inline]
    /// Number of threads Julia can use.
    pub fn n_threads() -> usize {
        unsafe { jl_n_threads as usize }
    }

    #[julia_version(since = "1.9")]
    #[inline]
    /// Number of threads Julia can use.
    pub fn n_threads() -> usize {
        unsafe { jl_n_threads.load(::std::sync::atomic::Ordering::Relaxed) as usize }
    }

    #[julia_version(since = "1.9")]
    /// Number of threads per thread pool.
    pub fn n_threads_per_pool() -> &'static [u32] {
        unsafe {
            let n_pools = jl_sys::jl_n_threadpools.get() as usize;
            let n_threads_per_pool = jl_sys::jl_n_threads_per_pool.get();
            std::slice::from_raw_parts(n_threads_per_pool as _, n_pools)
        }
    }

    #[julia_version(since = "1.10")]
    #[inline]
    /// Number of GC threads Julia can use.
    pub fn n_gc_threads() -> usize {
        unsafe { jl_sys::jl_n_gcthreads as usize }
    }

    /// Returns `true` if a debug build of Julia is used.
    #[inline]
    pub fn is_debugbuild() -> bool {
        unsafe { jl_is_debugbuild() != 0 }
    }

    /// Name and information of the kernel.
    #[inline]
    pub fn uname() -> StrOrBytes<'static> {
        unsafe {
            let cstr =
                Symbol::wrap_non_null(NonNull::new_unchecked(jl_get_UNAME()), Private).as_cstr();

            if let Ok(rstr) = cstr.to_str() {
                Ok(rstr)
            } else {
                Err(cstr.to_bytes())
            }
        }
    }

    // /// The CPU architecture.
    // #[inline]
    // pub fn arch() -> StrOrBytes<'static> {
    //     unsafe {
    //         let cstr =
    //             Symbol::wrap_non_null(NonNull::new_unchecked(jl_get_ARCH()), Private).as_cstr();

    //         if let Ok(rstr) = cstr.to_str() {
    //             Ok(rstr)
    //         } else {
    //             Err(cstr.to_bytes())
    //         }
    //     }
    // }

    /// The major version of Julia.
    #[inline]
    pub fn major_version() -> isize {
        unsafe { jl_ver_major() as isize }
    }

    /// The minor version of Julia.
    #[inline]
    pub fn minor_version() -> isize {
        unsafe { jl_ver_minor() as isize }
    }

    /// The patch version of Julia.
    #[inline]
    pub fn patch_version() -> isize {
        unsafe { jl_ver_patch() as isize }
    }

    /// Returns true if a release version of Julia is used.
    #[inline]
    pub fn is_release() -> bool {
        unsafe { jl_ver_is_release() != 0 }
    }

    /// Returns the version string of the used version of Julia.
    #[inline]
    pub fn version_string() -> &'static str {
        unsafe { CStr::from_ptr(jl_ver_string()).to_str().unwrap() }
    }
}

/// Alias for a result that contains either a valid UTF8-encoded string slice, or the raw byte
/// slice if the contents are not valid UTF8.
pub type StrOrBytes<'scope> = Result<&'scope str, &'scope [u8]>;

#[cfg(test)]
mod test {
    use super::Info;

    #[test]
    fn is_global() {
        assert_eq!(Info::major_version(), 1);
        assert_eq!(Info::n_threads(), 0);
    }
}
