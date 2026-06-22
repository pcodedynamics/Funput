// Shared settings bridge. The Tauri Settings app and this addon are separate
// processes, so they sync through ~/.config/Funput/settings.json (the same file
// the Windows shell writes via `dirs::config_dir()`). The addon reads it on
// startup and reloads when the file's mtime changes (cheap, checked on focus-in).

#ifndef FUNPUT_SETTINGS_H
#define FUNPUT_SETTINGS_H

#include <cstdint>
#include <string>
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
    Hotkey toggleHotkey = Hotkey::CtrlBacktick;
    // App identifiers (fcitx5 program() / WM_CLASS) that default to English on
    // focus. Owned by the Settings UI; the addon only reads them for matching.
    std::vector<std::string> excludedAppIds;

    // Whether `program` (fcitx5 InputContext::program()) is on the exclusion list.
    bool isExcluded(const std::string &program) const;

    // Absolute path to ~/.config/Funput/settings.json (XDG-aware).
    static std::string path();

    // Re-read from disk only if the file changed since last load. Returns true if
    // values changed. On missing/corrupt file, keeps current (default) values.
    bool reloadIfChanged();

    // Persist the current values (used when the VI/EN hotkey toggles `enabled`).
    void save() const;

private:
    int64_t lastMtime_ = -1;
};

} // namespace funput

#endif // FUNPUT_SETTINGS_H
