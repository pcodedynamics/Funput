#!/bin/sh
# Build Funput.app and install it as a user input method.
#
#   ./scripts/install.sh            # Debug build
#   CONFIGURATION=Release ./scripts/install.sh
#
# After the first install, enable it once in:
#   System Settings → Keyboard → Input Sources → + → Vietnamese → Funput
set -eu

# Release by default: a clean standalone binary. Debug builds split out a
# .debug.dylib that the input-method scanner can reject.
CONFIGURATION="${CONFIGURATION:-Release}"
PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
DEST="$HOME/Library/Input Methods"

cd "$PROJECT_DIR"

# Fixed DerivedData path so the build is deterministic — avoids stale copies in
# Xcode's hashed DerivedData dirs producing a mismatched bundle id / Info.plist.
DERIVED="$PROJECT_DIR/build/DerivedData"
APP_PATH="$DERIVED/Build/Products/$CONFIGURATION/Funput.app"

echo "Building Funput ($CONFIGURATION)…"
xcodebuild -project Funput.xcodeproj -scheme Funput -configuration "$CONFIGURATION" \
    -derivedDataPath "$DERIVED" \
    -destination 'platform=macOS' build >/dev/null

echo "Installing to $DEST"
mkdir -p "$DEST"
# Quit a running instance so the bundle can be replaced and reloaded.
killall Funput 2>/dev/null || true
rm -rf "$DEST/Funput.app"
cp -R "$APP_PATH" "$DEST/Funput.app"

# LaunchServices keys input sources by bundle id. The DerivedData build shares
# app.funput.inputmethod.Funput, so unregister those and register the installed copy — else
# the input source never shows up in System Settings.
LSR="/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister"
"$LSR" -dump 2>/dev/null | grep -oE '/Users/[^ ]*DerivedData[^ ]*Funput.app' | sort -u \
    | while read -r p; do "$LSR" -u "$p" 2>/dev/null || true; done
"$LSR" -f "$DEST/Funput.app"

# Register the input source with Text Input Sources so it appears immediately,
# without a logout/login. (lsregister alone does not refresh the TIS database.)
xcrun swift - "$DEST/Funput.app" <<'SWIFT' 2>/dev/null || true
import Carbon
import Foundation
let url = URL(fileURLWithPath: CommandLine.arguments[1]) as CFURL
exit(TISRegisterInputSource(url) == noErr ? 0 : 1)
SWIFT

# Launch so the IMKServer comes up; it then runs as a background agent.
open "$DEST/Funput.app"

echo "Done. If this is the first install, add Funput in"
echo "  System Settings → Keyboard → Input Sources → + → Vietnamese → Funput"
