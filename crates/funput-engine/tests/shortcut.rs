//! Text-expansion (gõ tắt) integration tests at the [`Engine`] level.
//!
//! A trigger matches the raw keystrokes since the last word boundary,
//! case-sensitively, and expands at the boundary — taking priority over English
//! restore. See `src/boundary.rs` for the matching logic.

mod support;

use funput_core::InputMethod;
use funput_engine::{Action, Engine};

/// Drive `keys` through an engine seeded with `shortcuts`, reconstructing the
/// resulting app text from the inject stream (None → append, Send → delete + append).
fn app_text_with_shortcuts(method: InputMethod, shortcuts: &[(&str, &str)], keys: &str) -> String {
    let mut engine = Engine::new();
    engine.set_method(method);
    for (trigger, expansion) in shortcuts {
        engine.add_shortcut(*trigger, *expansion);
    }
    let mut app = String::new();
    for key in keys.chars() {
        let r = engine.process_char(key);
        match r.action {
            Action::None => app.push(key),
            Action::Send => {
                for _ in 0..r.backspace {
                    app.pop();
                }
                app.push_str(&r.output);
            }
            Action::Restore => unreachable!("Restore not implemented yet"),
        }
    }
    app
}

#[test]
fn expands_trigger_on_space_telex() {
    let text = app_text_with_shortcuts(InputMethod::Telex, &[("vn", "Việt Nam")], "vn ");
    assert_eq!(text, "Việt Nam ");
}

#[test]
fn expands_trigger_on_space_vni() {
    let text = app_text_with_shortcuts(InputMethod::Vni, &[("vn", "Việt Nam")], "vn ");
    assert_eq!(text, "Việt Nam ");
}

#[test]
fn punctuation_boundary_is_kept() {
    let text = app_text_with_shortcuts(InputMethod::Telex, &[("kg", "không")], "kg,");
    assert_eq!(text, "không,");
}

#[test]
fn expansion_wins_over_english_restore() {
    // `card` would normally restore to its raw keys at the boundary; the trigger
    // takes priority and expands instead.
    let text = app_text_with_shortcuts(InputMethod::Telex, &[("card", "credit card")], "card ");
    assert_eq!(text, "credit card ");
}

#[test]
fn trigger_is_case_sensitive() {
    // Only lowercase `vn` is defined — `VN` falls through to normal handling.
    let text = app_text_with_shortcuts(InputMethod::Telex, &[("vn", "Việt Nam")], "VN ");
    assert_eq!(text, "VN ");
}

#[test]
fn shortcut_only_at_boundary_not_mid_word() {
    // `vna` (no boundary after `vn`) must not expand.
    let text = app_text_with_shortcuts(InputMethod::Telex, &[("vn", "Việt Nam")], "vna ");
    assert_ne!(text, "Việt Nama ");
}

#[test]
fn undefined_trigger_leaves_normal_behavior_unchanged() {
    // No shortcuts at all: a valid Vietnamese word still composes normally.
    let text = app_text_with_shortcuts(InputMethod::Telex, &[], "mas ");
    assert_eq!(text, "má ");
}

#[test]
fn remove_and_clear_shortcuts() {
    let mut engine = Engine::new();
    engine.add_shortcut("vn", "Việt Nam");
    engine.add_shortcut("kg", "không");
    assert_eq!(engine.shortcuts().len(), 2);

    engine.remove_shortcut("vn");
    assert!(!engine.shortcuts().contains_key("vn"));
    assert!(engine.shortcuts().contains_key("kg"));

    engine.clear_shortcuts();
    assert!(engine.shortcuts().is_empty());
}

#[test]
fn empty_trigger_is_ignored() {
    let mut engine = Engine::new();
    engine.add_shortcut("", "nothing");
    assert!(engine.shortcuts().is_empty());
}

#[test]
fn re_adding_trigger_overwrites() {
    let mut engine = Engine::new();
    engine.add_shortcut("vn", "Việt Nam");
    engine.add_shortcut("vn", "Vietnam");
    assert_eq!(engine.shortcuts().get("vn").map(String::as_str), Some("Vietnam"));
}
