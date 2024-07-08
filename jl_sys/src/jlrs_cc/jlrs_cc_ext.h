// Functions and types that expose some functionality that can't be directly used from Rust, or
// would require exposing many implementation details to Rust.

#ifndef JLRS_CC_EXT_H
#define JLRS_CC_EXT_H

#ifdef __cplusplus
extern "C" {
#endif //__cplusplus

    typedef enum
    {
        JLRS_CATCH_OK = 0,
        JLRS_CATCH_EXCEPTION = 1,
        JLRS_CATCH_PANIC = 2,
    } jlrs_catch_tag_t;

    typedef struct
    {
        jlrs_catch_tag_t tag;
        void *error;
    } jlrs_catch_t;

    typedef jlrs_catch_t (*jlrs_try_catch_trampoline_t)(void *callback, void *result);
    typedef void (*jlrs_unsized_scope_trampoline_t)(jl_gcframe_t *frame, void *callback, void *result);

    void jlrs_unsized_scope(size_t frame_size, jlrs_unsized_scope_trampoline_t trampoline, void *callback, void *result);
    jlrs_catch_t jlrs_try_catch(void *callback, jlrs_try_catch_trampoline_t trampoline, void *result);

    jl_datatype_t *jlrs_dimtuple_type(size_t rank);
    jl_value_t *jlrs_tuple_of(jl_value_t **values, size_t n);

    jl_value_t *jlrs_call_unchecked(jl_function_t *f, jl_value_t **args, uint32_t nargs);

    int jlrs_datatype_has_layout(jl_datatype_t *t);

    // datatype field getters
    uint32_t jlrs_datatype_nptrs(jl_datatype_t *ty);
    jl_typename_t *jlrs_datatype_typename(jl_datatype_t *ty);
    int32_t jlrs_datatype_first_ptr(jl_datatype_t *ty);
    uint32_t jlrs_field_offset(jl_datatype_t *st, int i);
    uint32_t jlrs_field_size(jl_datatype_t *st, int i);
    jl_datatype_t *jlrs_datatype_super(jl_datatype_t *ty);
    jl_svec_t *jlrs_datatype_parameters(jl_datatype_t *ty);
    jl_value_t *jlrs_datatype_instance(jl_datatype_t *ty);
    uint8_t jlrs_datatype_zeroinit(jl_datatype_t *ty);
    uint8_t jlrs_datatype_isconcretetype(jl_datatype_t *ty);

    uint8_t jlrs_datatype_isinlinealloc(jl_datatype_t *ty);
    uint8_t jlrs_datatype_abstract(jl_datatype_t *ty);
    uint8_t jlrs_datatype_mutable(jl_datatype_t *ty);

    // option field setters
    void jlrs_set_nthreads(int16_t nthreads);

#if JULIA_VERSION_MINOR >= 9
    void jlrs_set_nthreadpools(int8_t nthreadpools);
#endif
#if JULIA_VERSION_MINOR >= 9
    void jlrs_set_nthreads_per_pool(const int16_t *nthreads_per_pool);
#endif
    // tvar field getters
    jl_sym_t *jlrs_tvar_name(jl_tvar_t *tvar);
    jl_value_t *jlrs_tvar_lb(jl_tvar_t *tvar);
    jl_value_t *jlrs_tvar_ub(jl_tvar_t *tvar);

    // unionall field getters
    jl_value_t *jlrs_unionall_body(jl_unionall_t *ua);
    jl_tvar_t *jlrs_unionall_tvar(jl_unionall_t *ua);

    // typename field getters
    jl_sym_t *jlrs_typename_name(jl_typename_t *tn);
    jl_module_t *jlrs_typename_module(jl_typename_t *tn);
    jl_value_t *jlrs_typename_wrapper(jl_typename_t *tn);

#if JULIA_VERSION_MINOR >= 7
    const uint32_t *jlrs_typename_atomicfields(jl_typename_t *tn);
    uint8_t jlrs_typename_abstract(jl_typename_t *tn);
    uint8_t jlrs_typename_mutable(jl_typename_t *tn);
    uint8_t jlrs_typename_mayinlinealloc(jl_typename_t *tn);
#endif // JULIA_VERSION_MINOR >= 7

    jl_svec_t *jlrs_typename_names(jl_typename_t *tn);

#if JULIA_VERSION_MINOR >= 8
    const uint32_t *jlrs_typename_constfields(jl_typename_t *tn);
#endif // JULIA_VERSION_MINOR >= 8

    // union field getters
    jl_value_t *jlrs_union_a(jl_uniontype_t *u);
    jl_value_t *jlrs_union_b(jl_uniontype_t *u);

    // module field getters
    jl_sym_t *jlrs_module_name(jl_module_t *m);
    jl_module_t *jlrs_module_parent(jl_module_t *m);

    // expr heaf field getter
    jl_sym_t *jlrs_expr_head(jl_expr_t *expr);

    uintptr_t jlrs_symbol_hash(jl_sym_t *sym);

#if JULIA_VERSION_MINOR >= 9
    // enter / exit threaded region.
    void jl_enter_threaded_region(void);
    void jl_exit_threaded_region(void);
#endif // JULIA_VERSION_MINOR >= 9

    // Removed array functions
    jl_value_t *jlrs_arrayref(jl_array_t *a, size_t i);
    void jlrs_arrayset(jl_array_t *a, jl_value_t *v, size_t i);
    jl_value_t *jlrs_array_data_owner(jl_array_t *a);
    char *jlrs_array_typetagdata(jl_array_t *a);

    int jlrs_array_is_pointer_array(jl_array_t *a);
    int jlrs_array_is_union_array(jl_array_t *a);
    int jlrs_array_has_pointers(jl_array_t *a);
    int jlrs_array_how(jl_array_t *a);

#if JULIA_VERSION_MINOR <= 10
    const jl_datatype_layout_t *jl_datatype_layout(jl_datatype_t *t);
#endif

    void jlrs_set_global(jl_module_t *m JL_ROOTING_ARGUMENT, jl_sym_t *var, jl_value_t *val JL_ROOTED_ARGUMENT);
#ifdef __cplusplus
}
#endif // __cplusplus

#endif // JLRS_CC_EXT_H
