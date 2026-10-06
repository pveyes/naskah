#!/usr/bin/env bash
# Build the wasm demo, then serve it locally with the Cloudflare dev server.
set -euo pipefail
cd "$(dirname "$0")/.."

./scripts/build-demo.sh
cd demo/static
npm install --no-audit --no-fund
npx --no-install cf dev
