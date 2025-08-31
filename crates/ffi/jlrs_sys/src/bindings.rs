/// jlrs_cc functions
///
/// The jlrs_cc library is compiled by the build script and implements some missing functionality
/// in C so we can keep type layouts opaque.
pub mod jlrs_cc {
    unsafe extern "C" {
        pub fn jlrs_typeof(v: *mut crate::types::jl_value_t) -> *mut crate::types::jl_value_t;

        pub fn jlrs_gc_wb(parent: *mut std::ffi::c_void, ptr: *mut std::ffi::c_void);

        pub fn jlrs_svec_len(t: *mut crate::types::jl_svec_t) -> usize;

        pub fn jlrs_svec_data(
            t: *mut crate::types::jl_svec_t,
        ) -> *mut *mut crate::types::jl_value_t;

        pub fn jlrs_svecref(t: *mut std::ffi::c_void, i: usize) -> *mut crate::types::jl_value_t;

        pub fn jlrs_svecset(
            t: *mut std::ffi::c_void,
            i: usize,
            x: *mut std::ffi::c_void,
        ) -> *mut crate::types::jl_value_t;

        pub fn jlrs_array_len(a: *mut crate::types::jl_array_t) -> usize;

        pub fn jlrs_array_data(a: *mut crate::types::jl_array_t) -> *mut std::ffi::c_void;

        pub fn jlrs_exprarg(
            e: *mut crate::types::jl_expr_t,
            n: usize,
        ) -> *mut crate::types::jl_value_t;

        pub fn jlrs_exprargset(
            e: *mut crate::types::jl_expr_t,
            n: usize,
            v: *mut crate::types::jl_value_t,
        );

        pub fn jlrs_expr_nargs(e: *mut crate::types::jl_expr_t) -> usize;

        pub fn jlrs_nparams(t: *mut crate::types::jl_datatype_t) -> usize;

        pub fn jlrs_string_len(s: *mut crate::types::jl_value_t) -> usize;

        pub fn jlrs_get_fieldtypes(
            st: *mut crate::types::jl_datatype_t,
        ) -> *mut crate::types::jl_svec_t;

        pub fn jlrs_datatype_size(t: *mut crate::types::jl_datatype_t) -> u32;

        pub fn jlrs_datatype_align(t: *mut crate::types::jl_datatype_t) -> u16;

        pub fn jlrs_datatype_nfields(t: *mut crate::types::jl_datatype_t) -> u32;

        pub fn jlrs_symbol_name(s: *mut crate::types::jl_sym_t) -> *mut std::ffi::c_char;

        pub fn jlrs_field_isptr(
            st: *mut crate::types::jl_datatype_t,
            i: std::ffi::c_int,
        ) -> std::ffi::c_int;

        pub fn jlrs_is_primitivetype(v: *mut std::ffi::c_void) -> std::ffi::c_int;

        pub fn jlrs_isbits(t: *mut std::ffi::c_void) -> std::ffi::c_int;

        pub fn jlrs_egal(
            a: *const crate::types::jl_value_t,
            b: *const crate::types::jl_value_t,
        ) -> std::ffi::c_int;

        pub fn jlrs_is_concrete_type(v: *mut crate::types::jl_value_t) -> std::ffi::c_int;

        pub fn jlrs_box_long(x: isize) -> *mut crate::types::jl_value_t;

        pub fn jlrs_box_ulong(x: usize) -> *mut crate::types::jl_value_t;

        pub fn jlrs_unbox_long(x: *mut crate::types::jl_value_t) -> isize;

        pub fn jlrs_unbox_ulong(x: *mut crate::types::jl_value_t) -> usize;

        pub fn jlrs_current_task() -> *mut crate::types::jl_task_t;

        pub fn jlrs_unsized_scope(
            frame_size: usize,
            trampoline: crate::types::jlrs_unsized_scope_trampoline_t,
            callback: *mut std::ffi::c_void,
            result: *mut std::ffi::c_void,
        );

        pub fn jlrs_try_catch(
            callback: *mut std::ffi::c_void,
            trampoline: crate::types::jlrs_try_catch_trampoline_t,
            result: *mut std::ffi::c_void,
        ) -> crate::types::jlrs_catch_t;

        pub fn jlrs_dimtuple_type(rank: usize) -> *mut crate::types::jl_datatype_t;

        pub fn jlrs_tuple_of(
            values: *mut *mut crate::types::jl_value_t,
            n: usize,
        ) -> *mut crate::types::jl_value_t;

        pub fn jlrs_call_unchecked(
            f: *mut crate::types::jl_value_t,
            args: *mut *mut crate::types::jl_value_t,
            nargs: u32,
        ) -> *mut crate::types::jl_value_t;

        pub fn jlrs_datatype_layout(
            t: *mut crate::types::jl_datatype_t,
        ) -> *const crate::types::jl_datatype_layout_t;

        pub fn jlrs_datatype_has_layout(t: *mut crate::types::jl_datatype_t) -> std::ffi::c_int;

        pub fn jlrs_datatype_typename(
            ty: *mut crate::types::jl_datatype_t,
        ) -> *mut crate::types::jl_typename_t;

        pub fn jlrs_datatype_first_ptr(ty: *mut crate::types::jl_datatype_t) -> i32;

        pub fn jlrs_field_offset(st: *mut crate::types::jl_datatype_t, i: std::ffi::c_int) -> u32;

        pub fn jlrs_field_size(st: *mut crate::types::jl_datatype_t, i: std::ffi::c_int) -> u32;

        pub fn jlrs_datatype_super(
            ty: *mut crate::types::jl_datatype_t,
        ) -> *mut crate::types::jl_datatype_t;

        pub fn jlrs_datatype_parameters(
            ty: *mut crate::types::jl_datatype_t,
        ) -> *mut crate::types::jl_svec_t;

        pub fn jlrs_datatype_parameter(
            ty: *mut crate::types::jl_datatype_t,
            n: usize,
        ) -> *mut crate::types::jl_value_t;

        pub fn jlrs_datatype_instance(
            ty: *mut crate::types::jl_datatype_t,
        ) -> *mut crate::types::jl_value_t;

        pub fn jlrs_datatype_isinlinealloc(ty: *mut crate::types::jl_datatype_t) -> u8;

        pub fn jlrs_datatype_abstract(ty: *mut crate::types::jl_datatype_t) -> u8;

        pub fn jlrs_datatype_mutable(ty: *mut crate::types::jl_datatype_t) -> u8;

        pub fn jlrs_datatype_zeroinit(ty: *mut crate::types::jl_datatype_t) -> u8;

        pub fn jlrs_set_nthreads(nthreads: i16);

        pub fn jlrs_gc_safe_enter(ptls: *mut crate::types::jl_tls_states_t) -> i8;

        pub fn jlrs_gc_unsafe_enter(ptls: *mut crate::types::jl_tls_states_t) -> i8;

        pub fn jlrs_gc_safe_leave(ptls: *mut crate::types::jl_tls_states_t, state: i8);

        pub fn jlrs_gc_unsafe_leave(ptls: *mut crate::types::jl_tls_states_t, state: i8);

        pub fn jlrs_tvar_name(tvar: *mut crate::types::jl_tvar_t) -> *mut crate::types::jl_sym_t;

        pub fn jlrs_tvar_lb(tvar: *mut crate::types::jl_tvar_t) -> *mut crate::types::jl_value_t;

        pub fn jlrs_tvar_ub(tvar: *mut crate::types::jl_tvar_t) -> *mut crate::types::jl_value_t;

        pub fn jlrs_unionall_body(
            ua: *mut crate::types::jl_unionall_t,
        ) -> *mut crate::types::jl_value_t;

        pub fn jlrs_unionall_tvar(
            ua: *mut crate::types::jl_unionall_t,
        ) -> *mut crate::types::jl_tvar_t;

        pub fn jlrs_typename_name(
            tn: *mut crate::types::jl_typename_t,
        ) -> *mut crate::types::jl_sym_t;

        pub fn jlrs_typename_module(
            tn: *mut crate::types::jl_typename_t,
        ) -> *mut crate::types::jl_module_t;

        pub fn jlrs_typename_wrapper(
            tn: *mut crate::types::jl_typename_t,
        ) -> *mut crate::types::jl_value_t;

        pub fn jlrs_union_a(u: *mut crate::types::jl_uniontype_t) -> *mut crate::types::jl_value_t;

        pub fn jlrs_union_b(u: *mut crate::types::jl_uniontype_t) -> *mut crate::types::jl_value_t;

        pub fn jlrs_module_name(m: *mut crate::types::jl_module_t) -> *mut crate::types::jl_sym_t;

        pub fn jlrs_module_parent(
            m: *mut crate::types::jl_module_t,
        ) -> *mut crate::types::jl_module_t;

        pub fn jlrs_expr_head(expr: *mut crate::types::jl_expr_t) -> *mut crate::types::jl_sym_t;

        pub fn jlrs_symbol_hash(sym: *mut crate::types::jl_sym_t) -> usize;

        pub fn jlrs_arrayref(
            a: *mut crate::types::jl_array_t,
            i: usize,
        ) -> *mut crate::types::jl_value_t;

        pub fn jlrs_arrayset(
            a: *mut crate::types::jl_array_t,
            v: *mut crate::types::jl_value_t,
            i: usize,
        );

        pub fn jlrs_array_data_owner(
            a: *mut crate::types::jl_array_t,
        ) -> *mut crate::types::jl_value_t;

        pub fn jlrs_array_typetagdata(a: *mut crate::types::jl_array_t) -> *mut std::ffi::c_char;

        pub fn jlrs_array_is_pointer_array(a: *mut crate::types::jl_array_t) -> std::ffi::c_int;

        pub fn jlrs_array_is_union_array(a: *mut crate::types::jl_array_t) -> std::ffi::c_int;

        pub fn jlrs_array_has_pointers(a: *mut crate::types::jl_array_t) -> std::ffi::c_int;

        pub fn jlrs_array_how(a: *mut crate::types::jl_array_t) -> std::ffi::c_int;

        pub fn jlrs_get_ptls_states() -> *mut crate::types::jl_tls_states_t;

        pub fn jlrs_ptls_from_gcstack(
            pgcstack: *mut *mut crate::types::jl_gcframe_t,
        ) -> *mut crate::types::jl_tls_states_t;

        pub fn jlrs_task_gc_state() -> i8;

        pub fn jlrs_clear_gc_stack();

        pub fn jlrs_typename_names(
            tn: *mut crate::types::jl_typename_t,
        ) -> *mut crate::types::jl_svec_t;

        pub fn jlrs_typename_atomicfields(tn: *mut crate::types::jl_typename_t) -> *const u32;

        pub fn jlrs_typename_abstract(tn: *mut crate::types::jl_typename_t) -> u8;

        pub fn jlrs_typename_mutable(tn: *mut crate::types::jl_typename_t) -> u8;

        pub fn jlrs_typename_mayinlinealloc(tn: *mut crate::types::jl_typename_t) -> u8;

        pub fn jlrs_lock_value(v: *mut crate::types::jl_value_t);

        pub fn jlrs_unlock_value(v: *mut crate::types::jl_value_t);

        pub fn jlrs_typename_constfields(tn: *mut crate::types::jl_typename_t) -> *const u32;

        pub fn jlrs_set_nthreadpools(nthreadpools: i8);

        pub fn jlrs_set_nthreads_per_pool(nthreads_per_pool: *const i16);

        pub fn jlrs_init_missing_functions();

        // Added in Julia 1.12

        #[cfg(not(any(julia_1_10, julia_1_11)))]
        pub fn jlrs_declare_constant_val(
            b: *mut crate::types::jl_binding_t,
            m: *mut crate::types::jl_module_t,
            var: *mut crate::types::jl_sym_t,
            val: *mut crate::types::jl_value_t,
        ) -> *mut crate::types::jl_binding_partition_t;
    }
}
