//! The Settings window: an `AdwPreferencesWindow` with one page per settings group.
//! Every control reads the current value from `settings.json` on build and writes
//! back through `Settings::update` on change — the engine reloads on its next
//! focus-in. Mirrors the panes of the retired Svelte UI (`platforms/ui/src/lib/
//! settings/panes/*`), minus the "Gõ thử" try-typing box (dropped for a cleaner
//! native UI; users type in real apps).
//!
//! Each page lives in its own submodule under `settings_window/`.

mod about;
mod apps;
mod general;
mod input_method;
mod keyboard;
mod shortcuts;
mod smart;

use adw::prelude::*;
use adw::{Application, PreferencesWindow};

pub fn build(app: &Application) -> PreferencesWindow {
    let window = PreferencesWindow::builder()
        .title("Funput — Cài đặt")
        .default_width(640)
        .default_height(520)
        .build();
    window.set_application(Some(app));
    window.set_search_enabled(false);

    window.add(&input_method::page());
    window.add(&smart::page());
    window.add(&shortcuts::page());
    window.add(&keyboard::page());
    // Per-app auto-switch is Fcitx5-only; hide the page entirely on IBus (where it
    // would do nothing) rather than show a dead pane.
    if crate::framework::per_app_supported() {
        window.add(&apps::page());
    }
    window.add(&general::page());
    window.add(&about::page(&window));

    window
}
