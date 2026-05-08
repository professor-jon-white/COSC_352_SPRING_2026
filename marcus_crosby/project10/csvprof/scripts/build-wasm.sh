#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "$ROOT_DIR"
rustup target add wasm32-unknown-unknown >/dev/null
cargo build --release --target wasm32-unknown-unknown --lib

mkdir -p web/pkg
cp target/wasm32-unknown-unknown/release/csvprof.wasm web/pkg/csvprof.wasm

echo "WASM bundle written to web/pkg/csvprof.wasm"
