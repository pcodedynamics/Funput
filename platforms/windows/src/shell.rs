//! Global engine + settings state shared between the keyboard-hook thread, the
//! tray, and the UI callbacks. The hook callback is a bare `extern "system"`
//! function with no user pointer, so this lives in a process-global behind a mutex.
//! No Windows APIs here.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use funput_core::{InputMethod, ToneStyle as CoreToneStyle};
use funput_engine::{Engine, ImeResult};

use crate::settings::{ExcludedApp, Hotkey, Method, Settings, ToneStyle};

/// Tag stamped into `dwExtraInfo` of every event we synthesize via `SendInput`, so
/// the hook can recognize and ignore its own injected keystrokes (no re-entrancy).
pub const INJECT_TAG: usize = 0x4655_4E50; // "FUNP"

/// How many recently-focused apps to keep for the Settings "recent apps" picker.
const RECENT_CAP: usize = 12;

struct Shell {
    engine: Engine,
    settings: Settings,
    /// Recently-focused apps (most recent first), fed by the foreground hook. Not
    /// persisted — it's just a convenience source for the Settings UI.
    recent: Vec<ExcludedApp>,
    /// Per-app manual VI/EN overrides (runtime only, not persisted). A manual
    /// toggle records the user's choice here so the per-app auto-switch honours it
    /// on the next focus change instead of reverting to the exclusion-list default.
    /// Keyed by the lowercased exe id (e.g. "code.exe").
    overrides: HashMap<String, bool>,
    /// A manual toggle whose target app isn't known yet. The tray and the Settings
    /// toggle steal foreground (the taskbar/Settings window becomes foreground), so
    /// the choice is parked here and bound to the next app the user focuses.
    pending_override: Option<bool>,
}

static SHELL: OnceLock<Mutex<Shell>> = OnceLock::new();

fn apply_to_engine(engine: &mut Engine, s: &Settings) {
    engine.set_method(s.method.core());
    engine.set_tone_style(s.tone_style.core());
    engine.set_enabled(s.enabled);
    engine.set_smart_restore(s.smart_restore);
    engine.set_eager_restore(s.eager_restore);
    engine.clear();
}

fn shell() -> &'static Mutex<Shell> {
    SHELL.get_or_init(|| {
        let settings = Settings::load();
        let mut engine = Engine::new();
        apply_to_engine(&mut engine, &settings);
        Mutex::new(Shell {
            engine,
            settings,
            recent: Vec::new(),
            overrides: HashMap::new(),
            pending_override: None,
        })
    })
}

fn with<R>(f: impl FnOnce(&mut Shell) -> R) -> R {
    let mut guard = shell().lock().expect("shell mutex poisoned");
    f(&mut guard)
}

/// Apply a VI/EN state to both the persisted settings and the live engine. Callers
/// persist (`save`) themselves, since some batch this with other field writes.
fn set_enabled_state(s: &mut Shell, on: bool) {
    s.settings.enabled = on;
    s.engine.set_enabled(on);
    if !on {
        s.engine.clear();
    }
}

// --- reads -----------------------------------------------------------------

pub fn snapshot() -> Settings {
    with(|s| s.settings.clone())
}
pub fn excluded_apps() -> Vec<ExcludedApp> {
    with(|s| s.settings.excluded_apps.clone())
}
pub fn recent_apps() -> Vec<ExcludedApp> {
    with(|s| s.recent.clone())
}
pub fn enabled() -> bool {
    with(|s| s.settings.enabled)
}
pub fn method() -> InputMethod {
    with(|s| s.settings.method.core())
}
pub fn tone_style() -> CoreToneStyle {
    with(|s| s.settings.tone_style.core())
}
pub fn toggle_hotkey() -> Hotkey {
    with(|s| s.settings.toggle_hotkey)
}
pub fn is_composing() -> bool {
    with(|s| !s.engine.buffer().is_empty())
}

/// True when the most-recently-focused app is Google Chrome. Used to route text
/// injection through the Delete-primer path that works around Chrome's omnibox
/// autocomplete eating synthesized Backspaces. `recent[0]` is the current
/// foreground app (it is pushed there by the foreground hook). Chrome Beta/Dev/
/// Canary also report `chrome.exe`; Edge (`msedge.exe`) and Brave (`brave.exe`)
/// deliberately do not match — they are unaffected.
pub fn foreground_is_chrome() -> bool {
    with(|s| s.recent.first().map(|a| a.id == "chrome.exe").unwrap_or(false))
}

// --- writes (each persists) ------------------------------------------------

/// Flip VI/EN from the tray; returns the new state. The tray click steals
/// foreground, so the choice is parked as a pending override and bound to the next
/// app the user focuses (see [`apply_for_app`]) — otherwise the per-app auto-switch
/// would revert it the instant focus returns to a non-excluded app.
pub fn toggle_enabled() -> bool {
    with(|s| {
        let on = !s.settings.enabled;
        set_enabled_state(s, on);
        s.pending_override = Some(on);
        s.settings.save();
        on
    })
}

/// Flip VI/EN from the keyboard hotkey; returns the new state. Unlike the tray, the
/// hotkey fires while the target app is focused, so the choice binds to that app
/// (`recent[0]`) immediately and clears any stale pending override.
pub fn toggle_enabled_hotkey() -> bool {
    with(|s| {
        let on = !s.settings.enabled;
        set_enabled_state(s, on);
        if let Some(app) = s.recent.first() {
            s.overrides.insert(app.id.clone(), on);
        }
        s.pending_override = None;
        s.settings.save();
        on
    })
}

pub fn set_enabled(on: bool) {
    with(|s| {
        set_enabled_state(s, on);
        // The Settings window holds focus while this runs, so treat it like the
        // tray: bind the choice to the next app the user returns to.
        s.pending_override = Some(on);
        s.settings.save();
    });
}

pub fn set_method(method: InputMethod) {
    with(|s| {
        s.settings.method = Method::from_core(method);
        s.engine.set_method(method);
        s.engine.clear();
        s.settings.save();
    });
}

pub fn set_tone_style(style: CoreToneStyle) {
    with(|s| {
        s.settings.tone_style = ToneStyle::from_core(style);
        s.engine.set_tone_style(style);
        s.settings.save();
    });
}

pub fn set_smart_restore(on: bool) {
    with(|s| {
        s.settings.smart_restore = on;
        s.engine.set_smart_restore(on);
        s.settings.save();
    });
}

pub fn set_eager_restore(on: bool) {
    with(|s| {
        s.settings.eager_restore = on;
        s.engine.set_eager_restore(on);
        s.settings.save();
    });
}

pub fn set_toggle_hotkey(hotkey: Hotkey) {
    with(|s| {
        s.settings.toggle_hotkey = hotkey;
        s.settings.save();
    });
}

/// Persist the launch-at-login preference. The registry side effect (auto-launch)
/// is applied by `commands`, which owns the OS integration.
pub fn set_launch_at_login(on: bool) {
    with(|s| {
        s.settings.launch_at_login = on;
        s.settings.save();
    });
}

pub fn complete_onboarding() {
    with(|s| {
        s.settings.has_completed_onboarding = true;
        s.settings.save();
    });
}

pub fn add_excluded_app(app: ExcludedApp) {
    with(|s| {
        if !s.settings.excluded_apps.iter().any(|a| a.id == app.id) {
            s.settings.excluded_apps.push(app);
            s.settings.save();
        }
    });
}

pub fn remove_excluded_app(id: &str) {
    with(|s| {
        let before = s.settings.excluded_apps.len();
        s.settings.excluded_apps.retain(|a| a.id != id);
        if s.settings.excluded_apps.len() != before {
            s.settings.save();
        }
    });
}

// --- per-app auto-switch (called from the foreground hook) ------------------

/// Record the just-focused app for the Settings "recent apps" picker (deduped,
/// most-recent-first, capped). No-op for empty ids.
pub fn note_foreground(id: String, name: String) {
    if id.is_empty() {
        return;
    }
    with(|s| {
        s.recent.retain(|a| a.id != id);
        s.recent.insert(0, ExcludedApp { id, name });
        s.recent.truncate(RECENT_CAP);
    });
}

/// Decide VI/EN for the newly-focused app, in priority order:
///
/// 1. A pending manual toggle (from the tray / Settings, which steal foreground)
///    binds to this app — the user's choice lands on the app they return to.
/// 2. A remembered manual override for this app wins over the list default, so a
///    prior manual toggle survives leaving and re-focusing the app.
/// 3. Otherwise the exclusion-list default, mirroring the macOS shell: excluded
///    apps → English, every other app → Vietnamese. No-op when the list is empty,
///    so users who don't use the feature keep a plain global toggle.
///
/// Returns `Some(on)` when it flipped VI/EN (so the caller can refresh the tray),
/// `None` when nothing changed.
pub fn apply_for_app(id: &str) -> Option<bool> {
    with(|s| {
        let target = if let Some(on) = s.pending_override.take() {
            s.overrides.insert(id.to_string(), on);
            on
        } else if let Some(&on) = s.overrides.get(id) {
            on
        } else if s.settings.excluded_apps.is_empty() {
            return None;
        } else {
            !s.settings.excluded_apps.iter().any(|a| a.id == id)
        };

        if s.settings.enabled == target {
            return None;
        }
        set_enabled_state(s, target);
        s.settings.save();
        Some(target)
    })
}

// --- composition driving (called from the hook) ----------------------------

pub fn process_char(c: char) -> ImeResult {
    with(|s| s.engine.process_char(c))
}

/// Sync the engine after Backspace while composing; the physical Backspace then
/// passes through so the app deletes its own visible char (like `funput-term`).
pub fn on_backspace() {
    with(|s| {
        s.engine.on_backspace();
    });
}

pub fn clear() {
    with(|s| s.engine.clear());
}
