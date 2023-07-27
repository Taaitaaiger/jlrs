#[cfg(not(feature = "julia-1-6"))]
#[cfg_attr(
    all(
        any(windows, target_os = "windows", feature = "windows"),
        any(target_env = "msvc", feature = "yggdrasil"),
    ),
    link(name = "libjulia", kind = "raw-dylib")
)]
extern "C-unwind" {
    pub fn jl_setjmp(ptr: *mut ::std::ffi::c_void);
}
