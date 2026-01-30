#!/usr/bin/env bash

#unset JLRS_JULIA_DIR

cargo build || exit 1
cp ./target/debug/libjulia_module_test.* . || exit 1

if [ -z "$1" ]; then
	julia JuliaModuleTest.jl
else
	# Supply argument to run alternative script
	julia $1
fi
