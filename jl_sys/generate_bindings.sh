#!/usr/bin/env bash

# This script is used to generate the bindings provided by jl-sys.

target_dir=$(readlink -f $(dirname $0))
if [ "$(pwd)" != "$(readlink -f $(dirname $0))" ]; then
    echo "Error: generate_bindings.sh must be called from ${target_dir}." >&2
    exit 1
fi

if [ -z "$JULIA_STABLE_DIR" ]; then
    JULIA_STABLE_DIR=~/julia-1.7.3
fi
if [ -z "$JULIA_STABLE_DIR_32" ]; then
    JULIA_STABLE_DIR_32=$JULIA_STABLE_DIR-32
fi
if [ -z "$JULIA_STABLE_DIR_WIN" ]; then
    JULIA_STABLE_DIR_WIN=$JULIA_STABLE_DIR-win
fi
if [ -z "$JULIA_LTS_DIR" ]; then
    JULIA_LTS_DIR=~/julia-1.6.7
fi
if [ -z "$JULIA_LTS_DIR_32" ]; then
    JULIA_LTS_DIR_32=$JULIA_LTS_DIR-32
fi
if [ -z "$JULIA_LTS_DIR_WIN" ]; then
    JULIA_LTS_DIR_WIN=$JULIA_LTS_DIR-win
fi

if [ -z "$JULIA_RC_DIR" ]; then
    JULIA_RC_DIR=~/julia-1.8.0-rc3
fi
if [ -z "$JULIA_RC_DIR_32" ]; then
    JULIA_RC_DIR_32=$JULIA_RC_DIR-32
fi
if [ -z "$JULIA_RC_DIR_WIN" ]; then
    JULIA_RC_DIR_WIN=$JULIA_RC_DIR-win
fi

if [ ! -d "$JULIA_STABLE_DIR" ]; then
    echo "Error: $JULIA_STABLE_DIR does not exist" >&2
    exit 1
fi
if [ ! -d "$JULIA_STABLE_DIR_32" ]; then
    echo "Error: $JULIA_STABLE_DIR_32 does not exist" >&2
    exit 1
fi
if [ ! -d "$JULIA_STABLE_DIR_WIN" ]; then
    echo "Error: $JULIA_STABLE_DIR_WIN does not exist" >&2
    exit 1
fi
if [ ! -d "$JULIA_LTS_DIR" ]; then
    echo "Error: $JULIA_LTS_DIR does not exist" >&2
    exit 1
fi
if [ ! -d "$JULIA_LTS_DIR_32" ]; then
    echo "Error: $JULIA_LTS_DIR_32 does not exist" >&2
    exit 1
fi
if [ ! -d "$JULIA_LTS_DIR_WIN" ]; then
    echo "Error: $JULIA_LTS_DIR_WIN does not exist" >&2
    exit 1
fi
if [ ! -d "$JULIA_RC_DIR" ]; then
    echo "Error: $JULIA_RC_DIR does not exist" >&2
    exit 1
fi
if [ ! -d "$JULIA_RC_DIR_32" ]; then
    echo "Error: $JULIA_RC_DIR_32 does not exist" >&2
    exit 1
fi
if [ ! -d "$JULIA_RC_DIR_WIN" ]; then
    echo "Error: $JULIA_RC_DIR_WIN does not exist" >&2
    exit 1
fi

# Required for MSVC compatibility, the first expression globs the entire file: 
# sed -e '1h;2,$H;$!d;g' -r -E -e 's/(extern "C" \{\n\s+pub static)/#[link(name = \"libjulia\")]\n\1/g' bindings.rs
# https://unix.stackexchange.com/a/235016
cargo clean
JULIA_DIR=$JULIA_LTS_DIR_WIN LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$JULIA_DIR/lib" cargo build --features use-bindgen,windows-lts --target x86_64-pc-windows-gnu
updated=$(sed -e '1h;2,$H;$!d;g' -r -E -e 's/(extern "C" \{\n\s+pub static)/#[link(name = \"libjulia\")]\n\1/g' ../target/x86_64-pc-windows-gnu/debug/build/jl-sys*/out/bindings.rs)
echo "/* generated from Julia version 1.6.7 */" > ./src/bindings_1_6_x86_64_pc_windows_gnu.rs
echo -e "$updated" >> ./src/bindings_1_6_x86_64_pc_windows_gnu.rs

cargo clean
JULIA_DIR=$JULIA_LTS_DIR LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$JULIA_DIR/lib" cargo build --features use-bindgen,lts
echo "/* generated from Julia version 1.6.7 */" > ./src/bindings_1_6_x86_64_unknown_linux_gnu.rs
cat ../target/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_1_6_x86_64_unknown_linux_gnu.rs

cargo clean
JULIA_DIR=$JULIA_LTS_DIR_32 LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$JULIA_DIR/lib" cargo build --features use-bindgen,lts,i686 --target i686-unknown-linux-gnu
echo "/* generated from Julia version 1.6.7 */" > ./src/bindings_1_6_i686_unknown_linux_gnu.rs
cat ../target/i686-unknown-linux-gnu/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_1_6_i686_unknown_linux_gnu.rs

cargo clean
JULIA_DIR=$JULIA_STABLE_DIR_WIN LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$JULIA_DIR/lib" cargo build --features use-bindgen --target x86_64-pc-windows-gnu
updated=$(sed -e '1h;2,$H;$!d;g' -r -E -e 's/(extern "C" \{\n\s+pub static)/#[link(name = \"libjulia\")]\n\1/g' ../target/x86_64-pc-windows-gnu/debug/build/jl-sys*/out/bindings.rs)
echo "/* generated from Julia version 1.7.3 */" > ./src/bindings_1_7_x86_64_pc_windows_gnu.rs
echo -e "$updated" >> ./src/bindings_1_7_x86_64_pc_windows_gnu.rs

cargo clean
JULIA_DIR=$JULIA_STABLE_DIR LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$JULIA_DIR/lib" cargo build --features use-bindgen
echo "/* generated from Julia version 1.7.3 */" > ./src/bindings_1_7_x86_64_unknown_linux_gnu.rs
cat ../target/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_1_7_x86_64_unknown_linux_gnu.rs

cargo clean
JULIA_DIR=$JULIA_STABLE_DIR_32 LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$JULIA_DIR/lib" cargo build --features use-bindgen,i686 --target i686-unknown-linux-gnu
echo "/* generated from Julia version 1.7.3 */" > ./src/bindings_1_7_i686_unknown_linux_gnu.rs
cat ../target/i686-unknown-linux-gnu/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_1_7_i686_unknown_linux_gnu.rs

cargo clean
JULIA_DIR=$JULIA_RC_DIR_WIN LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$JULIA_DIR/lib" cargo build --features use-bindgen --target x86_64-pc-windows-gnu
updated=$(sed -e '1h;2,$H;$!d;g' -r -E -e 's/(extern "C" \{\n\s+pub static)/#[link(name = \"libjulia\")]\n\1/g' ../target/x86_64-pc-windows-gnu/debug/build/jl-sys*/out/bindings.rs)
echo "/* generated from Julia version 1.8.0-rc3 */" > ./src/bindings_1_8_x86_64_pc_windows_gnu.rs
echo -e "$updated" >> ./src/bindings_1_8_x86_64_pc_windows_gnu.rs

cargo clean
JULIA_DIR=$JULIA_RC_DIR LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$JULIA_DIR/lib" cargo build --features use-bindgen
echo "/* generated from Julia version 1.8.0-rc3 */" > ./src/bindings_1_8_x86_64_unknown_linux_gnu.rs
cat ../target/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_1_8_x86_64_unknown_linux_gnu.rs

cargo clean
JULIA_DIR=$JULIA_RC_DIR_32 LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$JULIA_DIR/lib" cargo build --features use-bindgen,i686 --target i686-unknown-linux-gnu
echo "/* generated from Julia version 1.8.0-rc3 */" > ./src/bindings_1_8_i686_unknown_linux_gnu.rs
cat ../target/i686-unknown-linux-gnu/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_1_8_i686_unknown_linux_gnu.rs
