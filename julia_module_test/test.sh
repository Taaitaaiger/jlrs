#!/usr/bin/env bash

version=julia-$(julia --version | grep -oEe "1.[0-9]+" | sed "s/\./-/g")
julia_dir="$(dirname $(dirname $(which julia)))"
JULIA_DIR=$julia_dir cargo build --features $version --release
cp ./target/release/libjulia_module_test.so .
julia JuliaModuleTest.jl
