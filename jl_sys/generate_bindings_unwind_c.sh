#!/usr/bin/env bash

export RUST_BACKTRACE=1
LD_LIBRARY_PATH=
NIGHTLY="n"
BETA="n"
ALL="n"

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
            --all)
                ALL="y"
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
    local spacing=$(printf %$((13 + ${#HOME}))s)

    echo "    generate_bindings.sh [--nightly] [--beta] [--all]"
    echo ""
    echo "This script can be used to generate Rust bindings to the Julia C API with"
    echo "bindgen. It can either generate bindings for all supported versions of Julia,"
    echo "or for the nightly version specifically with the --nightly flag."
    echo ""
    echo "To use it without the nightly flag, all versions of Julia supported by jlrs"
    echo "must be available. The following versions and default paths are expected, the"
    echo "default paths can be overridden with environment variables:"
    echo ""
    echo -e "\033[1m       Version              Default path${spacing}Override\033[0m"
    echo "    Linux 64-bit 1.9    $HOME/julia-1.9.3             JULIA_1_9_DIR"
    echo "    Linux 64-bit 1.8    $HOME/julia-1.8.5             JULIA_1_8_DIR"
    echo "    Linux 64-bit 1.7    $HOME/julia-1.7.3             JULIA_1_7_DIR"
    echo "    Linux 64-bit 1.6    $HOME/julia-1.6.7             JULIA_1_6_DIR"
    echo ""
    echo ""
    echo "When the nightly flag is set, the following is expected:"
    echo ""
    echo -e "\033[1m        Version             Default path${spacing}Override\033[0m"
    echo "    Linux 64-bit dev    $HOME/Projects/C/julia/usr    JULIA_NIGHTLY_DIR"
    echo ""
    echo ""
    echo "When the beta flag is set, the following is expected:"
    echo ""
    echo -e "\033[1m        Version             Default path${spacing}Override\033[0m"
    echo "    Linux 64-bit 1.10   $HOME/julia-1.10.0-beta3      JULIA_1_10_DIR"
    echo ""
    echo ""
    echo "All dependencies must have been installed before running this script. The"
    echo "following should be sufficient on Ubuntu:"
    echo ""
    echo "    apt install llvm-dev libclang-dev clang g++-multilib-i686-linux-gnu"
    echo "    rustup target add i686-unknown-linux-gnu"
    echo "    rustup toolchain install nightly"
    echo "    rustup toolchain install stable-i686-unknown-linux-gnu"
}

parse_args $@
check_dir

if [ "${NIGHTLY}" = "y" -o "${ALL}" = "y" ]; then
    if [ -z "$JULIA_NIGHTLY_DIR" ]; then
        JULIA_NIGHTLY_DIR=${HOME}/Projects/C/julia/usr
    fi
    if [ ! -d "$JULIA_NIGHTLY_DIR" ]; then
        echo "Error: $JULIA_NIGHTLY_DIR does not exist" >&2
        exit 1
    fi

    cargo clean
    JULIA_VERSION=$($JULIA_NIGHTLY_DIR/bin/julia --version)
    JULIA_COMMIT=$($JULIA_NIGHTLY_DIR/bin/julia -E "Base.GIT_VERSION_INFO.commit_short" | grep -oEe "[^\"]+")
    JULIA_COMMIT_DATE=$($JULIA_NIGHTLY_DIR/bin/julia -E "Base.GIT_VERSION_INFO.date_string" | grep -oEe "[^\"]+")
    JULIA_DIR=$JULIA_NIGHTLY_DIR cargo +nightly build --features use-bindgen,c-unwind,julia-1-11
    echo "/* generated from $JULIA_VERSION (Commit: $JULIA_COMMIT $JULIA_COMMIT_DATE) */" > ./src/bindings_unwind/bindings_unwind_1_11_64.rs
    cat ../target/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_unwind/bindings_unwind_1_11_64.rs

    cargo clean
    JULIA_DIR=$JULIA_NIGHTLY_DIR RUSTC_BOOTSTRAP=1 cargo build --features use-bindgen,c-unwind,i686,julia-1-11 --target i686-unknown-linux-gnu
    echo "/* generated from $JULIA_VERSION (Commit: $JULIA_COMMIT $JULIA_COMMIT_DATE) */" > ./src/bindings_unwind/bindings_unwind_1_11_32.rs
    cat ../target/i686-unknown-linux-gnu/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_unwind/bindings_unwind_1_11_32.rs

    if [ "${BETA}" != "y" -a "${ALL}" != "y"  ]; then
        cargo +nightly fmt -- ./src/bindings_unwind/bindings_unwind_*
        exit
    fi
fi

if [ "${BETA}" = "y" -o "${ALL}" = "y" ]; then
    if [ -z "$JULIA_BETA_DIR" ]; then
        JULIA_BETA_DIR=${HOME}/julia-1.10.0-beta3
    fi
    if [ ! -d "$JULIA_BETA_DIR" ]; then
        echo "Error: $JULIA_BETA_DIR does not exist" >&2
        exit 1
    fi

    cargo clean
    JULIA_VERSION=$($JULIA_BETA_DIR/bin/julia --version)
    JULIA_DIR=$JULIA_BETA_DIR cargo +nightly build --features use-bindgen,c-unwind,julia-1-10
    echo "/* generated from $JULIA_VERSION */" > ./src/bindings_unwind/bindings_unwind_1_10_64.rs
    cat ../target/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_unwind/bindings_unwind_1_10_64.rs

    cargo clean
    JULIA_DIR=$JULIA_BETA_DIR RUSTC_BOOTSTRAP=1 cargo build --features use-bindgen,c-unwind,i686,julia-1-10 --target i686-unknown-linux-gnu
    echo "/* generated from $JULIA_VERSION */" > ./src/bindings_unwind/bindings_unwind_1_10_32.rs
    cat ../target/i686-unknown-linux-gnu/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_unwind/bindings_unwind_1_10_32.rs

    # echo "Warning: there is no known beta version. Skipping beta bindings" >&2
    if [ "${ALL}" != "y"  ]; then
        cargo +nightly fmt -- ./src/bindings_unwind/bindings_unwind_*
        exit
    fi
fi

if [ -z "$JULIA_1_9_DIR" ]; then
    JULIA_1_9_DIR=${HOME}/julia-1.9.3
fi
if [ ! -d "$JULIA_1_9_DIR" ]; then
    echo "Error: $JULIA_1_9_DIR does not exist" >&2
    exit 1
fi

if [ -z "$JULIA_1_8_DIR" ]; then
    JULIA_1_8_DIR=${HOME}/julia-1.8.5
fi
if [ ! -d "$JULIA_1_8_DIR" ]; then
    echo "Error: $JULIA_1_8_DIR does not exist" >&2
    exit 1
fi

if [ -z "$JULIA_1_7_DIR" ]; then
    JULIA_1_7_DIR=${HOME}/julia-1.7.3
fi
if [ ! -d "$JULIA_1_7_DIR" ]; then
    echo "Error: $JULIA_1_7_DIR does not exist" >&2
    exit 1
fi

if [ -z "$JULIA_1_6_DIR" ]; then
    JULIA_1_6_DIR=${HOME}/julia-1.6.7
fi
if [ ! -d "$JULIA_1_6_DIR" ]; then
    echo "Error: $JULIA_1_6_DIR does not exist" >&2
    exit 1
fi

cargo clean
JULIA_VERSION=$($JULIA_1_6_DIR/bin/julia --version)
JULIA_DIR=$JULIA_1_6_DIR cargo +nightly build --features use-bindgen,c-unwind,julia-1-6
echo "/* generated from $JULIA_VERSION */" > ./src/bindings_unwind/bindings_unwind_1_6_64.rs
cat ../target/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_unwind/bindings_unwind_1_6_64.rs

cargo clean
JULIA_DIR=$JULIA_1_6_DIR RUSTC_BOOTSTRAP=1 cargo build --features use-bindgen,c-unwind,julia-1-6,i686 --target i686-unknown-linux-gnu
echo "/* generated from $JULIA_VERSION */" > ./src/bindings_unwind/bindings_unwind_1_6_32.rs
cat ../target/i686-unknown-linux-gnu/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_unwind/bindings_unwind_1_6_32.rs

cargo clean
JULIA_VERSION=$($JULIA_1_7_DIR/bin/julia --version)
JULIA_DIR=$JULIA_1_7_DIR cargo +nightly build --features use-bindgen,c-unwind,julia-1-7
echo "/* generated from $JULIA_VERSION */" > ./src/bindings_unwind/bindings_unwind_1_7_64.rs
cat ../target/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_unwind/bindings_unwind_1_7_64.rs

cargo clean
JULIA_DIR=$JULIA_1_7_DIR RUSTC_BOOTSTRAP=1 cargo build --features use-bindgen,c-unwind,i686,julia-1-7 --target i686-unknown-linux-gnu
echo "/* generated from $JULIA_VERSION */" > ./src/bindings_unwind/bindings_unwind_1_7_32.rs
cat ../target/i686-unknown-linux-gnu/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_unwind/bindings_unwind_1_7_32.rs

cargo clean
JULIA_VERSION=$($JULIA_1_8_DIR/bin/julia --version)
JULIA_DIR=$JULIA_1_8_DIR cargo +nightly build --features use-bindgen,c-unwind,julia-1-8
echo "/* generated from $JULIA_VERSION */" > ./src/bindings_unwind/bindings_unwind_1_8_64.rs
cat ../target/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_unwind/bindings_unwind_1_8_64.rs

cargo clean
JULIA_DIR=$JULIA_1_8_DIR RUSTC_BOOTSTRAP=1 cargo build --features use-bindgen,c-unwind,i686,julia-1-8 --target i686-unknown-linux-gnu
echo "/* generated from $JULIA_VERSION */" > ./src/bindings_unwind/bindings_unwind_1_8_32.rs
cat ../target/i686-unknown-linux-gnu/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_unwind/bindings_unwind_1_8_32.rs

cargo clean
JULIA_VERSION=$($JULIA_1_9_DIR/bin/julia --version)
JULIA_DIR=$JULIA_1_9_DIR cargo +nightly build --features use-bindgen,c-unwind,julia-1-9
echo "/* generated from $JULIA_VERSION */" > ./src/bindings_unwind/bindings_unwind_1_9_64.rs
cat ../target/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_unwind/bindings_unwind_1_9_64.rs

cargo clean
JULIA_DIR=$JULIA_1_9_DIR RUSTC_BOOTSTRAP=1 cargo build --features use-bindgen,c-unwind,i686,julia-1-9 --target i686-unknown-linux-gnu
echo "/* generated from $JULIA_VERSION */" > ./src/bindings_unwind/bindings_unwind_1_9_32.rs
cat ../target/i686-unknown-linux-gnu/debug/build/jl-sys*/out/bindings.rs >> ./src/bindings_unwind/bindings_unwind_1_9_32.rs

cargo +nightly fmt -- ./src/bindings_unwind/bindings_unwind_*
