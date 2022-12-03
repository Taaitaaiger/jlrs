//! System and Julia version information.

use std::{ffi::CStr, ptr::NonNull};

use cfg_if::cfg_if;
use jl_sys::{
    jl_cpu_threads, jl_get_ARCH, jl_get_UNAME, jl_getallocationgranularity, jl_getpagesize,
    jl_git_branch, jl_git_commit, jl_is_debugbuild, jl_n_threads, jl_ver_is_release, jl_ver_major,
    jl_ver_minor, jl_ver_patch, jl_ver_string,
};

use crate::{
    data::managed::{private::ManagedPriv, symbol::Symbol},
    private::Private,
};

/// Global Julia information.
pub struct Info;

impl Info {
    /// Number of threads the CPU supports.
    pub fn n_cpu_threads() -> usize {
        unsafe { jl_cpu_threads() as usize }
    }

    /// Number of threads Julia can use.
    pub fn n_threads() -> usize {
        cfg_if! {
            if #[cfg(any(feature = "julia-1-10", feature = "julia-1-9"))] {
                unsafe { jl_n_threads.load(::std::sync::atomic::Ordering::Relaxed) as usize }
            } else {
                unsafe { jl_n_threads as usize }
            }
        }
    }

    /// The page size used by the garbage collector.
    pub fn page_size() -> usize {
        unsafe { jl_getpagesize() as usize }
    }

    /// The allocation granularity.
    pub fn allocation_granularity() -> usize {
        unsafe { jl_getallocationgranularity() as usize }
    }

    /// Returns `true` if a debug build of Julia is used.
    pub fn is_debugbuild() -> bool {
        unsafe { jl_is_debugbuild() != 0 }
    }

    /// Name and information of the kernel.
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

    /// The CPU architecture.
    pub fn arch() -> StrOrBytes<'static> {
        unsafe {
            let cstr =
                Symbol::wrap_non_null(NonNull::new_unchecked(jl_get_ARCH()), Private).as_cstr();

            if let Ok(rstr) = cstr.to_str() {
                Ok(rstr)
            } else {
                Err(cstr.to_bytes())
            }
        }
    }

    /// The major version of Julia.
    pub fn major_version() -> isize {
        unsafe { jl_ver_major() as isize }
    }

    /// The minor version of Julia.
    pub fn minor_version() -> isize {
        unsafe { jl_ver_minor() as isize }
    }

    /// The patch version of Julia.
    pub fn patch_version() -> isize {
        unsafe { jl_ver_patch() as isize }
    }

    /// Returns true if a release version of Julia is used.
    pub fn is_release() -> bool {
        unsafe { jl_ver_is_release() != 0 }
    }

    /// Returns the git branch that was used to compile the used version of Julia.
    pub fn git_branch() -> StrOrBytes<'static> {
        unsafe {
            let cstr = CStr::from_ptr(jl_git_branch());

            if let Ok(rstr) = cstr.to_str() {
                Ok(rstr)
            } else {
                Err(cstr.to_bytes())
            }
        }
    }

    /// Returns the git commit that was used to compile the used version of Julia.
    pub fn git_commit() -> &'static str {
        unsafe { CStr::from_ptr(jl_git_commit()).to_str().unwrap() }
    }

    /// Returns the version string of the used version of Julia.
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
