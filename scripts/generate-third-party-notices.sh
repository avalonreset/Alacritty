#!/usr/bin/env sh
set -eu

OUTPUT="${1:-THIRD_PARTY_NOTICES.html}"

# Requires `cargo-about` to be installed: `cargo install --locked cargo-about`
cd "$(dirname "$0")/.."

cargo about generate --workspace --locked about.hbs -o "$OUTPUT"

