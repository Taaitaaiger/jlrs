#!/usr/bin/env bash

unset JLRS_JULIA_DIR

cd ../julia_module_test
cargo build || exit 1
cd -

JULIA_MODULE_TEST_LIB_DIR=$(pwd)/../julia_module_test/target/debug julia JuliaModuleTest.jl
