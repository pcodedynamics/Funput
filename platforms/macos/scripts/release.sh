#!/bin/sh
# Build, sign (Developer ID), notarize, and package Funput.app into a .pkg
# installer for direct distribution (GitHub Releases). Funput is an input method
# and cannot be sandboxed, so the Mac App Store is not an option. A .pkg (not a
# DMG + shell-script installer) is used because a flat script cannot carry a
# notarization ticket and so always trips Gatekeeper on download.
#
#   ./scripts/release.sh             # full release: Developer ID + notarize + staple
#   DRY_RUN=1 ./scripts/release.sh   # ad-hoc sign, skip notarize — test the pipeline
#
# Notarization credentials (pick one):
#   - Local:  a notarytool keychain profile named "$NOTARY_PROFILE" (default funput)
#       xcrun notarytool store-credentials funput \
#         --apple-id <email> --team-id RSARFZ5CD3 --password <app-specific-password>
#   - CI:     set NOTARY_APPLE_ID + NOTARY_PASSWORD (app-specific) env vars; the
#             script uses them directly, no stored profile needed.
#
# VERSION can be overridden (e.g. CI passes VERSION=${tag#v}); otherwise it is read
# from the project's MARKETING_VERSION.
set -eu

CONFIGURATION="${CONFIGURATION:-Release}"
SIGN_ID="${SIGN_ID:-Developer ID Application}"
SIGN_INSTALLER_ID="${SIGN_INSTALLER_ID:-Developer ID Installer}"
PKG_IDENTIFIER="${PKG_IDENTIFIER:-com.funput.installer}"
TEAM_ID="${TEAM_ID:-RSARFZ5CD3}"
NOTARY_PROFILE="${NOTARY_PROFILE:-funput}"
DRY_RUN="${DRY_RUN:-}"

# notarytool auth: env credentials (CI) take precedence over the keychain profile.
USE_ENV_NOTARY=
if [ -n "${NOTARY_APPLE_ID:-}" ] && [ -n "${NOTARY_PASSWORD:-}" ]; then
    USE_ENV_NOTARY=1
fi

# Submit a file to the notary service and block until Apple finishes processing.
notarize() {
    if [ -n "$USE_ENV_NOTARY" ]; then
        xcrun notarytool submit "$1" --apple-id "$NOTARY_APPLE_ID" \
            --team-id "$TEAM_ID" --password "$NOTARY_PASSWORD" --wait
    else
        xcrun notarytool submit "$1" --keychain-profile "$NOTARY_PROFILE" --wait
    fi
}

PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_DIR"

OUT="$PROJECT_DIR/build/release"
DERIVED="$OUT/DerivedData"
ARCHIVE="$OUT/Funput.xcarchive"
EXPORT="$OUT/export"
APP="$EXPORT/Funput.app"
mkdir -p "$OUT" # fresh checkout (CI) has no build/ yet; logs + zip + pkg land here

# --- 1. Version (names the DMG) -------------------------------------------------
if [ -z "${VERSION:-}" ]; then
    VERSION="$(xcodebuild -project Funput.xcodeproj -scheme Funput \
        -configuration "$CONFIGURATION" -showBuildSettings 2>/dev/null \
        | awk '/ MARKETING_VERSION = /{print $3; exit}')"
fi
VERSION="${VERSION:-0.0.0}"
PKG="$OUT/Funput-$VERSION.pkg"
APPZIP="$OUT/Funput-$VERSION.app.zip"
echo "Releasing Funput $VERSION (${DRY_RUN:+DRY_RUN }$CONFIGURATION)…"

# --- 2. Preflight ---------------------------------------------------------------
if [ -n "$DRY_RUN" ]; then
    SIGN_ID="-" # ad-hoc
else
    if ! security find-identity -v -p codesigning | grep -q "Developer ID Application"; then
        cat >&2 <<EOF
error: no "Developer ID Application" certificate found in the keychain.
Create one in Xcode → Settings → Accounts → Manage Certificates → + →
"Developer ID Application", then re-run. (Use DRY_RUN=1 to test packaging now.)
EOF
        exit 1
    fi
    # The .pkg installer is signed with a *separate* "Developer ID Installer"
    # certificate (basic codesigning lists do not show it; search all identities).
    if ! security find-identity -v | grep -q "Developer ID Installer"; then
        cat >&2 <<EOF
error: no "Developer ID Installer" certificate found in the keychain.
This is a different cert from "Developer ID Application" and is required to sign
the .pkg. Create one in Xcode → Settings → Accounts → Manage Certificates → + →
"Developer ID Installer", then re-run. (Use DRY_RUN=1 to test packaging now.)
EOF
        exit 1
    fi
    if [ -z "$USE_ENV_NOTARY" ] \
        && ! xcrun notarytool history --keychain-profile "$NOTARY_PROFILE" >/dev/null 2>&1; then
        cat >&2 <<EOF
error: no notary credentials. Either set NOTARY_APPLE_ID + NOTARY_PASSWORD, or
create a keychain profile "$NOTARY_PROFILE":
  xcrun notarytool store-credentials $NOTARY_PROFILE \\
    --apple-id <email> --team-id $TEAM_ID --password <app-specific-password>
EOF
        exit 1
    fi
fi

rm -rf "$ARCHIVE" "$EXPORT" "$OUT/pkgroot" "$OUT/Funput-component.pkg" "$PKG" "$APPZIP"

# Run a build/export step quietly, but dump the captured xcodebuild log on failure
# (xcodebuild writes errors to stdout, so swallowing it would hide the reason — as
# happened in early CI runs).
run_xcode() {
    log="$OUT/xcodebuild.log"
    mkdir -p "$OUT"
    if ! "$@" >"$log" 2>&1; then
        echo "xcodebuild failed:" >&2
        tail -n 40 "$log" >&2
        exit 1
    fi
}

# --- 3. Archive (universal: arm64 + x86_64) -------------------------------------
# Sign at archive time with the Developer ID identity directly (Manual). The
# project defaults to Automatic signing, which needs a logged-in Apple account and
# a development cert — neither exists on a CI runner, so Automatic fails there. The
# Developer ID cert in the keychain is all we need for direct distribution; no
# provisioning profile is required (the app is not sandboxed). DRY_RUN skips signing
# entirely and ad-hoc signs afterwards.
echo "Archiving (universal)…"
# Build args via positional params so values with spaces ("Developer ID
# Application", "arm64 x86_64") survive as single arguments.
# CURRENT_PROJECT_VERSION (→ CFBundleVersion) is stamped to the same tag value as
# MARKETING_VERSION. The project default is a static "1", but Sparkle compares
# CFBundleVersion to decide if an update is newer — a frozen value would make every
# release look identical and updates would never be offered. The calver (e.g.
# 1.2026.1) compares correctly under Sparkle's SUStandardVersionComparator.
set -- -project Funput.xcodeproj -scheme Funput -configuration "$CONFIGURATION" \
    -derivedDataPath "$DERIVED" -archivePath "$ARCHIVE" \
    -destination "generic/platform=macOS" \
    "ARCHS=arm64 x86_64" ONLY_ACTIVE_ARCH=NO \
    "MARKETING_VERSION=$VERSION" "CURRENT_PROJECT_VERSION=$VERSION"
if [ -n "$DRY_RUN" ]; then
    set -- "$@" CODE_SIGNING_ALLOWED=NO
else
    set -- "$@" CODE_SIGN_STYLE=Manual "CODE_SIGN_IDENTITY=$SIGN_ID" \
        "DEVELOPMENT_TEAM=$TEAM_ID" "PROVISIONING_PROFILE_SPECIFIER="
fi
run_xcode xcodebuild "$@" archive

# --- 4. Export the signed .app --------------------------------------------------
if [ -n "$DRY_RUN" ]; then
    # No Developer ID export profile in dry runs: pull the app out of the archive
    # and ad-hoc re-sign it with the hardened runtime so packaging is exercised.
    echo "Exporting (ad-hoc, hardened runtime)…"
    mkdir -p "$EXPORT"
    cp -R "$ARCHIVE/Products/Applications/Funput.app" "$APP"
    codesign --force --options runtime --timestamp=none --deep --sign "-" "$APP"
else
    echo "Exporting with Developer ID…"
    run_xcode xcodebuild -exportArchive -archivePath "$ARCHIVE" -exportPath "$EXPORT" \
        -exportOptionsPlist "$PROJECT_DIR/ExportOptions.plist"
fi

# --- 5. Verify the signature ----------------------------------------------------
echo "Verifying signature…"
codesign --verify --deep --strict --verbose=2 "$APP"
if ! codesign -dvv "$APP" 2>&1 | grep -q 'flags=.*runtime'; then
    echo "error: hardened runtime flag missing on Funput.app" >&2
    exit 1
fi
lipo -info "$APP/Contents/MacOS/Funput"

# --- 6. Notarize the app + staple (so the copied-out app validates offline) -----
if [ -z "$DRY_RUN" ]; then
    echo "Notarizing app…"
    ditto -c -k --keepParent "$APP" "$OUT/Funput-app.zip"
    notarize "$OUT/Funput-app.zip"
    xcrun stapler staple "$APP"
    rm -f "$OUT/Funput-app.zip"
fi

# --- 7. Build the signed .pkg installer -----------------------------------------
# A double-clickable .pkg replaces the old "Install Funput.command": unlike a flat
# shell script (which cannot carry a notarization ticket and so always trips
# Gatekeeper), a .pkg is signed with "Developer ID Installer", notarized, and
# stapled — Installer.app opens it with no warning. The payload installs Funput.app
# directly into /Library/Input Methods (a valid, system-wide input-method search
# location), so the install is correct from the payload alone; the postinstall only
# does best-effort LaunchServices registration (see scripts/pkg/postinstall).
echo "Building .pkg…"
PKGROOT="$OUT/pkgroot"
SCRIPTS="$OUT/pkgscripts"
COMPONENT="$OUT/Funput-component.pkg"
rm -rf "$PKGROOT" "$SCRIPTS"
mkdir -p "$PKGROOT/Library/Input Methods" "$SCRIPTS"
cp -R "$APP" "$PKGROOT/Library/Input Methods/Funput.app"
cp "$PROJECT_DIR/scripts/pkg/postinstall" "$SCRIPTS/postinstall"
chmod +x "$SCRIPTS/postinstall"

pkgbuild --root "$PKGROOT" --scripts "$SCRIPTS" \
    --identifier "$PKG_IDENTIFIER" --version "$VERSION" \
    --install-location "/" "$COMPONENT" >/dev/null

if [ -n "$DRY_RUN" ]; then
    # No Installer identity in dry runs: emit an unsigned product archive so the
    # packaging path is still exercised end to end.
    productbuild --package "$COMPONENT" "$PKG" >/dev/null
else
    productbuild --package "$COMPONENT" --sign "$SIGN_INSTALLER_ID" "$PKG" >/dev/null
fi
rm -rf "$PKGROOT" "$SCRIPTS" "$COMPONENT"

# --- 8. Notarize the .pkg + staple ----------------------------------------------
if [ -z "$DRY_RUN" ]; then
    echo "Notarizing .pkg…"
    notarize "$PKG"
    xcrun stapler staple "$PKG"
fi

# --- 8b. Zip the stapled app (no-admin / per-user install) ----------------------
# Secondary artifact for users who cannot authenticate as admin: the .pkg installs
# system-wide (/Library/Input Methods) and so needs an admin password, but the app
# is notarized + stapled (step 6), so a non-admin can just unzip this and drop
# Funput.app into their own ~/Library/Input Methods — no privileges, no Gatekeeper
# warning. keepParent so the archive expands to "Funput.app", not loose contents.
echo "Zipping app (no-admin install)…"
ditto -c -k --keepParent "$APP" "$APPZIP"

# --- 9. Report ------------------------------------------------------------------
echo ""
echo "Built: $PKG"
shasum -a 256 "$PKG"
echo "Built: $APPZIP"
shasum -a 256 "$APPZIP"
if [ -n "$DRY_RUN" ]; then
    echo "(DRY_RUN: ad-hoc signed app, UNSIGNED pkg, NOT notarized — pipeline test only.)"
fi
