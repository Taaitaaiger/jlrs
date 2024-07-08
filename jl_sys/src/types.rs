#![allow(non_camel_case_types)]

use std::{cell::Cell, ffi::c_void, ptr::null_mut};

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum jlrs_catch_tag_t {
    Ok = 0,
    Exception = 1,
    Panic = 2,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct jl_array_t {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct jl_datatype_layout_t {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct jl_datatype_t {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct jl_expr_t {
    _unused: [u8; 0],
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum jl_gc_collection_t {
    Auto = 0,
    Full = 1,
    Incremental = 2,
}

pub type GcCollection = jl_gc_collection_t;

#[repr(C)]
#[derive(Debug)]
pub struct jl_gcframe_t {
    pub(crate) n_roots: usize,
    pub(crate) prev: Cell<*mut c_void>,
}

impl jl_gcframe_t {
    #[inline]
    pub const fn new<const N: usize>() -> Self {
        jl_gcframe_t {
            n_roots: N << 2,
            prev: Cell::new(null_mut()),
        }
    }

    #[inline]
    pub const fn new_split(m: usize, n: usize) -> Self {
        jl_gcframe_t {
            n_roots: (m + n) << 2,
            prev: Cell::new(null_mut()),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
#[cfg(not(any(
    feature = "julia-1-6",
    feature = "julia-1-7",
    feature = "julia-1-8",
    feature = "julia-1-9",
    feature = "julia-1-10",
)))]
pub struct jl_genericmemory_t {
    _unused: [u8; 0],
}

pub type jl_markfunc_t =
    unsafe extern "C" fn(ptls: *mut jl_tls_states_t, obj: *mut jl_value_t) -> usize;
pub type jl_sweepfunc_t = unsafe extern "C" fn(obj: *mut jl_value_t);

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct jl_module_t {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct jlrs_catch_t {
    pub tag: jlrs_catch_tag_t,
    pub error: *mut c_void,
}

pub type jlrs_try_catch_trampoline_t =
    unsafe extern "C" fn(callback: *mut c_void, result: *mut c_void) -> jlrs_catch_t;
pub type jlrs_unsized_scope_trampoline_t = unsafe extern "C-unwind" fn(
    frame: *mut jl_gcframe_t,
    callback: *mut c_void,
    result: *mut c_void,
);

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct JL_STREAM {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct jl_svec_t {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct jl_sym_t {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct jl_task_t {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct jl_tls_states_t {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct jl_tvar_t {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct jl_typename_t {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct jl_unionall_t {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct jl_uniontype_t {
    _unused: [u8; 0],
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct jl_value_t {
    _unused: [u8; 0],
}
