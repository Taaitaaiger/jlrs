#include "jlrs_cc_windows.h"
#include <julia.h>

#ifdef __cplusplus
extern "C"
{
#endif
    jl_tls_states_t *jlrs_get_ptls_states(void);
    jl_tls_states_t *jlrs_ptls_from_gcstack(jl_gcframe_t **pgcstack);
    int8_t jlrs_task_gc_state();
    void jlrs_clear_gc_stack(void);

    // pgcstack getter
    jl_gcframe_t **jlrs_ppgcstack(void);
#ifdef __cplusplus
} // extern "C"
#endif
