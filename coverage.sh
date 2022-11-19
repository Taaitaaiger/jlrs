#!/usr/bin/env bash

set -euxo pipefail

JULIA_DIR=$HOME/julia-1.8.3
LD_LIBRARY_PATH=$JULIA_DIR/lib:$JULIA_DIR/lib/julia
echo "backend: Gtk3Agg" > matplotlibrc

cargo llvm-cov clean --workspace;
cargo llvm-cov --features full --workspace --no-report -- --test-threads=1
cargo llvm-cov --example ccall --no-report -- --test-threads=1
cargo llvm-cov --example ccall_with_threads --no-report -- --test-threads=1
cargo llvm-cov run --example async_tasks --no-report
cargo llvm-cov run --example call_julia --no-report
cargo llvm-cov run --example fully_async_async_std --no-report
cargo llvm-cov run --example fully_async_tokio --no-report
cargo llvm-cov run --example nested_async_scopes --no-report
cargo llvm-cov run --example persistent_tasks --no-report
cargo llvm-cov run --example plot --no-report
rm matplotlibrc

cargo llvm-cov --no-run --open --hide-instantiations --ignore-filename-regex "(ptr/internal|jl_sys)"
