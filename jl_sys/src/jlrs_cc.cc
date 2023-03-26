#include "jlrs_cc.h"

#ifdef __cplusplus
extern "C"
{
#endif
    jlrs_catch_t jlrs_catch_wrapper(void *callback, jlrs_callback_caller_t caller, void *result, void *frame_slice)
    {
        jlrs_catch_t res;

#if !defined(JLRS_WINDOWS_LTS)
        JL_TRY
        {
#endif
            res = caller(callback, frame_slice, result);
#if !defined(JLRS_WINDOWS_LTS)
        }
        JL_CATCH
        {
            res = {.tag = JLRS_CATCH_EXCEPTION, .error = jl_current_exception()};
        }
#endif
        return res;
    }

    uint_t jlrs_array_data_owner_offset(uint16_t n_dims)
    {
        return jl_array_data_owner_offset(n_dims);
    }

    void jlrs_gc_queue_multiroot(jl_value_t *parent, jl_datatype_t *dt, const void *ptr) JL_NOTSAFEPOINT
    {
        // first check if this is really necessary
        // TODO: should we store this info in one of the extra gc bits?
        const jl_datatype_layout_t *ly = dt->layout;
        uint32_t npointers = ly->npointers;
        if (npointers == 0)
            return;
        jl_value_t *ptrf = ((jl_value_t **)ptr)[ly->first_ptr];
        if (ptrf && (jl_astaggedvalue(ptrf)->bits.gc & 1) == 0)
        {
            // this pointer was young, move the barrier back now
            jl_gc_wb_back(parent);
            return;
        }
        const uint8_t *ptrs8 = (const uint8_t *)jl_dt_layout_ptrs(ly);
        const uint16_t *ptrs16 = (const uint16_t *)jl_dt_layout_ptrs(ly);
        const uint32_t *ptrs32 = (const uint32_t *)jl_dt_layout_ptrs(ly);
        for (size_t i = 1; i < npointers; i++)
        {
            uint32_t fld;
            if (ly->fielddesc_type == 0)
            {
                fld = ptrs8[i];
            }
            else if (ly->fielddesc_type == 1)
            {
                fld = ptrs16[i];
            }
            else
            {
                assert(ly->fielddesc_type == 2);
                fld = ptrs32[i];
            }
            jl_value_t *ptrf = ((jl_value_t **)ptr)[fld];
            if (ptrf && (jl_astaggedvalue(ptrf)->bits.gc & 1) == 0)
            {
                // this pointer was young, move the barrier back now
                jl_gc_wb_back(parent);
                return;
            }
        }
    }

#if !defined(JULIA_1_6)
    void jlrs_lock(jl_value_t *v)
    {
        jl_task_t *self = jl_current_task;
        jl_mutex_t *lock = (jl_mutex_t *)v;

        jl_task_t *owner = jl_atomic_load_relaxed(&lock->owner);
        if (owner == self)
        {
            lock->count++;
            return;
        }
        while (1)
        {
            if (owner == NULL && jl_atomic_cmpswap(&lock->owner, &owner, self))
            {
                lock->count = 1;
                return;
            }

            jl_cpu_pause();
            owner = jl_atomic_load_relaxed(&lock->owner);
        }
    }

    void jlrs_unlock(jl_value_t *v)
    {
        jl_mutex_t *lock = (jl_mutex_t *)v;

        if (--lock->count == 0)
        {
            jl_atomic_store_release(&lock->owner, (jl_task_t *)NULL);
            jl_cpu_wake();
        }
    }
#endif
#ifdef __cplusplus
}
#endif
