#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::c_void;
use std::mem::size_of;
use std::ptr::{null, null_mut};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[inline(always)]
pub unsafe fn jl_init() {
    jl_init__threading()
}

#[inline(always)]
pub unsafe fn jl_astaggedvalue(v: *mut jl_value_t) -> *mut jl_taggedvalue_t {
    if v == null_mut() {
        return null_mut();
    }

    (v as *mut char as usize - size_of::<jl_taggedvalue_t>()) as *mut jl_taggedvalue_t
}

#[inline(always)]
pub unsafe fn jl_valueof(v: *mut jl_value_t) -> *mut jl_value_t {
    if v == null_mut() {
        return null_mut();
    }

    (v as *mut char as usize + size_of::<jl_taggedvalue_t>()) as *mut jl_value_t
}

#[inline(always)]
pub unsafe fn jl_typeof(v: *mut jl_value_t) -> *mut jl_value_t {
    if v == null_mut() {
        return null_mut();
    }

    ((*jl_astaggedvalue(v)).__bindgen_anon_1.header as usize & !15usize) as *mut jl_value_t
}

#[inline(always)]
pub unsafe fn jl_array_data(array: *mut jl_value_t) -> *mut c_void {
    if array == null_mut() {
        return null_mut();
    }

    (&*(array as *mut jl_array_t)).data as *mut std::ffi::c_void
}

#[inline(always)]
pub unsafe fn jl_typeis(v: *mut jl_value_t, t: *mut jl_datatype_t) -> bool {
    jl_typeof(v) == t as *mut jl_value_t
}

#[inline(always)]
pub unsafe fn jl_is_datatype(v: *mut jl_value_t) -> bool {
    jl_typeis(v, jl_datatype_type)
}

#[inline(always)]
pub unsafe fn jl_is_array_type(v: *mut jl_value_t) -> bool {
    jl_is_datatype(v) && (&*(v as *mut jl_datatype_t)).name == jl_array_typename
}

#[inline(always)]
pub unsafe fn jl_is_array(v: *mut jl_value_t) -> bool {
    jl_is_array_type(jl_typeof(v))
}

#[inline(always)]
pub unsafe fn jl_is_string(v: *mut jl_value_t) -> bool {
    jl_typeof(v) == jl_string_type  as _
}

#[inline(always)]
pub unsafe fn jl_gc_wb(parent: *mut jl_value_t, ptr: *mut jl_value_t) {
    let parent = &*jl_astaggedvalue(parent);
    let ptr = &*jl_astaggedvalue(ptr);

    if parent.__bindgen_anon_1.bits.gc() == 3 && (ptr.__bindgen_anon_1.bits.gc() & 1) == 0 {
        jl_gc_queue_root(parent as *const jl_taggedvalue_t as *mut jl_value_t)
    }
}

#[inline(always)]
pub unsafe fn jl_array_ndims(array: *mut jl_array_t) -> u16 {
    (&*array).flags.ndims()
}

#[inline(always)]
pub unsafe fn jl_array_dim(array: *mut jl_array_t, i: usize) -> u64 {
    let x = &(&*array).nrows as *const u64;
    *x.offset(i as isize)
}

#[inline(always)]
pub unsafe fn jl_array_dims<'a>(array: *mut jl_array_t, ndims: usize) -> &'a [u64] {
    let x = &(&*array).nrows as *const u64;
    std::slice::from_raw_parts(x, ndims)
}

#[inline(always)]
pub unsafe fn jl_array_dim0(array: *mut jl_array_t) -> u64 {
    (&*array).nrows
}

#[inline(always)]
pub unsafe fn jl_array_nrows(array: *mut jl_array_t) -> u64 {
    (&*array).nrows
}

#[inline(always)]
pub unsafe fn jl_string_data(s: *mut jl_value_t) -> *const u8 {
    if s == null_mut() {
        return null();
    }

    (s as *const u8).offset(size_of::<usize>() as _)
}

#[inline(always)]
pub unsafe fn jl_string_len(s: *mut jl_value_t) -> usize {
    if s == null_mut() {
        return 0;
    }

    *(s as *const usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanity() {
        unsafe {
            jl_init();
            assert!(jl_is_initialized() != 0);

            assert!(jl_exception_occurred().is_null());

            jl_atexit_hook(0);
        }
    }
}
