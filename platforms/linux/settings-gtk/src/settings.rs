//! Persisted user settings (`~/.config/Funput/settings.json`) — the same file the
//! Fcitx5 addon and IBus engine read. Field names serialize to camelCase so the
//! schema stays identical to the C++ reader (`platforms/linux/common/settings.cpp`)
//! and the previous Tauri shell. This process never drives the engine; it only
//! reads and writes this file (the engine picks changes up on its next focus-in).
//!
//! Ported verbatim from the retired Tauri shell; keep the serde attributes
//! unchanged or the engine will stop reading the file.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Method {
    Telex,
    Vni,
}

/// Tone-mark placement style (traditional `hòa` vs modern `hoà`). This process only
/// persists it to settings.json; the engine reads it and applies it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToneStyle {
    #[default]
    Traditional,
    Modern,
}

/// VI/EN toggle hotkey presets. The engine maps these to keysyms. `AltShift` is not
/// yet supported by the Linux engine (see README); the UI surfaces it with a note.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Hotkey {
    CtrlBacktick,
    CtrlSpace,
    AltShift,
}

/// An app excluded from Vietnamese input. `id` is the fcitx5 program()/WM_CLASS
/// (e.g. "code"); `name` is a friendly label for the Settings UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExcludedApp {
    pub id: String,
    pub name: String,
}

/// A text-expansion shortcut (gõ tắt): typing `trigger` then a word boundary expands
/// to `expansion` (`vn` → `Việt Nam`). Read by the C++ engines from settings.json.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Shortcut {
    pub trigger: String,
    pub expansion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub method: Method,
    /// `#[serde(default)]` keeps older settings files (without this key) loadable.
    #[serde(default)]
    pub tone_style: ToneStyle,
    pub enabled: bool,
    pub smart_restore: bool,
    pub eager_restore: bool,
    /// Spell-check ("Kiểm tra chính tả"): only place a diacritic that forms a valid
    /// Vietnamese syllable. `#[serde(default)]` keeps older settings files loadable.
    #[serde(default)]
    pub spell_check: bool,
    /// Auto-capitalize ("Tự động viết hoa"): uppercase the first letter at the start of
    /// a sentence. `#[serde(default)]` keeps older settings files loadable.
    #[serde(default)]
    pub auto_capitalize: bool,
    pub toggle_hotkey: Hotkey,
    pub launch_at_login: bool,
    pub has_completed_onboarding: bool,
    /// Apps that default to English on focus. `#[serde(default)]` keeps older
    /// settings files (without this key) loadable instead of resetting to defaults.
    #[serde(default)]
    pub excluded_apps: Vec<ExcludedApp>,
    /// Text-expansion shortcuts (gõ tắt). `#[serde(default)]` keeps older settings
    /// files (without this key) loadable.
    #[serde(default)]
    pub shortcuts: Vec<Shortcut>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            method: Method::Vni,
            tone_style: ToneStyle::Traditional,
            enabled: true,
            smart_restore: true,
            eager_restore: true,
            spell_check: false,
            auto_capitalize: false,
            toggle_hotkey: Hotkey::CtrlBacktick,
            launch_at_login: false,
            has_completed_onboarding: false,
            excluded_apps: Vec::new(),
            shortcuts: Vec::new(),
        }
    }
}

fn settings_path() -> Option<PathBuf> {
    // ~/.config/Funput/settings.json (XDG-aware via `dirs`).
    dirs::config_dir().map(|d| d.join("Funput").join("settings.json"))
}

/// Read the recently-focused apps the Fcitx5 addon recorded in
/// `~/.config/Funput/recent-apps.json`. Empty on missing/corrupt file.
pub fn recent_apps() -> Vec<ExcludedApp> {
    dirs::config_dir()
        .map(|d| d.join("Funput").join("recent-apps.json"))
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

impl Settings {
    /// Load from disk, falling back to defaults on any error (first run, corrupt).
    pub fn load() -> Self {
        settings_path()
            .and_then(|p| fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Persist to disk (best effort).
    pub fn save(&self) {
        let Some(path) = settings_path() else { return };
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, json);
        }
    }

    /// Load, mutate one field, save — the shape every control's change handler needs.
    /// Loading fresh each time avoids clobbering writes the engine made meanwhile
    /// (it rewrites the file when VI/EN is toggled from the keyboard).
    pub fn update(f: impl FnOnce(&mut Settings)) -> Settings {
        let mut s = Settings::load();
        f(&mut s);
        s.save();
        s
    }
}
