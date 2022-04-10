#!/usr/bin/env bash

set -euxo pipefail

cargo clean

LLVM_PROFILE_FILE="jlrs-%m-%p.profraw" \
RUSTFLAGS="-C instrument-coverage" \
cargo test --features sync-rt,tokio-rt,jlrs-derive,f16,jlrs-ndarray,ccall,uv --tests -- --test-threads=1

rust-profdata merge *.profraw -o jlrs.profdata

rust-cov report \
    $( \
      for file in \
        $( \
          RUSTFLAGS="-C instrument-coverage" \
            cargo test  --features sync-rt,tokio-rt,jlrs-derive,f16,jlrs-ndarray,ccall,uv --tests --no-run --message-format=json -- --test-threads=1 \
              | jq -r "select(.profile.test == true) | .filenames[]" \
              | grep -v dSYM - \
        ); \
      do \
        printf "%s %s " -object $file; \
      done \
    ) \
  --instr-profile=jlrs.profdata --summary-only

find . -name "*.profraw" -print0 | xargs -0 rm
rm ./*.profdata
