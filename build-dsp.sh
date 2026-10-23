#!/usr/bin/env bash
# Builds the DSP core to wasm and drops it where the Dioxus app picks it up.
set -euo pipefail
cd "$(dirname "$0")"
cargo build --release -p strange-dsp --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/strange_dsp.wasm app/assets/dsp.wasm
