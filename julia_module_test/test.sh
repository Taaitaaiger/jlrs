#!/usr/bin/env bash

version=julia-$(julia --version | grep -oEe "1.[0-9]+" | sed "s/\./-/g")
julia_dir="$(dirname $(dirname $(which julia)))"
JULIA_DIR=$julia_dir RUSTFLAGS="-Clinker-plugin-lto -Clinker=clang-18 -Clink-arg=-fuse-ld=lld -Copt-level=3" CC=clang-18 cargo build --release --features $version,lto || exit 1
cp ./target/release/libjulia_module_test.so . || exit 1
julia JuliaModuleTest.jl
