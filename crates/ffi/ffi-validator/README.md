# FFI validator

A hacky crate that generates a C file that verifies all globals and functions exported by jl_sys exist, and that the functions have been exported with the correct signature.

```
Usage: ffi-validator [OPTIONS] <FILE>

Arguments:
  <FILE>

Options:
  -p, --print-types  Print all types used by the bindings and exit
  -h, --help         Print help
  -V, --version      Print version
```
