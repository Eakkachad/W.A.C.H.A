#!/usr/bin/env bash
# Build the offline WASM app: compile wacha-wasm → wasm32, copy the .wasm into
# web/, report raw + gzip size. Serve web/ with any static server (offline after
# first load thanks to the service worker). No wasm-bindgen tooling required.
#
#   cd wacha-wasm && bash web/build.sh && (cd web && python3 -m http.server 8099)
set -eu
cd "$(dirname "$0")/.."   # -> wacha-wasm/
cargo build --release --target wasm32-unknown-unknown
WASM=target/wasm32-unknown-unknown/release/wacha_wasm.wasm
cp "$WASM" web/wacha_wasm.wasm
raw=$(wc -c < web/wacha_wasm.wasm)
gz=$(gzip -9 -c web/wacha_wasm.wasm | wc -c)
echo "wasm raw:  ${raw} bytes ($(echo "scale=2; $raw/1048576" | bc) MB)"
echo "wasm gzip: ${gz} bytes ($(echo "scale=2; $gz/1048576" | bc) MB)   [budget: ~25 MB]"
