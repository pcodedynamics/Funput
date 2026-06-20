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

bool Settings::reloadIfChanged() {
    const std::string p = path();
    if (p.empty()) return false;

    struct stat st {};
    if (::stat(p.c_str(), &st) != 0) return false; // missing → keep current values
    const auto mtime = static_cast<int64_t>(st.st_mtime);
    if (mtime == lastMtime_) return false;
    lastMtime_ = mtime;

    std::ifstream f(p);
    if (!f) return false;
    std::stringstream ss;
    ss << f.rdbuf();

    json j = json::parse(ss.str(), nullptr, /*allow_exceptions=*/false);
    if (j.is_discarded() || !j.is_object()) return false;

    const Settings prev = *this;
    method = parseMethod(j.value("method", std::string(methodStr(method))));
    enabled = j.value("enabled", enabled);
    smartRestore = j.value("smartRestore", smartRestore);
    eagerRestore = j.value("eagerRestore", eagerRestore);
    toggleHotkey = parseHotkey(j.value("toggleHotkey", std::string(hotkeyStr(toggleHotkey))));

    return method != prev.method || enabled != prev.enabled ||
           smartRestore != prev.smartRestore || eagerRestore != prev.eagerRestore ||
           toggleHotkey != prev.toggleHotkey;
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
    j["enabled"] = enabled;
    j["smartRestore"] = smartRestore;
    j["eagerRestore"] = eagerRestore;
    j["toggleHotkey"] = hotkeyStr(toggleHotkey);

    std::ofstream out(p, std::ios::trunc);
    if (out) out << j.dump(2);
}

} // namespace funput
