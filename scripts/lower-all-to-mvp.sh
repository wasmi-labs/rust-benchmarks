#!/usr/bin/env bash
#
# Optimize every benchmark's `out.wasm` in place with `wasm-opt`.
#
# Run this whenever a benchmark has changed or after a new Rust, LLVM or
# `wasm-opt` version has been released, following `cargo run --package build_benches`.
#
# Usage:
#     ./scripts/wasm-opt.sh
#
# Works from any directory: paths are resolved relative to the repo root.

set -euo pipefail

# Resolve the repo root from this script's own location.
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root_dir="$(cd "$script_dir/.." && pwd)"
cd "$root_dir"

if ! command -v wasm-opt >/dev/null 2>&1; then
    echo "error: 'wasm-opt' not found on PATH (install binaryen)" >&2
    exit 1
fi

shopt -s nullglob
wasm_files=(cases/*/out.wasm)

if [ ${#wasm_files[@]} -eq 0 ]; then
    echo "error: no 'cases/*/out.wasm' files found" >&2
    exit 1
fi

for wasm in "${wasm_files[@]}"; do
    case_name="$(basename "$(dirname "$wasm")")"
    echo "optimizing $case_name ..."
    wasm-opt "$wasm" \
        -O3 \
        --enable-sign-ext \
        --enable-mutable-globals \
        --enable-tail-call \
        --enable-reference-types \
        --enable-nontrapping-float-to-int \
        --enable-bulk-memory \
        --llvm-memory-copy-fill-lowering \
        --llvm-nontrapping-fptoint-lowering \
        --signext-lowering \
        -o "$wasm"
done

echo "done: optimized ${#wasm_files[@]} benchmark(s)"
