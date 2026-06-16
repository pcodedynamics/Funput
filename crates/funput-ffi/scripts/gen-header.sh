#!/usr/bin/env bash
# Regenerate the committed C header from the Rust source.
# Requires: cargo install cbindgen
set -euo pipefail

crate_dir="$(cd "$(dirname "$0")/.." && pwd)"
cbindgen --config "$crate_dir/cbindgen.toml" \
         --crate funput-ffi \
         --output "$crate_dir/include/funput.h" \
         "$crate_dir"
echo "wrote $crate_dir/include/funput.h"
