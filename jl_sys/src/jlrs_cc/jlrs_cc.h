// This library exists for several reasons. It exports several functions which only exists as
// macros or static inline functions in the Julia C API. It reimplements several functions which
// are not fully exported by that API. And, it exports several types and functions that can't be
// written in Rust or would require exposing many implementation details to do so.

#ifndef JLRS_CC_H
#define JLRS_CC_H

#ifndef JULIA_VERSION_MINOR
#include <julia_version.h>
#endif

#include "jlrs_cc_windows.h"
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wunused-parameter"
#include <julia.h>
#pragma GCC diagnostic pop
#include <julia_gcext.h>

#include "jlrs_cc_hacks.h"
#include "jlrs_cc_ext.h"
#include "jlrs_cc_reexport.h"

#endif