# jlrs

[![Rust Docs](https://docs.rs/jlrs/badge.svg)](https://docs.rs/jlrs)
[![License:MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

jlrs is a crate that provides access to the Julia C API. It can be used to embed Julia in Rust
applications and to write interop libraries to Rust crates that can be used by Julia.

Julia versions 1.10, 1.11 and 1.12 are currently supported. In general jlrs aims to support all
versions starting at the current LTS version, but only the LTS and stable versions are actively
tested. Using the current stable version of Julia is highly recommended. The minimum supported
Rust version is currently 1.79.

This readme only contains information about what features are supported by jlrs, what
prerequisites must be met, and how to meet them. A complete tutorial is available
[here](https://taaitaaiger.github.io/jlrs-tutorial/). For more information and examples about how
to use jlrs, please read the [docs](https://docs.rs/jlrs). All documentation assumes you are
already familiar with the Julia and Rust programming languages.

## Overview

An incomplete list of features that are currently supported by jlrs:

- Access arbitrary Julia modules and their content.
- Call Julia functions, including functions that take keyword arguments.
- Handle exceptions or convert them to an error message, optionally with color.
- Include and call your own Julia code.
- Use custom system images.
- Create values that Julia can use, and convert them back to Rust, from Rust.
- Access the type information and fields of such values. Inline and bits-union fields can be
  accessed directly.
- Create and use n-dimensional arrays. The `jlrs-ndarray` feature can be enabled for
  integration with ndarray.
- Map Julia structs to Rust structs, the Rust implementation can be generated with the
  JlrsCore package.
- Structs that can be mapped to Rust include those with type parameters and bits unions.
- Use Julia from multiple threads either directly or via Julia-aware thread pools.
- Export Rust types, methods and functions to Julia with the `julia_module` macro.
- Libraries that use `julia_module` can be compiled with BinaryBuilder and distributed as JLLs.

## Prerequisites

To use jlrs, supported versions of Rust and Julia must have been installed. Currently, Julia 1.10,
1.11 and 1.12 are supported, the minimum supported Rust version is 1.79. Some features may require
a more recent version of Rust. jlrs uses the JlrsCore package for Julia, if this package has not
been installed, the latest version will be installed automatically by default.

### Linux

The recommended way to install Julia is to download the binaries from the official website,
which is distributed as an archive containing a directory called `julia-x.y.z`. This directory
contains several other directories, including a `bin` directory containing the `julia`
executable.

During compilation, the paths to the header and library are normally detected automatically by
executing the command `which julia`. The path to `julia.h` must be
`$(which julia)/../include/julia/julia.h` and the path to the library
`$(which julia)/../lib/libjulia.so`. If you want to override this default behaviour the
`JULIA_DIR` environment variable must be set to the path to the appropriate `julia.x-y-z`
directory, in this case `$JULIA_DIR/include/julia/julia.h` and
`$JULIA_DIR/lib/libjulia.so` are used instead.

In order to be able to load `libjulia.so` this file must be on the library search path. If
this is not the case you must add `/path/to/julia-x.y.z/lib` to the `LD_LIBRARY_PATH`
environment variable.

### macOS

Follow the instructions for Linux, but replace `LD_LIBRARY_PATH` with `DYLD_LIBRARY_PATH`.

### Windows

Julia can be installed using juliaup, or with the installer or portable installation
downloaded from the official website. In the first case, Julia has been likely installed in
`%USERPROFILE%\.julia\juliaup\julia-x.y.z+0~x64`, using the installer or extracting allows you to
pick the destination. After installation or extraction a folder called `Julia-x.y.z` exists, which
contains several folders including a `bin` folder containing `julia.exe`. The path to the `bin`
folder must be added to the `Path` environment variable.

Julia is automatically detected by executing the command `where julia`. If this returns
multiple locations the first one is used. The default can be overridden by setting the
`JULIA_DIR` environment variable. This doesn't work correctly with juliaup, in this case
the environment variable must be set.

## Features

Most functionality of jlrs is only available if the proper features are enabled. These
features generally belong to one of two categories: runtimes and utilities.

### Runtimes

A runtime lets initialize Julia from Rust application, the following features enable a runtime:

- `local-rt`

  Enables the local runtime. The local runtime provides single-threaded, blocking access to Julia.

- `async-rt`

  Enables the async runtime. The async runtime runs on a separate thread and can be used from
  multiple threads. This feature requires using at least Rust 1.85.

- `tokio-rt`

  The async runtime requires an executor. This feature provides a tokio-based executor. This
  feature requires using at least Rust 1.85.

- `multi-rt`

  Enables the multithreaded runtime. The multithreaded runtime lets you call Julia from arbitrary
  threads. It can be combined with the `async-rt` feature to create Julia-aware thread pools.

**WARNING**: Runtime features must only be enabled by applications that embed Julia. Libraries
must never enable a runtime feature.

**WARNING**: When building an application that embeds Julia, set
`RUSTFLAGS="-Clink-args=-rdynamic"` if you want fast code.

### Utilities

All other features are called utility features. The following are available:

- `async`

  Enable the features of the async runtime which don't depend on the executor. This
  can be used in libraries which provide implementations of tasks that the async runtime can
  handle. This feature requires using at least Rust 1.85.

- `jlrs-derive`

  This feature should be used in combination with the code generation provided by the Reflect
  module in the JlrsCore package. This module lets you generate Rust implementations of Julia
  structs, this generated code uses custom derive macros made available with this feature to
  enable the safe conversion of data from Julia to Rust, and from Rust to Julia in some cases.

- `jlrs-ndarray`

  Access the content of a Julia array as an `ArrayView` or `ArrayViewMut` from ndarray.

- `f16`

  Adds support for working with Julia's `Float16` type from Rust using half's `f16` type.

- `complex`
  Adds support for working with Julia's `Complex` type from Rust using num's `Complex` type.

- `ccall`

  Julia's `ccall` interface can be used to call functions written in Rust from Julia. No
  runtime can be used in this case because Julia has already been initialized, when this
  feature is enabled the `CCall` struct is available which offers the same functionality as
  the local runtime without initializing Julia. The `julia_module` macro is provided to
  easily export functions, types, and data in combination with the macros from the Wrap
  module in the JlrsCore package.

- `lto`

  jlrs depends on a support library written in C, if this feature is enabled this support library
  is built with support for cross-language LTO which can provide a significant performance boost.

  This feature has only been tested on Linux and requires building the support library using a
  version of `clang` with the same major version as `rustc`'s LLVM version; e.g. rust 1.78.0 uses
  LLVM 18.1.2, so it requires `clang-18`. You can check what version you need by executing
  `rustc -vV`.

  You must set the `RUSTFLAGS` environment variable if this feature is enabled, and possibly the
  `CC` environment variable. Setting `RUSTFLAGS` overrides the default flags that jlrs sets, so
  you must set at least the following flags:
  `RUSTFLAGS="-Clinker-plugin-lto -Clinker=clang-XX -Clink-arg=-fuse-ld=lld -Clink-args=-rdynamic"`.
  The last one is particularly important for embedders, forgetting it is guaranteed to kill
  performance.

- `i686`

  Link with a 32-bit build of Julia on Linux, only used for cross-compilation.

- `windows`

  Flag that must be enabled when cross-compiling for Windows from Linux.

- `debug`

  Link with a debug build of Julia on Linux.

- `no-link`

  Don't link Julia.

- `yggdrasil`

  Flag that must be enabled when compiling with BinaryBuilder.

You can enable all features except `debug`, `i686`, `windows`, `no-link` and `yggdrasil` by
enabling the `full` feature. If you don't want to enable any runtimes either, you can use
`full-no-rt`.

## Environment variables

It's possible to override certain defaults of jlrs and Julia by setting environment variables.
Many of the environment variables mentioned
[here](https://docs.julialang.org/en/v1/manual/environment-variables/) should apply to applications
that use jlrs as well, but this is mostly untested.

Several additional environment variables can be set which only affect applications that use jlrs.

- `JLRS_CORE_VERSION=major.minor.patch`
Installs the set version of JlrsCore before loading it.

- `JLRS_CORE_REVISION=rev`
Installs the set revision of JlrsCore before loading it.

- `JLRS_CORE_REPO=repo-url`
Can be used with `JLRS_CORE_REVISION` to set the repository JlrsCore will be downloaded from.

- `JLRS_CORE_NO_INSTALL=...`
Don't install JlrsCore, its value is ignored.

`JLRS_CORE_NO_INSTALL` takes priority over `JLRS_CORE_REVISION`, which takes priority over
`JLRS_CORE_VERSION`.
