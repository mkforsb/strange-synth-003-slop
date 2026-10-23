#!/usr/bin/env bash
# Full production build: DSP wasm -> Dioxus web bundle in target/dx/strange-app/release/web/public
set -euo pipefail
cd "$(dirname "$0")"
./build-dsp.sh
(cd app && dx build --release --platform web)
echo "bundle: $(pwd)/target/dx/strange-app/release/web/public"
