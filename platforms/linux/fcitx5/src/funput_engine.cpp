#include "funput_engine.h"

#include <fstream>
#include <sstream>

#include <fcitx/inputpanel.h>
#include <fcitx/text.h>
#include <fcitx/userinterface.h>
#include <fcitx-utils/keysym.h>
#include <nlohmann/json.hpp>

namespace {

using json = nlohmann::json;

// ~/.config/Funput/recent-apps.json — derived from the settings.json path so it
// lands in the same XDG config dir.
std::string recentAppsPath() {
    std::string p = funput::Settings::path();
    const std::string from = "settings.json";
    if (auto pos = p.rfind(from); pos != std::string::npos) {
        p.replace(pos, from.size(), "recent-apps.json");
        return p;
    }
    return {};
}

// Word boundary — ASCII whitespace or punctuation, digits excluded (VNI uses
// them as tone modifiers). Mirrors funput_core's rule and the macOS shell.
bool isBoundary(char32_t s) {
    if (s > 0x7F) return false;
    if (s == U' ' || s == U'\t' || s == U'\n' || s == U'\r') return true;
    const uint32_t v = static_cast<uint32_t>(s);
    return (v >= 0x21 && v <= 0x2F) || (v >= 0x3A && v <= 0x40) ||
           (v >= 0x5B && v <= 0x60) || (v >= 0x7B && v <= 0x7E);
}

} // namespace

FunputEngine::FunputEngine(fcitx::Instance *instance) : instance_(instance) {
    // Apply defaults now; activate() reloads the real values from disk on focus-in.
    applySettings();
}

void FunputEngine::applySettings() {
    handle_.setMethod(static_cast<uint8_t>(settings_.method));
    handle_.setToneStyle(static_cast<uint8_t>(settings_.toneStyle));
    effectiveEnabled_ = settings_.enabled; // baseline; activate() refines it per-app
    handle_.setEnabled(effectiveEnabled_);
    handle_.setSmartRestore(settings_.smartRestore);
    handle_.setEagerRestore(settings_.eagerRestore);
    handle_.clear();
}

// excluded app → English; any other app → Vietnamese. No-op when the list is empty
// (keeps the plain global toggle for users who don't use the feature).
void FunputEngine::applyPerAppDefault(const std::string &program) {
    const bool eff = settings_.excludedAppIds.empty()
                         ? settings_.enabled
                         : !settings_.isExcluded(program);
    if (eff == effectiveEnabled_) return;
    effectiveEnabled_ = eff;
    handle_.setEnabled(eff);
    handle_.clear();
}

// Append a newly-seen app to ~/.config/Funput/recent-apps.json (capped, deduped).
// Only writes when the program is new, so it doesn't churn on every focus change.
void FunputEngine::noteRecentApp(const std::string &program) {
    if (program.empty()) return;
    const std::string p = recentAppsPath();
    if (p.empty()) return;

    json arr = json::array();
    if (std::ifstream in(p); in) {
        std::stringstream ss;
        ss << in.rdbuf();
        json parsed = json::parse(ss.str(), nullptr, false);
        if (parsed.is_array()) arr = std::move(parsed);
    }
    for (const auto &e : arr) {
        if (e.is_object() && e.value("id", std::string()) == program) return; // already known
    }
    arr.insert(arr.begin(), json{{"id", program}, {"name", program}});
    if (arr.size() > 12) arr.erase(arr.begin() + 12, arr.end());

    std::ofstream out(p, std::ios::trunc);
    if (out) out << arr.dump(2);
}

void FunputEngine::updatePreedit(fcitx::InputContext *ic) {
    const std::string s = handle_.buffer();
    fcitx::Text preedit;
    if (!s.empty()) preedit.append(s, fcitx::TextFormatFlag::Underline);
    preedit.setCursor(static_cast<int>(s.size()));

    auto &panel = ic->inputPanel();
    if (ic->capabilityFlags().test(fcitx::CapabilityFlag::Preedit)) {
        panel.setClientPreedit(preedit);
    } else {
        panel.setPreedit(preedit);
    }
    ic->updatePreedit();
    ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
}

void FunputEngine::clearPreedit(fcitx::InputContext *ic) {
    ic->inputPanel().reset();
    ic->updatePreedit();
    ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
}

void FunputEngine::commitBuffer(fcitx::InputContext *ic) {
    const std::string s = handle_.buffer();
    if (!s.empty()) ic->commitString(s);
    handle_.clear();
    clearPreedit(ic);
}

// Boundary key (space / punctuation) while composing. The engine decides
// English-restore, then clears the session: on restore it returns ACTION_SEND
// with output = rawKeys + boundaryChar; otherwise we keep the composed buffer.
bool FunputEngine::handleBoundary(fcitx::InputContext *ic, char32_t scalar) {
    const std::string pre = handle_.buffer();
    if (pre.empty()) return false; // not composing → let the app handle the key

    const FunputResult r = handle_.process(static_cast<uint32_t>(scalar));
    std::string word;
    if (r.action == ACTION_SEND) {
        word = funput::Handle::output(r);
        if (!word.empty()) word.pop_back(); // drop the trailing boundary char (ASCII)
    } else {
        word = pre;
    }

    std::string boundary;
    funput::appendUtf8(boundary, static_cast<uint32_t>(scalar));
    ic->commitString(word + boundary);
    handle_.clear();
    clearPreedit(ic);
    return true;
}

bool FunputEngine::matchesToggle(const fcitx::Key &key) const {
    const auto st = key.states();
    switch (settings_.toggleHotkey) {
    case funput::Hotkey::CtrlBacktick:
        return st.test(fcitx::KeyState::Ctrl) && key.sym() == FcitxKey_grave;
    case funput::Hotkey::CtrlSpace:
        return st.test(fcitx::KeyState::Ctrl) && key.sym() == FcitxKey_space;
    case funput::Hotkey::AltShift:
        return false; // modifier-only combo; not supported in v1
    }
    return false;
}

void FunputEngine::toggleEnabled(fcitx::InputContext *ic) {
    commitBuffer(ic); // commit any in-progress word first
    effectiveEnabled_ = !effectiveEnabled_;
    settings_.enabled = effectiveEnabled_; // persist baseline; holds until next focus
    handle_.setEnabled(effectiveEnabled_);
    settings_.save(); // persist so the Settings UI reflects the new state
}

void FunputEngine::keyEvent(const fcitx::InputMethodEntry &, fcitx::KeyEvent &keyEvent) {
    if (keyEvent.isRelease()) return;

    auto *ic = keyEvent.inputContext();
    const fcitx::Key key = keyEvent.key();

    if (matchesToggle(key)) {
        toggleEnabled(ic);
        keyEvent.filterAndAccept();
        return;
    }

    if (!effectiveEnabled_) return; // English mode: pass everything through

    // Keyboard shortcuts (Ctrl/Alt/Super combos) are not text: end composition and
    // let the app handle them.
    const auto st = key.states();
    if (st.test(fcitx::KeyState::Ctrl) || st.test(fcitx::KeyState::Alt) ||
        st.test(fcitx::KeyState::Super)) {
        commitBuffer(ic);
        return;
    }

    // Backspace inside the composition: drop the last composed char.
    if (key.sym() == FcitxKey_BackSpace) {
        if (!handle_.buffer().empty()) {
            handle_.backspace();
            updatePreedit(ic);
            keyEvent.filterAndAccept();
        }
        return; // not composing → pass through so the app deletes its own char
    }

    // Navigation / function / editing keys (arrows, Home/End, Esc, Enter, Tab, …)
    // carry no Unicode value — end composition and let the app handle them.
    const uint32_t uc = fcitx::Key::keySymToUnicode(key.sym());
    if (uc == 0) {
        commitBuffer(ic);
        return;
    }

    const char32_t scalar = static_cast<char32_t>(uc);
    if (isBoundary(scalar)) {
        if (handleBoundary(ic, scalar)) keyEvent.filterAndAccept();
        return;
    }

    // Compose: feed the scalar and show the updated buffer as underlined preedit.
    handle_.process(static_cast<uint32_t>(scalar));
    updatePreedit(ic);
    keyEvent.filterAndAccept();
}

void FunputEngine::reset(const fcitx::InputMethodEntry &, fcitx::InputContextEvent &event) {
    handle_.clear();
    clearPreedit(event.inputContext());
}

void FunputEngine::activate(const fcitx::InputMethodEntry &, fcitx::InputContextEvent &event) {
    // Pick up settings changed by the Tauri Settings app (cheap mtime check).
    if (settings_.reloadIfChanged()) applySettings();
    // Per-app auto-switch on focus-in, mirroring the macOS shell's activateServer.
    const std::string program = event.inputContext()->program();
    applyPerAppDefault(program);
    noteRecentApp(program);
}

void FunputEngine::deactivate(const fcitx::InputMethodEntry &, fcitx::InputContextEvent &event) {
    commitBuffer(event.inputContext());
}

FCITX_ADDON_FACTORY(FunputEngineFactory)
