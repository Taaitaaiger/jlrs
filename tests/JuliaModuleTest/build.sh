#!/usr/bin/env bash

unset JLRS_JULIA_DIR

cd ../julia_module_test
cargo build || exit 1
cd -
