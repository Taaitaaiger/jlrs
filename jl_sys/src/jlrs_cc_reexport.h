// Static inline functions and macros. Such "functions" can't be directly called from Rust, so
// they're reexported as functions.

#ifndef JLRS_CC_REEXPORT_H
#define JLRS_CC_REEXPORT_H

#ifdef __cplusplus
extern "C"
{
#endif
    jl_value_t *jlrs_typeof(jl_value_t *v);
#if JULIA_VERSION_MINOR <= 6
    void jlrs_gc_wb(void *parent, void *ptr);
#endif
#if JULIA_VERSION_MINOR >= 7
    void jlrs_gc_wb(const void *parent, const void *ptr);
#endif
    size_t jlrs_svec_len(jl_svec_t *t);
    jl_value_t **jlrs_svec_data(jl_svec_t *t);
    jl_value_t *jlrs_svecref(void *t, size_t i);          // X
    jl_value_t *jlrs_svecset(void *t, size_t i, void *x); // X
    size_t jlrs_array_len(jl_array_t *a);
#if JULIA_VERSION_MINOR <= 10
    void *jlrs_array_data(jl_array_t *a);
#endif
#if JULIA_VERSION_MINOR >= 11
    void *jlrs_array_data(jl_array_t *a);
#endif
    size_t jlrs_array_ndims(jl_array_t *a);
    jl_value_t *jlrs_exprarg(jl_expr_t *e, size_t n);            // X
    void jlrs_exprargset(jl_expr_t *e, size_t n, jl_value_t *v); // X
    size_t jlrs_expr_nargs(jl_expr_t *e);                        // X
    size_t jlrs_nparams(jl_datatype_t *t);
    size_t jlrs_string_len(jl_value_t *s);
    jl_svec_t *jlrs_get_fieldtypes(jl_datatype_t *st);
    uint32_t jlrs_datatype_size(jl_datatype_t *t);
    uint16_t jlrs_datatype_align(jl_datatype_t *t);
    uint32_t jlrs_datatype_nfields(jl_datatype_t *t);
    char *jlrs_symbol_name(jl_sym_t *s);
    int jlrs_field_isptr(jl_datatype_t *st, int i);
    uint32_t jlrs_ptr_offset(jl_datatype_t *st, int i); // X
    int jlrs_is_primitivetype(void *v);
    int jlrs_isbits(void *t);
    int jlrs_egal(const jl_value_t *a, const jl_value_t *b);
    int jlrs_is_concrete_type(jl_value_t *v);
    jl_value_t *jlrs_box_long(intptr_t x);
    jl_value_t *jlrs_box_ulong(size_t x);
    intptr_t jlrs_unbox_long(jl_value_t *x);
    size_t jlrs_unbox_ulong(jl_value_t *x);
    jl_value_t *jlrs_apply(jl_value_t **args, uint32_t nargs); // X
    jl_task_t *jlrs_current_task();                            // X
    const jl_datatype_layout_t *jlrs_datatype_layout(jl_datatype_t *t);
#ifdef __cplusplus
}
#endif
#endif