#!/bin/sh
# Build funput-ffi as a (universal) static library for the Xcode app to link.
# Invoked from an Xcode "Run Script" build phase before the Compile Sources phase.
#
# Respects $ARCHS so Debug (active arch only) stays fast and Release goes universal.
set -eu

# Xcode strips the user PATH; cargo/rustup live under ~/.cargo by default.
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:$PATH"

# SRCROOT is platforms/macos; the Cargo workspace root is two levels up.
REPO_ROOT="$(cd "$SRCROOT/../.." && pwd)"
cd "$REPO_ROOT"

ARCHS="${ARCHS:-arm64}"
libs=""
for arch in $ARCHS; do
    case "$arch" in
        arm64) rust_target="aarch64-apple-darwin" ;;
        x86_64) rust_target="x86_64-apple-darwin" ;;
        *) echo "build-ffi: skipping unknown arch '$arch'" >&2; continue ;;
    esac
    rustup target add "$rust_target" >/dev/null 2>&1 || true
    cargo build -p funput-ffi --release --target "$rust_target"
    libs="$libs $REPO_ROOT/target/$rust_target/release/libfunput_ffi.a"
done

mkdir -p "$SRCROOT/Vendor"
# shellcheck disable=SC2086
lipo -create $libs -output "$SRCROOT/Vendor/libfunput_ffi.a"
echo "build-ffi: wrote $SRCROOT/Vendor/libfunput_ffi.a ($ARCHS)"
