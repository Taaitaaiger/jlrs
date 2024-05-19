#include "jlrs_cc_fast_tls.h"

#ifdef __cplusplus
extern "C"
{
#endif

#ifdef JLRS_FAST_TLS
#if JULIA_VERSION_MINOR == 6
    JULIA_DEFINE_FAST_TLS()
#else
    JULIA_DEFINE_FAST_TLS
#endif
#endif

    jl_tls_states_t *jlrs_get_ptls_states(void)
    {
#if JULIA_VERSION_MINOR == 6
        return jl_get_ptls_states();
#else
    jl_gcframe_t **pgcstack = jl_get_pgcstack();
    if (pgcstack == NULL)
    {
        return NULL;
    }

    jl_task_t *task = container_of(pgcstack, jl_task_t, gcstack);
    return task->ptls;
#endif
    }

    jl_gcframe_t **jlrs_ppgcstack(void)
    {
#if JULIA_VERSION_MINOR == 6
        jl_tls_states_t *ptls = jl_get_ptls_states();
        return &(ptls->pgcstack);
#else
    return jl_get_pgcstack();
#endif
    }

    jl_tls_states_t *jlrs_ptls_from_gcstack(jl_gcframe_t **pgcstack)
    {
#if JULIA_VERSION_MINOR == 6
        (void)pgcstack;
        return jl_get_ptls_states();
#else
    jl_task_t *task = container_of(pgcstack, jl_task_t, gcstack);
    return task->ptls;
#endif
    }

    int8_t jlrs_task_gc_state(void)
    {
#if JULIA_VERSION_MINOR == 6
        jl_gcframe_t **pgcstack = &jl_pgcstack;
        if (pgcstack == NULL)
        {
            return -1;
        }

        return jl_get_ptls_states()->gc_state;
#else
    jl_gcframe_t **pgcstack = jl_get_pgcstack();
    if (pgcstack == NULL)
    {
        return -1;
    }

    jl_task_t *task = container_of(pgcstack, jl_task_t, gcstack);
    return jl_atomic_load_relaxed(&task->ptls->gc_state);
#endif
    }

    void jlrs_clear_gc_stack(void)
    {
        while (jl_pgcstack != NULL)
        {
            JL_GC_POP();
        }
    }

#ifdef __cplusplus
}
#endif