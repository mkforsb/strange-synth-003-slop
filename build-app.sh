#!/usr/bin/env bash
# Full production build: DSP wasm -> Dioxus web bundle in target/dx/strange-app/release/web/public
set -euo pipefail
cd "$(dirname "$0")"
./build-dsp.sh
(cd app && dx build --release --platform web)
public=target/dx/strange-app/release/web/public

# dx only knows absolute asset paths ("/./assets/..." or "/<base_path>/assets/..."). Make the loader paths in
# index.html and the wasm-bindgen bootstrap page-relative so the bundle can be served from any subfolder.
# (Runtime asset!() URLs are made relative in app/src/main.rs: asset_url.)
loaders=("$public/index.html" "$public"/assets/strange-app-*.js)
sed -i -E 's#(["'\''])/(\./)?assets/#\1./assets/#g' "${loaders[@]}"
if grep -nE '["'\'']/(\./)?assets/' "${loaders[@]}"; then
    echo "error: absolute asset paths left in bundle" >&2
    exit 1
fi
echo "bundle: $(pwd)/$public"
