#!/usr/bin/env bash
# Build the wasm demo, then deploy it to Cloudflare Workers with the cf CLI.
# Log in first with `cf auth login`, or set CLOUDFLARE_API_TOKEN and CLOUDFLARE_ACCOUNT_ID.
set -euo pipefail
cd "$(dirname "$0")/.."

./scripts/build-demo.sh
cd demo/static
npm ci --no-audit --no-fund
npx --no-install cf deploy "$@"
