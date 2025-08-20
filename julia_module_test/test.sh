#!/usr/bin/env bash

julia_dir="$(dirname $(dirname $(which julia)))"
JULIA_DIR=$julia_dir cargo build || exit 1
cp ./target/debug/libjulia_module_test.so . || exit 1
julia JuliaModuleTest.jl
