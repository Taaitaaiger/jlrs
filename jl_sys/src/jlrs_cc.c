#include "jlrs_cc.h"

#ifdef JLRS_FAST_TLS
#ifdef JULIA_1_6
JULIA_DEFINE_FAST_TLS()
#else
JULIA_DEFINE_FAST_TLS
#endif
#endif

#ifdef __cplusplus
extern "C"
{
#endif
    jlrs_catch_t jlrs_catch_wrapper(void *callback, jlrs_callback_caller_t caller, void *result)
    {
        jlrs_catch_t res = {.tag = JLRS_CATCH_OK, .error = NULL};

#ifndef JLRS_WINDOWS_LTS
        JL_TRY
        {
            res = caller(callback, result);
        }
        JL_CATCH
        {
            res.tag = JLRS_CATCH_EXCEPTION;
            res.error = jl_current_exception();
        }
#else
    res = caller(callback, result);
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

    int8_t jlrs_gc_safe_enter(jl_ptls_t ptls)
    {
        return jl_gc_safe_enter(ptls);
    }

    int8_t jlrs_gc_unsafe_enter(jl_ptls_t ptls)
    {
        return jl_gc_unsafe_enter(ptls);
    }

    void jlrs_gc_safe_leave(jl_ptls_t ptls, int8_t state)
    {
        jl_gc_safe_leave(ptls, state);
    }

    void jlrs_gc_unsafe_leave(jl_ptls_t ptls, int8_t state)
    {
        jl_gc_unsafe_leave(ptls, state);
    }

    jl_datatype_t *jlrs_dimtuple_type(size_t rank)
    {
        jl_value_t **params = (jl_value_t **)alloca(rank);
        if (sizeof(void *) == 4)
        {

            for (size_t i = 0; i < rank; ++i)
            {
                params[i] = (jl_value_t *)jl_int32_type;
            }
        }
        else
        {
            for (size_t i = 0; i < rank; ++i)
            {
                params[i] = (jl_value_t *)jl_int64_type;
            }
        }

        return (jl_datatype_t *)jl_apply_tuple_type_v(params, rank);
    }

    jl_value_t *jlrs_tuple_of(jl_value_t **values, size_t n)
    {
        jl_value_t **types = (jl_value_t **)alloca(n);
        for (size_t i = 0; i < n; ++i)
        {
            types[i] = jl_typeof(values[i]);
        }

        // Should be a leaf type
        jl_datatype_t *tupty = (jl_datatype_t *)jl_apply_tuple_type_v(types, n);

        return jl_new_structv(tupty, values, n);
    }

#ifndef JULIA_1_6
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

#ifdef JULIA_1_6
    jl_gcframe_t **jlrs_pgcstack(jl_tls_states_t *ptls)
    {
        return &(ptls->pgcstack);
    }
#endif

#if !defined(JULIA_1_6) && !defined(JULIA_1_7) && !defined(JULIA_1_8) && !defined(JULIA_1_9)
    jl_datatype_t *jlrs_typeof(jl_value_t *v)
    {
        return (jl_datatype_t *)jl_typeof(v);
    }
#endif

#ifdef __cplusplus
}
#endif
