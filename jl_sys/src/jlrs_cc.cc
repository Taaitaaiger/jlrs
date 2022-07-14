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
            res = {.tag = JLRS_CATCH_EXCECPTION, .error = jl_current_exception()};
        }
        jl_exception_clear();

        return res;
    }
#endif

    uint_t jlrs_array_data_owner_offset(uint16_t n_dims)
    {
        return jl_array_data_owner_offset(n_dims);
    }

    void jlrs_lock(jl_value_t *v)
    {
        JL_LOCK_NOGC((jl_mutex_t *)v);
    }

    void jlrs_unlock(jl_value_t *v)
    {
        JL_UNLOCK_NOGC((jl_mutex_t *)v);
    }
#ifdef __cplusplus
}
#endif
