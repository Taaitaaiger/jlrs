STATIC_INLINE int jl_array_ndimwords(uint32_t ndims) JL_NOTSAFEPOINT
{
    return (ndims < 3 ? 0 : ndims-2);
}
STATIC_INLINE void jl_gc_wb(void *parent, void *ptr) JL_NOTSAFEPOINT
{
    // parent and ptr isa jl_value_t*
    if (__unlikely(jl_astaggedvalue(parent)->bits.gc == 3 && // parent is old and not in remset
                   (jl_astaggedvalue(ptr)->bits.gc & 1) == 0)) // ptr is young
        jl_gc_queue_root((jl_value_t*)parent);
}
STATIC_INLINE void jl_gc_wb_back(void *ptr) JL_NOTSAFEPOINT // ptr isa jl_value_t*
{
    // if ptr is old
    if (__unlikely(jl_astaggedvalue(ptr)->bits.gc == 3)) {
        jl_gc_queue_root((jl_value_t*)ptr);
    }
}
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
STATIC_INLINE jl_value_t *jl_svecref(void *t JL_PROPAGATES_ROOT, size_t i) JL_NOTSAFEPOINT
{
    assert(jl_typeis(t,jl_simplevector_type));
    assert(i < jl_svec_len(t));
    return jl_svec_data(t)[i];
}
STATIC_INLINE jl_value_t *jl_array_ptr_ref(void *a JL_PROPAGATES_ROOT, size_t i) JL_NOTSAFEPOINT
{
    assert(((jl_array_t*)a)->flags.ptrarray);
    assert(i < jl_array_len(a));
    return ((jl_value_t**)(jl_array_data(a)))[i];
}
STATIC_INLINE uint8_t jl_array_uint8_ref(void *a, size_t i) JL_NOTSAFEPOINT
{
    assert(i < jl_array_len(a));
    assert(jl_typeis(a, jl_array_uint8_type));
    return ((uint8_t*)(jl_array_data(a)))[i];
}
STATIC_INLINE void jl_array_uint8_set(void *a, size_t i, uint8_t x) JL_NOTSAFEPOINT
{
    assert(i < jl_array_len(a));
    assert(jl_typeis(a, jl_array_uint8_type));
    ((uint8_t*)(jl_array_data(a)))[i] = x;
}
STATIC_INLINE jl_svec_t *jl_field_names(jl_datatype_t *st) JL_NOTSAFEPOINT
{
    jl_svec_t *names = st->names;
    if (!names)
        names = st->name->names;
    return names;
}
STATIC_INLINE jl_sym_t *jl_field_name(jl_datatype_t *st, size_t i) JL_NOTSAFEPOINT
{
    return (jl_sym_t*)jl_svecref(jl_field_names(st), i);
}
STATIC_INLINE jl_value_t *jl_field_type(jl_datatype_t *st JL_PROPAGATES_ROOT, size_t i)
{
    return jl_svecref(jl_get_fieldtypes(st), i);
}
STATIC_INLINE jl_value_t *jl_field_type_concrete(jl_datatype_t *st JL_PROPAGATES_ROOT, size_t i) JL_NOTSAFEPOINT
{
    assert(st->types);
    return jl_svecref(st->types, i);
}
STATIC_INLINE char *jl_symbol_name_(jl_sym_t *s) JL_NOTSAFEPOINT
{
    return (char*)s + LLT_ALIGN(sizeof(jl_sym_t), sizeof(void*));
}
STATIC_INLINE int jl_is_kind(jl_value_t *v) JL_NOTSAFEPOINT
{
    return (v==(jl_value_t*)jl_uniontype_type || v==(jl_value_t*)jl_datatype_type ||
            v==(jl_value_t*)jl_unionall_type || v==(jl_value_t*)jl_typeofbottom_type);
}
STATIC_INLINE int jl_is_type(jl_value_t *v) JL_NOTSAFEPOINT
{
    return jl_is_kind(jl_typeof(v));
}
STATIC_INLINE int jl_is_primitivetype(void *v) JL_NOTSAFEPOINT
{
    return (jl_is_datatype(v) && jl_is_immutable(v) &&
            ((jl_datatype_t*)(v))->layout &&
            jl_datatype_nfields(v) == 0 &&
            jl_datatype_size(v) > 0);
}
STATIC_INLINE int jl_is_structtype(void *v) JL_NOTSAFEPOINT
{
    return (jl_is_datatype(v) &&
            !((jl_datatype_t*)(v))->abstract &&
            !jl_is_primitivetype(v));
}
STATIC_INLINE int jl_isbits(void *t) JL_NOTSAFEPOINT // corresponding to isbits() in julia
{
    return (jl_is_datatype(t) && ((jl_datatype_t*)t)->isbitstype);
}
STATIC_INLINE int jl_is_datatype_singleton(jl_datatype_t *d) JL_NOTSAFEPOINT
{
    return (d->instance != NULL);
}
STATIC_INLINE int jl_is_abstracttype(void *v) JL_NOTSAFEPOINT
{
    return (jl_is_datatype(v) && ((jl_datatype_t*)(v))->abstract);
}
STATIC_INLINE int jl_is_array_type(void *t) JL_NOTSAFEPOINT
{
    return (jl_is_datatype(t) &&
            ((jl_datatype_t*)(t))->name == jl_array_typename);
}
STATIC_INLINE int jl_is_array(void *v) JL_NOTSAFEPOINT
{
    jl_value_t *t = jl_typeof(v);
    return jl_is_array_type(t);
}
STATIC_INLINE int jl_is_cpointer_type(jl_value_t *t) JL_NOTSAFEPOINT
{
    return (jl_is_datatype(t) &&
            ((jl_datatype_t*)(t))->name == ((jl_datatype_t*)jl_pointer_type->body)->name);
}
STATIC_INLINE int jl_is_llvmpointer_type(jl_value_t *t) JL_NOTSAFEPOINT
{
    return (jl_is_datatype(t) &&
            ((jl_datatype_t*)(t))->name == jl_llvmpointer_typename);
}
STATIC_INLINE int jl_is_abstract_ref_type(jl_value_t *t) JL_NOTSAFEPOINT
{
    return (jl_is_datatype(t) &&
            ((jl_datatype_t*)(t))->name == ((jl_datatype_t*)jl_ref_type->body)->name);
}
STATIC_INLINE int jl_is_tuple_type(void *t) JL_NOTSAFEPOINT
{
    return (jl_is_datatype(t) &&
            ((jl_datatype_t*)(t))->name == jl_tuple_typename);
}
STATIC_INLINE int jl_is_namedtuple_type(void *t) JL_NOTSAFEPOINT
{
    return (jl_is_datatype(t) &&
            ((jl_datatype_t*)(t))->name == jl_namedtuple_typename);
}
STATIC_INLINE int jl_is_vecelement_type(jl_value_t* t) JL_NOTSAFEPOINT
{
    return (jl_is_datatype(t) &&
            ((jl_datatype_t*)(t))->name == jl_vecelement_typename);
}
STATIC_INLINE int jl_is_type_type(jl_value_t *v) JL_NOTSAFEPOINT
{
    return (jl_is_datatype(v) &&
            ((jl_datatype_t*)(v))->name == ((jl_datatype_t*)jl_type_type->body)->name);
}
STATIC_INLINE int jl_is_dispatch_tupletype(jl_value_t *v) JL_NOTSAFEPOINT
{
    return jl_is_datatype(v) && ((jl_datatype_t*)v)->isdispatchtuple;
}
STATIC_INLINE int jl_is_concrete_type(jl_value_t *v) JL_NOTSAFEPOINT
{
    return jl_is_datatype(v) && ((jl_datatype_t*)v)->isconcretetype;
}
STATIC_INLINE int jl_is_vararg_type(jl_value_t *v) JL_NOTSAFEPOINT
{
    v = jl_unwrap_unionall(v);
    return (jl_is_datatype(v) &&
            ((jl_datatype_t*)(v))->name == jl_vararg_typename);
}
STATIC_INLINE jl_value_t *jl_unwrap_vararg(jl_value_t *v) JL_NOTSAFEPOINT
{
    return jl_tparam0(jl_unwrap_unionall(v));
}
STATIC_INLINE size_t jl_vararg_length(jl_value_t *v) JL_NOTSAFEPOINT
{
    assert(jl_is_vararg_type(v));
    jl_value_t *len = jl_tparam1(jl_unwrap_unionall(v));
    assert(jl_is_long(len));
    return jl_unbox_long(len);
}
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
STATIC_INLINE int jl_is_va_tuple(jl_datatype_t *t) JL_NOTSAFEPOINT
{
    assert(jl_is_tuple_type(t));
    size_t l = jl_svec_len(t->parameters);
    return (l>0 && jl_is_vararg_type(jl_tparam(t,l-1)));
}
STATIC_INLINE jl_vararg_kind_t jl_va_tuple_kind(jl_datatype_t *t) JL_NOTSAFEPOINT
{
    t = (jl_datatype_t*)jl_unwrap_unionall((jl_value_t*)t);
    assert(jl_is_tuple_type(t));
    size_t l = jl_svec_len(t->parameters);
    if (l == 0)
        return JL_VARARG_NONE;
    return jl_vararg_kind(jl_tparam(t,l-1));
}
STATIC_INLINE jl_function_t *jl_get_function(jl_module_t *m, const char *name)
{
    return (jl_function_t*)jl_get_global(m, jl_symbol(name));
}
STATIC_INLINE int jl_vinfo_sa(uint8_t vi)
{
    return (vi&16)!=0;
}
STATIC_INLINE int jl_vinfo_usedundef(uint8_t vi)
{
    return (vi&32)!=0;
}
