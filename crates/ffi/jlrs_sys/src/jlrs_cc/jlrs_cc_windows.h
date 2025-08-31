#ifndef JLRS_CC_WINDOW_H
#define JLRS_CC_WINDOW_H

#ifdef _MSC_VER
#include <winsock2.h>
#include <windows.h>

template <typename T>
static inline T jl_atomic_load_relaxed(volatile T *obj)
{
    return jl_atomic_load_acquire(obj);
}

#endif // _MSC_VER
#endif // JLRS_CC_WINDOW_H