#include "engine.h"

#include <string>

#include <glib-unix.h>

#include "boundary.h"       // from platforms/linux/common (via funput_linux_common)
#include "ffi_handle.h"     // from platforms/linux/common (via funput_linux_common)
#include "settings.h"       // from platforms/linux/common
#include "settings_watch.h" // from platforms/linux/common

namespace {

// Per-engine C++ state. GObject instances are C structs, so we heap-allocate this
// in instance_init and delete it in finalize, holding only a pointer in the struct.
struct EngineState {
    funput::Handle handle_;
    funput::Settings settings_;
    // Runtime VI/EN actually in effect. Starts from settings_.enabled; the toggle
    // overrides it. (Per-app auto-switch is intentionally not in the IBus v1 shell —
    // IBus exposes no reliable focused-app id, especially on Wayland.)
    bool effectiveEnabled_ = true;
    // Live settings reload: an inotify fd watched on the GLib main loop, so changes
    // from the Settings app apply immediately (not just on the next focus-in).
    funput::SettingsWatcher settingsWatcher_;
    guint settingsSource_ = 0; // g_unix_fd_add id; removed in finalize
};

} // namespace

// --- GObject type ----------------------------------------------------------
struct _IBusFunputEngine {
    IBusEngine parent;
    EngineState *state;
};

struct _IBusFunputEngineClass {
    IBusEngineClass parent;
};

G_DEFINE_TYPE(IBusFunputEngine, ibus_funput_engine, IBUS_TYPE_ENGINE)

// --- Helpers (operate on the C++ state) ------------------------------------
namespace {

// Push settings_ into the engine. Called on construct and after a settings reload.
void applySettings(EngineState *st) {
    st->handle_.setMethod(static_cast<uint8_t>(st->settings_.method));
    st->handle_.setToneStyle(static_cast<uint8_t>(st->settings_.toneStyle));
    st->effectiveEnabled_ = st->settings_.enabled;
    st->handle_.setEnabled(st->effectiveEnabled_);
    st->handle_.setSmartRestore(st->settings_.smartRestore);
    st->handle_.setEagerRestore(st->settings_.eagerRestore);
    st->handle_.clear();
}

void clearPreedit(IBusEngine *engine) {
    ibus_engine_hide_preedit_text(engine);
}

// Show buffer() as preedit, cursor at the end. We don't set any underline
// attribute: how preedit is drawn is up to the client app, which renders its
// default composing style (typically underlined). Trying to override it isn't
// reliable since many clients ignore the attribute.
void updatePreedit(IBusEngine *engine, EngineState *st) {
    const std::string s = st->handle_.buffer();
    IBusText *text = ibus_text_new_from_string(s.c_str());
    const glong len = g_utf8_strlen(s.c_str(), -1);
    // ibus_engine_update_preedit_text sinks the floating ref on `text`.
    ibus_engine_update_preedit_text(engine, text, static_cast<guint>(len),
                                    s.empty() ? FALSE : TRUE);
}

// Commit buffer() and end composition.
void commitBuffer(IBusEngine *engine, EngineState *st) {
    const std::string s = st->handle_.buffer();
    if (!s.empty()) {
        ibus_engine_commit_text(engine, ibus_text_new_from_string(s.c_str()));
    }
    st->handle_.clear();
    clearPreedit(engine);
}

// Boundary key (space / punctuation) while composing. The engine decides
// English-restore, then we clear the session: on restore it returns ACTION_SEND
// with output = rawKeys + boundaryChar; otherwise we keep the composed buffer.
bool handleBoundary(IBusEngine *engine, EngineState *st, char32_t scalar) {
    const std::string pre = st->handle_.buffer();
    if (pre.empty()) return false; // not composing → let the app handle the key

    const FunputResult r = st->handle_.process(static_cast<uint32_t>(scalar));
    std::string word;
    if (r.action == ACTION_SEND) {
        word = funput::Handle::output(r);
        if (!word.empty()) word.pop_back(); // drop the trailing boundary char (ASCII)
    } else {
        word = pre;
    }

    std::string boundary;
    funput::appendUtf8(boundary, static_cast<uint32_t>(scalar));
    const std::string full = word + boundary;
    ibus_engine_commit_text(engine, ibus_text_new_from_string(full.c_str()));
    st->handle_.clear();
    clearPreedit(engine);
    return true;
}

bool matchesToggle(EngineState *st, guint keyval, guint modifiers) {
    const bool ctrl = (modifiers & IBUS_CONTROL_MASK) != 0;
    switch (st->settings_.toggleHotkey) {
    case funput::Hotkey::CtrlBacktick:
        return ctrl && keyval == IBUS_KEY_grave;
    case funput::Hotkey::CtrlSpace:
        return ctrl && keyval == IBUS_KEY_space;
    case funput::Hotkey::AltShift:
        return false; // modifier-only combo; not supported in v1
    }
    return false;
}

void toggleEnabled(IBusEngine *engine, EngineState *st) {
    commitBuffer(engine, st); // commit any in-progress word first
    st->effectiveEnabled_ = !st->effectiveEnabled_;
    st->settings_.enabled = st->effectiveEnabled_; // persist baseline
    st->handle_.setEnabled(st->effectiveEnabled_);
    st->settings_.save(); // persist so the Settings UI reflects the new state
}

} // namespace

// Live settings reload: the inotify fd became readable (the Settings app rewrote
// settings.json). Force a re-read — inotify already signalled the change and
// st_mtime's 1-second resolution would otherwise miss rapid edits.
static gboolean funput_on_settings_fd(gint /*fd*/, GIOCondition /*cond*/, gpointer user_data) {
    EngineState *st = FUNPUT_ENGINE(user_data)->state;
    if (st->settingsWatcher_.drain() && st->settings_.reload()) {
        applySettings(st);
    }
    return G_SOURCE_CONTINUE;
}

// --- IBusEngine vfuncs -----------------------------------------------------
static gboolean ibus_funput_engine_process_key_event(IBusEngine *engine, guint keyval,
                                                 guint /*keycode*/, guint modifiers) {
    EngineState *st = FUNPUT_ENGINE(engine)->state;

    if (modifiers & IBUS_RELEASE_MASK) return FALSE; // ignore key release

    if (matchesToggle(st, keyval, modifiers)) {
        toggleEnabled(engine, st);
        return TRUE;
    }

    if (!st->effectiveEnabled_) return FALSE; // English mode: pass everything through

    // Keyboard shortcuts (Ctrl/Alt/Super combos) are not text: end composition and
    // let the app handle them.
    if (modifiers & (IBUS_CONTROL_MASK | IBUS_MOD1_MASK | IBUS_SUPER_MASK)) {
        commitBuffer(engine, st);
        return FALSE;
    }

    // Backspace inside the composition: drop the last composed char.
    if (keyval == IBUS_KEY_BackSpace) {
        if (!st->handle_.buffer().empty()) {
            st->handle_.backspace();
            updatePreedit(engine, st);
            return TRUE;
        }
        return FALSE; // not composing → pass through so the app deletes its own char
    }

    // Navigation / function / editing keys (arrows, Home/End, Esc, Enter, Tab, …)
    // carry no Unicode value — end composition and let the app handle them.
    const guint32 uc = ibus_keyval_to_unicode(keyval);
    if (uc == 0) {
        commitBuffer(engine, st);
        return FALSE;
    }

    const char32_t scalar = static_cast<char32_t>(uc);
    if (funput::isBoundary(scalar)) {
        return handleBoundary(engine, st, scalar) ? TRUE : FALSE;
    }

    // Compose: feed the scalar and show the updated buffer as preedit.
    st->handle_.process(static_cast<uint32_t>(scalar));
    updatePreedit(engine, st);
    return TRUE;
}

// Focus-in / enable: fallback mtime check for settings changed while unfocused (the
// live watcher applies changes during focus). No per-app auto-switch in the IBus v1
// shell (see EngineState).
static void ibus_funput_engine_focus_in(IBusEngine *engine) {
    EngineState *st = FUNPUT_ENGINE(engine)->state;
    if (st->settings_.reloadIfChanged()) applySettings(st);
}

static void ibus_funput_engine_enable(IBusEngine *engine) {
    EngineState *st = FUNPUT_ENGINE(engine)->state;
    if (st->settings_.reloadIfChanged()) applySettings(st);
}

static void ibus_funput_engine_focus_out(IBusEngine *engine) {
    commitBuffer(engine, FUNPUT_ENGINE(engine)->state);
}

static void ibus_funput_engine_disable(IBusEngine *engine) {
    commitBuffer(engine, FUNPUT_ENGINE(engine)->state);
}

static void ibus_funput_engine_reset(IBusEngine *engine) {
    EngineState *st = FUNPUT_ENGINE(engine)->state;
    st->handle_.clear();
    clearPreedit(engine);
}

// --- Lifecycle -------------------------------------------------------------
static void ibus_funput_engine_init(IBusFunputEngine *self) {
    self->state = new EngineState();
    applySettings(self->state); // focus_in reloads the real values from disk

    // Watch settings.json on the GLib main loop so Settings-app changes apply live.
    if (self->state->settingsWatcher_.fd() >= 0) {
        self->state->settingsSource_ = g_unix_fd_add(
            self->state->settingsWatcher_.fd(), G_IO_IN, funput_on_settings_fd, self);
    }
}

static void ibus_funput_engine_finalize(GObject *object) {
    IBusFunputEngine *self = FUNPUT_ENGINE(object);
    if (self->state && self->state->settingsSource_ != 0) {
        g_source_remove(self->state->settingsSource_);
        self->state->settingsSource_ = 0;
    }
    delete self->state;
    self->state = nullptr;
    G_OBJECT_CLASS(ibus_funput_engine_parent_class)->finalize(object);
}

static void ibus_funput_engine_class_init(IBusFunputEngineClass *klass) {
    GObjectClass *object_class = G_OBJECT_CLASS(klass);
    IBusEngineClass *engine_class = IBUS_ENGINE_CLASS(klass);

    object_class->finalize = ibus_funput_engine_finalize;

    engine_class->process_key_event = ibus_funput_engine_process_key_event;
    engine_class->focus_in = ibus_funput_engine_focus_in;
    engine_class->focus_out = ibus_funput_engine_focus_out;
    engine_class->enable = ibus_funput_engine_enable;
    engine_class->disable = ibus_funput_engine_disable;
    engine_class->reset = ibus_funput_engine_reset;
}
