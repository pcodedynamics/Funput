#!/usr/bin/env bash
# Build the full Funput Linux package: Rust core + Settings UI + Fcitx5 addon,
# bundled into a single .deb. Run on Debian/Ubuntu with the build deps installed
# (see platforms/linux/README.md). Usage: platforms/linux/build.sh [build-dir]
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_ROOT="$(cd "${HERE}/../.." && pwd)"
BUILD_DIR="${1:-${HERE}/build}"

echo "==> [1/4] Rust core (funput-ffi cdylib)"
cargo build --release -p funput-ffi --manifest-path "${APP_ROOT}/Cargo.toml"

echo "==> [2/4] Settings UI (Svelte → dist)"
pnpm --dir "${APP_ROOT}/platforms/ui" install --frozen-lockfile
pnpm --dir "${APP_ROOT}/platforms/ui" build

echo "==> [3/4] Settings app (Tauri)"
cargo build --release --manifest-path "${HERE}/src-tauri/Cargo.toml"

echo "==> [4/4] Fcitx5 addon + .deb (CMake/CPack)"
cmake -S "${HERE}/fcitx5" -B "${BUILD_DIR}" -DCMAKE_BUILD_TYPE=Release
cmake --build "${BUILD_DIR}" --parallel
( cd "${BUILD_DIR}" && cpack -G DEB )

echo "==> Done. Package:"
ls -1 "${BUILD_DIR}"/*.deb
