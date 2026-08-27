// Static inline functions and macros. Such "functions" can't be directly called from Rust, so
// they're reexported as functions.

#ifndef JLRS_CC_REEXPORT_H
#define JLRS_CC_REEXPORT_H

#define PREDICATES_10(XX)                           \
    XX(is_nothing, jl_is_nothing)                   \
    XX(is_tuple, jl_is_tuple)                       \
    XX(is_namedtuple, jl_is_namedtuple)             \
    XX(is_svec, jl_is_svec)                         \
    XX(is_datatype, jl_is_datatype)                 \
    XX(is_mutable, jl_is_mutable)                   \
    XX(is_mutable_datatype, jl_is_mutable_datatype) \
    XX(is_immutable, jl_is_immutable)               \
    XX(is_uniontype, jl_is_uniontype)               \
    XX(is_typevar, jl_is_typevar)                   \
    XX(is_unionall, jl_is_unionall)                 \
    XX(is_vararg, jl_is_vararg)                     \
    XX(is_typename, jl_is_typename)                 \
    XX(is_int8, jl_is_int8)                         \
    XX(is_int16, jl_is_int16)                       \
    XX(is_int32, jl_is_int32)                       \
    XX(is_int64, jl_is_int64)                       \
    XX(is_uint8, jl_is_uint8)                       \
    XX(is_uint16, jl_is_uint16)                     \
    XX(is_uint32, jl_is_uint32)                     \
    XX(is_uint64, jl_is_uint64)                     \
    XX(is_bool, jl_is_bool)                         \
    XX(is_symbol, jl_is_symbol)                     \
    XX(is_expr, jl_is_expr)                         \
    XX(is_binding, jl_is_binding)                   \
    XX(is_module, jl_is_module)                     \
    XX(is_task, jl_is_task)                         \
    XX(is_string, jl_is_string)                     \
    XX(is_uint8pointer, jl_is_uint8pointer)

#define PREDICATES_12(XX)                                       \
    XX(may_be_immutable_datatype, jl_may_be_immutable_datatype) \
    XX(is_array_any, jl_is_array_any)

#define PREDICATES_14(XX)       \
    XX(is_typeeq, jl_is_typeeq) \
    XX(is_typeegal, jl_is_typeegal)

#ifdef __cplusplus
extern "C"
{
#endif
    jl_value_t *jlrs_typeof(jl_value_t *v);
    void jlrs_gc_wb(void *parent, void *ptr);
    size_t jlrs_svec_len(jl_svec_t *t);
    jl_value_t **jlrs_svec_data(jl_svec_t *t);
    jl_value_t *jlrs_svecref(void *t, size_t i);
    jl_value_t *jlrs_svecset(void *t, size_t i, void *x);
    size_t jlrs_array_len(jl_array_t *a);
    void *jlrs_array_data(jl_array_t *a);
    jl_value_t *jlrs_exprarg(jl_expr_t *e, size_t n);
    void jlrs_exprargset(jl_expr_t *e, size_t n, jl_value_t *v);
    size_t jlrs_expr_nargs(jl_expr_t *e);
    size_t jlrs_nparams(jl_datatype_t *t);
    size_t jlrs_string_len(jl_value_t *s);
    jl_svec_t *jlrs_get_fieldtypes(jl_datatype_t *st);
    uint32_t jlrs_datatype_size(jl_datatype_t *t);
    uint16_t jlrs_datatype_align(jl_datatype_t *t);
    uint32_t jlrs_datatype_nfields(jl_datatype_t *t);
    char *jlrs_symbol_name(jl_sym_t *s);
    int jlrs_field_isptr(jl_datatype_t *st, int i);
    int jlrs_is_primitivetype(void *v);
    int jlrs_isbits(void *t);
    int jlrs_egal(const jl_value_t *a, const jl_value_t *b);
    int jlrs_is_concrete_type(jl_value_t *v);
    jl_value_t *jlrs_box_long(intptr_t x);
    jl_value_t *jlrs_box_ulong(size_t x);
    intptr_t jlrs_unbox_long(jl_value_t *x);
    size_t jlrs_unbox_ulong(jl_value_t *x);
    jl_task_t *jlrs_current_task();
    const jl_datatype_layout_t *jlrs_datatype_layout(jl_datatype_t *t);
    int8_t jlrs_gc_safe_enter(jl_ptls_t ptls);
    int8_t jlrs_gc_unsafe_enter(jl_ptls_t ptls);
    void jlrs_gc_safe_leave(jl_ptls_t ptls, int8_t state);
    void jlrs_gc_unsafe_leave(jl_ptls_t ptls, int8_t state);

#define XX(name, function) int jlrs_##name(jl_value_t *v);
    PREDICATES_10(XX)
#if JULIA_VERSION_MINOR >= 12
    PREDICATES_12(XX)
#endif
#if JULIA_VERSION_MINOR >= 14
    PREDICATES_14(XX)
#endif
#undef XX

#ifdef __cplusplus
}
#endif
#endif
