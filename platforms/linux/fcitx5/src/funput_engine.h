// Fcitx5 input method addon for Funput. Drives `funput-engine` (via the C ABI)
// using Fcitx5's preedit/commit model — the same shape as the macOS IMKit shell
// (platforms/macos/.../FunputInputController.swift), NOT the Windows backspace-
// injection path. The composing word is shown as underlined preedit and committed
// on a word boundary, navigation key, focus change, or VI/EN toggle.

#ifndef FUNPUT_ENGINE_H
#define FUNPUT_ENGINE_H

#include <memory>
#include <string>

#include <fcitx/addonfactory.h>
#include <fcitx/addonmanager.h>
#include <fcitx/inputcontext.h>
#include <fcitx/inputmethodengine.h>
#include <fcitx/instance.h>
#include <fcitx-utils/event.h>
#include <fcitx-utils/key.h>

#include "ffi_handle.h"
#include "settings.h"
#include "settings_watch.h"

class FunputEngine : public fcitx::InputMethodEngineV2 {
public:
    explicit FunputEngine(fcitx::Instance *instance);

    void keyEvent(const fcitx::InputMethodEntry &entry, fcitx::KeyEvent &keyEvent) override;
    void reset(const fcitx::InputMethodEntry &entry, fcitx::InputContextEvent &event) override;
    void activate(const fcitx::InputMethodEntry &entry, fcitx::InputContextEvent &event) override;
    void deactivate(const fcitx::InputMethodEntry &entry, fcitx::InputContextEvent &event) override;

private:
    void applySettings();                         // push settings_ into the engine
    void updatePreedit(fcitx::InputContext *ic);  // show buffer() as underlined preedit
    void clearPreedit(fcitx::InputContext *ic);
    void commitBuffer(fcitx::InputContext *ic);   // commit buffer(), end composition
    bool handleBoundary(fcitx::InputContext *ic, char32_t scalar);
    bool matchesToggle(const fcitx::Key &key) const;
    void toggleEnabled(fcitx::InputContext *ic);
    // Per-app auto-switch (mirrors the macOS shell): excluded apps default to
    // English on focus, every other app to Vietnamese. No-op when the list is empty.
    void applyPerAppDefault(const std::string &program);
    void noteRecentApp(const std::string &program); // record for the Settings picker
    // Reload settings live when the watcher fires (Settings app wrote the file), and
    // re-apply the per-app default for the currently-focused app.
    void onSettingsChanged();

    fcitx::Instance *instance_;
    funput::Handle handle_;
    funput::Settings settings_;
    // Runtime VI/EN actually in effect. Starts from settings_.enabled but is driven
    // per-app on focus; the toggle overrides it until the next focus change.
    bool effectiveEnabled_ = true;
    // Program() of the most recently focused app, so a live settings reload can
    // re-apply the per-app default without waiting for the next focus-in.
    std::string lastProgram_;
    // Live settings reload: an inotify fd (settingsWatcher_) wired into Fcitx5's
    // event loop (settingsWatch_).
    funput::SettingsWatcher settingsWatcher_;
    std::unique_ptr<fcitx::EventSourceIO> settingsWatch_;
};

class FunputEngineFactory : public fcitx::AddonFactory {
public:
    fcitx::AddonInstance *create(fcitx::AddonManager *manager) override {
        return new FunputEngine(manager->instance());
    }
};

#endif // FUNPUT_ENGINE_H
