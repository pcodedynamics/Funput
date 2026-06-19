#!/bin/sh
# Install the signed Funput.app that ships beside this script on the mounted DMG.
# Double-click this file in Finder to install - no Terminal needed.
#
# Unlike scripts/install.sh, this does NOT build; it copies the notarized bundle
# into ~/Library/Input Methods and registers it as an input source.
set -eu

HERE="$(cd "$(dirname "$0")" && pwd)"
APP="$HERE/Funput.app"
DEST="$HOME/Library/Input Methods"

if [ ! -d "$APP" ]; then
    echo "Funput.app not found next to this installer ($APP)." >&2
    exit 1
fi

echo "Installing Funput to ${DEST} ..."
mkdir -p "$DEST"
# Quit a running instance so the bundle can be replaced and reloaded.
killall Funput 2>/dev/null || true
rm -rf "$DEST/Funput.app"
cp -R "$APP" "$DEST/Funput.app"

# LaunchServices keys input sources by bundle id; refresh its record.
LSR="/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister"
"$LSR" -f "$DEST/Funput.app"

# Register with Text Input Sources so Funput appears without a logout/login.
xcrun swift - "$DEST/Funput.app" <<'SWIFT' 2>/dev/null || true
import Carbon
import Foundation
let url = URL(fileURLWithPath: CommandLine.arguments[1]) as CFURL
exit(TISRegisterInputSource(url) == noErr ? 0 : 1)
SWIFT

# Launch so the IMKServer comes up; it then runs as a background agent.
open "$DEST/Funput.app"

echo ""
echo "Done. Enable Funput in:"
echo "  System Settings > Keyboard > Input Sources > + > Vietnamese > Funput"
echo "(If it does not appear yet, log out and back in once.)"
echo ""
echo "Press Return to close this window."
read -r _
