#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../demo"

# Use the rustup toolchain even when an older cargo (for example from Homebrew)
# comes first on PATH, because wasm-pack runs whichever cargo it finds there.
# `cargo install` also puts wasm-pack in ~/.cargo/bin, which is not on PATH in every shell.
export PATH="$(dirname "$(rustup which cargo)"):${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"

rustup target add wasm32-unknown-unknown
command -v wasm-pack >/dev/null || cargo install wasm-pack
wasm-pack build --release --target web --out-name wasm --out-dir ./static/assets
