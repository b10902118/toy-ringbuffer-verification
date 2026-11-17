#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
bash scripts/verify-flux.sh
bash scripts/verify-verus.sh
