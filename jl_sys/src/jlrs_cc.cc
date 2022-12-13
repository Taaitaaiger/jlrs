#include "jlrs_cc.h"

#ifdef __cplusplus
extern "C"
{
#endif
#if !defined(JLRS_WINDOWS_LTS)
    jlrs_catch_t jlrs_catch_wrapper(void *callback, jlrs_callback_caller_t caller, void *result, void *frame_slice)
    {
        jlrs_catch_t res;

        JL_TRY
        {
            res = caller(callback, frame_slice, result);
        }
        JL_CATCH
        {
            res = {.tag = JLRS_CATCH_EXCEPTION, .error = jl_current_exception()};
        }

        return res;
    }
#endif

    uint_t jlrs_array_data_owner_offset(uint16_t n_dims)
    {
        return jl_array_data_owner_offset(n_dims);
    }

#if !defined(JULIA_1_6)
    void jlrs_lock(jl_value_t *v)
    {
        jl_task_t *self = jl_current_task;
        jl_mutex_t *lock = (jl_mutex_t *) v;

        jl_task_t *owner = jl_atomic_load_relaxed(&lock->owner);
        if (owner == self) {
            lock->count++;
            return;
        }
        while (1) {
            if (owner == NULL && jl_atomic_cmpswap(&lock->owner, &owner, self)) {
                lock->count = 1;
                return;
            }

            jl_cpu_pause();
            owner = jl_atomic_load_relaxed(&lock->owner);
        }
    }

    void jlrs_unlock(jl_value_t *v)
    {
        jl_mutex_t *lock = (jl_mutex_t *) v;

        if (--lock->count == 0) {
            jl_atomic_store_release(&lock->owner, (jl_task_t*)NULL);
            jl_cpu_wake();
        }
    }
#endif
#ifdef __cplusplus
}
#endif
