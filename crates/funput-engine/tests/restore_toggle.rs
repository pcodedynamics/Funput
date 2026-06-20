//! Smart / eager English-restore toggles on the engine.

use funput_core::InputMethod;
use funput_engine::{Action, Engine};

fn telex() -> Engine {
    let mut e = Engine::new();
    e.set_method(InputMethod::Telex);
    e
}

fn typed(e: &mut Engine, s: &str) {
    for c in s.chars() {
        e.process_char(c);
    }
}

#[test]
fn defaults_restore_eagerly() {
    // Both toggles default on: "text" flips to raw the instant it dead-ends.
    let mut e = telex();
    typed(&mut e, "text");
    assert_eq!(e.buffer(), "text");
}

#[test]
fn smart_off_keeps_composed_word() {
    // No restore at all: the composed Vietnamese form is kept even for English.
    let mut e = telex();
    e.set_smart_restore(false);
    typed(&mut e, "text");
    assert_eq!(e.buffer(), "tẽt"); // not restored mid-word
    let space = e.process_char(' ');
    assert_eq!(space.action, Action::None); // nor at the boundary
}

#[test]
fn eager_off_restores_only_at_boundary() {
    // Smart on, eager off: keep composing mid-word, restore when the word ends.
    let mut e = telex();
    e.set_eager_restore(false);
    typed(&mut e, "text");
    assert_eq!(e.buffer(), "tẽt"); // no instant restore
    let space = e.process_char(' ');
    assert_eq!(space.action, Action::Send);
    assert_eq!(space.output, "text "); // restored at the boundary
}

#[test]
fn toggles_do_not_disturb_valid_vietnamese() {
    // A real syllable is kept regardless of the toggles.
    for (smart, eager) in [(true, true), (true, false), (false, false)] {
        let mut e = telex();
        e.set_smart_restore(smart);
        e.set_eager_restore(eager);
        typed(&mut e, "vieejt");
        assert_eq!(e.buffer(), "việt");
    }
}
