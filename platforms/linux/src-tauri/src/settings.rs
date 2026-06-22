//! Persisted user settings (`~/.config/Funput/settings.json`) — the same file the
//! Fcitx5 addon reads. Field names serialize to camelCase to match the web UI
//! (`platforms/ui/src/lib/api.ts`). Unlike the Windows shell, this process never
//! drives the engine; it only reads and writes this file.

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
/// persists it to settings.json; the Fcitx5 addon reads it and drives the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToneStyle {
    #[default]
    Traditional,
    Modern,
}

/// VI/EN toggle hotkey presets. The Fcitx5 addon maps these to keysyms.
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
    pub toggle_hotkey: Hotkey,
    pub launch_at_login: bool,
    pub has_completed_onboarding: bool,
    /// Apps that default to English on focus. `#[serde(default)]` keeps older
    /// settings files (without this key) loadable instead of resetting to defaults.
    #[serde(default)]
    pub excluded_apps: Vec<ExcludedApp>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            method: Method::Vni,
            tone_style: ToneStyle::Traditional,
            enabled: true,
            smart_restore: true,
            eager_restore: true,
            toggle_hotkey: Hotkey::CtrlBacktick,
            launch_at_login: false,
            has_completed_onboarding: false,
            excluded_apps: Vec::new(),
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

    /// Load, mutate one field, save — the shape every setter command needs.
    pub fn update(f: impl FnOnce(&mut Settings)) -> Settings {
        let mut s = Settings::load();
        f(&mut s);
        s.save();
        s
    }
}
