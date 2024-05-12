
// Work-arounds for some issues with unexported functions.

#ifndef JLRS_CC_HACKS_H
#define JLRS_CC_HACKS_H

#ifdef __cplusplus
extern "C"
{
#endif

    // Enter / exit gc (un)safe region
    int8_t jlrs_gc_safe_enter(jl_ptls_t ptls);
    int8_t jlrs_gc_unsafe_enter(jl_ptls_t ptls);
    void jlrs_gc_safe_leave(jl_ptls_t ptls, int8_t state);
    void jlrs_gc_unsafe_leave(jl_ptls_t ptls, int8_t state);

#if JULIA_VERSION_MINOR >= 7
    // acquire and release lock
    void jlrs_lock(jl_value_t *v);
    void jlrs_unlock(jl_value_t *v);
    void jlrs_lock_nogc(jl_value_t *v);
    void jlrs_unlock_nogc(jl_value_t *v);
#endif // JULIA_VERSION_MINOR >= 7

#if JULIA_VERSION_MINOR >= 11
    int jlrs_memoryref_isassigned(jl_genericmemoryref_t m, int isatomic);
    int jlrs_find_union_component(jl_value_t *haystack, jl_value_t *needle, unsigned *nth) JL_NOTSAFEPOINT;
    jl_genericmemoryref_t jlrs_memoryrefindex(jl_genericmemoryref_t m JL_ROOTING_ARGUMENT, size_t idx);
    char *jlrs_genericmemory_typetagdata(jl_genericmemory_t *m);
    void jlrs_memoryrefset(jl_genericmemoryref_t m JL_ROOTING_ARGUMENT, jl_value_t *rhs JL_ROOTED_ARGUMENT JL_MAYBE_UNROOTED, int isatomic);

    static inline void jlrs_memmove_refs(_Atomic(void *) *dstp, _Atomic(void *) *srcp, size_t n) JL_NOTSAFEPOINT
    {
        size_t i;
        if (dstp < srcp || dstp > srcp + n)
        {
            for (i = 0; i < n; i++)
            {
                jl_atomic_store_release(dstp + i, jl_atomic_load_relaxed(srcp + i));
            }
        }
        else
        {
            for (i = 0; i < n; i++)
            {
                jl_atomic_store_release(dstp + n - i - 1, jl_atomic_load_relaxed(srcp + n - i - 1));
            }
        }
    }

    static inline void jlrs_memassign_safe(int hasptr, char *dst, const jl_value_t *src, size_t nb) JL_NOTSAFEPOINT
    {
        assert(nb == jl_datatype_size(jl_typeof(src)));
        if (hasptr)
        {
            size_t nptr = nb / sizeof(void *);
            jlrs_memmove_refs((_Atomic(void *) *)dst, (_Atomic(void *) *)src, nptr);
            nb -= nptr * sizeof(void *);
            if (__likely(nb == 0))
                return;
            src = (jl_value_t *)((char *)src + nptr * sizeof(void *));
            dst = dst + nptr * sizeof(void *);
        }
        else if (nb >= 16)
        {
            memcpy(dst, jl_assume_aligned(src, 16), nb);
            return;
        }
        memcpy(dst, jl_assume_aligned(src, sizeof(void *)), nb);
    }
#endif

#ifdef __cplusplus
}
#endif
#endif