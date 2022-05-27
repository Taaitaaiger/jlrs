#!/usr/bin/env bash

cargo llvm-cov clean --workspace; 
cargo llvm-cov --all-features --workspace --open --ignore-filename-regex "(ptr/internal|jl_sys)" -- --test-threads=1
