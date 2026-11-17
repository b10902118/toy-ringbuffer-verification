#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
verus_bin="${VERUS_BIN:-$PWD/.tools/verus-x86-linux/verus}"
if [[ ! -x "$verus_bin" ]]; then
    echo 'Verus is missing. See README.md, or set VERUS_BIN to its absolute path.' >&2
    exit 1
fi
# Check the implementation as a library; no entry point or executable needed.
"$verus_bin" --crate-type=lib src/lib.rs
