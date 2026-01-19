pub use functions::*;
pub use globals::*;

/// Globals from libjulia used by jlrs
pub mod globals {
    #[cfg_attr(
        any(windows, target_os = "windows", feature = "windows"),
        link(name = "libjulia", kind = "raw-dylib")
    )]
    unsafe extern "C" {
        pub static mut jl_typeofbottom_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_datatype_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_uniontype_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_unionall_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_tvar_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_any_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_type_type: *mut crate::types::jl_unionall_t;

        pub static mut jl_typename_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_type_typename: *mut crate::types::jl_typename_t;

        pub static mut jl_symbol_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_const_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_simplevector_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_tuple_typename: *mut crate::types::jl_typename_t;

        pub static mut jl_vecelement_typename: *mut crate::types::jl_typename_t;

        pub static mut jl_anytuple_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_emptytuple_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_anytuple_type_type: *mut crate::types::jl_unionall_t;

        pub static mut jl_function_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_bottom_type: *mut crate::types::jl_value_t;

        pub static mut jl_module_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_abstractarray_type: *mut crate::types::jl_unionall_t;

        pub static mut jl_densearray_type: *mut crate::types::jl_unionall_t;

        pub static mut jl_array_type: *mut crate::types::jl_unionall_t;

        pub static mut jl_array_typename: *mut crate::types::jl_typename_t;

        pub static mut jl_abstractstring_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_string_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_errorexception_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_argumenterror_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_loaderror_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_initerror_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_typeerror_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_methoderror_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_undefvarerror_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_stackovf_exception: *mut crate::types::jl_value_t;

        pub static mut jl_memory_exception: *mut crate::types::jl_value_t;

        pub static mut jl_readonlymemory_exception: *mut crate::types::jl_value_t;

        pub static mut jl_diverror_exception: *mut crate::types::jl_value_t;

        pub static mut jl_undefref_exception: *mut crate::types::jl_value_t;

        pub static mut jl_interrupt_exception: *mut crate::types::jl_value_t;

        pub static mut jl_boundserror_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_an_empty_vec_any: *mut crate::types::jl_value_t;

        pub static mut jl_an_empty_string: *mut crate::types::jl_value_t;

        pub static mut jl_bool_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_char_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_int8_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_uint8_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_int16_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_uint16_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_int32_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_uint32_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_int64_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_uint64_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_float16_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_float32_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_float64_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_floatingpoint_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_number_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_nothing_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_signed_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_voidpointer_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_uint8pointer_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_pointer_type: *mut crate::types::jl_unionall_t;

        pub static mut jl_llvmpointer_type: *mut crate::types::jl_unionall_t;

        pub static mut jl_ref_type: *mut crate::types::jl_unionall_t;

        pub static mut jl_pointer_typename: *mut crate::types::jl_typename_t;

        pub static mut jl_llvmpointer_typename: *mut crate::types::jl_typename_t;

        pub static mut jl_namedtuple_typename: *mut crate::types::jl_typename_t;

        pub static mut jl_namedtuple_type: *mut crate::types::jl_unionall_t;

        pub static mut jl_task_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_array_uint8_type: *mut crate::types::jl_value_t;

        pub static mut jl_array_any_type: *mut crate::types::jl_value_t;

        pub static mut jl_array_symbol_type: *mut crate::types::jl_value_t;

        pub static mut jl_array_int32_type: *mut crate::types::jl_value_t;

        pub static mut jl_expr_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_emptysvec: *mut crate::types::jl_svec_t;

        pub static mut jl_emptytuple: *mut crate::types::jl_value_t;

        pub static mut jl_true: *mut crate::types::jl_value_t;

        pub static mut jl_false: *mut crate::types::jl_value_t;

        pub static mut jl_nothing: *mut crate::types::jl_value_t;

        pub static mut jl_main_module: *mut crate::types::jl_module_t;

        pub static mut jl_core_module: *mut crate::types::jl_module_t;

        pub static mut jl_base_module: *mut crate::types::jl_module_t;

        pub static mut jl_vararg_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_atomicerror_type: *mut crate::types::jl_datatype_t;

        pub static mut jl_pair_type: *mut crate::types::jl_value_t;

        pub static mut jl_array_uint64_type: *mut crate::types::jl_value_t;

        pub static jl_n_threads: std::sync::atomic::AtomicI32;

        pub static jl_n_threadpools: std::cell::UnsafeCell<std::ffi::c_int>;

        pub static jl_n_threads_per_pool: std::cell::UnsafeCell<*mut std::ffi::c_int>;

        pub static mut jl_n_gcthreads: std::ffi::c_int;

        // Added in Julia 1.11

        #[cfg(not(julia_1_10))]
        pub static mut jl_genericmemory_type: *mut crate::types::jl_unionall_t;

        #[cfg(not(julia_1_10))]
        pub static mut jl_genericmemory_typename: *mut crate::types::jl_typename_t;

        #[cfg(not(julia_1_10))]
        pub static mut jl_genericmemoryref_type: *mut crate::types::jl_unionall_t;

        #[cfg(not(julia_1_10))]
        pub static mut jl_genericmemoryref_typename: *mut crate::types::jl_typename_t;

        #[cfg(not(julia_1_10))]
        pub static mut jl_an_empty_memory_any: *mut crate::types::jl_value_t;

        #[cfg(not(julia_1_10))]
        pub static mut jl_bfloat16_type: *mut crate::types::jl_datatype_t;

        #[cfg(not(julia_1_10))]
        pub static mut jl_array_uint32_type: *mut crate::types::jl_value_t;

        // Removed in Julia 1.12

        #[cfg(any(julia_1_10, julia_1_11))]
        pub static mut jl_kwcall_func: *mut crate::types::jl_value_t;

    }
}

/// Functions from libjulia used by jlrs
pub mod functions {
    use crate::jl_gcframe_t;

    #[cfg_attr(
        any(windows, target_os = "windows", feature = "windows"),
        link(name = "libjulia", kind = "raw-dylib")
    )]
    unsafe extern "C" {
        pub fn jl_gc_enable(on: std::ffi::c_int) -> std::ffi::c_int;

        pub fn jl_gc_is_enabled() -> std::ffi::c_int;

        pub fn jl_get_pgcstack() -> *mut *mut jl_gcframe_t;

        pub fn jl_gc_collect(arg0: crate::types::jl_gc_collection_t);

        pub fn jl_gc_add_finalizer(
            v: *mut crate::types::jl_value_t,
            f: *mut crate::types::jl_value_t,
        );

        pub fn jl_gc_add_ptr_finalizer(
            ptls: *mut crate::types::jl_tls_states_t,
            v: *mut crate::types::jl_value_t,
            f: *mut std::ffi::c_void,
        );

        pub fn jl_subtype(
            a: *mut crate::types::jl_value_t,
            b: *mut crate::types::jl_value_t,
        ) -> std::ffi::c_int;

        pub fn jl_object_id(v: *mut crate::types::jl_value_t) -> usize;

        pub fn jl_has_free_typevars(v: *mut crate::types::jl_value_t) -> std::ffi::c_int;

        pub fn jl_has_typevar(
            t: *mut crate::types::jl_value_t,
            v: *mut crate::types::jl_tvar_t,
        ) -> std::ffi::c_int;

        pub fn jl_isa(
            a: *mut crate::types::jl_value_t,
            t: *mut crate::types::jl_value_t,
        ) -> std::ffi::c_int;

        pub fn jl_type_union(
            ts: *mut *mut crate::types::jl_value_t,
            n: usize,
        ) -> *mut crate::types::jl_value_t;

        pub fn jl_type_unionall(
            v: *mut crate::types::jl_tvar_t,
            body: *mut crate::types::jl_value_t,
        ) -> *mut crate::types::jl_value_t;

        pub fn jl_typename_str(v: *mut crate::types::jl_value_t) -> *const std::ffi::c_char;

        pub fn jl_typeof_str(v: *mut crate::types::jl_value_t) -> *const std::ffi::c_char;

        pub fn jl_new_typevar(
            name: *mut crate::types::jl_sym_t,
            lb: *mut crate::types::jl_value_t,
            ub: *mut crate::types::jl_value_t,
        ) -> *mut crate::types::jl_tvar_t;

        pub fn jl_apply_type(
            tc: *mut crate::types::jl_value_t,
            params: *mut *mut crate::types::jl_value_t,
            n: usize,
        ) -> *mut crate::types::jl_value_t;

        pub fn jl_new_datatype(
            name: *mut crate::types::jl_sym_t,
            module: *mut crate::types::jl_module_t,
            sup: *mut crate::types::jl_datatype_t,
            parameters: *mut crate::types::jl_svec_t,
            fnames: *mut crate::types::jl_svec_t,
            ftypes: *mut crate::types::jl_svec_t,
            fattrs: *mut crate::types::jl_svec_t,
            abstr: std::ffi::c_int,
            mutabl: std::ffi::c_int,
            ninitialized: std::ffi::c_int,
        ) -> *mut crate::types::jl_datatype_t;

        pub fn jl_new_structv(
            ty: *mut crate::types::jl_datatype_t,
            args: *mut *mut crate::types::jl_value_t,
            na: u32,
        ) -> *mut crate::types::jl_value_t;

        pub fn jl_new_struct_uninit(
            ty: *mut crate::types::jl_datatype_t,
        ) -> *mut crate::types::jl_value_t;

        pub fn jl_alloc_svec(n: usize) -> *mut crate::types::jl_svec_t;

        pub fn jl_alloc_svec_uninit(n: usize) -> *mut crate::types::jl_svec_t;

        pub fn jl_svec_copy(a: *mut crate::types::jl_svec_t) -> *mut crate::types::jl_svec_t;

        pub fn jl_symbol_n(str: *const std::ffi::c_char, len: usize)
        -> *mut crate::types::jl_sym_t;

        pub fn jl_gensym() -> *mut crate::types::jl_sym_t;

        pub fn jl_tagged_gensym(
            str: *const std::ffi::c_char,
            len: usize,
        ) -> *mut crate::types::jl_sym_t;

        pub fn jl_box_bool(x: i8) -> *mut crate::types::jl_value_t;

        pub fn jl_box_int8(x: i8) -> *mut crate::types::jl_value_t;

        pub fn jl_box_uint8(x: u8) -> *mut crate::types::jl_value_t;

        pub fn jl_box_int16(x: i16) -> *mut crate::types::jl_value_t;

        pub fn jl_box_uint16(x: u16) -> *mut crate::types::jl_value_t;

        pub fn jl_box_int32(x: i32) -> *mut crate::types::jl_value_t;

        pub fn jl_box_uint32(x: u32) -> *mut crate::types::jl_value_t;

        pub fn jl_box_char(x: u32) -> *mut crate::types::jl_value_t;

        pub fn jl_box_int64(x: i64) -> *mut crate::types::jl_value_t;

        pub fn jl_box_uint64(x: u64) -> *mut crate::types::jl_value_t;

        pub fn jl_box_float32(x: f32) -> *mut crate::types::jl_value_t;

        pub fn jl_box_float64(x: f64) -> *mut crate::types::jl_value_t;

        pub fn jl_box_voidpointer(x: *mut std::ffi::c_void) -> *mut crate::types::jl_value_t;

        pub fn jl_unbox_bool(v: *mut crate::types::jl_value_t) -> i8;

        pub fn jl_unbox_int8(v: *mut crate::types::jl_value_t) -> i8;

        pub fn jl_unbox_uint8(v: *mut crate::types::jl_value_t) -> u8;

        pub fn jl_unbox_int16(v: *mut crate::types::jl_value_t) -> i16;

        pub fn jl_unbox_uint16(v: *mut crate::types::jl_value_t) -> u16;

        pub fn jl_unbox_int32(v: *mut crate::types::jl_value_t) -> i32;

        pub fn jl_unbox_uint32(v: *mut crate::types::jl_value_t) -> u32;

        pub fn jl_unbox_int64(v: *mut crate::types::jl_value_t) -> i64;

        pub fn jl_unbox_uint64(v: *mut crate::types::jl_value_t) -> u64;

        pub fn jl_unbox_float32(v: *mut crate::types::jl_value_t) -> f32;

        pub fn jl_unbox_float64(v: *mut crate::types::jl_value_t) -> f64;

        pub fn jl_unbox_voidpointer(v: *mut crate::types::jl_value_t) -> *mut std::ffi::c_void;

        pub fn jl_field_index(
            t: *mut crate::types::jl_datatype_t,
            fld: *mut crate::types::jl_sym_t,
            err: std::ffi::c_int,
        ) -> std::ffi::c_int;

        pub fn jl_get_nth_field(
            v: *mut crate::types::jl_value_t,
            i: usize,
        ) -> *mut crate::types::jl_value_t;

        pub fn jl_get_nth_field_noalloc(
            v: *mut crate::types::jl_value_t,
            i: usize,
        ) -> *mut crate::types::jl_value_t;

        pub fn jl_set_nth_field(
            v: *mut crate::types::jl_value_t,
            i: usize,
            rhs: *mut crate::types::jl_value_t,
        );

        pub fn jl_islayout_inline(
            eltype: *mut crate::types::jl_value_t,
            fsz: *mut usize,
            al: *mut usize,
        ) -> std::ffi::c_int;

        pub fn jl_ptr_to_array_1d(
            atype: *mut crate::types::jl_value_t,
            data: *mut std::ffi::c_void,
            nel: usize,
            own_buffer: std::ffi::c_int,
        ) -> *mut crate::types::jl_array_t;

        pub fn jl_ptr_to_array(
            atype: *mut crate::types::jl_value_t,
            data: *mut std::ffi::c_void,
            dims: *mut crate::types::jl_value_t,
            own_buffer: std::ffi::c_int,
        ) -> *mut crate::types::jl_array_t;

        pub fn jl_alloc_array_1d(
            atype: *mut crate::types::jl_value_t,
            nr: usize,
        ) -> *mut crate::types::jl_array_t;

        pub fn jl_alloc_array_2d(
            atype: *mut crate::types::jl_value_t,
            nr: usize,
            nc: usize,
        ) -> *mut crate::types::jl_array_t;

        pub fn jl_alloc_array_3d(
            atype: *mut crate::types::jl_value_t,
            nr: usize,
            nc: usize,
            z: usize,
        ) -> *mut crate::types::jl_array_t;

        pub fn jl_pchar_to_array(
            str: *const std::ffi::c_char,
            len: usize,
        ) -> *mut crate::types::jl_array_t;

        pub fn jl_pchar_to_string(
            str: *const std::ffi::c_char,
            len: usize,
        ) -> *mut crate::types::jl_value_t;

        pub fn jl_array_to_string(
            a: *mut crate::types::jl_array_t,
        ) -> *mut crate::types::jl_value_t;

        pub fn jl_alloc_vec_any(n: usize) -> *mut crate::types::jl_array_t;

        pub fn jl_array_grow_end(a: *mut crate::types::jl_array_t, inc: usize);

        pub fn jl_array_del_end(a: *mut crate::types::jl_array_t, dec: usize);

        pub fn jl_array_ptr_1d_push(
            a: *mut crate::types::jl_array_t,
            item: *mut crate::types::jl_value_t,
        );

        pub fn jl_array_ptr_1d_append(
            a: *mut crate::types::jl_array_t,
            a2: *mut crate::types::jl_array_t,
        );

        pub fn jl_apply_array_type(
            ty: *mut crate::types::jl_value_t,
            dim: usize,
        ) -> *mut crate::types::jl_value_t;

        pub fn jl_array_eltype(a: *mut crate::types::jl_value_t) -> *mut std::ffi::c_void;

        pub fn jl_array_rank(a: *mut crate::types::jl_value_t) -> std::ffi::c_int;

        pub fn jl_string_ptr(s: *mut crate::types::jl_value_t) -> *const std::ffi::c_char;

        pub fn jl_is_const(
            m: *mut crate::types::jl_module_t,
            var: *mut crate::types::jl_sym_t,
        ) -> std::ffi::c_int;

        pub fn jl_get_global(
            m: *mut crate::types::jl_module_t,
            var: *mut crate::types::jl_sym_t,
        ) -> *mut crate::types::jl_value_t;

        pub fn jl_set_global(
            m: *mut crate::types::jl_module_t,
            var: *mut crate::types::jl_sym_t,
            val: *mut crate::types::jl_value_t,
        );

        pub fn jl_cpu_threads() -> std::ffi::c_int;

        pub fn jl_is_debugbuild() -> std::ffi::c_int;

        pub fn jl_get_UNAME() -> *mut crate::types::jl_sym_t;

        pub fn jl_exception_occurred() -> *mut crate::types::jl_value_t;

        pub fn jl_is_initialized() -> std::ffi::c_int;

        pub fn jl_atexit_hook(status: std::ffi::c_int);

        pub fn jl_eval_string(str: *const std::ffi::c_char) -> *mut crate::types::jl_value_t;

        pub fn jl_call(
            f: *mut crate::types::jl_value_t,
            args: *mut *mut crate::types::jl_value_t,
            nargs: u32,
        ) -> *mut crate::types::jl_value_t;

        pub fn jl_call0(f: *mut crate::types::jl_value_t) -> *mut crate::types::jl_value_t;

        pub fn jl_call1(
            f: *mut crate::types::jl_value_t,
            a: *mut crate::types::jl_value_t,
        ) -> *mut crate::types::jl_value_t;

        pub fn jl_call2(
            f: *mut crate::types::jl_value_t,
            a: *mut crate::types::jl_value_t,
            b: *mut crate::types::jl_value_t,
        ) -> *mut crate::types::jl_value_t;

        pub fn jl_call3(
            f: *mut crate::types::jl_value_t,
            a: *mut crate::types::jl_value_t,
            b: *mut crate::types::jl_value_t,
            c: *mut crate::types::jl_value_t,
        ) -> *mut crate::types::jl_value_t;

        pub fn jl_throw(e: *mut crate::types::jl_value_t) -> !;

        pub fn jl_rethrow() -> !;

        pub fn jl_print_backtrace();

        pub fn jl_rethrow_other(e: *mut crate::types::jl_value_t) -> !;

        pub fn jl_stdout_stream() -> *mut crate::types::JL_STREAM;

        pub fn jl_stderr_stream() -> *mut crate::types::JL_STREAM;

        pub fn jl_stdout_obj() -> *mut crate::types::jl_value_t;

        pub fn jl_stderr_obj() -> *mut crate::types::jl_value_t;

        pub fn jl_static_show(
            out: *mut crate::types::JL_STREAM,
            v: *mut crate::types::jl_value_t,
        ) -> usize;

        pub fn jl_ver_major() -> std::ffi::c_int;

        pub fn jl_ver_minor() -> std::ffi::c_int;

        pub fn jl_ver_patch() -> std::ffi::c_int;

        pub fn jl_ver_is_release() -> std::ffi::c_int;

        pub fn jl_ver_string() -> *const std::ffi::c_char;

        pub fn jl_gc_set_cb_pre_gc(cb: *const extern "C" fn(std::ffi::c_int) -> (), enable: std::ffi::c_int);
        pub fn jl_gc_set_cb_post_gc(cb: *const extern "C" fn(std::ffi::c_int) -> (), enable: std::ffi::c_int);

        pub fn jl_new_foreign_type(
            name: *mut crate::types::jl_sym_t,
            module: *mut crate::types::jl_module_t,
            sup: *mut crate::types::jl_datatype_t,
            markfunc: crate::types::jl_markfunc_t,
            sweepfunc: crate::types::jl_sweepfunc_t,
            haspointers: std::ffi::c_int,
            large: std::ffi::c_int,
        ) -> *mut crate::types::jl_datatype_t;

        pub fn jl_gc_alloc_typed(
            ptls: *mut crate::types::jl_tls_states_t,
            sz: usize,
            ty: *mut std::ffi::c_void,
        ) -> *mut std::ffi::c_void;

        pub fn jl_gc_mark_queue_obj(
            ptls: *mut crate::types::jl_tls_states_t,
            obj: *mut crate::types::jl_value_t,
        ) -> std::ffi::c_int;

        pub fn jl_gc_mark_queue_objarray(
            ptls: *mut crate::types::jl_tls_states_t,
            parent: *mut crate::types::jl_value_t,
            objs: *mut *mut crate::types::jl_value_t,
            nobjs: usize,
        );

        pub fn jl_gc_schedule_foreign_sweepfunc(
            ptls: *mut crate::types::jl_tls_states_t,
            bj: *mut crate::types::jl_value_t,
        );

        pub fn jl_gc_set_cb_root_scanner(
            cb: crate::jl_gc_cb_root_scanner_t,
            enable: std::ffi::c_int,
        );

        pub fn jl_dlopen(
            filename: *const std::ffi::c_char,
            flags: std::ffi::c_uint,
        ) -> *mut std::ffi::c_void;

        pub fn jl_dlclose(handle: *mut std::ffi::c_void) -> std::ffi::c_int;

        pub fn jl_gc_safepoint();

        pub fn jl_init();

        pub fn jl_init_with_image(
            julia_bindir: *const std::os::raw::c_char,
            image_path: *const std::os::raw::c_char,
        );

        pub fn jl_symbol(str: *const std::ffi::c_char) -> *mut crate::types::jl_sym_t;

        pub fn jl_egal(
            a: *const crate::types::jl_value_t,
            b: *const crate::types::jl_value_t,
        ) -> std::os::raw::c_int;

        pub fn jl_adopt_thread() -> *mut *mut crate::types::jl_gcframe_t;

        pub fn jl_reinit_foreign_type(
            dt: *mut crate::types::jl_datatype_t,
            markfunc: crate::types::jl_markfunc_t,
            sweepfunc: crate::types::jl_sweepfunc_t,
        ) -> std::ffi::c_int;

        pub fn jl_enter_threaded_region();

        pub fn jl_exit_threaded_region();

        // Removed in Julia 1.11

        #[cfg(julia_1_10)]
        pub fn jl_new_array(
            atype: *mut crate::types::jl_value_t,
            dims: *mut crate::types::jl_value_t,
        ) -> *mut crate::types::jl_array_t;

        // Added in Julia 1.11

        #[cfg(not(julia_1_10))]
        pub fn jl_alloc_array_nd(
            atype: *mut crate::types::jl_value_t,
            dims: *mut usize,
            ndims: usize,
        ) -> *mut crate::types::jl_array_t;

        // Removed in Julia 1.12

        #[cfg(any(julia_1_10, julia_1_11))]
        pub fn jl_set_const(
            m: *mut crate::types::jl_module_t,
            var: *mut crate::types::jl_sym_t,
            val: *mut crate::types::jl_value_t,
        );

        // Added in Julia 1.12

        #[cfg(not(any(julia_1_10, julia_1_11)))]
        pub fn jl_declare_constant_val(
            b: *mut crate::types::jl_binding_t,
            m: *mut crate::types::jl_module_t,
            var: *mut crate::types::jl_sym_t,
            val: *mut crate::types::jl_value_t,
        ) -> *mut crate::types::jl_binding_partition_t;
    }
}

/// On Windows we use raw dylib linkage to avoid having to create an import lib for Julia. If a
/// symbol is used in jlrs_cc, either directly or inside a macro or static inline function, we
/// need to mention them.
#[cfg(any(windows, target_os = "windows", feature = "windows"))]
mod indirect {
    #[link(name = "libjulia", kind = "raw-dylib")]
    unsafe extern "C" {
        #![allow(unused)]

        // TODO: is this ok? It's unused, but compiling with BinaryBuilder complains
        // about jl_options being undefined.
        pub static mut jl_options: [u8; 0];

        pub static mut jl_small_typeof: *mut std::ffi::c_void;

        pub static mut jl_excstack_state: *mut std::ffi::c_void;

        pub static mut jl_enter_handler: *mut std::ffi::c_void;

        pub static mut jl_eh_restore_state: *mut std::ffi::c_void;

        pub static mut jl_eh_restore_state_noexcept: *mut std::ffi::c_void;

        pub static mut jl_apply_generic: *mut std::ffi::c_void;

        pub static mut jl_gc_queue_multiroot: *mut std::ffi::c_void;

        pub static mut jl_gc_queue_root: *mut std::ffi::c_void;

        pub static mut jl_compute_fieldtypes: *mut std::ffi::c_void;

        pub static mut jl_setjmp: *mut std::ffi::c_void;

        pub static mut jl_current_exception: *mut std::ffi::c_void;

        pub static mut jl_get_world_counter: *mut std::ffi::c_void;

        pub static mut jl_egal__bitstag: *mut std::ffi::c_void;

        pub static mut jl_error: *mut std::ffi::c_void;

        pub static mut jl_restore_excstack: *mut std::ffi::c_void;

        // Removed in Julia 1.11

        #[cfg(julia_1_10)]
        pub static mut jl_arrayref: *mut std::ffi::c_void;

        #[cfg(julia_1_10)]
        pub static mut jl_arrayset: *mut std::ffi::c_void;

        #[cfg(julia_1_10)]
        pub static mut jl_array_typetagdata: *mut std::ffi::c_void;

        // Added in Julia 1.11

        #[cfg(not(julia_1_10))]
        pub static mut jl_unwrap_unionall: *mut std::ffi::c_void;

        #[cfg(not(julia_1_10))]
        pub static mut jl_genericmemoryref: *mut std::ffi::c_void;
    }
}
