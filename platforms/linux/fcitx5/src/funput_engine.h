// Fcitx5 input method addon for Funput. Drives `funput-engine` (via the C ABI)
// using Fcitx5's preedit/commit model — the same shape as the macOS IMKit shell
// (platforms/macos/.../FunputInputController.swift), NOT the Windows backspace-
// injection path. The composing word is shown as underlined preedit and committed
// on a word boundary, navigation key, focus change, or VI/EN toggle.

#ifndef FUNPUT_ENGINE_H
#define FUNPUT_ENGINE_H

#include <fcitx/addonfactory.h>
#include <fcitx/addonmanager.h>
#include <fcitx/inputcontext.h>
#include <fcitx/inputmethodengine.h>
#include <fcitx/instance.h>
#include <fcitx-utils/key.h>

#include "ffi_handle.h"
#include "settings.h"

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

    fcitx::Instance *instance_;
    funput::Handle handle_;
    funput::Settings settings_;
};

class FunputEngineFactory : public fcitx::AddonFactory {
public:
    fcitx::AddonInstance *create(fcitx::AddonManager *manager) override {
        return new FunputEngine(manager->instance());
    }
};

#endif // FUNPUT_ENGINE_H
