// Shared settings bridge. The Settings app and this addon are separate processes,
// so they sync through ~/.config/Funput/settings.json (the same file the Windows
// shell writes via `dirs::config_dir()`). The addon reads it on startup, reloads
// live when a file watcher (SettingsWatcher) fires, and also re-checks the mtime on
// focus-in as a fallback.

#ifndef FUNPUT_SETTINGS_H
#define FUNPUT_SETTINGS_H

#include <cstdint>
#include <string>
#include <utility>
#include <vector>

namespace funput {

enum class Method : uint8_t { Telex = 0, Vni = 1 };
enum class ToneStyle : uint8_t { Traditional = 0, Modern = 1 };
enum class Hotkey { CtrlBacktick, CtrlSpace, AltShift };

struct Settings {
    Method method = Method::Vni;
    ToneStyle toneStyle = ToneStyle::Traditional;
    bool enabled = true;
    bool smartRestore = true;
    bool eagerRestore = true;
    // Spell-check ("Kiểm tra chính tả"): only place a diacritic that forms a valid
    // Vietnamese syllable. Off by default.
    bool spellCheck = false;
    // Auto-capitalize ("Tự động viết hoa"): uppercase the first letter at the start of
    // a sentence. Off by default.
    bool autoCapitalize = false;
    Hotkey toggleHotkey = Hotkey::CtrlBacktick;
    // App identifiers (fcitx5 program() / WM_CLASS) that default to English on
    // focus. Owned by the Settings UI; the addon only reads them for matching.
    std::vector<std::string> excludedAppIds;
    // Text-expansion shortcuts (gõ tắt): (trigger, expansion) pairs. Owned by the
    // Settings UI; the addon only reads them and pushes them into the engine.
    std::vector<std::pair<std::string, std::string>> shortcuts;

    // Whether `program` (fcitx5 InputContext::program()) is on the exclusion list.
    bool isExcluded(const std::string &program) const;

    // Absolute path to ~/.config/Funput/settings.json (XDG-aware).
    static std::string path();

    // Re-read from disk only if the mtime changed since last load. Returns true if
    // values changed. On missing/corrupt file, keeps current (default) values.
    bool reloadIfChanged();

    // Force a re-read regardless of mtime (used by the file watcher: inotify already
    // told us the file changed, and st_mtime's 1-second resolution would otherwise
    // miss two writes within the same second). Returns true if values changed.
    bool reload();

    // Persist the current values (used when the VI/EN hotkey toggles `enabled`).
    void save() const;

private:
    int64_t lastMtime_ = -1;
};

} // namespace funput

#endif // FUNPUT_SETTINGS_H
