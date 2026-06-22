//! Persisted user settings (`%APPDATA%\Funput\settings.json`) + the enums shared
//! with the web UI. Field names serialize to camelCase to match `api.ts`.

use std::fs;
use std::path::PathBuf;

use funput_core::{InputMethod, ToneStyle as CoreToneStyle};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Method {
    Telex,
    Vni,
}

impl Method {
    pub fn core(self) -> InputMethod {
        match self {
            Method::Telex => InputMethod::Telex,
            Method::Vni => InputMethod::Vni,
        }
    }
    pub fn from_core(m: InputMethod) -> Self {
        match m {
            InputMethod::Telex => Method::Telex,
            InputMethod::Vni => Method::Vni,
        }
    }
}

/// Tone-mark placement style (traditional `hòa` vs modern `hoà`). Mirrors `Method`:
/// a serde-friendly enum bridged to the engine's `funput_core::ToneStyle`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToneStyle {
    #[default]
    Traditional,
    Modern,
}

impl ToneStyle {
    pub fn core(self) -> CoreToneStyle {
        match self {
            ToneStyle::Traditional => CoreToneStyle::Traditional,
            ToneStyle::Modern => CoreToneStyle::Modern,
        }
    }
    pub fn from_core(ts: CoreToneStyle) -> Self {
        match ts {
            CoreToneStyle::Traditional => ToneStyle::Traditional,
            CoreToneStyle::Modern => ToneStyle::Modern,
        }
    }
}

/// VI/EN toggle hotkey presets. The hook maps these to virtual-keys in `keymap`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Hotkey {
    CtrlBacktick,
    CtrlSpace,
    AltShift,
}

/// An app excluded from Vietnamese input. `id` is the lowercased exe file name
/// (e.g. "code.exe"); `name` is a friendly label for the Settings UI.
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
    // %APPDATA%\Funput\settings.json
    dirs::config_dir().map(|d| d.join("Funput").join("settings.json"))
}

impl Settings {
    /// Load from disk, falling back to defaults on any error (first run, corrupt).
    pub fn load() -> Self {
        settings_path()
            .and_then(|p| fs::read_to_string(p).ok())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Persist to disk (best effort — ignore IO errors so typing is never blocked).
    pub fn save(&self) {
        let Some(path) = settings_path() else { return };
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, json);
        }
    }
}
