#![allow(non_camel_case_types)]

use std::ffi::c_void;

pub use jl_sys::types::*;

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum jlrs_catch_tag_t {
    Ok = 0,
    Exception = 1,
    Panic = 2,
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
