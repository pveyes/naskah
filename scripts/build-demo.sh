#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../demo"

rustup target add wasm32-unknown-unknown
command -v wasm-pack >/dev/null || cargo install wasm-pack
wasm-pack build --release --target web --out-name wasm --out-dir ./static/assets
