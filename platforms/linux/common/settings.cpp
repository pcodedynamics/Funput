#include "settings.h"

#include <cstdlib>
#include <fstream>
#include <sstream>
#include <sys/stat.h>

#include <nlohmann/json.hpp>

namespace funput {

using json = nlohmann::json;

std::string Settings::path() {
    // Mirror dirs::config_dir(): $XDG_CONFIG_HOME or ~/.config, then /Funput.
    const char *xdg = std::getenv("XDG_CONFIG_HOME");
    std::string base;
    if (xdg && *xdg) {
        base = xdg;
    } else if (const char *home = std::getenv("HOME"); home && *home) {
        base = std::string(home) + "/.config";
    } else {
        return {};
    }
    return base + "/Funput/settings.json";
}

static Method parseMethod(const std::string &s) {
    return s == "telex" ? Method::Telex : Method::Vni;
}

static const char *methodStr(Method m) {
    return m == Method::Telex ? "telex" : "vni";
}

static ToneStyle parseToneStyle(const std::string &s) {
    return s == "modern" ? ToneStyle::Modern : ToneStyle::Traditional;
}

static const char *toneStyleStr(ToneStyle t) {
    return t == ToneStyle::Modern ? "modern" : "traditional";
}

static Hotkey parseHotkey(const std::string &s) {
    if (s == "ctrl_space") return Hotkey::CtrlSpace;
    if (s == "alt_shift") return Hotkey::AltShift;
    return Hotkey::CtrlBacktick;
}

static const char *hotkeyStr(Hotkey h) {
    switch (h) {
    case Hotkey::CtrlSpace: return "ctrl_space";
    case Hotkey::AltShift: return "alt_shift";
    case Hotkey::CtrlBacktick:
    default: return "ctrl_backtick";
    }
}

static FlipHotkey parseFlipHotkey(const std::string &s) {
    if (s == "ctrl_shift_z") return FlipHotkey::CtrlShiftZ;
    if (s == "ctrl_shift_x") return FlipHotkey::CtrlShiftX;
    return FlipHotkey::Off;
}

static const char *flipHotkeyStr(FlipHotkey h) {
    switch (h) {
    case FlipHotkey::CtrlShiftZ: return "ctrl_shift_z";
    case FlipHotkey::CtrlShiftX: return "ctrl_shift_x";
    case FlipHotkey::Off:
    default: return "off";
    }
}

bool Settings::reloadIfChanged() {
    const std::string p = path();
    if (p.empty()) return false;

    struct stat st {};
    if (::stat(p.c_str(), &st) != 0) return false; // missing → keep current values
    if (static_cast<int64_t>(st.st_mtime) == lastMtime_) return false;
    return reload();
}

bool Settings::reload() {
    const std::string p = path();
    if (p.empty()) return false;

    struct stat st {};
    if (::stat(p.c_str(), &st) != 0) return false; // missing → keep current values
    lastMtime_ = static_cast<int64_t>(st.st_mtime);

    std::ifstream f(p);
    if (!f) return false;
    std::stringstream ss;
    ss << f.rdbuf();

    json j = json::parse(ss.str(), nullptr, /*allow_exceptions=*/false);
    if (j.is_discarded() || !j.is_object()) return false;

    const Settings prev = *this;
    method = parseMethod(j.value("method", std::string(methodStr(method))));
    toneStyle = parseToneStyle(j.value("toneStyle", std::string(toneStyleStr(toneStyle))));
    enabled = j.value("enabled", enabled);
    smartRestore = j.value("smartRestore", smartRestore);
    eagerRestore = j.value("eagerRestore", eagerRestore);
    spellCheck = j.value("spellCheck", spellCheck);
    autoCapitalize = j.value("autoCapitalize", autoCapitalize);
    toggleHotkey = parseHotkey(j.value("toggleHotkey", std::string(hotkeyStr(toggleHotkey))));
    flipHotkey = parseFlipHotkey(j.value("flipHotkey", std::string(flipHotkeyStr(flipHotkey))));

    // excludedApps: [{ "id": "code", "name": "Code" }, ...] — keep just the ids.
    excludedAppIds.clear();
    if (auto it = j.find("excludedApps"); it != j.end() && it->is_array()) {
        for (const auto &app : *it) {
            if (app.is_object() && app.contains("id") && app["id"].is_string()) {
                excludedAppIds.push_back(app["id"].get<std::string>());
            }
        }
    }

    // shortcuts: [{ "trigger": "vn", "expansion": "Việt Nam" }, ...] — gõ tắt.
    shortcuts.clear();
    if (auto it = j.find("shortcuts"); it != j.end() && it->is_array()) {
        for (const auto &sc : *it) {
            if (sc.is_object() && sc.contains("trigger") && sc["trigger"].is_string() &&
                sc.contains("expansion") && sc["expansion"].is_string()) {
                shortcuts.emplace_back(sc["trigger"].get<std::string>(),
                                       sc["expansion"].get<std::string>());
            }
        }
    }

    return method != prev.method || toneStyle != prev.toneStyle ||
           enabled != prev.enabled || smartRestore != prev.smartRestore ||
           eagerRestore != prev.eagerRestore || spellCheck != prev.spellCheck ||
           autoCapitalize != prev.autoCapitalize ||
           toggleHotkey != prev.toggleHotkey || flipHotkey != prev.flipHotkey ||
           excludedAppIds != prev.excludedAppIds || shortcuts != prev.shortcuts;
}

bool Settings::isExcluded(const std::string &program) const {
    if (program.empty()) return false;
    for (const auto &id : excludedAppIds) {
        if (id == program) return true;
    }
    return false;
}

void Settings::save() const {
    const std::string p = path();
    if (p.empty()) return;

    // Preserve any keys we don't own (launchAtLogin, hasCompletedOnboarding) by
    // merging into the existing file rather than overwriting it wholesale.
    json j = json::object();
    if (std::ifstream in(p); in) {
        std::stringstream ss;
        ss << in.rdbuf();
        json existing = json::parse(ss.str(), nullptr, false);
        if (existing.is_object()) j = std::move(existing);
    }
    j["method"] = methodStr(method);
    j["toneStyle"] = toneStyleStr(toneStyle);
    j["enabled"] = enabled;
    j["smartRestore"] = smartRestore;
    j["eagerRestore"] = eagerRestore;
    j["spellCheck"] = spellCheck;
    j["autoCapitalize"] = autoCapitalize;
    j["toggleHotkey"] = hotkeyStr(toggleHotkey);
    j["flipHotkey"] = flipHotkeyStr(flipHotkey);

    std::ofstream out(p, std::ios::trunc);
    if (out) out << j.dump(2);
}

} // namespace funput
