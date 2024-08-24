#include "jlrs_cc.h"

#ifdef __cplusplus
extern "C"
{
#endif

    void jlrs_unsized_scope(size_t frame_size, jlrs_unsized_scope_trampoline_t trampoline, void *callback, void *result)
    {
        jl_value_t **args;
        JL_GC_PUSHARGS(args, frame_size);
        trampoline((jl_gcframe_t *)&(((void **)args)[-2]), callback, result);
        JL_GC_POP();
    }

    jlrs_catch_t jlrs_try_catch(void *callback, jlrs_try_catch_trampoline_t trampoline, void *result)
    {
        jlrs_catch_t res = {JLRS_CATCH_OK, 0};

#ifndef JLRS_WINDOWS_LTS
        JL_TRY
        {
            res = trampoline(callback, result);
        }
        JL_CATCH
        {
#if JULIA_VERSION_MINOR >= 11
            jl_value_t *exc = jl_current_exception(jl_current_task);
#else
            jl_value_t *exc = jl_current_exception();
#endif
            jlrs_catch_t the_exc = {JLRS_CATCH_EXCEPTION, exc};
            return the_exc;
        }
#else
    res = trampoline(callback, result);
#endif
        return res;
    }

    jl_value_t *jlrs_call_unchecked(jl_function_t *f, jl_value_t **args, uint32_t nargs)
    {
#if JULIA_VERSION_MINOR == 6
        jl_value_t *v;
        nargs++; // add f to args
        jl_value_t **argv;
        JL_GC_PUSHARGS(argv, nargs);
        argv[0] = (jl_value_t *)f;
        for (unsigned int i = 1; i < nargs; i++)
            argv[i] = args[i - 1];
        size_t last_age = jl_get_ptls_states()->world_age;
        jl_get_ptls_states()->world_age = jl_get_world_counter();
        v = jl_apply(argv, nargs);
        jl_get_ptls_states()->world_age = last_age;
        JL_GC_POP();
        return v;
#else
    jl_value_t *v;
    jl_task_t *ct = jl_current_task;
    nargs++; // add f to args
    jl_value_t **argv;
    JL_GC_PUSHARGS(argv, nargs);
    argv[0] = (jl_value_t *)f;
    for (unsigned int i = 1; i < nargs; i++)
        argv[i] = args[i - 1];
    size_t last_age = ct->world_age;
    ct->world_age = jl_get_world_counter();
    v = jl_apply(argv, nargs);
    ct->world_age = last_age;
    JL_GC_POP();
    return v;
#endif
    }

#if JULIA_VERSION_MINOR <= 10
    const jl_datatype_layout_t *jl_datatype_layout(jl_datatype_t *t)
    {
        return t->layout;
    }
#endif

    uint32_t jlrs_datatype_nptrs(jl_datatype_t *ty)
    {
        return jl_datatype_layout(ty)->npointers;
    }

    jl_typename_t *jlrs_datatype_typename(jl_datatype_t *ty)
    {
        return ty->name;
    }

    int32_t jlrs_datatype_first_ptr(jl_datatype_t *ty)
    {
        return jl_datatype_layout(ty)->first_ptr;
    }

    uint32_t jlrs_field_offset(jl_datatype_t *st, int i)
    {
        return jl_field_offset(st, i);
    }

    uint32_t jlrs_field_size(jl_datatype_t *st, int i)
    {
        return jl_field_size(st, i);
    }

    void jlrs_set_nthreads(int16_t nthreads)
    {
        jl_options.nthreads = nthreads;
    }

#if JULIA_VERSION_MINOR >= 9
    void jlrs_set_nthreadpools(int8_t nthreadpools)
    {
        jl_options.nthreadpools = nthreadpools;
    }
#endif

#if JULIA_VERSION_MINOR >= 9
    void jlrs_set_nthreads_per_pool(const int16_t *nthreads_per_pool)
    {
        jl_options.nthreads_per_pool = nthreads_per_pool;
    }
#endif

    jl_datatype_t *jlrs_dimtuple_type(size_t rank)
    {
        // printf("Rank %zu\n", rank);
        jl_value_t **params = (jl_value_t **)alloca(rank * sizeof(void *));
        if (sizeof(void *) == 4)
        {

            for (size_t i = 0; i < rank; ++i)
            {
                params[i] = (jl_value_t *)jl_int32_type;
            }
        }
        else
        {
            for (size_t i = 0; i < rank; ++i)
            {
                params[i] = (jl_value_t *)jl_int64_type;
            }
        }

        return (jl_datatype_t *)jl_apply_tuple_type_v(params, rank);
    }

    jl_value_t *jlrs_tuple_of(jl_value_t **values, size_t n)
    {
        jl_value_t **types = (jl_value_t **)alloca(n);
        for (size_t i = 0; i < n; ++i)
        {
            types[i] = jl_typeof(values[i]);
        }

        // Should be a leaf type
        jl_datatype_t *tupty = (jl_datatype_t *)jl_apply_tuple_type_v(types, n);

        return jl_new_structv(tupty, values, n);
    }

    uintptr_t jlrs_symbol_hash(jl_sym_t *sym)
    {
        return sym->hash;
    }

    jl_sym_t *jlrs_tvar_name(jl_tvar_t *tvar)
    {
        return tvar->name;
    }

    jl_value_t *jlrs_tvar_lb(jl_tvar_t *tvar)
    {
        return tvar->lb;
    }

    jl_value_t *jlrs_tvar_ub(jl_tvar_t *tvar)
    {
        return tvar->ub;
    }

    jl_value_t *jlrs_unionall_body(jl_unionall_t *ua)
    {
        return ua->body;
    }

    jl_tvar_t *jlrs_unionall_tvar(jl_unionall_t *ua)
    {
        return ua->var;
    }

    jl_sym_t *jlrs_typename_name(jl_typename_t *tn)
    {
        return tn->name;
    }

    jl_module_t *jlrs_typename_module(jl_typename_t *tn)
    {
        return tn->module;
    }

    jl_value_t *jlrs_typename_wrapper(jl_typename_t *tn)
    {
        return tn->wrapper;
    }

#if JULIA_VERSION_MINOR >= 7
    const uint32_t *jlrs_typename_atomicfields(jl_typename_t *tn)
    {
        return tn->atomicfields;
    }

    uint8_t jlrs_typename_abstract(jl_typename_t *tn)
    {
        return tn->abstract;
    }

    uint8_t jlrs_typename_mutable(jl_typename_t *tn)
    {
        return tn->mutabl;
    }

    uint8_t jlrs_typename_mayinlinealloc(jl_typename_t *tn)
    {
        return tn->mayinlinealloc;
    }
#endif

    jl_svec_t *jlrs_typename_names(jl_typename_t *tn)
    {
        return tn->names;
    }

#if JULIA_VERSION_MINOR >= 8
    const uint32_t *jlrs_typename_constfields(jl_typename_t *tn)
    {
        return tn->constfields;
    }
#endif

    jl_value_t *jlrs_union_a(jl_uniontype_t *u)
    {
        return u->a;
    }

    jl_value_t *jlrs_union_b(jl_uniontype_t *u)
    {
        return u->b;
    }

    jl_datatype_t *jlrs_datatype_super(jl_datatype_t *ty)
    {
        return ty->super;
    }

    jl_svec_t *jlrs_datatype_parameters(jl_datatype_t *ty)
    {
        return ty->parameters;
    }

    jl_value_t *jlrs_datatype_instance(jl_datatype_t *ty)
    {
        return ty->instance;
    }

    uint8_t jlrs_datatype_zeroinit(jl_datatype_t *ty)
    {
        return ty->zeroinit;
    }

    uint8_t jlrs_datatype_isconcretetype(jl_datatype_t *ty)
    {
        return ty->isconcretetype;
    }

    int jlrs_datatype_has_layout(jl_datatype_t *t)
    {
        return t->layout != NULL;
    }

    uint8_t jlrs_datatype_isinlinealloc(jl_datatype_t *ty)
    {
#if JULIA_VERSION_MINOR == 6
        return ty->isinlinealloc;
#else
    if (ty->layout && jl_datatype_layout(ty))
    {
        return ty->name->mayinlinealloc;
    }
    else
    {
        return 0;
    }

#endif
    }

    uint8_t jlrs_datatype_abstract(jl_datatype_t *ty)
    {
#if JULIA_VERSION_MINOR == 6
        return ty->abstract;
#else
    return ty->name->abstract;
#endif
    }

    uint8_t jlrs_datatype_mutable(jl_datatype_t *ty)
    {
#if JULIA_VERSION_MINOR == 6
        return ty->mutabl;
#else
    return ty->name->mutabl;
#endif
    }

    jl_sym_t *jlrs_module_name(jl_module_t *m)
    {
        return m->name;
    }

    jl_module_t *jlrs_module_parent(jl_module_t *m)
    {
        return m->parent;
    }

    jl_sym_t *jlrs_expr_head(jl_expr_t *expr)
    {
        return expr->head;
    }

    jl_value_t *jlrs_arrayref(jl_array_t *a, size_t i)
    {
#if JULIA_VERSION_MINOR >= 11
        return jl_genericmemoryref(a->ref.mem, i);
#else
    return jl_arrayref(a, i);
#endif
    }

    void jlrs_arrayset(jl_array_t *a, jl_value_t *rhs, size_t i)
    {
#if JULIA_VERSION_MINOR >= 11
        static _Atomic(jl_value_t *) s = NULL;
        jl_value_t *s2 = jl_atomic_load_relaxed(&s);
        if (s2 == NULL)
        {
            s2 = (jl_value_t *)jl_symbol("atomic");
            jl_atomic_store_relaxed(&s, s2);
        }

        jl_genericmemoryref_t m = jlrs_memoryrefindex(a->ref, i);
        int isatomic = jl_tparam0(jl_typetagof(m.mem)) == s2;
        jlrs_memoryrefset(m, rhs, isatomic);
#else
    jl_arrayset(a, rhs, i);
#endif
    }

    jl_value_t *jlrs_array_data_owner(jl_array_t *a)
    {
#if JULIA_VERSION_MINOR >= 11
        return jl_array_owner(a);
#else
    return jl_array_data_owner(a);
#endif
    }

    char *jlrs_array_typetagdata(jl_array_t *a)
    {
#if JULIA_VERSION_MINOR >= 11
        return jlrs_genericmemory_typetagdata(a->ref.mem);
#else
    return jl_array_typetagdata(a);
#endif
    }

    int jlrs_array_is_pointer_array(jl_array_t *a)
    {
#if JULIA_VERSION_MINOR >= 11
        return ((jl_datatype_t *)jl_typetagof((a)->ref.mem))->layout->flags.arrayelem_isboxed;
#else
    return a->flags.ptrarray != 0;
#endif
    }

    int jlrs_array_is_union_array(jl_array_t *a)
    {
#if JULIA_VERSION_MINOR >= 11
        return ((jl_datatype_t *)jl_typetagof((a)->ref.mem))->layout->flags.arrayelem_isunion;
#else
    return jl_array_isbitsunion(a);
#endif
    }

    int jlrs_array_has_pointers(jl_array_t *a)
    {
#if JULIA_VERSION_MINOR >= 11
        if (jlrs_array_is_pointer_array(a))
        {
            return 0;
        }

        jl_datatype_t *eltype = (jl_datatype_t *)jl_tparam0(jl_typeof(a));
        return jlrs_datatype_first_ptr(eltype) != -1;
#else
    return a->flags.hasptr != 0;
#endif
    }

    int jlrs_array_how(jl_array_t *a)
    {
#if JULIA_VERSION_MINOR >= 11
        return jl_genericmemory_how(a->ref.mem);
#else
    return (int)a->flags.how;
#endif
    }

    void jlrs_set_global(jl_module_t *m JL_ROOTING_ARGUMENT, jl_sym_t *var, jl_value_t *val JL_ROOTED_ARGUMENT)
    {
#if JULIA_VERSION_MINOR >= 11
        jl_binding_t *bp = jl_get_binding_wr(m, var, 1);
        jl_checked_assignment(bp, m, var, val);
#else
    jl_set_global(m, var, val);
#endif
    }
#ifdef __cplusplus
}
#endif