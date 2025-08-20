#include "jlrs_cc.h"

#ifdef __cplusplus
extern "C"
{
#endif

    typedef void (*jl_lock_value_func_t)(void *);

    static jl_lock_value_func_t jl_lock_value_func;
    static jl_lock_value_func_t jl_unlock_value_func;

#if JULIA_VERSION_MINOR >= 11
    typedef jl_genericmemoryref_t (*jl_memoryrefindex_func_t)(jl_genericmemoryref_t, size_t);
    typedef void (*jl_memoryrefset_func_t)(jl_genericmemoryref_t, jl_value_t *, int);
    typedef char *(*jl_genericmemory_typetagdata_func_t)(jl_genericmemory_t *);

    static jl_memoryrefindex_func_t jl_memoryrefindex_func;
    static jl_memoryrefset_func_t jl_memoryrefset_func;
    static jl_genericmemory_typetagdata_func_t jl_genericmemory_typetagdata_func;
#endif

#if JULIA_VERSION_MINOR >= 12
    typedef jl_binding_partition_t *(*jl_declare_constant_val_func_t)(jl_binding_t *, jl_module_t *, jl_sym_t *, jl_value_t *);
    static jl_declare_constant_val_func_t jl_declare_constant_val_func;
#endif

    void jlrs_init_missing_functions(void)
    {
        void ***libjulia_internal_handle_ref_v = (void ***)jl_eval_string("cglobal(:jl_libjulia_internal_handle)");
        void *libjulia_internal_handle = **libjulia_internal_handle_ref_v;

        int found_jl_lock_value = jl_dlsym(libjulia_internal_handle, "jl_lock_value", (void **)&jl_lock_value_func, 0);
        assert(found_jl_lock_value);

        int found_jl_unlock_value = jl_dlsym(libjulia_internal_handle, "jl_unlock_value", (void **)&jl_unlock_value_func, 0);
        assert(found_jl_unlock_value);

#if JULIA_VERSION_MINOR >= 11
        int found_jl_memoryrefindex = jl_dlsym(libjulia_internal_handle, "jl_memoryrefindex", (void **)&jl_memoryrefindex_func, 0);
        assert(found_jl_memoryrefindex);

        int found_jl_memoryrefset = jl_dlsym(libjulia_internal_handle, "jl_memoryrefset", (void **)&jl_memoryrefset_func, 0);
        assert(found_jl_memoryrefset);

        int found_jl_genericmemory_typetagdata = jl_dlsym(libjulia_internal_handle, "jl_genericmemory_typetagdata", (void **)&jl_genericmemory_typetagdata_func, 0);
        assert(found_jl_genericmemory_typetagdata);
#endif

#if JULIA_VERSION_MINOR >= 12
        int found_jl_declare_constant_val = jl_dlsym(libjulia_internal_handle, "jl_declare_constant_val", (void **)&jl_declare_constant_val_func, 0);
        assert(found_jl_declare_constant_val);
#endif
    }

    void jlrs_lock_value(jl_value_t *v)
    {
        assert(jl_lock_value_func && "jl_lock_value_func not loaded");
        jl_lock_value_func(v);
    }

    void jlrs_unlock_value(jl_value_t *v)
    {
        assert(jl_unlock_value_func && "jl_unlock_value_func not loaded");
        jl_unlock_value_func(v);
    }

#if JULIA_VERSION_MINOR >= 11
    jl_genericmemoryref_t jlrs_memoryrefindex(jl_genericmemoryref_t m, size_t idx)
    {
        assert(jl_memoryrefindex_func && "jl_memoryrefindex_func not loaded");
        return jl_memoryrefindex_func(m, idx);
    }

    void jlrs_memoryrefset(jl_genericmemoryref_t m, jl_value_t *rhs, int isatomic)
    {
        assert(jl_memoryrefset_func && "jl_memoryrefset_func not loaded");
        jl_memoryrefset_func(m, rhs, isatomic);
    }

    char *jlrs_genericmemory_typetagdata(jl_genericmemory_t *m)
    {
        assert(jl_genericmemory_typetagdata_func && "jl_genericmemory_typetagdata_func not loaded");
        return jl_genericmemory_typetagdata_func(m);
    }
#endif

#if JULIA_VERSION_MINOR >= 12
    jl_binding_partition_t *jlrs_declare_constant_val(jl_binding_t *b, jl_module_t *m, jl_sym_t *var, jl_value_t *val)
    {
        assert(jl_declare_constant_val_func && "jl_declare_constant_val_func not loaded");
        return jl_declare_constant_val_func(b, m, var, val);
    }
#endif

#ifdef __cplusplus
}
#endif