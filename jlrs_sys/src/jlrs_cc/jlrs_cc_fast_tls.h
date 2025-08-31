#include "jlrs_cc_windows.h"
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wunused-parameter"
#include <julia.h>
#pragma GCC diagnostic pop

#ifdef __cplusplus
extern "C"
{
#endif
    JL_CONST_FUNC jl_tls_states_t *jlrs_get_ptls_states(void);
    jl_tls_states_t *jlrs_ptls_from_gcstack(jl_gcframe_t **pgcstack);
    int8_t jlrs_task_gc_state();
    void jlrs_clear_gc_stack(void);
#ifdef __cplusplus
} // extern "C"
#endif
