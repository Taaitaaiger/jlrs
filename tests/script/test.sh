#!/usr/bin/env bash

unset JLRS_JULIA_DIR

cd ../julia_module_test
jlrs-launcher run +1.12 cargo build || exit 1
cd -

JULIA_MODULE_TEST_LIB_DIR=$(pwd)/../julia_module_test/target/debug julia +1.12 JuliaModuleTest.jl
