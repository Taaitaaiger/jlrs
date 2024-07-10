#ifndef JLRS_CC_WINDOW_H
#define JLRS_CC_WINDOW_H

#ifdef _MSC_VER
#include <winsock2.h>
#include <windows.h>

#if JLRS_EXPECTED_MINOR_VERSION == 6
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

#endif // JULIA_VERSION_MINOR == 6
#endif // _MSC_VER
#endif // JLRS_CC_WINDOW_H