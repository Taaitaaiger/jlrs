#!/usr/bin/env bash

# This script is used to generate the bindings provided by jl-sys. It's currently as bare-bones as it can be.

cargo clean
JULIA_DIR=~/julia-1.6.6-win LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$JULIA_DIR/lib:$JULIA_DIR/lib/julia" cargo build --features use-bindgen,windows-lts --target x86_64-pc-windows-gnu

# Required for MSVC compatibility, the first expression globs the entire file: 
# https://unix.stackexchange.com/a/235016
# 
# TODO: check if only static data needs to be annotated
updated=$(sed \
    -e '1h;2,$H;$!d;g' \
    -r -E \
    -e 's/(extern "C" \{\n\s+pub (static|fn uv|fn jl[^r]))/#[link(name = "libjulia")]\n\1/g' \
    ../target/x86_64-pc-windows-gnu/debug/build/jl-sys*/out/bindings.rs)

echo "#![allow(deref_nullptr)]" > ./src/bindings_1_6_x86_64_pc_windows_gnu.rs
echo "/* generated from Julia version 1.6.6 */" >> ./src/bindings_1_6_x86_64_pc_windows_gnu.rs
echo -e "$updated" >> ./src/bindings_1_6_x86_64_pc_windows_gnu.rs

cargo clean
JULIA_DIR=~/julia-1.6.6 LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$JULIA_DIR/lib:$JULIA_DIR/lib/julia" cargo build --features use-bindgen,lts
echo "#![allow(deref_nullptr)]" > ./src/bindings_1_6_x86_64_unknown_linux_gnu.rs
echo "/* generated from Julia version 1.6.6 */" >> ./src/bindings_1_6_x86_64_unknown_linux_gnu.rs
cat ../target/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_1_6_x86_64_unknown_linux_gnu.rs

cargo clean
JULIA_DIR=~/julia-1.8.0-beta3-win/ LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$JULIA_DIR/lib:$JULIA_DIR/lib/julia" cargo build --features use-bindgen --target x86_64-pc-windows-gnu
updated=$(sed \
    -e '1h;2,$H;$!d;g' \
    -r -E \
    -e 's/(extern "C" \{\n\s+pub (static|fn uv|fn jl[^r]))/#[link(name = "libjulia")]\n\1/g' \
    ../target/x86_64-pc-windows-gnu/debug/build/jl-sys*/out/bindings.rs)

echo "#![allow(deref_nullptr)]" > ./src/bindings_1_8_x86_64_pc_windows_gnu.rs
echo "/* generated from Julia version 1.8.0-beta3 */" >> ./src/bindings_1_8_x86_64_pc_windows_gnu.rs
echo -e "$updated" >> ./src/bindings_1_8_x86_64_pc_windows_gnu.rs

cargo clean
JULIA_DIR=~/julia-1.8.0-beta3/ LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$JULIA_DIR/lib:$JULIA_DIR/lib/julia" cargo build --features use-bindgen
echo "#![allow(deref_nullptr)]" > ./src/bindings_1_8_x86_64_unknown_linux_gnu.rs
echo "/* generated from Julia version 1.8.0-beta3 */" >> ./src/bindings_1_8_x86_64_unknown_linux_gnu.rs
cat ../target/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_1_8_x86_64_unknown_linux_gnu.rs
