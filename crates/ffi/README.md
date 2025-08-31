# FFI crates

Two FFI crates are used by jlrs: jl-sys and jlrs-sys. The first provides bindings to libjulia, the second provides low-level extensions.

The extensions provided by jlrs-sys are mostly implemented in C, and include accessors for specific struct fields, wrappers for inline functions and functional macros, and functions that serve as a trampoline to make use of features like try-catch blocks and non-statically-sized GC frames.

The C code of jlrs-sys is compiled as a static library to support fast TLS and enable cross-language LTO.
