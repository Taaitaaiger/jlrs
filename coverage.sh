#!/usr/bin/env bash

set -euxo pipefail

cargo llvm-cov clean --workspace;
cargo llvm-cov --features full,ccall --workspace --no-report
cargo llvm-cov --example ccall --no-report -- --test-threads=1
cargo llvm-cov --example ccall_throw_exception --no-report -- --test-threads=1
cargo llvm-cov run --example async_tasks --no-report
cargo llvm-cov run --example call_julia --no-report
cargo llvm-cov run --example nested_async_scopes --no-report
cargo llvm-cov run --example persistent_tasks --no-report
cargo llvm-cov run --example with_rayon --no-report

cargo llvm-cov --no-run --open --hide-instantiations --ignore-filename-regex "(jl_sys|jlrs_sys|build_utils)"
