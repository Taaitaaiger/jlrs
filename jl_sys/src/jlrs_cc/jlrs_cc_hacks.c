#include "jlrs_cc.h"

#ifdef __cplusplus
extern "C"
{
#endif
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

#if JULIA_VERSION_MINOR >= 7
    void jlrs_small_arraylist_grow(small_arraylist_t *a, uint32_t n)
    {
        size_t len = a->len;
        size_t newlen = len + n;
        if (newlen > a->max)
        {
            if (a->items == &a->_space[0])
            {
                void **p = (void **)LLT_ALLOC((a->len + n) * sizeof(void *));
                if (p == NULL)
                    return;
                memcpy(p, a->items, len * sizeof(void *));
                a->items = p;
                a->max = newlen;
            }
            else
            {
                size_t nm = a->max * 2;
                if (nm == 0)
                    nm = 1;
                while (newlen > nm)
                    nm *= 2;
                void **p = (void **)LLT_REALLOC(a->items, nm * sizeof(void *));
                if (p == NULL)
                    return;
                a->items = p;
                a->max = nm;
            }
        }
        a->len = newlen;
    }

    static void jlrs_lock_frame_push(jl_task_t *self, jl_mutex_t *lock)
    {
        jl_ptls_t ptls = self->ptls;
        small_arraylist_t *locks = &ptls->locks;
        uint32_t len = locks->len;
        if (__unlikely(len >= locks->max))
        {
            jlrs_small_arraylist_grow(locks, 1);
        }
        else
        {
            locks->len = len + 1;
        }
        locks->items[len] = (void *)lock;
    }

    static void jlrs_lock_frame_pop(jl_task_t *self)
    {
        jl_ptls_t ptls = self->ptls;
        assert(ptls->locks.len > 0);
        ptls->locks.len--;
    }

    static void jlrs_mutex_wait(jl_task_t *self, jl_mutex_t *lock, int safepoint)
    {
        jl_task_t *owner = jl_atomic_load_relaxed(&lock->owner);
        if (owner == self)
        {
            lock->count++;
            return;
        }

        if (owner == NULL && jl_atomic_cmpswap(&lock->owner, &owner, self))
        {
            lock->count = 1;
            return;
        }

        while (1)
        {
            if (owner == NULL && jl_atomic_cmpswap(&lock->owner, &owner, self))
            {
                lock->count = 1;
                return;
            }
            if (safepoint)
            {
                jl_gc_safepoint_(self->ptls);
            }

#if JULIA_VERSION_MINOR <= 9
            jl_cpu_pause();
#else
            jl_cpu_suspend();
#endif
            owner = jl_atomic_load_relaxed(&lock->owner);
        }
    }

    static void jlrs_mutex_unlock_nogc(jl_mutex_t *lock)
    {
        assert(jl_atomic_load_relaxed(&lock->owner) == jl_current_task &&
               "Unlocking a lock in a different thread.");

        if (--lock->count == 0)
        {
            jl_atomic_store_release(&lock->owner, (jl_task_t *)NULL);
            jl_cpu_wake();
        }
    }

    void jlrs_lock(jl_value_t *v)
    {
        jl_task_t *self = jl_current_task;
        jl_mutex_t *lock = (jl_mutex_t *)v;

#if JULIA_VERSION_MINOR <= 8
        JL_SIGATOMIC_BEGIN();
#else
        JL_SIGATOMIC_BEGIN_self();
#endif

        jlrs_mutex_wait(self, lock, 1);
        jlrs_lock_frame_push(self, lock);
    }

    void jlrs_lock_nogc(jl_value_t *v)
    {
        jl_task_t *self = jl_current_task;
        jl_mutex_t *lock = (jl_mutex_t *)v;
        jlrs_mutex_wait(self, lock, 0);
    }

    void jlrs_unlock(jl_value_t *v)
    {
        jl_task_t *self = jl_current_task;
        jl_mutex_t *lock = (jl_mutex_t *)v;

        jlrs_mutex_unlock_nogc(lock);
        jlrs_lock_frame_pop(self);

#if JULIA_VERSION_MINOR <= 8
        JL_SIGATOMIC_END();
#else
        JL_SIGATOMIC_END_self();
#endif

        // FIXME
        // if (jl_atomic_load_relaxed(&jl_gc_have_pending_finalizers))
        // {
        //     jl_gc_run_pending_finalizers(self); // may GC
        // }
    }

    void jlrs_unlock_nogc(jl_value_t *v)
    {
        jl_mutex_t *lock = (jl_mutex_t *)v;
        jlrs_mutex_unlock_nogc(lock);
    }
#endif

#if JULIA_VERSION_MINOR >= 11

#define JL_SMALL_BYTE_ALIGNMENT 16

    int jlrs_memoryref_isassigned(jl_genericmemoryref_t m, int isatomic)
    {
        const jl_datatype_layout_t *layout = ((jl_datatype_t *)jl_typetagof(m.mem))->layout;
        _Atomic(jl_value_t *) *elem = (_Atomic(jl_value_t *) *)m.ptr_or_offset;
        if (layout->flags.arrayelem_isboxed)
        {
        }
        else if (layout->first_ptr >= 0)
        {
            int needlock = isatomic && layout->size > MAX_ATOMIC_SIZE;
            if (needlock)
                elem = elem + LLT_ALIGN(sizeof(jl_mutex_t), JL_SMALL_BYTE_ALIGNMENT) / sizeof(jl_value_t *);
            elem = &elem[layout->first_ptr];
        }
        else
        {
            return 1;
        }
        return (isatomic ? jl_atomic_load(elem) : jl_atomic_load_relaxed(elem)) != NULL;
    }

    int jlrs_find_union_component(jl_value_t *haystack, jl_value_t *needle, unsigned *nth) JL_NOTSAFEPOINT
    {
        while (jl_is_uniontype(haystack))
        {
            jl_uniontype_t *u = (jl_uniontype_t *)haystack;
            if (jlrs_find_union_component(u->a, needle, nth))
                return 1;
            haystack = u->b;
        }
        if (needle == haystack)
            return 1;
        (*nth)++;
        return 0;
    }

    jl_genericmemoryref_t jlrs_memoryrefindex(jl_genericmemoryref_t m JL_ROOTING_ARGUMENT, size_t idx)
    {
        const jl_datatype_layout_t *layout = ((jl_datatype_t *)jl_typetagof(m.mem))->layout;
        if ((layout->flags.arrayelem_isboxed || !layout->flags.arrayelem_isunion) && layout->size != 0)
        {
            m.ptr_or_offset = (void *)((char *)m.ptr_or_offset + idx * layout->size);
            assert((size_t)((char *)m.ptr_or_offset - (char *)m.mem->ptr) < (size_t)(layout->size * m.mem->length));
        }
        else
        {
            m.ptr_or_offset = (void *)((size_t)m.ptr_or_offset + idx);
            assert((size_t)m.ptr_or_offset < m.mem->length);
        }
        return m;
    }

    void jlrs_memoryrefset(jl_genericmemoryref_t m JL_ROOTING_ARGUMENT, jl_value_t *rhs JL_ROOTED_ARGUMENT JL_MAYBE_UNROOTED, int isatomic)
    {
        // Caller must guarantee this, jl_atomic_sym is not exported by the C API.
        // assert(isatomic == (jl_tparam0(jl_typetagof(m.mem)) == (jl_value_t*)jl_atomic_sym));
        jl_value_t *eltype = jl_tparam1(jl_typetagof(m.mem));
        if (eltype != (jl_value_t *)jl_any_type && !jl_typeis(rhs, eltype))
        {
            JL_GC_PUSH1(&rhs);
            if (!jl_isa(rhs, eltype))
                jl_type_error("memoryrefset!", eltype, rhs);
            JL_GC_POP();
        }
        const jl_datatype_layout_t *layout = ((jl_datatype_t *)jl_typetagof(m.mem))->layout;
        if (layout->flags.arrayelem_isboxed)
        {
            assert((size_t)((char *)m.ptr_or_offset - (char *)m.mem->ptr) < (size_t)(sizeof(jl_value_t *) * m.mem->length));
            if (isatomic)
                jl_atomic_store((_Atomic(jl_value_t *) *)m.ptr_or_offset, rhs);
            else
                jl_atomic_store_release((_Atomic(jl_value_t *) *)m.ptr_or_offset, rhs);
            jl_gc_wb(jl_genericmemory_owner(m.mem), rhs);
            return;
        }
        int hasptr;
        char *data = (char *)m.ptr_or_offset;
        if (layout->flags.arrayelem_isunion)
        {
            assert(!isatomic);
            assert(jl_is_uniontype(eltype));
            size_t i = (size_t)data;
            assert(i < m.mem->length);
            // uint8_t *psel = (uint8_t*)jl_genericmemory_typetagdata(m.mem) + i;
            uint8_t *psel = (uint8_t *)jlrs_genericmemory_typetagdata(m.mem) + i;
            unsigned nth = 0;
            // if (!jl_find_union_component(eltype, jl_typeof(rhs), &nth))
            if (!jlrs_find_union_component(eltype, jl_typeof(rhs), &nth))
                assert(0 && "invalid genericmemoryset to isbits union");
            *psel = nth;
            hasptr = 0;
            data = (char *)m.mem->ptr + i * layout->size;
        }
        else
        {
            hasptr = layout->first_ptr >= 0;
        }
        if (layout->size != 0)
        {
            assert((size_t)(data - (char *)m.mem->ptr) < (size_t)(layout->size * m.mem->length));
            int needlock = isatomic && layout->size > MAX_ATOMIC_SIZE;
            size_t fsz = jl_datatype_size((jl_datatype_t *)jl_typeof(rhs)); // need to shrink-wrap the final copy
            if (isatomic && !needlock)
            {
                jl_atomic_store_bits(data, rhs, fsz);
            }
            else if (needlock)
            {
                // jl_lock_field((jl_mutex_t*)data);
                // memassign_safe(hasptr, data + LLT_ALIGN(sizeof(jl_mutex_t), JL_SMALL_BYTE_ALIGNMENT), rhs, fsz);
                // jl_unlock_field((jl_mutex_t*)data);
                jlrs_lock_nogc((jl_value_t *)data);
                jlrs_memassign_safe(hasptr, data + LLT_ALIGN(sizeof(jl_mutex_t), JL_SMALL_BYTE_ALIGNMENT), rhs, fsz);
                jlrs_unlock_nogc((jl_value_t *)data);
            }
            else
            {
                // memassign_safe(hasptr, data, rhs, fsz);
                jlrs_memassign_safe(hasptr, data, rhs, fsz);
            }
            if (hasptr)
                jl_gc_multi_wb(jl_genericmemory_owner(m.mem), rhs); // rhs is immutable
        }
    }

    char *jlrs_genericmemory_typetagdata(jl_genericmemory_t *m)
    {
        const jl_datatype_layout_t *layout = ((jl_datatype_t *)jl_typetagof(m))->layout;
        assert(layout->flags.arrayelem_isunion);
        return (char *)m->ptr + m->length * layout->size;
    }
#endif

#ifdef __cplusplus
}
#endif