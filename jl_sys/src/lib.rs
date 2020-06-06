#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

//! The documentation found on docs.rs corresponds to Julia version 1.4.1, however when
//! compiled locally, the bindings will match the version installed locally.

use std::ffi::c_void;
use std::mem::size_of;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[inline(always)]
pub unsafe fn jl_init() {
    jl_init__threading()
}

#[inline(always)]
pub unsafe fn jl_astaggedvalue(v: *mut jl_value_t) -> *mut jl_taggedvalue_t {
    let v_usize = v as *mut char as usize;
    let sz = size_of::<jl_taggedvalue_t>();

    (v_usize - sz) as *mut jl_taggedvalue_t
}

#[inline(always)]
pub unsafe fn jl_valueof(v: *mut jl_value_t) -> *mut jl_value_t {
    (v as *mut char as usize + size_of::<jl_taggedvalue_t>()) as *mut jl_value_t
}

#[inline(always)]
pub unsafe fn jl_typeof(v: *mut jl_value_t) -> *mut jl_value_t {
    ((*jl_astaggedvalue(v)).__bindgen_anon_1.header as usize & !15usize) as *mut jl_value_t
}

#[inline(always)]
pub unsafe fn jl_array_data(array: *mut jl_value_t) -> *mut c_void {
    (&*(array as *mut jl_array_t)).data as *mut std::ffi::c_void
}

#[inline(always)]
pub unsafe fn jl_typeis(v: *mut jl_value_t, t: *mut jl_datatype_t) -> bool {
    jl_typeof(v) == t as *mut jl_value_t
}

#[inline(always)]
pub unsafe fn jl_is_nothing(v: *mut jl_value_t) -> bool {
    v == jl_nothing.cast()
}

#[inline(always)]
pub unsafe fn jl_is_tuple(v: *mut jl_value_t) -> bool {
    (&*jl_typeof(v).cast::<jl_datatype_t>()).name == jl_tuple_typename
}

#[inline(always)]
pub unsafe fn jl_is_namedtuple(v: *mut jl_value_t) -> bool {
    (&*jl_typeof(v).cast::<jl_datatype_t>()).name == jl_namedtuple_typename
}

#[inline(always)]
pub unsafe fn jl_is_immutable(v: *mut jl_value_t) -> bool {
    (&*jl_typeof(v).cast::<jl_datatype_t>()).mutabl == 0
}

#[inline(always)]
pub unsafe fn jl_is_svec(v: *mut jl_value_t) -> bool {
    jl_typeis(v, jl_simplevector_type)
}

#[inline(always)]
pub unsafe fn jl_is_uniontype(v: *mut jl_value_t) -> bool {
    jl_typeis(v, jl_uniontype_type)
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
    jl_typeof(v) == jl_string_type as _
}

#[inline(always)]
pub unsafe fn jl_is_symbol(v: *mut jl_value_t) -> bool {
    jl_typeis(v, jl_symbol_type)
}

#[inline(always)]
pub unsafe fn jl_is_module(v: *mut jl_value_t) -> bool {
    jl_typeis(v, jl_module_type)
}

#[inline(always)]
pub unsafe fn jl_is_task(v: *mut jl_value_t) -> bool {
    jl_typeis(v, jl_task_type)
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
pub unsafe fn jl_array_dim(array: *mut jl_array_t, i: usize) -> usize {
    let x = &(&*array).nrows as *const usize;
    *x.add(i)
}

#[inline(always)]
pub unsafe fn jl_array_dims<'a>(array: *mut jl_array_t, ndims: usize) -> &'a [usize] {
    let x = &(&*array).nrows as *const usize;
    std::slice::from_raw_parts(x, ndims)
}

#[inline(always)]
pub unsafe fn jl_array_dim0(array: *mut jl_array_t) -> usize {
    (&*array).nrows
}

#[inline(always)]
pub unsafe fn jl_array_nrows(array: *mut jl_array_t) -> usize {
    (&*array).nrows
}

#[inline(always)]
pub unsafe fn jl_string_data(s: *mut jl_value_t) -> *const u8 {
    (s as *const u8).add(size_of::<usize>())
}

#[inline(always)]
pub unsafe fn jl_string_len(s: *mut jl_value_t) -> usize {
    *(s as *const usize)
}

#[inline(always)]
pub unsafe fn jl_field_names(st: *mut jl_datatype_t) -> *mut jl_svec_t {
    let st = &mut *st;
    if !st.names.is_null() {
        return st.names;
    }

    return (&mut *st.name).names;
}

#[inline(always)]
pub unsafe fn jl_svec_len(t: *mut jl_svec_t) -> usize {
    (&*t).length
}

#[inline(always)]
pub unsafe fn jl_svec_data(t: *mut jl_svec_t) -> *mut *mut jl_value_t {
    t.cast::<u8>().add(size_of::<jl_svec_t>()).cast()
}

macro_rules! llt_align {
    ($x:expr, $sz:expr) => {
        (($x) + ($sz) - 1) & !(($sz) - 1)
    };
}

#[inline(always)]
pub unsafe fn jl_symbol_name(s: *mut jl_sym_t) -> *mut u8 {
    s.cast::<u8>()
        .add(llt_align!(size_of::<jl_sym_t>(), size_of::<*mut c_void>()))
}

#[inline(always)]
pub unsafe fn jl_datatype_size(t: *mut jl_datatype_t) -> i32 {
    (&*(t)).size
}

#[inline(always)]
pub unsafe fn jl_datatype_align(t: *mut jl_datatype_t) -> u16 {
    (&*(&*(t)).layout).alignment
}

#[inline(always)]
pub unsafe fn jl_datatype_nbits(t: *mut jl_datatype_t) -> i32 {
    (&*(t)).size * 8
}

#[inline(always)]
pub unsafe fn jl_datatype_nfields(t: *mut jl_datatype_t) -> u32 {
    (&*(&*(t)).layout).nfields
}

#[inline(always)]
pub unsafe fn jl_nfields(v: *mut jl_value_t) -> u32 {
    jl_datatype_nfields(jl_typeof(v).cast())
}

#[inline(always)]
pub unsafe fn jl_datatype_isinlinealloc(t: *mut jl_datatype_t) -> u8 {
    (&*(t)).isinlinealloc
}

#[inline(always)]
pub unsafe fn jl_fieldref(s: *mut jl_value_t, i: usize) -> *mut jl_value_t {
    jl_get_nth_field(s, i)
}

#[inline(always)]
pub unsafe fn jl_fieldref_noalloc(s: *mut jl_value_t, i: usize) -> *mut jl_value_t {
    jl_get_nth_field_noalloc(s, i)
}

#[inline(always)]
#[cfg(feature = "stable")]
pub unsafe fn jl_get_fieldtypes(st: *mut jl_datatype_t) -> *mut jl_svec_t {
    if (&*st).types.is_null() {
        jl_compute_fieldtypes(st)
    } else {
        (&*st).types
    }
}

#[inline(always)]
#[cfg(feature = "beta")]
pub unsafe fn jl_get_fieldtypes(st: *mut jl_datatype_t) -> *mut jl_svec_t {
    if (&*st).types.is_null() {
        jl_compute_fieldtypes(st, std::ptr::null_mut())
    } else {
        (&*st).types
    }
}

#[inline(always)]
pub unsafe fn jl_is_kind(v: *mut jl_value_t) -> bool {
    v == jl_uniontype_type.cast()
        || v == jl_datatype_type.cast()
        || v == jl_unionall_type.cast()
        || v == jl_typeofbottom_type.cast()
}

#[inline(always)]
pub unsafe fn jl_is_type(v: *mut jl_value_t) -> bool {
    jl_is_kind(jl_typeof(v))
}

#[inline(always)]
pub unsafe fn jl_is_primitivetype(v: *mut jl_value_t) -> bool {
    jl_is_datatype(v)
        && jl_is_immutable(v)
        && !(&*v.cast::<jl_datatype_t>()).layout.is_null()
        && jl_datatype_nfields(v.cast()) == 0
        && jl_datatype_size(v.cast()) > 0
}

#[inline(always)]
pub unsafe fn jl_is_structtype(v: *mut jl_value_t) -> bool {
    jl_is_datatype(v)
        && jl_is_immutable(v)
        && (&*v.cast::<jl_datatype_t>()).abstract_ == 0
        && jl_datatype_nfields(v.cast()) == 0
        && jl_datatype_size(v.cast()) > 0
}

#[inline(always)]
pub unsafe fn jl_is_datatype_singleton(v: *mut jl_datatype_t) -> bool {
    !(&*v).instance.is_null()
}

#[inline(always)]
pub unsafe fn jl_is_abstracttype(v: *mut c_void) -> bool {
    jl_is_datatype(v.cast()) && (&*v.cast::<jl_datatype_t>()).abstract_ > 0
}

#[inline(always)]
pub unsafe fn jl_isbits(t: *mut c_void) -> bool {
    jl_is_datatype(t.cast()) && (&*t.cast::<jl_datatype_t>()).isbitstype != 0
}

#[inline(always)]
pub unsafe fn jl_is_cpointer_type(v: *mut c_void) -> bool {
    jl_is_datatype(v.cast())
        && (&*v.cast::<jl_datatype_t>()).name
            == (&*(&*jl_pointer_type).body.cast::<jl_datatype_t>()).name
}

#[inline(always)]
pub unsafe fn jl_is_addrspace_ptr_type(v: *mut c_void) -> bool {
    jl_is_datatype(v.cast()) && (&*v.cast::<jl_datatype_t>()).name == jl_addrspace_pointer_typename
}

#[inline(always)]
pub unsafe fn jl_is_abstract_ref_type(v: *mut c_void) -> bool {
    jl_is_datatype(v.cast())
        && (&*v.cast::<jl_datatype_t>()).name
            == (&*(&*jl_ref_type).body.cast::<jl_datatype_t>()).name
}

#[inline(always)]
pub unsafe fn jl_is_tuple_type(v: *mut c_void) -> bool {
    jl_is_datatype(v.cast()) && (&*v.cast::<jl_datatype_t>()).name == jl_tuple_typename
}

#[inline(always)]
pub unsafe fn jl_is_namedtuple_type(v: *mut c_void) -> bool {
    jl_is_datatype(v.cast()) && (&*v.cast::<jl_datatype_t>()).name == jl_namedtuple_typename
}

#[inline(always)]
pub unsafe fn jl_is_vecelement_type(v: *mut c_void) -> bool {
    jl_is_datatype(v.cast()) && (&*v.cast::<jl_datatype_t>()).name == jl_vecelement_typename
}

#[inline(always)]
pub unsafe fn jl_is_type_type(v: *mut c_void) -> bool {
    jl_is_datatype(v.cast())
        && (&*v.cast::<jl_datatype_t>()).name
            == (&*(&*jl_type_type).body.cast::<jl_datatype_t>()).name
}

#[inline(always)]
pub unsafe fn jl_array_isbitsunion(a: *mut jl_array_t) -> bool {
    (&*a).flags.ptrarray() > 0 && jl_is_uniontype(jl_tparam0(jl_typeof(a.cast()).cast()))
}

#[inline(always)]
pub unsafe fn jl_nparams(t: *mut jl_datatype_t) -> usize {
    jl_svec_len((&*t).parameters)
}

#[inline(always)]
pub unsafe fn jl_tparam0(t: *mut jl_datatype_t) -> *mut jl_value_t {
    jl_svecref((&*t).parameters.cast(), 0)
}

#[inline(always)]
pub unsafe fn jl_tparam1(t: *mut jl_datatype_t) -> *mut jl_value_t {
    jl_svecref((&*t).parameters.cast(), 1)
}

#[inline(always)]
pub unsafe fn jl_tparam(t: *mut jl_datatype_t, i: usize) -> *mut jl_value_t {
    jl_svecref((&*t).parameters.cast(), i)
}

#[inline(always)]
pub unsafe fn jl_svecref(t: *mut c_void, i: usize) -> *mut jl_value_t {
    assert!(jl_typeis(t.cast(), jl_simplevector_type));
    assert!(i < jl_svec_len(t.cast()));
    std::slice::from_raw_parts_mut(jl_svec_data(t.cast()), jl_svec_len(t.cast()))[i]
}

/*
#define jl_dt_layout_fields(d) ((const char*)(d) + sizeof(jl_datatype_layout_t))

static inline uint32_t jl_fielddesc_size(int8_t fielddesc_type) JL_NOTSAFEPOINT
{
    return 2 << fielddesc_type;
    //if (fielddesc_type == 0) {
    //    return sizeof(jl_fielddesc8_t);
    //}
    //else if (fielddesc_type == 1) {
    //    return sizeof(jl_fielddesc16_t);
    //}
    //else {
    //    return sizeof(jl_fielddesc32_t);
    //}
}

static inline int jl_field_isptr(jl_datatype_t *st, int i) JL_NOTSAFEPOINT
{
    const jl_datatype_layout_t *ly = st->layout;
    assert(i >= 0 && (size_t)i < ly->nfields);
    return ((const jl_fielddesc8_t*)(jl_dt_layout_fields(ly) + jl_fielddesc_size(ly->fielddesc_type) * i))->isptr;
}
*/

pub unsafe fn jl_dt_layout_fields(d: *const u8) -> *const u8 {
    d.add(size_of::<jl_datatype_layout_t>())
}

pub unsafe fn jl_fielddesc_size(fielddesc_type: i8) -> u32 {
    2 << fielddesc_type
}

pub unsafe fn jl_field_isptr(st: *mut jl_datatype_t, i: i32) -> bool {
    let ly = &*(&*st).layout;
    assert!(i >= 0 && (i as u32) < ly.nfields);
    (&*jl_dt_layout_fields(ly as *const _ as *const u8)
        .add(jl_fielddesc_size(ly.fielddesc_type() as i8) as usize * i as usize)
        .cast::<jl_fielddesc8_t>())
        .isptr()
        != 0
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
