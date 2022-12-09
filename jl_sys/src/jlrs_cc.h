#include <uv.h>

#ifdef _MSC_VER
#include <windows.h>

template <typename T>
static inline T jl_atomic_load_relaxed(volatile T *obj)
{
    return jl_atomic_load_acquire(obj);
}
#endif

#include <julia.h>
#include <julia_gcext.h>

#if defined(JLRS_WINDOWS) && defined(JULIA_1_6)
#define JLRS_WINDOWS_LTS
#endif

#ifdef __cplusplus
extern "C"
{
#endif
#if !defined(JLRS_WINDOWS_LTS)
    typedef enum
    {
        JLRS_CATCH_OK = 0,
        JLRS_CATCH_ERR = 1,
        JLRS_CATCH_EXCEPTION = 2,
        JLRS_CATCH_PANIC = 3,
    } jlrs_catch_tag_t;

    typedef struct
    {
        jlrs_catch_tag_t tag;
        void *error;
    } jlrs_catch_t;

    typedef jlrs_catch_t (*jlrs_callback_caller_t)(void *, void *, void *);
    jlrs_catch_t jlrs_catch_wrapper(void *callback, jlrs_callback_caller_t caller, void *result, void *frame_slice);
#endif

    uint_t jlrs_array_data_owner_offset(uint16_t n_dims);

#if !defined(JLRS_WINDOWS_LTS)
    void jlrs_lock(jl_value_t *v);
    void jlrs_unlock(jl_value_t *v);
#endif

#if !defined(JULIA_1_6) && !defined(JULIA_1_7) && !defined(JULIA_1_8)
    void jl_enter_threaded_region(void);
    void jl_exit_threaded_region(void);
#endif
#ifdef __cplusplus
}
#endif
