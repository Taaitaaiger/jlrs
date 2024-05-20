#include "jlrs_cc.h"

#ifdef __cplusplus
extern "C"
{
#endif
    jl_value_t *jlrs_typeof(jl_value_t *v) { return jl_typeof(v); }
    void jlrs_gc_wb(void *parent, void *ptr) { jl_gc_wb(parent, ptr); }
    size_t jlrs_svec_len(jl_svec_t *t) { return jl_svec_len(t); }
    jl_value_t **jlrs_svec_data(jl_svec_t *t) { return jl_svec_data(t); }
    jl_value_t *jlrs_svecref(void *t, size_t i) { return jl_svecref(t, i); }
    jl_value_t *jlrs_svecset(void *t, size_t i, void *x) { return jl_svecset(t, i, x); }
    size_t jlrs_array_len(jl_array_t *a) { return jl_array_len(a); }
#if JULIA_VERSION_MINOR <= 10
    void *jlrs_array_data(jl_array_t *a) { return jl_array_data(a); }
#endif
#if JULIA_VERSION_MINOR >= 11
    void *jlrs_array_data(jl_array_t *a)
    {
        // Copied from jl_array_ptr.
        const jl_datatype_layout_t *layout = ((jl_datatype_t *)jl_typetagof(a->ref.mem))->layout;
        if (layout->flags.arrayelem_isunion || layout->size == 0)
            return (char *)a->ref.mem->ptr + (size_t)jl_array_data_(a);
        return jl_array_data_(a);
    }
#endif
    size_t jlrs_array_ndims(jl_array_t *a) { return jl_array_ndims(a); }
    jl_value_t *jlrs_exprarg(jl_expr_t *e, size_t n) { return jl_exprarg(e, n); }
    void jlrs_exprargset(jl_expr_t *e, size_t n, jl_value_t *v) { jl_exprargset(e, n, v); }
    size_t jlrs_expr_nargs(jl_expr_t *e) { return jl_expr_nargs(e); }
    size_t jlrs_nparams(jl_datatype_t *t) { return jl_nparams(t); }
    size_t jlrs_string_len(jl_value_t *s) { return jl_string_len(s); }
    jl_svec_t *jlrs_get_fieldtypes(jl_datatype_t *st) { return jl_get_fieldtypes(st); }
    uint32_t jlrs_datatype_size(jl_datatype_t *t) { return jl_datatype_size(t); }
    uint16_t jlrs_datatype_align(jl_datatype_t *t) { return jl_datatype_align(t); }
    uint32_t jlrs_datatype_nfields(jl_datatype_t *t) { return jl_datatype_nfields(t); }
    char *jlrs_symbol_name(jl_sym_t *s) { return jl_symbol_name(s); }
    int jlrs_field_isptr(jl_datatype_t *st, int i) { return jl_field_isptr(st, i); }
    uint32_t jlrs_ptr_offset(jl_datatype_t *st, int i) { return jl_ptr_offset(st, i); }
    int jlrs_is_primitivetype(void *v) { return jl_is_primitivetype(v); }
    int jlrs_isbits(void *t) { return jl_isbits(t); }
    int jlrs_egal(const jl_value_t *a, const jl_value_t *b) { return jl_egal((jl_value_t *)a, (jl_value_t *)b); }
    int jlrs_is_concrete_type(jl_value_t *v) { return jl_is_concrete_type(v); }
    jl_value_t *jlrs_box_long(intptr_t x) { return jl_box_long(x); }
    jl_value_t *jlrs_box_ulong(size_t x) { return jl_box_ulong(x); }
    intptr_t jlrs_unbox_long(jl_value_t *x) { return jl_unbox_long(x); }
    size_t jlrs_unbox_ulong(jl_value_t *x) { return jl_unbox_ulong(x); }
    jl_value_t *jlrs_apply(jl_value_t **args, uint32_t nargs) { return jl_apply(args, nargs); }
    jl_task_t *jlrs_current_task()
    {
#if JULIA_VERSION_MINOR == 6
        return jl_current_task;
#else
    jl_gcframe_t **pgcstack = jl_get_pgcstack();
    if (pgcstack == NULL)
    {
        return NULL;
    }

    return container_of(pgcstack, jl_task_t, gcstack);
#endif
    }
    const jl_datatype_layout_t *jlrs_datatype_layout(jl_datatype_t *t) { return jl_datatype_layout(t); }
    int8_t jlrs_gc_safe_enter(jl_ptls_t ptls)
    {
        return jl_gc_safe_enter(ptls);
    }

    int8_t jlrs_gc_unsafe_enter(jl_ptls_t ptls)
    {
        return jl_gc_unsafe_enter(ptls);
    }

    void jlrs_gc_safe_leave(jl_ptls_t ptls, int8_t state)
    {
        jl_gc_safe_leave(ptls, state);
    }

    void jlrs_gc_unsafe_leave(jl_ptls_t ptls, int8_t state)
    {
        jl_gc_unsafe_leave(ptls, state);
    }
#ifdef __cplusplus
}
#endif
