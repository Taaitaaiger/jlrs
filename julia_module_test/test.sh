#!/usr/bin/env bash

version=julia-$(julia --version | grep -oEe "1.[0-9]+" | sed "s/\./-/g")
julia_dir="$(dirname $(dirname $(which julia)))"
JULIA_DIR=$julia_dir cargo +1.79 build --release --features $version || exit 1
cp ./target/release/libjulia_module_test.so . || exit 1
julia JuliaModuleTest.jl
