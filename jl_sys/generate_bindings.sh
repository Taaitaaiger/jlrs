#!/usr/bin/env bash

# This script is used to generate the bindings provided by jl-sys.

function print_help() {
    echo "    generate_bindings.sh [--nightly]"
    echo ""
    echo "This script can be used to generate Rust bindings to the Julia C API with"
    echo "bindgen. It can either generate bindings for all supported versions of Julia,"
    echo "or for the nightly version specifically with the --nightly flag."
    echo ""
    echo "To use it without the nightly flag, all versions of Julia supported by jlrs"
    echo "must be available. The following versions and default paths are expected, the"
    echo "default paths can be overridden with environment variables:"
    echo ""
    echo -e "\033[1m      Version                   Default path            Override\033[0m"
    echo "    Linux 64 bit stable:      ~/julia-1.8.0           JULIA_STABLE_DIR"
    echo "    Linux 64 bit lts:         ~/julia-1.6.7           JULIA_LTS_DIR"
    echo "    Linux 32 bit stable:      ~/julia-1.8.0-32        JULIA_STABLE_DIR_32"
    echo "    Linux 32 bit lts:         ~/julia-1.6.7-32        JULIA_LTS_DIR_32"
    echo "    Windows 64 bit stable:    ~/julia-1.8.0-win       JULIA_STABLE_DIR_WIN"
    echo "    Windows 64 bit lts:       ~/julia-1.6.7-win       JULIA_LTS_DIR_WIN"
    echo ""
    echo "When the nightly flag is set, the following is expected:"
    echo ""
    echo -e "\033[1m      Version                   Default path            Override\033[0m"
    echo "    Linux 64 bit nightly:     ~/Projects/C/julia/usr  JULIA_NIGHTLY_DIR"
    echo ""
    echo "All dependencies must have been installed before running this script:"
    echo ""
    echo "    apt install llvm-dev libclang-dev clang \\"
    echo "                g++-multilib-i686-linux-gnu \\"
    echo "                g++-mingw-w64-x86-64"
    echo "    rustup target add i686-unknown-linux-gnu"
    echo "    rustup toolchain install stable-i686-unknown-linux-gnu"
    echo "    rustup target add x86_64-pc-windows-gnu"
    echo "    rustup toolchain install stable-x86_64-pc-windows-gnu"


}

target_dir=$(readlink -f $(dirname $0))
if [ "$(pwd)" != "$(readlink -f $(dirname $0))" ]; then
    echo "Error: generate_bindings.sh must be called from ${target_dir}." >&2
    exit 1
fi

LD_LIBRARY_PATH=

while [ -n "$1" ]; do
    case $1 in
        --nightly)
            if [ -z "$JULIA_NIGHTLY_DIR" ]; then
                JULIA_NIGHTLY_DIR=~/Projects/C/julia/usr
            fi

            if [ ! -d "$JULIA_NIGHTLY_DIR" ]; then
                echo "Error: $JULIA_NIGHTLY_DIR does not exist" >&2
                exit 1
            fi

            cargo clean
            JULIA_DIR=$JULIA_NIGHTLY_DIR cargo build --features use-bindgen
            echo "/* generated from Julia version 1.9.0-dev */" > ./src/bindings_nightly_x86_64_unknown_linux_gnu.rs
            cat ../target/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_nightly_x86_64_unknown_linux_gnu.rs

            cargo fmt

            exit
            ;;
        -h | --help)
            print_help
            exit
            ;;
        *)
            echo "Unknown option: $1"
            print_help
            exit 1
            ;;
    esac
done

if [ -z "$JULIA_STABLE_DIR" ]; then
    JULIA_STABLE_DIR=~/julia-1.8.0
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

# Required for MSVC compatibility, the first expression globs the entire file:
# sed -e '1h;2,$H;$!d;g' -r -E -e 's/(extern "C" \{\n\s+pub static)/#[link(name = \"libjulia\")]\n\1/g' bindings.rs
# https://unix.stackexchange.com/a/235016
cargo clean
JULIA_DIR=$JULIA_LTS_DIR_WIN cargo build --features use-bindgen,windows-lts --target x86_64-pc-windows-gnu
echo "/* generated from Julia version 1.6.7 */" > ./src/bindings_1_6_x86_64_pc_windows_gnu.rs
cat ../target/x86_64-pc-windows-gnu/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_1_6_x86_64_pc_windows_gnu.rs

cargo clean
JULIA_DIR=$JULIA_LTS_DIR cargo build --features use-bindgen,lts
echo "/* generated from Julia version 1.6.7 */" > ./src/bindings_1_6_x86_64_unknown_linux_gnu.rs
cat ../target/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_1_6_x86_64_unknown_linux_gnu.rs

cargo clean
JULIA_DIR=$JULIA_LTS_DIR_32 cargo build --features use-bindgen,lts,i686 --target i686-unknown-linux-gnu
echo "/* generated from Julia version 1.6.7 */" > ./src/bindings_1_6_i686_unknown_linux_gnu.rs
cat ../target/i686-unknown-linux-gnu/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_1_6_i686_unknown_linux_gnu.rs

cargo clean
JULIA_DIR=$JULIA_STABLE_DIR_WIN cargo build --features use-bindgen,windows --target x86_64-pc-windows-gnu
echo "/* generated from Julia version 1.8.0 */" > ./src/bindings_1_8_x86_64_pc_windows_gnu.rs
cat ../target/x86_64-pc-windows-gnu/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_1_8_x86_64_pc_windows_gnu.rs

cargo clean
JULIA_DIR=$JULIA_STABLE_DIR cargo build --features use-bindgen
echo "/* generated from Julia version 1.8.0 */" > ./src/bindings_1_8_x86_64_unknown_linux_gnu.rs
cat ../target/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_1_8_x86_64_unknown_linux_gnu.rs

cargo clean
JULIA_DIR=$JULIA_STABLE_DIR_32 cargo build --features use-bindgen,i686 --target i686-unknown-linux-gnu
echo "/* generated from Julia version 1.8.0 */" > ./src/bindings_1_8_i686_unknown_linux_gnu.rs
cat ../target/i686-unknown-linux-gnu/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_1_8_i686_unknown_linux_gnu.rs

cargo fmt