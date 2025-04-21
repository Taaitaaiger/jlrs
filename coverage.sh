#!/usr/bin/env bash

set -euxo pipefail

#export JULIA_DIR=$HOME/julia-1.9.3
#export LD_LIBRARY_PATH=$JULIA_DIR/lib:$JULIA_DIR/lib/julia
echo "backend: Gtk3Agg" > matplotlibrc

cargo llvm-cov clean --workspace;
cargo llvm-cov --features full,ccall --workspace --no-report
cargo llvm-cov --example ccall --no-report -- --test-threads=1
cargo llvm-cov run --example async_tasks --no-report
cargo llvm-cov run --example call_julia --no-report
cargo llvm-cov run --example fully_async_async_std --no-report
cargo llvm-cov run --example fully_async_tokio --no-report
cargo llvm-cov run --example nested_async_scopes --no-report
cargo llvm-cov run --example persistent_tasks --no-report
cargo llvm-cov run --example plot --no-report
rm matplotlibrc

cargo llvm-cov --no-run --open --hide-instantiations --ignore-filename-regex "(managed/internal|jl_sys)"
