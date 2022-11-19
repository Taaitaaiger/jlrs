#!/usr/bin/env bash

LD_LIBRARY_PATH=
NIGHTLY="n"
BETA="n"

function parse_args() {
    local help="n"

    while [ -n "$1" ]; do
        case $1 in
            --nightly)
                NIGHTLY="y"
                shift
                ;;
            --beta)
                BETA="y"
                shift
                ;;
            -h | --help)
                help="y"
                shift
                ;;
            *)
                echo "Unknown option: $1" >&2
                print_help
                exit 1
                ;;
        esac
    done

    if [ $help = "y" ]; then
        print_help
        exit
    fi
}

function check_dir() {
    local target_dir=$(readlink -f $(dirname $0))
    local current_dir=$(readlink -f $(pwd))

    if [ "$current_dir" != "$target_dir" ]; then
        echo "Error: generate_bindings.sh must be called from ${target_dir}" >&2
        exit 1
    fi
}

function print_help() {
    local spacing=$(printf %$((15 + ${#HOME}))s)

    echo "    generate_bindings.sh [--nightly] [--beta]"
    echo ""
    echo "This script can be used to generate Rust bindings to the Julia C API with"
    echo "bindgen. It can either generate bindings for all supported versions of Julia,"
    echo "or for the nightly version specifically with the --nightly flag."
    echo ""
    echo "To use it without the nightly flag, all versions of Julia supported by jlrs"
    echo "must be available. The following versions and default paths are expected, the"
    echo "default paths can be overridden with environment variables:"
    echo ""
    echo -e "\033[1m      Version                   Default path${spacing}Override\033[0m"
    echo "  Linux 64-bit stable:      $HOME/julia-1.8.3               JULIA_STABLE_DIR"
    echo "  Linux 64-bit lts:         $HOME/julia-1.6.7               JULIA_LTS_DIR"
    echo "  Linux 32-bit stable:      $HOME/julia-1.8.3-32            JULIA_STABLE_DIR_32"
    echo "  Linux 32-bit lts:         $HOME/julia-1.6.7-32            JULIA_LTS_DIR_32"
    echo "  Windows 64-bit stable:    $HOME/julia-1.8.3-win           JULIA_STABLE_DIR_WIN"
    echo "  Windows 64-bit lts:       $HOME/julia-1.6.7-win           JULIA_LTS_DIR_WIN"
    echo ""
    echo ""
    echo "When the nightly flag is set, the following is expected:"
    echo ""
    echo -e "\033[1m      Version                   Default path${spacing}Override\033[0m"
    echo "  Linux 64-bit nightly:     $HOME/Projects/C/julia/usr      JULIA_NIGHTLY_DIR"
    echo ""
    echo ""
    echo "When the beta flag is set, the following is expected:"
    echo ""
    echo "  Linux 64-bit beta:        $HOME/julia-1.9.0-alpha1        JULIA_BETA_DIR"
    echo "  Linux 32-bit beta:        $HOME/julia-1.9.0-alpha1-32     JULIA_BETA_DIR_32"
    echo "  Windows 64-bit beta:      $HOME/julia-1.9.0-alpha1-win    JULIA_BETA_DIR_WIN"
    echo ""
    echo ""
    echo "All dependencies must have been installed before running this script. The"
    echo "following should be sufficient on Ubuntu:"
    echo ""
    echo "    apt install llvm-dev libclang-dev clang \\"
    echo "                g++-multilib-i686-linux-gnu \\"
    echo "                g++-mingw-w64-x86-64"
    echo "    rustup target add i686-unknown-linux-gnu"
    echo "    rustup toolchain install stable-i686-unknown-linux-gnu"
    echo "    rustup target add x86_64-pc-windows-gnu"
    echo "    rustup toolchain install stable-x86_64-pc-windows-gnu"
}

parse_args $@
check_dir

if [ "${NIGHTLY}" = "y" ]; then
    if [ -z "$JULIA_NIGHTLY_DIR" ]; then
        JULIA_NIGHTLY_DIR=${HOME}/Projects/C/julia/usr
    fi
    if [ ! -d "$JULIA_NIGHTLY_DIR" ]; then
        echo "Error: $JULIA_NIGHTLY_DIR does not exist" >&2
        exit 1
    fi

    cargo clean
    JULIA_DIR=$JULIA_NIGHTLY_DIR cargo build --features use-bindgen,nightly
    echo "/* generated from Julia version 1.9.0-dev */" > ./src/bindings_nightly_x86_64_unknown_linux_gnu.rs
    cat ../target/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_nightly_x86_64_unknown_linux_gnu.rs

    cargo fmt

    if [ "${BETA}" != "y" ]; then
        exit
    fi
fi

if [ "${BETA}" = "y" ]; then
    if [ -z "$JULIA_BETA_DIR" ]; then
        JULIA_BETA_DIR=${HOME}/julia-1.9.0-alpha1
    fi
    if [ ! -d "$JULIA_BETA_DIR" ]; then
        echo "Error: $JULIA_BETA_DIR does not exist" >&2
        exit 1
    fi

    if [ -z "$JULIA_BETA_DIR_32" ]; then
        JULIA_BETA_DIR_32=${HOME}/julia-1.9.0-alpha1-32
    fi
    if [ ! -d "$JULIA_BETA_DIR_32" ]; then
        echo "Error: $JULIA_BETA_DIR_32 does not exist" >&2
        exit 1
    fi

    if [ -z "$JULIA_BETA_DIR_WIN" ]; then
        JULIA_BETA_DIR_WIN=${HOME}/julia-1.9.0-alpha1-win
    fi
    if [ ! -d "$JULIA_BETA_DIR_WIN" ]; then
        echo "Error: $JULIA_BETA_DIR_WIN does not exist" >&2
        exit 1
    fi

    cargo clean
    JULIA_DIR=$JULIA_BETA_DIR cargo build --features use-bindgen,beta
    echo "/* generated from Julia version 1.9.0-alpha1 */" > ./src/bindings_1_9_x86_64_unknown_linux_gnu.rs
    cat ../target/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_1_9_x86_64_unknown_linux_gnu.rs

    cargo clean
    JULIA_DIR=$JULIA_BETA_DIR_32 cargo build --features use-bindgen,i686,beta --target i686-unknown-linux-gnu
    echo "/* generated from Julia version 1.9.0-alpha1 */" > ./src/bindings_1_9_i686_unknown_linux_gnu.rs
    cat ../target/i686-unknown-linux-gnu/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_1_9_i686_unknown_linux_gnu.rs

    cargo clean
    JULIA_DIR=$JULIA_BETA_DIR_WIN cargo build --features use-bindgen,windows,beta --target x86_64-pc-windows-gnu
    echo "/* generated from Julia version 1.9.0-alpha1 */" > ./src/bindings_1_9_x86_64_pc_windows_gnu.rs
    cat ../target/x86_64-pc-windows-gnu/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_1_9_x86_64_pc_windows_gnu.rs

    cargo fmt

    exit
fi

if [ -z "$JULIA_STABLE_DIR" ]; then
    JULIA_STABLE_DIR=${HOME}/julia-1.8.3
fi
if [ ! -d "$JULIA_STABLE_DIR" ]; then
    echo "Error: $JULIA_STABLE_DIR does not exist" >&2
    exit 1
fi

if [ -z "$JULIA_STABLE_DIR_32" ]; then
    JULIA_STABLE_DIR_32=$JULIA_STABLE_DIR-32
fi
if [ ! -d "$JULIA_STABLE_DIR_32" ]; then
    echo "Error: $JULIA_STABLE_DIR_32 does not exist" >&2
    exit 1
fi

if [ -z "$JULIA_STABLE_DIR_WIN" ]; then
    JULIA_STABLE_DIR_WIN=$JULIA_STABLE_DIR-win
fi
if [ ! -d "$JULIA_STABLE_DIR_WIN" ]; then
    echo "Error: $JULIA_STABLE_DIR_WIN does not exist" >&2
    exit 1
fi

if [ -z "$JULIA_LTS_DIR" ]; then
    JULIA_LTS_DIR=${HOME}/julia-1.6.7
fi
if [ ! -d "$JULIA_LTS_DIR" ]; then
    echo "Error: $JULIA_LTS_DIR does not exist" >&2
    exit 1
fi

if [ -z "$JULIA_LTS_DIR_32" ]; then
    JULIA_LTS_DIR_32=$JULIA_LTS_DIR-32
fi
if [ ! -d "$JULIA_LTS_DIR_32" ]; then
    echo "Error: $JULIA_LTS_DIR_32 does not exist" >&2
    exit 1
fi

if [ -z "$JULIA_LTS_DIR_WIN" ]; then
    JULIA_LTS_DIR_WIN=$JULIA_LTS_DIR-win
fi
if [ ! -d "$JULIA_LTS_DIR_WIN" ]; then
    echo "Error: $JULIA_LTS_DIR_WIN does not exist" >&2
    exit 1
fi

cargo clean
JULIA_DIR=$JULIA_LTS_DIR cargo build --features use-bindgen,lts
echo "/* generated from Julia version 1.6.7 */" > ./src/bindings_1_6_x86_64_unknown_linux_gnu.rs
cat ../target/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_1_6_x86_64_unknown_linux_gnu.rs

cargo clean
JULIA_DIR=$JULIA_LTS_DIR_32 cargo build --features use-bindgen,lts,i686 --target i686-unknown-linux-gnu
echo "/* generated from Julia version 1.6.7 */" > ./src/bindings_1_6_i686_unknown_linux_gnu.rs
cat ../target/i686-unknown-linux-gnu/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_1_6_i686_unknown_linux_gnu.rs

cargo clean
JULIA_DIR=$JULIA_LTS_DIR_WIN cargo build --features use-bindgen,windows,lts --target x86_64-pc-windows-gnu
echo "/* generated from Julia version 1.6.7 */" > ./src/bindings_1_6_x86_64_pc_windows_gnu.rs
cat ../target/x86_64-pc-windows-gnu/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_1_6_x86_64_pc_windows_gnu.rs

cargo clean
JULIA_DIR=$JULIA_STABLE_DIR cargo build --features use-bindgen
echo "/* generated from Julia version 1.8.3 */" > ./src/bindings_1_8_x86_64_unknown_linux_gnu.rs
cat ../target/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_1_8_x86_64_unknown_linux_gnu.rs

cargo clean
JULIA_DIR=$JULIA_STABLE_DIR_32 cargo build --features use-bindgen,i686 --target i686-unknown-linux-gnu
echo "/* generated from Julia version 1.8.3 */" > ./src/bindings_1_8_i686_unknown_linux_gnu.rs
cat ../target/i686-unknown-linux-gnu/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_1_8_i686_unknown_linux_gnu.rs

cargo clean
JULIA_DIR=$JULIA_STABLE_DIR_WIN cargo build --features use-bindgen,windows --target x86_64-pc-windows-gnu
echo "/* generated from Julia version 1.8.3 */" > ./src/bindings_1_8_x86_64_pc_windows_gnu.rs
cat ../target/x86_64-pc-windows-gnu/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_1_8_x86_64_pc_windows_gnu.rs

cargo fmt
