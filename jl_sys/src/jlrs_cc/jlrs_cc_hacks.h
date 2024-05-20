
// Work-arounds for some issues with unexported functions.

#ifndef JLRS_CC_HACKS_H
#define JLRS_CC_HACKS_H

#ifdef __cplusplus
extern "C"
{
#endif

    // There are several functions that are marked as JL_DLLEXPORT but not present in
    // jl_exported_funcs.inc. These functions are unavailable in libjulia, but can be found in
    // libjulia_internal. So, we acquire a handle to that library and load the missing symbols at
    // runtime.
    //
    // This is obviously a hack, but less so than than manually reimplementing these functions.
    void jlrs_init_missing_functions(void);

#if JULIA_VERSION_MINOR >= 7
    void jlrs_lock_value(jl_value_t *v);
    void jlrs_unlock_value(jl_value_t *v);
#endif // JULIA_VERSION_MINOR >= 7

#if JULIA_VERSION_MINOR >= 11
    jl_genericmemoryref_t jlrs_memoryrefindex(jl_genericmemoryref_t m JL_ROOTING_ARGUMENT, size_t idx);
    void jlrs_memoryrefset(jl_genericmemoryref_t m JL_ROOTING_ARGUMENT, jl_value_t *rhs JL_ROOTED_ARGUMENT JL_MAYBE_UNROOTED, int isatomic);
    char *jlrs_genericmemory_typetagdata(jl_genericmemory_t *m);
#endif

#ifdef __cplusplus
}
#endif
#endif