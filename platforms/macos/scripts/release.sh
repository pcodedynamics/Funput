#!/bin/sh
# Build, sign (Developer ID), notarize, and package Funput.app into a DMG for
# direct distribution (GitHub Releases). Funput is an input method and cannot be
# sandboxed, so the Mac App Store is not an option.
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
mkdir -p "$OUT" # fresh checkout (CI) has no build/ yet; logs + zip + dmg land here

# --- 1. Version (names the DMG) -------------------------------------------------
if [ -z "${VERSION:-}" ]; then
    VERSION="$(xcodebuild -project Funput.xcodeproj -scheme Funput \
        -configuration "$CONFIGURATION" -showBuildSettings 2>/dev/null \
        | awk '/ MARKETING_VERSION = /{print $3; exit}')"
fi
VERSION="${VERSION:-0.0.0}"
DMG="$OUT/Funput-$VERSION.dmg"
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

rm -rf "$ARCHIVE" "$EXPORT" "$OUT/dmg" "$DMG"

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
set -- -project Funput.xcodeproj -scheme Funput -configuration "$CONFIGURATION" \
    -derivedDataPath "$DERIVED" -archivePath "$ARCHIVE" \
    -destination "generic/platform=macOS" \
    "ARCHS=arm64 x86_64" ONLY_ACTIVE_ARCH=NO
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

# --- 7. Build the DMG (app + installer helper + readme) -------------------------
echo "Building DMG…"
STAGE="$OUT/dmg"
mkdir -p "$STAGE"
cp -R "$APP" "$STAGE/Funput.app"
cp "$PROJECT_DIR/scripts/dmg/Install Funput.command" "$STAGE/"
cp "$PROJECT_DIR/scripts/dmg/README.txt" "$STAGE/"
chmod +x "$STAGE/Install Funput.command"
hdiutil create -volname "Funput" -srcfolder "$STAGE" -format UDZO -ov "$DMG" >/dev/null
rm -rf "$STAGE"

# --- 8. Notarize the DMG + staple -----------------------------------------------
if [ -z "$DRY_RUN" ]; then
    echo "Notarizing DMG…"
    notarize "$DMG"
    xcrun stapler staple "$DMG"
fi

# --- 9. Report ------------------------------------------------------------------
echo ""
echo "Built: $DMG"
shasum -a 256 "$DMG"
if [ -n "$DRY_RUN" ]; then
    echo "(DRY_RUN: ad-hoc signed, NOT notarized — for pipeline testing only.)"
fi
