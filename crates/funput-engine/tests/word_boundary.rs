mod support;

use funput_core::InputMethod;
use funput_engine::{Action, Engine};

#[test]
fn telex_mas_then_space() {
    let mut engine = Engine::new();
    engine.process_char('m');
    engine.process_char('a');
    let tone = engine.process_char('s');
    assert_eq!(tone.action, Action::Send);
    assert_eq!(engine.buffer(), "má");

    let space = engine.process_char(' ');
    assert_eq!(space.action, Action::None);
    assert_eq!(space.backspace, 0);
    assert!(space.output.is_empty());
    assert_eq!(engine.buffer(), "");
    assert_eq!(engine.keys(), "");
}

#[test]
fn telex_multi_word() {
    assert_eq!(
        support::type_words(InputMethod::Telex, "xins chaof banj"),
        "xín chào bạn"
    );
}

#[test]
fn vni_multi_word() {
    assert_eq!(
        support::type_words(InputMethod::Vni, "xin1 chao2"),
        "xín chào"
    );
}

#[test]
fn type_words_leaves_buffer_empty_after_trailing_space() {
    let mut engine = Engine::new();
    for key in "mas ".chars() {
        engine.process_char(key);
    }
    assert_eq!(engine.buffer(), "");
}

#[test]
fn punctuation_is_a_boundary_no_cross_syllable_bleed() {
    // The modifier in the second chunk must not reach back to the first syllable.
    assert_eq!(support::app_text(InputMethod::Telex, "as,af"), "á,à");
    assert_eq!(support::app_text(InputMethod::Vni, "a1.a2"), "á.à");
    // Buffer resets after the punctuation so the second word composes cleanly.
    assert_eq!(support::app_text(InputMethod::Telex, "anhf-em"), "ành-em");
}

#[test]
fn english_word_restored_on_boundary() {
    // Words composing to a non-Vietnamese final are restored to raw keystrokes.
    assert_eq!(support::app_text(InputMethod::Telex, "card "), "card ");
    assert_eq!(support::app_text(InputMethod::Telex, "cool "), "cool ");
    assert_eq!(support::app_text(InputMethod::Telex, "hard."), "hard.");
    assert_eq!(support::app_text(InputMethod::Telex, "park "), "park ");
}

#[test]
fn valid_vietnamese_kept_on_boundary() {
    // A complete syllable is intentional — never restored, even if it also reads
    // as English (`test` → `tét`).
    assert_eq!(support::app_text(InputMethod::Telex, "mas "), "má ");
    assert_eq!(support::app_text(InputMethod::Telex, "test "), "tét ");
    assert_eq!(support::app_text(InputMethod::Telex, "vietj "), "việt ");
}

#[test]
fn mixed_english_and_vietnamese_words() {
    assert_eq!(support::app_text(InputMethod::Telex, "card mas "), "card má ");
}
