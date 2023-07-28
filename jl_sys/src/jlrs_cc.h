#include <uv.h>

#ifdef _MSC_VER
#include <windows.h>

#ifdef JULIA_1_6
template <typename T>
static inline T jl_atomic_load_relaxed(volatile T *obj)
{
    T val = *obj;
    _ReadWriteBarrier();
    return val;
}
#else
template <typename T>
static inline T jl_atomic_load_relaxed(volatile T *obj)
{
    return jl_atomic_load_acquire(obj);
}
#endif
#endif

#include <julia.h>
#include <julia_gcext.h>

#ifdef __cplusplus
extern "C"
{
#endif

    // A few types are replaced to get rid of any platform-specific fields, we don't care about them.

#ifndef JULIA_1_6
    /**
     * <div rustbindgen replaces="_jl_tls_states_t"></div>
     */
    struct jlrs_tls_states_t;
#else
/**
 * <div rustbindgen replaces="_jl_tls_states_t"></div>
 */
struct jlrs_tls_states_t
{
    void *pgcstack;
    size_t world_age;
};
#endif

    /**
     * <div rustbindgen replaces="_jl_handler_t"></div>
     */
    struct jlrs_handler_t;

#ifndef JULIA_1_6
    /**
     * <div rustbindgen replaces="jl_mutex_t"></div>
     */
    struct jlrs_mutex_t
    {
        // This field is atomic! Special handling in fix_bindings.rs is required because this struct
        // is not defined in julia.h
        jl_task_t *owner;
        uint32_t count;
    };
#else
/**
 * <div rustbindgen replaces="jl_mutex_t"></div>
 */
struct jlrs_mutex_t
{
    // Use unsigned long to avoid generating bindings for DWORD on Windows.
    unsigned long owner;
    uint32_t count;
};
#endif

#ifdef JULIA_1_11

    /**
     * <div rustbindgen replaces="_jl_task_t"></div>
     */
    struct jlrs_task_t
    {
        JL_DATA_TYPE
        jl_value_t *next;  // invasive linked list for scheduler
        jl_value_t *queue; // invasive linked list for scheduler
        jl_value_t *tls;
        jl_value_t *donenotify;
        jl_value_t *result;
        jl_value_t *logstate;
        jl_function_t *start;
        // 4 byte padding on 32-bit systems
        // uint32_t padding0;
        uint64_t rngState[JL_RNG_SIZE];
        _Atomic(uint8_t) _state;
        uint8_t sticky;                // record whether this Task can be migrated to a new thread
        _Atomic(uint8_t) _isexception; // set if `result` is an exception to throw or that we exited with
        // 1 byte padding
        // uint8_t padding1;
        // multiqueue priority
        uint16_t priority;

        // hidden state:

#ifdef USE_TRACY
        const char *name;
#endif
        // id of owning thread - does not need to be defined until the task runs
        _Atomic(int16_t) tid;
        // threadpool id
        int8_t threadpoolid;
        // Reentrancy bits
        // Bit 0: 1 if we are currently running inference/codegen
        // Bit 1-2: 0-3 counter of how many times we've reentered inference
        // Bit 3: 1 if we are writing the image and inference is illegal
        uint8_t reentrant_timing;
        // 2 bytes of padding on 32-bit, 6 bytes on 64-bit
        // uint16_t padding2_32;
        // uint48_t padding2_64;
        // saved gc stack top for context switches
        jl_gcframe_t *gcstack;
        size_t world_age;
        // quick lookup for current ptls
        jl_ptls_t ptls; // == jl_all_tls_states[tid]
    };
#endif

#ifdef JULIA_1_10

    /**
     * <div rustbindgen replaces="_jl_task_t"></div>
     */
    struct jlrs_task_t
    {
        JL_DATA_TYPE
        jl_value_t *next;  // invasive linked list for scheduler
        jl_value_t *queue; // invasive linked list for scheduler
        jl_value_t *tls;
        jl_value_t *donenotify;
        jl_value_t *result;
        jl_value_t *logstate;
        jl_function_t *start;
        // 4 byte padding on 32-bit systems
        // uint32_t padding0;
        uint64_t rngState[JL_RNG_SIZE];
        _Atomic(uint8_t) _state;
        uint8_t sticky;                // record whether this Task can be migrated to a new thread
        _Atomic(uint8_t) _isexception; // set if `result` is an exception to throw or that we exited with
        // 1 byte padding
        // uint8_t padding1;
        // multiqueue priority
        uint16_t priority;

        // hidden state:
        // id of owning thread - does not need to be defined until the task runs
        _Atomic(int16_t) tid;
        // threadpool id
        int8_t threadpoolid;
        // Reentrancy bits
        // Bit 0: 1 if we are currently running inference/codegen
        // Bit 1-2: 0-3 counter of how many times we've reentered inference
        // Bit 3: 1 if we are writing the image and inference is illegal
        uint8_t reentrant_timing;
        // 2 bytes of padding on 32-bit, 6 bytes on 64-bit
        // uint16_t padding2_32;
        // uint48_t padding2_64;
        // saved gc stack top for context switches
        jl_gcframe_t *gcstack;
        size_t world_age;
        // quick lookup for current ptls
        jl_ptls_t ptls; // == jl_all_tls_states[tid]
    };
#endif

#ifdef JULIA_1_9
    /**
     * <div rustbindgen replaces="_jl_task_t"></div>
     */
    struct jlrs_task_t
    {
        JL_DATA_TYPE
        jl_value_t *next;  // invasive linked list for scheduler
        jl_value_t *queue; // invasive linked list for scheduler
        jl_value_t *tls;
        jl_value_t *donenotify;
        jl_value_t *result;
        jl_value_t *logstate;
        jl_function_t *start;
        uint64_t rngState[4];
        _Atomic(uint8_t) _state;
        uint8_t sticky;                // record whether this Task can be migrated to a new thread
        _Atomic(uint8_t) _isexception; // set if `result` is an exception to throw or that we exited with
        // multiqueue priority
        uint16_t priority;

        // hidden state:
        // id of owning thread - does not need to be defined until the task runs
        _Atomic(int16_t) tid;
        // threadpool id
        int8_t threadpoolid;
        // saved gc stack top for context switches
        jl_gcframe_t *gcstack;
        size_t world_age;
        // quick lookup for current ptls
        jl_ptls_t ptls; // == jl_all_tls_states[tid]
    };
#endif

#ifdef JULIA_1_8
    /**
     * <div rustbindgen replaces="_jl_task_t"></div>
     */
    struct jlrs_task_t
    {
        JL_DATA_TYPE
        jl_value_t *next;  // invasive linked list for scheduler
        jl_value_t *queue; // invasive linked list for scheduler
        jl_value_t *tls;
        jl_value_t *donenotify;
        jl_value_t *result;
        jl_value_t *logstate;
        jl_function_t *start;
        uint64_t rngState[4];
        _Atomic(uint8_t) _state;
        uint8_t sticky;                // record whether this Task can be migrated to a new thread
        _Atomic(uint8_t) _isexception; // set if `result` is an exception to throw or that we exited with

        // hidden state:
        // id of owning thread - does not need to be defined until the task runs
        _Atomic(int16_t) tid;
        // multiqueue priority
        int16_t prio;
        // saved gc stack top for context switches
        jl_gcframe_t *gcstack;
        size_t world_age;
        // quick lookup for current ptls
        jl_ptls_t ptls; // == jl_all_tls_states[tid]
    };
#endif

#ifdef JULIA_1_7
    /**
     * <div rustbindgen replaces="_jl_task_t"></div>
     */
    struct jlrs_task_t
    {
        JL_DATA_TYPE
        jl_value_t *next;  // invasive linked list for scheduler
        jl_value_t *queue; // invasive linked list for scheduler
        jl_value_t *tls;
        jl_value_t *donenotify;
        jl_value_t *result;
        jl_value_t *logstate;
        jl_function_t *start;
        uint64_t rngState0; // really rngState[4], but more convenient to split
        uint64_t rngState1;
        uint64_t rngState2;
        uint64_t rngState3;
        _Atomic(uint8_t) _state;
        uint8_t sticky;                // record whether this Task can be migrated to a new thread
        _Atomic(uint8_t) _isexception; // set if `result` is an exception to throw or that we exited with

        // hidden state:
        // id of owning thread - does not need to be defined until the task runs
        _Atomic(int16_t) tid;
        // multiqueue priority
        int16_t prio;
        // saved gc stack top for context switches
        jl_gcframe_t *gcstack;
        size_t world_age;
        // quick lookup for current ptls
        jl_ptls_t ptls; // == jl_all_tls_states[tid]
    };
#endif

#ifdef JULIA_1_6
    /**
     * <div rustbindgen replaces="_jl_task_t"></div>
     */
    struct jlrs_task_t
    {
        JL_DATA_TYPE
        jl_value_t *next;  // invasive linked list for scheduler
        jl_value_t *queue; // invasive linked list for scheduler
        jl_value_t *tls;
        jl_value_t *donenotify;
        jl_value_t *result;
        jl_value_t *logstate;
        jl_function_t *start;
        uint8_t _state;
        uint8_t sticky;       // record whether this Task can be migrated to a new thread
        uint8_t _isexception; // set if `result` is an exception to throw or that we exited with
    };
#endif

    typedef enum
    {
        JLRS_CATCH_OK = 0,
        JLRS_CATCH_EXCEPTION = 1,
        JLRS_CATCH_PANIC = 2,
    } jlrs_catch_tag_t;

    typedef struct
    {
        jlrs_catch_tag_t tag;
        void *error;
    } jlrs_catch_t;

    typedef jlrs_catch_t (*jlrs_callback_caller_t)(void *, void *);
    jlrs_catch_t jlrs_catch_wrapper(void *callback, jlrs_callback_caller_t caller, void *result);

    uint_t jlrs_array_data_owner_offset(uint16_t n_dims);
    void jlrs_gc_queue_multiroot(jl_value_t *parent, jl_datatype_t *dt, const void *ptr) JL_NOTSAFEPOINT;

    int8_t jlrs_gc_safe_enter(jl_ptls_t ptls);
    int8_t jlrs_gc_unsafe_enter(jl_ptls_t ptls);
    void jlrs_gc_safe_leave(jl_ptls_t ptls, int8_t state);
    void jlrs_gc_unsafe_leave(jl_ptls_t ptls, int8_t state);

    jl_datatype_t *jlrs_dimtuple_type(size_t rank);
    jl_value_t *jlrs_tuple_of(jl_value_t **values, size_t n);

#ifdef JULIA_1_6
    jl_gcframe_t **jlrs_pgcstack(jl_tls_states_t *ptls);
#endif

#ifndef JULIA_1_6
    void jlrs_lock(jl_value_t *v);
    void jlrs_unlock(jl_value_t *v);
#endif

#if !defined(JULIA_1_6) && !defined(JULIA_1_7) && !defined(JULIA_1_8)
    void jl_enter_threaded_region(void);
    void jl_exit_threaded_region(void);
#endif

#if !defined(JULIA_1_6) && !defined(JULIA_1_7) && !defined(JULIA_1_8) && !defined(JULIA_1_9)
    jl_datatype_t *jlrs_typeof(jl_value_t *v);
#endif
#ifdef __cplusplus
}
#endif
