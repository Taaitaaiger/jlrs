#!/usr/bin/env bash

set -euxo pipefail

cargo clean

LLVM_PROFILE_FILE="jlrs-%p-%m.profraw" \
RUSTFLAGS="-Zinstrument-coverage" \
cargo +nightly test --features sync-rt,tokio-rt,jlrs-derive,f16,jlrs-ndarray,ccall,uv \
--profile=coverage -- --test-threads=1

grcov . -s . --binary-path ./target/coverage/ -t html --branch --ignore-not-existing -o \
./target/coverage/coverage/ --ignore "jl_sys/*" --ignore "jlrs/tests/*" --ignore \
"jlrs_derive_tests/*" --ignore "jlrs_async_tests/*" --ignore "jlrs/src/wrappers/ptr/internal/*"

find . -name "*.profraw" -print0 | xargs -0 rm
