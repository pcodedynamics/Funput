//! Settings window creation, state population, and external update state.

use std::cell::RefCell;

use slint::{ComponentHandle, Weak};

use super::models;
use super::settings_callbacks;
use crate::{commands, shell, SettingsWindow};

thread_local! {
    static WINDOW: RefCell<Option<Weak<SettingsWindow>>> = const { RefCell::new(None) };
}

pub(super) fn open() {
    if let Some(window) = current() {
        populate(&window);
        let _ = window.show();
        return;
    }

    let window = SettingsWindow::new().expect("create settings window");
    populate(&window);
    settings_callbacks::wire(&window);
    let _ = window.show();
    WINDOW.with(|cell| *cell.borrow_mut() = Some(window.as_weak()));
}

fn current() -> Option<SettingsWindow> {
    WINDOW.with(|cell| cell.borrow().as_ref().and_then(Weak::upgrade))
}

fn populate(window: &SettingsWindow) {
    let settings = shell::snapshot();
    window.set_method(settings.method.id().into());
    window.set_tone_style(settings.tone_style.id().into());
    window.set_hotkey(settings.toggle_hotkey.id().into());
    window.set_hotkey_caps(models::caps(settings.toggle_hotkey));
    window.set_smart_restore(settings.smart_restore);
    window.set_eager_restore(settings.eager_restore);
    window.set_spell_check(settings.spell_check);
    window.set_auto_capitalize(settings.auto_capitalize);
    window.set_launch_at_login(settings.launch_at_login);
    window.set_version(env!("CARGO_PKG_VERSION").into());
    window.set_update_state("idle".into());
    window.set_update_version("".into());
    window.set_update_message("".into());
    refresh_apps(window);
    window.set_shortcuts(models::shortcuts(&shell::shortcuts()));
}

/// Reflect an asynchronous updater step if Settings is still open.
pub(crate) fn set_update_state(state: &str, version: &str, message: &str) {
    if let Some(window) = current() {
        window.set_update_state(state.into());
        window.set_update_version(version.into());
        window.set_update_message(message.into());
    }
}

pub(super) fn open_and_check_updates() {
    open();
    if let Some(window) = current() {
        window.set_active("about".into());
    }
    commands::check_for_updates();
}

pub(super) fn refresh_apps(window: &SettingsWindow) {
    let excluded = shell::excluded_apps();
    let addable = shell::recent_apps()
        .into_iter()
        .filter(|recent| !excluded.iter().any(|item| item.id == recent.id))
        .collect::<Vec<_>>();
    window.set_excluded_apps(models::apps(&excluded));
    window.set_recent_apps(models::apps(&addable));
}
