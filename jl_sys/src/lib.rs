#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

//! The documentation found on docs.rs corresponds to Julia version 1.4.1, however when
//! compiled locally, the bindings will match the version installed locally.

macro_rules! llt_align {
    ($x:expr, $sz:expr) => {
        (($x) + ($sz) - 1) & !(($sz) - 1)
    };
}

use std::ffi::c_void;
use std::mem::size_of;
use std::sync::atomic::{AtomicPtr, Ordering};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

/*mod dynamic {
    use libc::{dlopen, dlsym, RTLD_NOW, RTLD_GLOBAL, RTLD_NODELETE};

    macro_rules! define_dynamic_function {
        ($name:ident, ($($args:ty,)+), $out:ty) => {
            pub static mut $name: ::std::mem::MaybeUninit<unsafe extern "C" fn($(args),+) -> $out> = ::std::mem::MaybeUninit::uninit()
        };
        ($name:ident, ($args:ty), $out:ty) => {
            pub static mut $name: ::std::mem::MaybeUninit<unsafe extern "C" fn($args) -> $out> = ::std::mem::MaybeUninit::uninit();
        }
    }

    define_dynamic_function!(jl_eval_string, (*const ::std::os::raw::c_char), *mut super::jl_value_t);
    // pub static mut jl_eval_string: std::mem::MaybeUninit<unsafe extern "C" fn(*const std::os::raw::c_char) -> *mut super::jl_value_t> = unsafe { std::mem::transmute(std::ptr::null_mut::<std::ffi::c_void>()) };

    pub unsafe fn load() {
        let path = std::ffi::CStr::from_bytes_with_nul_unchecked(b"/home/thomas/julia-1.5.3/lib/libjulia.so\0");
        let lib = dlopen(path.as_ptr(), RTLD_NOW | RTLD_GLOBAL | RTLD_NODELETE);

        {
            let fnname = std::ffi::CStr::from_bytes_with_nul_unchecked(b"jl_eval_string\0");
            let sym: *mut std::ffi::c_void = dlsym(lib, fnname.as_ptr()).cast();
            assert!(!sym.is_null());
            jl_eval_string = std::mem::transmute(sym);
        }
    }
}

pub fn dyn_eval_string() {
    unsafe {
        dynamic::load();
        let s = std::ffi::CStr::from_bytes_with_nul_unchecked(b"println(\"hello dyn\")\0");
        dynamic::jl_eval_string.assume_init()(s.as_ptr());
    }
}*/

// define
/*
#define container_of(ptr, type, member) \
    ((type *) ((char *)(ptr) - offsetof(type, member)))
*/
// Not implemented

/*
#define jl_astaggedvalue(v)                                             \
    ((jl_taggedvalue_t*)((char*)(v) - sizeof(jl_taggedvalue_t)))
*/
#[inline(always)]
pub unsafe fn jl_astaggedvalue(v: *mut jl_value_t) -> *mut jl_taggedvalue_t {
    let v_usize = v as *mut char as usize;
    let sz = size_of::<jl_taggedvalue_t>();

    (v_usize - sz) as *mut jl_taggedvalue_t
}

/*
#define jl_valueof(v)                                           \
    ((jl_value_t*)((char*)(v) + sizeof(jl_taggedvalue_t)))
*/
#[inline(always)]
pub unsafe fn jl_valueof(v: *mut jl_value_t) -> *mut jl_value_t {
    (v as *mut char as usize + size_of::<jl_taggedvalue_t>()) as *mut jl_value_t
}

/*
#define jl_typeof(v)                                                    \
    ((jl_value_t*)(jl_astaggedvalue(v)->header & ~(uintptr_t)15))
*/
#[inline(always)]
pub unsafe fn jl_typeof(v: *mut jl_value_t) -> *mut jl_value_t {
    ((*jl_astaggedvalue(v)).__bindgen_anon_1.header as usize & !15usize) as *mut jl_value_t
}

/*
#define jl_typeis(v,t) (jl_typeof(v)==(jl_value_t*)(t))
*/
#[inline(always)]
pub unsafe fn jl_typeis(v: *mut jl_value_t, t: *mut jl_datatype_t) -> bool {
    jl_typeof(v) == t as *mut jl_value_t
}

/*
#define jl_tuple_type jl_anytuple_type
*/
// Not implemented

/*
#define jl_pgcstack (jl_get_ptls_states()->pgcstack)
*/
// Not implemented

/*
#define jl_svec_len(t)              (((jl_svec_t*)(t))->length)
*/
#[inline(always)]
pub unsafe fn jl_svec_len(t: *mut jl_svec_t) -> usize {
    (&*t).length
}

/*
#define jl_svec_set_len_unsafe(t,n) (((jl_svec_t*)(t))->length=(n))
*/
// Not implemented

/*
#define jl_svec_data(t) ((jl_value_t**)((char*)(t) + sizeof(jl_svec_t)))
*/
#[inline(always)]
pub unsafe fn jl_svec_data(t: *mut jl_svec_t) -> *mut *mut jl_value_t {
    t.cast::<u8>().add(size_of::<jl_svec_t>()).cast()
}

/*
#define jl_array_len(a)   (((jl_array_t*)(a))->length)
*/
#[inline(always)]
pub unsafe fn jl_array_len(a: *mut jl_array_t) -> usize {
    (&*a).length
}

/*
#define jl_array_data(a)  ((void*)((jl_array_t*)(a))->data)
*/
#[inline(always)]
pub unsafe fn jl_array_data(array: *mut jl_value_t) -> *mut c_void {
    (&*(array as *mut jl_array_t)).data as *mut std::ffi::c_void
}

/*
#define jl_array_dim(a,i) ((&((jl_array_t*)(a))->nrows)[i])
*/
#[inline(always)]
pub unsafe fn jl_array_dim(array: *mut jl_array_t, i: usize) -> usize {
    let x = &(&*array).nrows as *const usize;
    *x.add(i)
}

/*
#define jl_array_dim0(a)  (((jl_array_t*)(a))->nrows)
*/
#[inline(always)]
pub unsafe fn jl_array_dim0(array: *mut jl_array_t) -> usize {
    (&*array).nrows
}

/*
#define jl_array_nrows(a) (((jl_array_t*)(a))->nrows)
*/
#[inline(always)]
pub unsafe fn jl_array_nrows(array: *mut jl_array_t) -> usize {
    (&*array).nrows
}

/*
#define jl_array_ndims(a) ((int32_t)(((jl_array_t*)a)->flags.ndims))
*/
#[inline(always)]
pub unsafe fn jl_array_ndims(array: *mut jl_array_t) -> u16 {
    (&*array).flags.ndims()
}

/*
#define jl_array_data_owner_offset(ndims) (offsetof(jl_array_t,ncols) + sizeof(size_t)*(1+jl_array_ndimwords(ndims))) // in bytes
*/
pub unsafe fn jl_array_data_owner_offset(ndims: u16) -> usize {
    // While there is a memoffset crate which provides the functionality offsetof does, it's UB.
    // Until a sound alternative is available, calculate the offset manually.
    // Assumption: JL_ARRAY_LEN is defined.

    // data
    size_of::<*mut c_void>() +
    // length
    size_of::<usize>() +
    // flags
    2 +
    //elsize
    2 +
    // offset
    4 +
    // nrows
    size_of::<usize>() +
    size_of::<usize>() * (1 + jl_array_ndimwords(ndims as _)) as usize
}

/*
#define jl_array_data_owner(a) (*((jl_value_t**)((char*)a + jl_array_data_owner_offset(jl_array_ndims(a)))))
*/
pub unsafe fn jl_array_data_owner(a: *mut jl_array_t) -> *mut jl_value_t {
    a.cast::<u8>()
        .add(jl_array_data_owner_offset(jl_array_ndims(a)))
        .cast::<jl_value_t>()
}

/*
#define jl_array_ptr_data(a)  ((jl_value_t**)((jl_array_t*)(a))->data)
*/
// Not implemented

/*
#define jl_exprarg(e,n) jl_array_ptr_ref(((jl_expr_t*)(e))->args, n)
*/
// Not implemented

/*
#define jl_exprargset(e, n, v) jl_array_ptr_set(((jl_expr_t*)(e))->args, n, v)
*/
// Not implemented

/*
#define jl_expr_nargs(e) jl_array_len(((jl_expr_t*)(e))->args)
*/
// Not implemented

/*
#define jl_fieldref(s,i) jl_get_nth_field(((jl_value_t*)(s)),i)
*/
#[inline(always)]
pub unsafe fn jl_fieldref(s: *mut jl_value_t, i: usize) -> *mut jl_value_t {
    jl_get_nth_field(s, i)
}

/*
#define jl_fieldref_noalloc(s,i) jl_get_nth_field_noalloc(((jl_value_t*)(s)),i)
*/
#[inline(always)]
pub unsafe fn jl_fieldref_noalloc(s: *mut jl_value_t, i: usize) -> *mut jl_value_t {
    jl_get_nth_field_noalloc(s, i)
}

/*
#define jl_nfields(v)    jl_datatype_nfields(jl_typeof(v))
*/
#[inline(always)]
pub unsafe fn jl_nfields(v: *mut jl_value_t) -> u32 {
    jl_datatype_nfields(jl_typeof(v).cast())
}

/*
#define jl_linenode_line(x) (((intptr_t*)(x))[0])
*/
// Not implemented

/*
#define jl_linenode_file(x) (((jl_value_t**)(x))[1])
*/
// Not implemented

/*
#define jl_slot_number(x) (((intptr_t*)(x))[0])
*/
// Not implemented

/*
#define jl_typedslot_get_type(x) (((jl_value_t**)(x))[1])
*/
// Not implemented

/*
#define jl_gotonode_label(x) (((intptr_t*)(x))[0])
*/
// Not implemented

/*
#define jl_globalref_mod(s) (*(jl_module_t**)(s))
*/
// Not implemented

/*
#define jl_globalref_name(s) (((jl_sym_t**)(s))[1])
*/
// Not implemented

/*
#define jl_quotenode_value(x) (((jl_value_t**)x)[0])
*/
// Not implemented

/*
#define jl_nparams(t)  jl_svec_len(((jl_datatype_t*)(t))->parameters)
*/
#[inline(always)]
pub unsafe fn jl_nparams(t: *mut jl_datatype_t) -> usize {
    jl_svec_len((&*t).parameters)
}

/*
#define jl_tparam0(t)  jl_svecref(((jl_datatype_t*)(t))->parameters, 0)
*/
#[inline(always)]
pub unsafe fn jl_tparam0(t: *mut jl_datatype_t) -> *mut jl_value_t {
    jl_svecref((&*t).parameters.cast(), 0)
}

/*
#define jl_tparam1(t)  jl_svecref(((jl_datatype_t*)(t))->parameters, 1)
*/
#[inline(always)]
pub unsafe fn jl_tparam1(t: *mut jl_datatype_t) -> *mut jl_value_t {
    jl_svecref((&*t).parameters.cast(), 1)
}

/*
#define jl_tparam(t,i) jl_svecref(((jl_datatype_t*)(t))->parameters, i)
*/
#[inline(always)]
pub unsafe fn jl_tparam(t: *mut jl_datatype_t, i: usize) -> *mut jl_value_t {
    jl_svecref((&*t).parameters.cast(), i)
}

/*
#define jl_data_ptr(v)  ((jl_value_t**)v)
*/
// Not implemented

/*
#define jl_string_data(s) ((char*)s + sizeof(void*))
*/
#[inline(always)]
pub unsafe fn jl_string_data(s: *mut jl_value_t) -> *const u8 {
    (s as *const u8).add(size_of::<usize>())
}

/*
#define jl_string_len(s)  (*(size_t*)s)
*/
#[inline(always)]
pub unsafe fn jl_string_len(s: *mut jl_value_t) -> usize {
    *(s.cast())
}

/*
#define jl_gf_mtable(f) (((jl_datatype_t*)jl_typeof(f))->name->mt)
*/
// Not implemented

/*
#define jl_gf_name(f)   (jl_gf_mtable(f)->name)
*/
// Not implemented

/*
#define jl_get_fieldtypes(st) ((st)->types ? (st)->types : jl_compute_fieldtypes((st), NULL))
*/
#[inline(always)]
pub unsafe fn jl_get_fieldtypes(st: *mut jl_datatype_t) -> *mut jl_svec_t {
    if (&*st).types.is_null() {
        jl_compute_fieldtypes(st, std::ptr::null_mut())
    } else {
        (&*st).types
    }
}

/*
#define jl_datatype_size(t)    (((jl_datatype_t*)t)->size)
*/
#[inline(always)]
pub unsafe fn jl_datatype_size(t: *mut jl_datatype_t) -> i32 {
    (&*(t)).size
}

/*
#define jl_datatype_align(t)   (((jl_datatype_t*)t)->layout->alignment)
*/
#[inline(always)]
pub unsafe fn jl_datatype_align(t: *mut jl_datatype_t) -> u16 {
    (&*(&*(t)).layout).alignment
}

/*
#define jl_datatype_nbits(t)   ((((jl_datatype_t*)t)->size)*8)
*/
#[inline(always)]
pub unsafe fn jl_datatype_nbits(t: *mut jl_datatype_t) -> i32 {
    (&*(t)).size * 8
}

/*
#define jl_datatype_nfields(t) (((jl_datatype_t*)(t))->layout->nfields)
*/
#[inline(always)]
pub unsafe fn jl_datatype_nfields(t: *mut jl_datatype_t) -> u32 {
    (&*(&*(t)).layout).nfields
}

/*
#define jl_datatype_isinlinealloc(t) (((jl_datatype_t *)(t))->isinlinealloc)
*/
#[inline(always)]
pub unsafe fn jl_datatype_isinlinealloc(t: *mut jl_datatype_t) -> u8 {
    (&*(t)).isinlinealloc
}

/*
#define jl_symbol_name(s) jl_symbol_name_(s)
*/
// Not implemented

/*
#define jl_dt_layout_fields(d) ((const char*)(d) + sizeof(jl_datatype_layout_t))
*/
pub unsafe fn jl_dt_layout_fields(d: *const u8) -> *const u8 {
    d.add(size_of::<jl_datatype_layout_t>())
}

/*
#define jl_is_nothing(v)     (((jl_value_t*)(v)) == ((jl_value_t*)jl_nothing))
*/
#[inline(always)]
pub unsafe fn jl_is_nothing(v: *mut jl_value_t) -> bool {
    v == jl_nothing.cast()
}

/*
#define jl_is_tuple(v)       (((jl_datatype_t*)jl_typeof(v))->name == jl_tuple_typename)
*/
#[inline(always)]
pub unsafe fn jl_is_tuple(v: *mut jl_value_t) -> bool {
    (&*jl_typeof(v).cast::<jl_datatype_t>()).name == jl_tuple_typename
}

/*
#define jl_is_namedtuple(v)  (((jl_datatype_t*)jl_typeof(v))->name == jl_namedtuple_typename)
*/
#[inline(always)]
pub unsafe fn jl_is_namedtuple(v: *mut jl_value_t) -> bool {
    (&*jl_typeof(v).cast::<jl_datatype_t>()).name == jl_namedtuple_typename
}

/*
#define jl_is_svec(v)        jl_typeis(v,jl_simplevector_type)
*/
#[inline(always)]
pub unsafe fn jl_is_svec(v: *mut jl_value_t) -> bool {
    jl_typeis(v, jl_simplevector_type)
}

/*
#define jl_is_simplevector(v) jl_is_svec(v)
*/
// Not implemented

/*
#define jl_is_datatype(v)    jl_typeis(v,jl_datatype_type)
*/
#[inline(always)]
pub unsafe fn jl_is_datatype(v: *mut jl_value_t) -> bool {
    jl_typeis(v, jl_datatype_type)
}

/*
#define jl_is_mutable(t)     (((jl_datatype_t*)t)->mutabl)
*/
// Not implemented

/*
#define jl_is_mutable_datatype(t) (jl_is_datatype(t) && (((jl_datatype_t*)t)->mutabl))
*/
// Not implemented

/*
#define jl_is_immutable(t)   (!((jl_datatype_t*)t)->mutabl)
*/
#[inline(always)]
pub unsafe fn jl_is_immutable(v: *mut jl_value_t) -> bool {
    (&*jl_typeof(v).cast::<jl_datatype_t>()).mutabl == 0
}

/*
#define jl_is_immutable_datatype(t) (jl_is_datatype(t) && (!((jl_datatype_t*)t)->mutabl))
*/
// Not implemented

/*
#define jl_is_uniontype(v)   jl_typeis(v,jl_uniontype_type)
*/
#[inline(always)]
pub unsafe fn jl_is_uniontype(v: *mut jl_value_t) -> bool {
    jl_typeis(v, jl_uniontype_type)
}

/*
#define jl_is_typevar(v)     jl_typeis(v,jl_tvar_type)
*/
// Not implemented

/*
#define jl_is_unionall(v)    jl_typeis(v,jl_unionall_type)
*/
// Not implemented

/*
#define jl_is_typename(v)    jl_typeis(v,jl_typename_type)
*/
// Not implemented

/*
#define jl_is_int8(v)        jl_typeis(v,jl_int8_type)
*/
// Not implemented

/*
#define jl_is_int16(v)       jl_typeis(v,jl_int16_type)
*/
// Not implemented

/*
#define jl_is_int32(v)       jl_typeis(v,jl_int32_type)
*/
// Not implemented

/*
#define jl_is_int64(v)       jl_typeis(v,jl_int64_type)
*/
// Not implemented

/*
#define jl_is_uint8(v)       jl_typeis(v,jl_uint8_type)
*/
// Not implemented

/*
#define jl_is_uint16(v)      jl_typeis(v,jl_uint16_type)
*/
// Not implemented

/*
#define jl_is_uint32(v)      jl_typeis(v,jl_uint32_type)
*/
// Not implemented

/*
#define jl_is_uint64(v)      jl_typeis(v,jl_uint64_type)
*/
// Not implemented

/*
#define jl_is_bool(v)        jl_typeis(v,jl_bool_type)
*/
// Not implemented

/*
#define jl_is_symbol(v)      jl_typeis(v,jl_symbol_type)
*/
#[inline(always)]
pub unsafe fn jl_is_symbol(v: *mut jl_value_t) -> bool {
    jl_typeis(v, jl_symbol_type)
}

/*
#define jl_is_ssavalue(v)    jl_typeis(v,jl_ssavalue_type)
*/
// Not implemented

/*
#define jl_is_slot(v)        (jl_typeis(v,jl_slotnumber_type) || jl_typeis(v,jl_typedslot_type))
*/
// Not implemented

/*
#define jl_is_expr(v)        jl_typeis(v,jl_expr_type)
*/
// Not implemented

/*
#define jl_is_globalref(v)   jl_typeis(v,jl_globalref_type)
*/
// Not implemented

/*
#define jl_is_gotonode(v)    jl_typeis(v,jl_gotonode_type)
*/
// Not implemented

/*
#define jl_is_pinode(v)      jl_typeis(v,jl_pinode_type)
*/
// Not implemented

/*
#define jl_is_phinode(v)     jl_typeis(v,jl_phinode_type)
*/
// Not implemented

/*
#define jl_is_phicnode(v)    jl_typeis(v,jl_phicnode_type)
*/
// Not implemented

/*
#define jl_is_upsilonnode(v) jl_typeis(v,jl_upsilonnode_type)
*/
// Not implemented

/*
#define jl_is_quotenode(v)   jl_typeis(v,jl_quotenode_type)
*/
// Not implemented

/*
#define jl_is_newvarnode(v)  jl_typeis(v,jl_newvarnode_type)
*/
// Not implemented

/*
#define jl_is_linenode(v)    jl_typeis(v,jl_linenumbernode_type)
*/
// Not implemented

/*
#define jl_is_method_instance(v) jl_typeis(v,jl_method_instance_type)
*/
// Not implemented

/*
#define jl_is_code_instance(v) jl_typeis(v,jl_code_instance_type)
*/
// Not implemented

/*
#define jl_is_code_info(v)   jl_typeis(v,jl_code_info_type)
*/
// Not implemented

/*
#define jl_is_method(v)      jl_typeis(v,jl_method_type)
*/
// Not implemented

/*
#define jl_is_module(v)      jl_typeis(v,jl_module_type)
*/
#[inline(always)]
pub unsafe fn jl_is_module(v: *mut jl_value_t) -> bool {
    jl_typeis(v, jl_module_type)
}

/*
#define jl_is_mtable(v)      jl_typeis(v,jl_methtable_type)
*/
// Not implemented

/*
#define jl_is_task(v)        jl_typeis(v,jl_task_type)
*/
#[inline(always)]
pub unsafe fn jl_is_task(v: *mut jl_value_t) -> bool {
    jl_typeis(v, jl_task_type)
}

/*
#define jl_is_string(v)      jl_typeis(v,jl_string_type)
*/
#[inline(always)]
pub unsafe fn jl_is_string(v: *mut jl_value_t) -> bool {
    jl_typeof(v) == jl_string_type as _
}

/*
#define jl_is_cpointer(v)    jl_is_cpointer_type(jl_typeof(v))
*/
// Not implemented

/*
#define jl_is_pointer(v)     jl_is_cpointer_type(jl_typeof(v))
*/
// Not implemented

/*
#define jl_is_intrinsic(v)   jl_typeis(v,jl_intrinsic_type)
*/
// Not implemented

/*
#define jl_array_isbitsunion(a) (!(((jl_array_t*)(a))->flags.ptrarray) && jl_is_uniontype(jl_tparam0(jl_typeof(a))))
*/
#[inline(always)]
pub unsafe fn jl_array_isbitsunion(a: *mut jl_array_t) -> bool {
    (&*a).flags.ptrarray() > 0 && jl_is_uniontype(jl_tparam0(jl_typeof(a.cast()).cast()))
}

/*
#define jl_box_long(x)   jl_box_int64(x)
*/
// Not implemented

/*
#define jl_box_ulong(x)  jl_box_uint64(x)
*/
// Not implemented

/*
#define jl_unbox_long(x) jl_unbox_int64(x)
*/
// Not implemented

/*
#define jl_unbox_ulong(x) jl_unbox_uint64(x)
*/
// Not implemented

/*
#define jl_is_long(x)    jl_is_int64(x)
*/
// Not implemented

/*
#define jl_long_type     jl_int64_type
*/
// Not implemented

/*
#define jl_ulong_type    jl_uint64_type
*/
// Not implemented

/*
#define julia_init julia_init__threading
*/
// Not implemented

/*
#define jl_init jl_init__threading
*/
#[inline(always)]
pub unsafe fn jl_init() {
    jl_init__threading()
}

/*
#define jl_init_with_image jl_init_with_image__threading
*/
// Not implemented

/*
#define jl_setjmp_f    __sigsetjmp
*/
// Not implemented

/*
#define jl_setjmp_name "__sigsetjmp"
*/
// Not implemented

/*
#define jl_setjmp(a,b) sigsetjmp(a,b)
*/
// Not implemented

/*
#define jl_longjmp(a,b) siglongjmp(a,b)
*/
// Not implemented

/*
#define jl_current_task (jl_get_ptls_states()->current_task)
*/
// Not implemented

/*
#define jl_root_task (jl_get_ptls_states()->root_task)
*/
// Not implemented

/*
*/
#[inline(always)]
pub unsafe fn jl_symbol_name(s: *mut jl_sym_t) -> *mut u8 {
    jl_symbol_name_(s)
}

// STATIC_INLINE
/*
STATIC_INLINE int jl_array_ndimwords(uint32_t ndims) JL_NOTSAFEPOINT
{
    return (ndims < 3 ? 0 : ndims-2);
}
*/
#[inline]
pub unsafe fn jl_array_ndimwords(ndims: u32) -> i32 {
    if ndims < 3 {
        0
    } else {
        ndims as i32 - 2
    }
}

/*
STATIC_INLINE void jl_gc_wb(void *parent, void *ptr) JL_NOTSAFEPOINT
{
    // parent and ptr isa jl_value_t*
    if (__unlikely(jl_astaggedvalue(parent)->bits.gc == 3 && // parent is old and not in remset
                   (jl_astaggedvalue(ptr)->bits.gc & 1) == 0)) // ptr is young
        jl_gc_queue_root((jl_value_t*)parent);
}
*/
#[inline]
pub unsafe fn jl_gc_wb(parent: *mut jl_value_t, ptr: *mut jl_value_t) {
    let parent_tagged = &*jl_astaggedvalue(parent);
    let ptr = &*jl_astaggedvalue(ptr);

    if parent_tagged.__bindgen_anon_1.bits.gc() == 3 && (ptr.__bindgen_anon_1.bits.gc() & 1) == 0 {
        jl_gc_queue_root(parent)
    }
}

/*
STATIC_INLINE void jl_gc_wb_back(void *ptr) JL_NOTSAFEPOINT // ptr isa jl_value_t*
{
    // if ptr is old
    if (__unlikely(jl_astaggedvalue(ptr)->bits.gc == 3)) {
        jl_gc_queue_root((jl_value_t*)ptr);
    }
}
*/
// Not implemented

/*
STATIC_INLINE void jl_gc_multi_wb(void *parent, jl_value_t *ptr) JL_NOTSAFEPOINT
{
    // ptr is an immutable object
    if (__likely(jl_astaggedvalue(parent)->bits.gc != 3))
        return; // parent is young or in remset
    if (__likely(jl_astaggedvalue(ptr)->bits.gc == 3))
        return; // ptr is old and not in remset (thus it does not point to young)
    jl_datatype_t *dt = (jl_datatype_t*)jl_typeof(ptr);
    const jl_datatype_layout_t *ly = dt->layout;
    if (ly->npointers)
        jl_gc_queue_multiroot((jl_value_t*)parent, ptr);
}
*/
// Not implemented

/*
STATIC_INLINE jl_value_t *jl_svecref(void *t JL_PROPAGATES_ROOT, size_t i) JL_NOTSAFEPOINT
{
    assert(jl_typeis(t,jl_simplevector_type));
    assert(i < jl_svec_len(t));
    return jl_svec_data(t)[i];
}
*/
#[inline]
pub unsafe fn jl_svecref(t: *mut c_void, i: usize) -> *mut jl_value_t {
    assert!(jl_typeis(t.cast(), jl_simplevector_type));
    assert!(i < jl_svec_len(t.cast()));
    *jl_svec_data(t.cast()).add(i)
}

/*
STATIC_INLINE jl_value_t *jl_array_ptr_ref(void *a JL_PROPAGATES_ROOT, size_t i) JL_NOTSAFEPOINT
{
    assert(((jl_array_t*)a)->flags.ptrarray);
    assert(i < jl_array_len(a));
    return ((jl_value_t**)(jl_array_data(a)))[i];
}
*/
// Not implemented

/*
STATIC_INLINE uint8_t jl_array_uint8_ref(void *a, size_t i) JL_NOTSAFEPOINT
{
    assert(i < jl_array_len(a));
    assert(jl_typeis(a, jl_array_uint8_type));
    return ((uint8_t*)(jl_array_data(a)))[i];
}
*/
// Not implemented

/*
STATIC_INLINE void jl_array_uint8_set(void *a, size_t i, uint8_t x) JL_NOTSAFEPOINT
{
    assert(i < jl_array_len(a));
    assert(jl_typeis(a, jl_array_uint8_type));
    ((uint8_t*)(jl_array_data(a)))[i] = x;
}
*/
// Not implemented

/*STATIC_INLINE jl_svec_t *jl_field_names(jl_datatype_t *st) JL_NOTSAFEPOINT
{
    jl_svec_t *names = st->names;
    if (!names)
        names = st->name->names;
    return names;
}*/
#[inline]
pub unsafe fn jl_field_names(st: *mut jl_datatype_t) -> *mut jl_svec_t {
    let mut st = &mut *st;
    if !st.names.is_null() {
        return st.names;
    }

    st.names = (&mut *st.name).names;

    return st.names;
}

/*
STATIC_INLINE jl_sym_t *jl_field_name(jl_datatype_t *st, size_t i) JL_NOTSAFEPOINT
{
    return (jl_sym_t*)jl_svecref(jl_field_names(st), i);
}
*/
// Not implemented

/*
STATIC_INLINE jl_value_t *jl_field_type(jl_datatype_t *st JL_PROPAGATES_ROOT, size_t i)
{
    return jl_svecref(jl_get_fieldtypes(st), i);
}
*/
// Not implemented

/*
STATIC_INLINE jl_value_t *jl_field_type_concrete(jl_datatype_t *st JL_PROPAGATES_ROOT, size_t i) JL_NOTSAFEPOINT
{
    assert(st->types);
    return jl_svecref(st->types, i);
}
*/
// Not implemented

/*
STATIC_INLINE char *jl_symbol_name_(jl_sym_t *s) JL_NOTSAFEPOINT
{
    return (char*)s + LLT_ALIGN(sizeof(jl_sym_t), sizeof(void*));
}
*/
#[inline]
pub unsafe fn jl_symbol_name_(s: *mut jl_sym_t) -> *mut u8 {
    s.cast::<u8>()
        .add(llt_align!(size_of::<jl_sym_t>(), size_of::<*mut c_void>()))
}

/*
STATIC_INLINE int jl_is_kind(jl_value_t *v) JL_NOTSAFEPOINT
{
    return (v==(jl_value_t*)jl_uniontype_type || v==(jl_value_t*)jl_datatype_type ||
            v==(jl_value_t*)jl_unionall_type || v==(jl_value_t*)jl_typeofbottom_type);
}
*/
#[inline]
pub unsafe fn jl_is_kind(v: *mut jl_value_t) -> bool {
    v == jl_uniontype_type.cast()
        || v == jl_datatype_type.cast()
        || v == jl_unionall_type.cast()
        || v == jl_typeofbottom_type.cast()
}

/*
STATIC_INLINE int jl_is_type(jl_value_t *v) JL_NOTSAFEPOINT
{
    return jl_is_kind(jl_typeof(v));
}
*/
#[inline]
pub unsafe fn jl_is_type(v: *mut jl_value_t) -> bool {
    jl_is_kind(jl_typeof(v))
}

/*
STATIC_INLINE int jl_is_primitivetype(void *v) JL_NOTSAFEPOINT
{
    return (jl_is_datatype(v) && jl_is_immutable(v) &&
            ((jl_datatype_t*)(v))->layout &&
            jl_datatype_nfields(v) == 0 &&
            jl_datatype_size(v) > 0);
}
*/
#[inline]
pub unsafe fn jl_is_primitivetype(v: *mut jl_value_t) -> bool {
    jl_is_datatype(v)
        && jl_is_immutable(v)
        && !(&*v.cast::<jl_datatype_t>()).layout.is_null()
        && jl_datatype_nfields(v.cast()) == 0
        && jl_datatype_size(v.cast()) > 0
}

/*
STATIC_INLINE int jl_is_structtype(void *v) JL_NOTSAFEPOINT
{
    return (jl_is_datatype(v) &&
            !((jl_datatype_t*)(v))->abstract &&
            !jl_is_primitivetype(v));
}
*/
#[inline]
pub unsafe fn jl_is_structtype(v: *mut jl_value_t) -> bool {
    jl_is_datatype(v)
        && jl_is_immutable(v)
        && (&*v.cast::<jl_datatype_t>()).abstract_ == 0
        && jl_datatype_nfields(v.cast()) == 0
        && jl_datatype_size(v.cast()) > 0
}

/*
STATIC_INLINE int jl_isbits(void *t) JL_NOTSAFEPOINT // corresponding to isbits() in julia
{
    return (jl_is_datatype(t) && ((jl_datatype_t*)t)->isbitstype);
}
*/
#[inline]
pub unsafe fn jl_isbits(t: *mut c_void) -> bool {
    jl_is_datatype(t.cast()) && (&*t.cast::<jl_datatype_t>()).isbitstype != 0
}

/*
STATIC_INLINE int jl_is_datatype_singleton(jl_datatype_t *d) JL_NOTSAFEPOINT
{
    return (d->instance != NULL);
}
*/
#[inline]
pub unsafe fn jl_is_datatype_singleton(v: *mut jl_datatype_t) -> bool {
    !(&*v).instance.is_null()
}

/*
STATIC_INLINE int jl_is_abstracttype(void *v) JL_NOTSAFEPOINT
{
    return (jl_is_datatype(v) && ((jl_datatype_t*)(v))->abstract);
}
*/
#[inline]
pub unsafe fn jl_is_abstracttype(v: *mut c_void) -> bool {
    jl_is_datatype(v.cast()) && (&*v.cast::<jl_datatype_t>()).abstract_ > 0
}

/*
STATIC_INLINE int jl_is_array_type(void *t) JL_NOTSAFEPOINT
{
    return (jl_is_datatype(t) &&
            ((jl_datatype_t*)(t))->name == jl_array_typename);
}
*/
#[inline]
pub unsafe fn jl_is_array_type(v: *mut jl_value_t) -> bool {
    jl_is_datatype(v) && (&*(v as *mut jl_datatype_t)).name == jl_array_typename
}

/*
STATIC_INLINE int jl_is_array(void *v) JL_NOTSAFEPOINT
{
    jl_value_t *t = jl_typeof(v);
    return jl_is_array_type(t);
}
*/
#[inline]
pub unsafe fn jl_is_array(v: *mut jl_value_t) -> bool {
    jl_is_array_type(jl_typeof(v))
}

/*
STATIC_INLINE int jl_is_cpointer_type(jl_value_t *t) JL_NOTSAFEPOINT
{
    return (jl_is_datatype(t) &&
            ((jl_datatype_t*)(t))->name == ((jl_datatype_t*)jl_pointer_type->body)->name);
}
*/

#[inline]
pub unsafe fn jl_is_cpointer_type(v: *mut c_void) -> bool {
    jl_is_datatype(v.cast())
        && (&*v.cast::<jl_datatype_t>()).name
            == (&*(&*jl_pointer_type).body.cast::<jl_datatype_t>()).name
}

/*
STATIC_INLINE int jl_is_llvmpointer_type(jl_value_t *t) JL_NOTSAFEPOINT
{
    return (jl_is_datatype(t) &&
            ((jl_datatype_t*)(t))->name == jl_llvmpointer_typename);
}
*/
#[inline]
pub unsafe fn jl_is_llvmpointer_type(v: *mut c_void) -> bool {
    jl_is_datatype(v.cast()) && (&*v.cast::<jl_datatype_t>()).name == jl_llvmpointer_typename
}

/*
STATIC_INLINE int jl_is_abstract_ref_type(jl_value_t *t) JL_NOTSAFEPOINT
{
    return (jl_is_datatype(t) &&
            ((jl_datatype_t*)(t))->name == ((jl_datatype_t*)jl_ref_type->body)->name);
}
*/

#[inline]
pub unsafe fn jl_is_abstract_ref_type(v: *mut c_void) -> bool {
    jl_is_datatype(v.cast())
        && (&*v.cast::<jl_datatype_t>()).name
            == (&*(&*jl_ref_type).body.cast::<jl_datatype_t>()).name
}

/*
STATIC_INLINE int jl_is_tuple_type(void *t) JL_NOTSAFEPOINT
{
    return (jl_is_datatype(t) &&
            ((jl_datatype_t*)(t))->name == jl_tuple_typename);
}
*/
#[inline]
pub unsafe fn jl_is_tuple_type(v: *mut c_void) -> bool {
    jl_is_datatype(v.cast()) && (&*v.cast::<jl_datatype_t>()).name == jl_tuple_typename
}

/*
STATIC_INLINE int jl_is_namedtuple_type(void *t) JL_NOTSAFEPOINT
{
    return (jl_is_datatype(t) &&
            ((jl_datatype_t*)(t))->name == jl_namedtuple_typename);
}
*/
#[inline]
pub unsafe fn jl_is_namedtuple_type(v: *mut c_void) -> bool {
    jl_is_datatype(v.cast()) && (&*v.cast::<jl_datatype_t>()).name == jl_namedtuple_typename
}

/*
STATIC_INLINE int jl_is_vecelement_type(jl_value_t* t) JL_NOTSAFEPOINT
{
    return (jl_is_datatype(t) &&
            ((jl_datatype_t*)(t))->name == jl_vecelement_typename);
}
*/
#[inline]
pub unsafe fn jl_is_vecelement_type(v: *mut c_void) -> bool {
    jl_is_datatype(v.cast()) && (&*v.cast::<jl_datatype_t>()).name == jl_vecelement_typename
}

/*
STATIC_INLINE int jl_is_type_type(jl_value_t *v) JL_NOTSAFEPOINT
{
    return (jl_is_datatype(v) &&
            ((jl_datatype_t*)(v))->name == ((jl_datatype_t*)jl_type_type->body)->name);
}
*/
#[inline]
pub unsafe fn jl_is_type_type(v: *mut c_void) -> bool {
    jl_is_datatype(v.cast())
        && (&*v.cast::<jl_datatype_t>()).name
            == (&*(&*jl_type_type).body.cast::<jl_datatype_t>()).name
}

/*
STATIC_INLINE int jl_is_dispatch_tupletype(jl_value_t *v) JL_NOTSAFEPOINT
{
    return jl_is_datatype(v) && ((jl_datatype_t*)v)->isdispatchtuple;
}
*/
// Not implemented

/*
STATIC_INLINE int jl_is_concrete_type(jl_value_t *v) JL_NOTSAFEPOINT
{
    return jl_is_datatype(v) && ((jl_datatype_t*)v)->isconcretetype;
}
*/
// Not implemented

/*
STATIC_INLINE int jl_is_vararg_type(jl_value_t *v) JL_NOTSAFEPOINT
{
    v = jl_unwrap_unionall(v);
    return (jl_is_datatype(v) &&
            ((jl_datatype_t*)(v))->name == jl_vararg_typename);
}
*/
// Not implemented

/*
STATIC_INLINE jl_value_t *jl_unwrap_vararg(jl_value_t *v) JL_NOTSAFEPOINT
{
    return jl_tparam0(jl_unwrap_unionall(v));
}
*/
// Not implemented

/*
STATIC_INLINE size_t jl_vararg_length(jl_value_t *v) JL_NOTSAFEPOINT
{
    assert(jl_is_vararg_type(v));
    jl_value_t *len = jl_tparam1(jl_unwrap_unionall(v));
    assert(jl_is_long(len));
    return jl_unbox_long(len);
}
*/
// Not implemented

/*
STATIC_INLINE jl_vararg_kind_t jl_vararg_kind(jl_value_t *v) JL_NOTSAFEPOINT
{
    if (!jl_is_vararg_type(v))
        return JL_VARARG_NONE;
    jl_tvar_t *v1=NULL, *v2=NULL;
    if (jl_is_unionall(v)) {
        v1 = ((jl_unionall_t*)v)->var;
        v = ((jl_unionall_t*)v)->body;
        if (jl_is_unionall(v)) {
            v2 = ((jl_unionall_t*)v)->var;
            v = ((jl_unionall_t*)v)->body;
        }
    }
    assert(jl_is_datatype(v));
    jl_value_t *lenv = jl_tparam1(v);
    if (jl_is_long(lenv))
        return JL_VARARG_INT;
    if (jl_is_typevar(lenv) && lenv != (jl_value_t*)v1 && lenv != (jl_value_t*)v2)
        return JL_VARARG_BOUND;
    return JL_VARARG_UNBOUND;
}
*/
// Not implemented

/*
STATIC_INLINE int jl_is_va_tuple(jl_datatype_t *t) JL_NOTSAFEPOINT
{
    assert(jl_is_tuple_type(t));
    size_t l = jl_svec_len(t->parameters);
    return (l>0 && jl_is_vararg_type(jl_tparam(t,l-1)));
}
*/
// Not implemented

/*
STATIC_INLINE jl_vararg_kind_t jl_va_tuple_kind(jl_datatype_t *t) JL_NOTSAFEPOINT
{
    t = (jl_datatype_t*)jl_unwrap_unionall((jl_value_t*)t);
    assert(jl_is_tuple_type(t));
    size_t l = jl_svec_len(t->parameters);
    if (l == 0)
        return JL_VARARG_NONE;
    return jl_vararg_kind(jl_tparam(t,l-1));
}
*/
// Not implemented

/*
STATIC_INLINE jl_function_t *jl_get_function(jl_module_t *m, const char *name)
{
    return (jl_function_t*)jl_get_global(m, jl_symbol(name));
}
*/
// Not implemented

/*
STATIC_INLINE int jl_vinfo_sa(uint8_t vi)
{
    return (vi&16)!=0;
}
*/
// Not implemented

/*
STATIC_INLINE int jl_vinfo_usedundef(uint8_t vi)
{
    return (vi&32)!=0;
}
*/
// Not implemented

// static inline
/*
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
*/
pub unsafe fn jl_fielddesc_size(fielddesc_type: i8) -> u32 {
    2 << fielddesc_type
}

/*
static inline int jl_field_isptr(jl_datatype_t *st, int i) JL_NOTSAFEPOINT
{
    const jl_datatype_layout_t *ly = st->layout;
    assert(i >= 0 && (size_t)i < ly->nfields);
    return ((const jl_fielddesc8_t*)(jl_dt_layout_fields(ly) + jl_fielddesc_size(ly->fielddesc_type) * i))->isptr;
}
*/
pub unsafe fn jl_field_isptr(st: *mut jl_datatype_t, i: i32) -> bool {
    let ly = &*(&*st).layout;
    assert!(i >= 0 && (i as u32) < ly.nfields);
    (&*jl_dt_layout_fields(ly as *const _ as *const u8)
        .add(jl_fielddesc_size(ly.fielddesc_type() as i8) as usize * i as usize)
        .cast::<jl_fielddesc8_t>())
        .isptr()
        != 0
}

/*
#define DEFINE_FIELD_ACCESSORS(f)                                             \
    static inline uint32_t jl_field_##f(jl_datatype_t *st,                    \
                                        int i) JL_NOTSAFEPOINT                \
    {                                                                         \
        const jl_datatype_layout_t *ly = st->layout;                          \
        assert(i >= 0 && (size_t)i < ly->nfields);                            \
        if (ly->fielddesc_type == 0) {                                        \
            return ((const jl_fielddesc8_t*)jl_dt_layout_fields(ly))[i].f;    \
        }                                                                     \
        else if (ly->fielddesc_type == 1) {                                   \
            return ((const jl_fielddesc16_t*)jl_dt_layout_fields(ly))[i].f;   \
        }                                                                     \
        else {                                                                \
            return ((const jl_fielddesc32_t*)jl_dt_layout_fields(ly))[i].f;   \
        }                                                                     \
    }                                                                         \

DEFINE_FIELD_ACCESSORS(offset)
DEFINE_FIELD_ACCESSORS(size)
#undef DEFINE_FIELD_ACCESSORS
*/
pub unsafe fn jl_field_size(st: *mut jl_datatype_t, i: isize) -> u32 {
    let ly = &*(&*st).layout;
    assert!(i >= 0 && (i as u32) < ly.nfields);
    match ly.fielddesc_type() {
        0 => (&*(jl_dt_layout_fields((ly as *const jl_datatype_layout_t).cast())
            .cast::<jl_fielddesc8_t>()
            .offset(i)))
            .size() as u32,
        1 => (&*(jl_dt_layout_fields((ly as *const jl_datatype_layout_t).cast())
            .cast::<jl_fielddesc16_t>()
            .offset(i)))
            .size() as u32,
        _ => (&*(jl_dt_layout_fields((ly as *const jl_datatype_layout_t).cast())
            .cast::<jl_fielddesc32_t>()
            .offset(i)))
            .size(),
    }
}

pub unsafe fn jl_field_offset(st: *mut jl_datatype_t, i: isize) -> u32 {
    let ly = &*(&*st).layout;
    assert!(i >= 0 && (i as u32) < ly.nfields);
    match ly.fielddesc_type() {
        0 => {
            (&*(jl_dt_layout_fields((ly as *const jl_datatype_layout_t).cast())
                .cast::<jl_fielddesc8_t>()
                .offset(i)))
                .offset as u32
        }
        1 => {
            (&*(jl_dt_layout_fields((ly as *const jl_datatype_layout_t).cast())
                .cast::<jl_fielddesc16_t>()
                .offset(i)))
                .offset as u32
        }
        _ => {
            (&*(jl_dt_layout_fields((ly as *const jl_datatype_layout_t).cast())
                .cast::<jl_fielddesc32_t>()
                .offset(i)))
                .offset
        }
    }
}

#[inline(always)]
pub unsafe fn jl_array_dims<'a>(array: *mut jl_array_t, ndims: usize) -> &'a [usize] {
    let x = &(&*array).nrows as *const usize;
    std::slice::from_raw_parts(x, ndims)
}

/*#define jl_array_ptr_data(a)  ((jl_value_t**)((jl_array_t*)(a))->data)
STATIC_INLINE jl_value_t *jl_array_ptr_ref(void *a JL_PROPAGATES_ROOT, size_t i) JL_NOTSAFEPOINT
{
    assert(((jl_array_t*)a)->flags.ptrarray);
    assert(i < jl_array_len(a));
    return jl_atomic_load_relaxed(((jl_value_t**)(jl_array_data(a))) + i);
}
STATIC_INLINE jl_value_t *jl_array_ptr_set(
    void *a JL_ROOTING_ARGUMENT, size_t i,
    void *x JL_ROOTED_ARGUMENT) JL_NOTSAFEPOINT
{
    assert(((jl_array_t*)a)->flags.ptrarray);
    assert(i < jl_array_len(a));
    jl_atomic_store_relaxed(((jl_value_t**)(jl_array_data(a))) + i, (jl_value_t*)x);
    if (x) {
        if (((jl_array_t*)a)->flags.how == 3) {
            a = jl_array_data_owner(a);
        }
        jl_gc_wb(a, x);
    }
    return (jl_value_t*)x;
}*/

pub unsafe fn jl_array_ptr_set(a: *mut c_void, i: usize, x: *mut c_void) -> *mut jl_value_t {
    let a: *mut jl_array_t = a.cast();
    assert!((&*a).flags.ptrarray() != 0);
    assert!(i < jl_array_len(a));
    let a_data: *mut AtomicPtr<jl_value_t> = jl_array_data(a.cast()).cast();
    (&*a_data.add(i)).store(x.cast(), Ordering::Relaxed);

    if !x.is_null() {
        if (&*a).flags.how() == 3 {
            jl_gc_wb(jl_array_data_owner(a).cast(), x.cast());
        } else {
            jl_gc_wb(a.cast(), x.cast());
        }
    }

    x.cast()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanity() {
        unsafe {
            jl_init();
            assert!(jl_is_initialized() != 0);

            //dyn_eval_string();
            assert!(jl_exception_occurred().is_null());

            jl_atexit_hook(0);
        }
    }
}
