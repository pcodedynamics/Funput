//! Telex key classification — letter modifiers map to a shared [`KeyAction`].
//!
//! # Mapping (UniKey / OpenKey)
//!
//! | Telex | Action |
//! |-------|--------|
//! | `s` / `f` / `r` / `x` / `j` | sắc / huyền / hỏi / ngã / nặng |
//! | `dd` | stroke `đ` |
//! | `aa` / `ee` / `oo` | circumflex on `a` / `e` / `o` |
//! | `w` after `a` | breve `ă` |
//! | `w` after `o` / `u` | horn `ơ` / `ư` |
//! | `w` after `uo` | horn compound `ươ` |
//!
//! # Priority
//!
//! 1. Digraph stroke/shape (`dd`, `aa`, `ee`, `oo`) on plain vowels
//! 2. `w` — `uo` compound before single `o` / `u` / `a`; shaped-vowel revert
//! 3. Stroke/shape revert (`đ`+`d`, `â`+`a`, …)
//! 4. Tone keys `s` / `f` / `r` / `x` / `j`
//! 5. Normal character

use crate::composition::apply::uo_pair_in_vowel_cluster;
use crate::input_method::KeyAction;
use crate::unicode::marks::{is_vowel, tone_on_vowel, vowel_stem, Tone};
use crate::unicode::shapes::{shape_on_vowel, strip_shape, VowelShape};

/// Map Telex tone keys to tone marks.
pub fn tone_from_key(key: char) -> Option<Tone> {
    match key.to_ascii_lowercase() {
        's' => Some(Tone::Sac),
        'f' => Some(Tone::Huyen),
        'r' => Some(Tone::Hoi),
        'x' => Some(Tone::Nga),
        'j' => Some(Tone::Nang),
        _ => None,
    }
}

/// Classify a Telex keystroke into a method-agnostic [`KeyAction`].
pub fn classify_key(buffer: &str, key: char) -> KeyAction {
    if let Some(action) = classify_digraph(buffer, key) {
        return action;
    }
    if key.eq_ignore_ascii_case(&'w') && let Some(action) = classify_w(buffer) {
        return action;
    }
    if let Some(action) = classify_revert_stroke(buffer, key) {
        return action;
    }
    if let Some(action) = classify_revert_circumflex(buffer, key) {
        return action;
    }
    classify_tone(buffer, key).unwrap_or(KeyAction::Normal)
}

fn last_char(buffer: &str) -> Option<char> {
    buffer.chars().last()
}

fn is_plain_vowel(c: char, base: char) -> bool {
    vowel_stem(c).is_some_and(|stem| stem.eq_ignore_ascii_case(&base))
        && tone_on_vowel(c).is_none()
        && shape_on_vowel(c).is_none()
}

fn classify_w(buffer: &str) -> Option<KeyAction> {
    if uo_pair_in_vowel_cluster(buffer).is_some() {
        return Some(KeyAction::Shape(VowelShape::Horn));
    }

    let last = last_char(buffer)?;

    if let Some(shape) = shape_on_vowel(last) {
        return match shape {
            VowelShape::Breve | VowelShape::Horn => Some(KeyAction::Shape(shape)),
            VowelShape::Circumflex => None,
        };
    }

    if is_plain_vowel(last, 'a') {
        return Some(KeyAction::Shape(VowelShape::Breve));
    }
    if is_plain_vowel(last, 'o') || is_plain_vowel(last, 'u') {
        return Some(KeyAction::Shape(VowelShape::Horn));
    }

    None
}

fn classify_digraph(buffer: &str, key: char) -> Option<KeyAction> {
    let last = last_char(buffer)?;

    if last.eq_ignore_ascii_case(&'d') && key.eq_ignore_ascii_case(&'d') {
        return Some(KeyAction::Stroke);
    }

    for base in ['a', 'e', 'o'] {
        if is_plain_vowel(last, base) && key.eq_ignore_ascii_case(&base) {
            return Some(KeyAction::Shape(VowelShape::Circumflex));
        }
    }

    None
}

fn classify_revert_stroke(buffer: &str, key: char) -> Option<KeyAction> {
    let last = last_char(buffer)?;
    if matches!(last, 'đ' | 'Đ') && key.eq_ignore_ascii_case(&'d') {
        return Some(KeyAction::Stroke);
    }
    None
}

fn classify_revert_circumflex(buffer: &str, key: char) -> Option<KeyAction> {
    let last = last_char(buffer)?;
    if shape_on_vowel(last) != Some(VowelShape::Circumflex) {
        return None;
    }
    let plain = strip_shape(last)?;
    if key.eq_ignore_ascii_case(&plain) {
        Some(KeyAction::Shape(VowelShape::Circumflex))
    } else {
        None
    }
}

/// Tone keys (`s` `f` `r` `x` `j`) are ordinary Latin letters too, so only treat
/// one as a tone when there is a vowel to receive it. With no vowel the key stays
/// a literal character — a leading `f`/`j`, a consonant onset (`tr`), or an
/// English word the engine restores on the next word boundary — and is never
/// dropped.
fn classify_tone(buffer: &str, key: char) -> Option<KeyAction> {
    if !buffer.chars().any(is_vowel) {
        return None;
    }
    tone_from_key(key).map(KeyAction::Tone)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_tone_keys() {
        assert_eq!(classify_key("a", 's'), KeyAction::Tone(Tone::Sac));
        assert_eq!(classify_key("a", 'f'), KeyAction::Tone(Tone::Huyen));
        assert_eq!(classify_key("a", 'r'), KeyAction::Tone(Tone::Hoi));
        assert_eq!(classify_key("a", 'x'), KeyAction::Tone(Tone::Nga));
        assert_eq!(classify_key("a", 'j'), KeyAction::Tone(Tone::Nang));
        assert_eq!(classify_key("", 's'), KeyAction::Normal);
        assert_eq!(classify_key("a", 'm'), KeyAction::Normal);
    }

    #[test]
    fn classify_stroke_digraph() {
        assert_eq!(classify_key("d", 'd'), KeyAction::Stroke);
        assert_eq!(classify_key("D", 'd'), KeyAction::Stroke);
    }

    #[test]
    fn classify_shape_digraphs() {
        assert_eq!(
            classify_key("a", 'a'),
            KeyAction::Shape(VowelShape::Circumflex)
        );
        assert_eq!(
            classify_key("e", 'e'),
            KeyAction::Shape(VowelShape::Circumflex)
        );
        assert_eq!(
            classify_key("o", 'o'),
            KeyAction::Shape(VowelShape::Circumflex)
        );
    }

    #[test]
    fn classify_w_rules() {
        assert_eq!(classify_key("a", 'w'), KeyAction::Shape(VowelShape::Breve));
        assert_eq!(classify_key("o", 'w'), KeyAction::Shape(VowelShape::Horn));
        assert_eq!(classify_key("u", 'w'), KeyAction::Shape(VowelShape::Horn));
        assert_eq!(classify_key("uo", 'w'), KeyAction::Shape(VowelShape::Horn));
        assert_eq!(classify_key("tru", 'w'), KeyAction::Shape(VowelShape::Horn));
        assert_eq!(classify_key("nuoc", 'w'), KeyAction::Shape(VowelShape::Horn));
    }

    #[test]
    fn classify_uo_before_single_o() {
        assert_eq!(classify_key("uo", 'w'), KeyAction::Shape(VowelShape::Horn));
        assert_eq!(classify_key("o", 'w'), KeyAction::Shape(VowelShape::Horn));
    }

    #[test]
    fn tone_letter_without_vowel_stays_literal() {
        // No vowel to receive the tone → the letter is ordinary input, never lost.
        assert_eq!(classify_key("t", 'r'), KeyAction::Normal);
        assert_eq!(classify_key("ng", 's'), KeyAction::Normal);
        assert_eq!(classify_key("", 'x'), KeyAction::Normal);
        assert_eq!(classify_key("", 'f'), KeyAction::Normal);
        assert_eq!(classify_key("", 'j'), KeyAction::Normal);
    }

    #[test]
    fn classify_revert_triggers() {
        assert_eq!(classify_key("â", 'a'), KeyAction::Shape(VowelShape::Circumflex));
        assert_eq!(classify_key("ơ", 'w'), KeyAction::Shape(VowelShape::Horn));
        assert_eq!(classify_key("đ", 'd'), KeyAction::Stroke);
        assert_eq!(classify_key("á", 's'), KeyAction::Tone(Tone::Sac));
    }
}
