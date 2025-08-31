#!/usr/bin/env bash

unset JLRS_JULIA_DIR

cargo build || exit 1
cp ./target/debug/libjulia_module_test.so . || exit 1
julia JuliaModuleTest.jl
