#!/usr/bin/env bash
# Build the wasm demo, then serve it locally with the Workers runtime.
set -euo pipefail
cd "$(dirname "$0")/.."

./scripts/build-demo.sh
npx --yes wrangler@latest dev
