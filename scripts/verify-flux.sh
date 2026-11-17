#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
# Clean the Flux profile so repeated runs visibly recheck the proofs.
cargo flux clean
# Load Flux's standard-library contracts, including Option's is_some index.
flux_sysroot="${FLUX_SYSROOT:-$HOME/.flux}"
export RUSTFLAGS="${RUSTFLAGS:-} -L dependency=$flux_sysroot"
cargo flux --Fsysroot="$flux_sysroot" --Fstd-extern-specs --Fcheck-overflow=strict --Fno-panic
