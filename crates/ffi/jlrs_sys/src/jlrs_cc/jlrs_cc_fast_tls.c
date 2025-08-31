#include "jlrs_cc_fast_tls.h"

#ifdef __cplusplus
extern "C"
{
#endif

#ifdef JLRS_FAST_TLS
    JULIA_DEFINE_FAST_TLS
#endif

    JL_CONST_FUNC jl_tls_states_t *jlrs_get_ptls_states(void)
    {
        jl_gcframe_t **pgcstack = jl_get_pgcstack();
        if (pgcstack == NULL)
        {
            return NULL;
        }

        jl_task_t *task = container_of(pgcstack, jl_task_t, gcstack);
        return task->ptls;
    }

    jl_tls_states_t *jlrs_ptls_from_gcstack(jl_gcframe_t **pgcstack)
    {
        jl_task_t *task = container_of(pgcstack, jl_task_t, gcstack);
        return task->ptls;
    }

    int8_t jlrs_task_gc_state(void)
    {
        jl_gcframe_t **pgcstack = jl_get_pgcstack();
        if (pgcstack == NULL)
        {
            return -1;
        }

        jl_task_t *task = container_of(pgcstack, jl_task_t, gcstack);
        return jl_atomic_load_relaxed(&task->ptls->gc_state);
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