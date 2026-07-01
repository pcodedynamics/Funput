//! Telex key classification — letter modifiers map to a shared [`KeyAction`].
//!
//! # Mapping (UniKey / OpenKey)
//!
//! | Telex | Action |
//! |-------|--------|
//! | `s` / `f` / `r` / `x` / `j` | sắc / huyền / hỏi / ngã / nặng |
//! | `z` | remove tone mark (xóa dấu) |
//! | `dd` | stroke `đ` |
//! | `aa` / `ee` / `oo` | circumflex on `a` / `e` / `o` |
//! | `w` after `a` | breve `ă` |
//! | `w` after `o` / `u` | horn `ơ` / `ư` |
//! | `w` after `uo` | horn compound `ươ` |
//!
//! # Priority
//!
//! 1. Stroke `đ` — adjacent `dd`, `đ`+`d` revert, or a `d`/`đ` already past the
//!    nucleus so the `d` can be typed anywhere in the syllable (`dược`+`d` → `được`)
//! 2. Digraph shape (`aa`, `ee`, `oo`) on plain vowels
//! 3. `w` — `uo` compound before single `o` / `u` / `a`; shaped-vowel revert
//! 4. Shape revert (`â`+`a`, …)
//! 5. Tone keys `s` / `f` / `r` / `x` / `j`
//! 6. Normal character

use crate::composition::apply::uo_pair_in_vowel_cluster;
use crate::input_method::KeyAction;
use crate::unicode::marks::{is_vowel, tone_on_vowel, vowel_stem, Tone};
use crate::unicode::shapes::{shape_on_vowel, shape_target_index, strip_shape, VowelShape};

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
    if let Some(action) = classify_stroke(buffer, key) {
        return action;
    }
    if let Some(action) = classify_digraph(buffer, key) {
        return action;
    }
    if key.eq_ignore_ascii_case(&'w') && let Some(action) = classify_w(buffer) {
        return action;
    }
    if let Some(action) = classify_revert_circumflex(buffer, key) {
        return action;
    }
    if key.eq_ignore_ascii_case(&'z') {
        return KeyAction::RemoveTone;
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

/// Whether the buffer ends in a `ua`-family cluster (`u`/`ú`/`ư` + `a`) on which a
/// `w` should act on the `u` slot rather than putting a breve on the `a` — there is
/// no `uă` rhyme. All three cases resolve through the shared apply/revert layer once
/// `classify_w` returns [`VowelShape::Horn`]:
///
/// - plain `ua` → horn the `u`, forming `ưa` (`mua` + `w` → `mưa`, `nua` → `nưa`)
/// - toned `úa` → horn the `u`, keeping the tone (`úa` + `w` → `ứa`)
/// - already-horned `ưa` → a repeat `w` reverts it (`nưa` + `w` → `nua`)
///
/// The `qu` glide is excluded: there the `u` belongs to the onset and the `a` takes
/// the breve (`qua` + `w` → `quă`), mirroring the `uo` → `ươ` compound rule.
fn ends_with_ua_horn_target(buffer: &str) -> bool {
    let mut rev = buffer.chars().rev();
    if !rev.next().is_some_and(|a| is_plain_vowel(a, 'a')) {
        return false;
    }
    // The `u` slot: any `u`- or `ư`-based vowel (plain, toned, or already horned).
    let is_u_slot = rev
        .next()
        .and_then(vowel_stem)
        .is_some_and(|stem| matches!(stem, 'u' | 'U' | 'ư' | 'Ư'));
    if !is_u_slot {
        return false;
    }
    !matches!(rev.next(), Some('q' | 'Q'))
}

fn classify_w(buffer: &str) -> Option<KeyAction> {
    if uo_pair_in_vowel_cluster(buffer).is_some() {
        return Some(KeyAction::Shape(VowelShape::Horn));
    }

    let last = last_char(buffer)?;

    // Adjacent rules on the last vowel, tried first so an immediately-preceding
    // vowel still wins: revert a shaped vowel (`ơ` + `w` → `o`), or shape a plain
    // `a`/`o`/`u` typed right before `w`. Circumflex is never reverted by `w` and
    // returns early so an `ô` is left literal rather than re-horned below.
    if is_vowel(last) {
        if let Some(shape) = shape_on_vowel(last) {
            return match shape {
                VowelShape::Breve | VowelShape::Horn => Some(KeyAction::Shape(shape)),
                VowelShape::Circumflex => None,
            };
        }
        if is_plain_vowel(last, 'a') {
            // A `ua`-family cluster horns the `u` (→ `ưa`, and a repeat `w` reverts
            // it); a bare or `oa` `a` takes the breve.
            let shape = if ends_with_ua_horn_target(buffer) {
                VowelShape::Horn
            } else {
                VowelShape::Breve
            };
            return Some(KeyAction::Shape(shape));
        }
        if is_plain_vowel(last, 'o') || is_plain_vowel(last, 'u') {
            return Some(KeyAction::Shape(VowelShape::Horn));
        }
    }

    // Free position: place the breve/horn on whichever nucleus vowel can receive
    // it, wherever it sits — so the key works after the coda (`con` + `w` → `cơn`)
    // or after a trailing vowel that can't itself take the shape (`moi` + `w` →
    // `mơi`, `doi` + `w` → `dơi`), not only on a plain `a`/`o`/`u` typed right
    // before `w`. This lets the user place the horn/breve at any point in the
    // syllable, like VNI's position-free 7/8. A Vietnamese syllable never holds a
    // literal `w`, so this is unambiguous; with no shapeable nucleus (`eng`) it
    // returns None and `w` stays literal. Breve (only `a` takes it) before horn.
    if shape_target_index(buffer, VowelShape::Breve).is_some() {
        return Some(KeyAction::Shape(VowelShape::Breve));
    }
    if shape_target_index(buffer, VowelShape::Horn).is_some() {
        return Some(KeyAction::Shape(VowelShape::Horn));
    }

    None
}

/// A `d`/`đ`/`Đ` already sits before a vowel, so a later `d` keystroke is the
/// stroke modifier wherever it lands. A Vietnamese syllable never has a `d` after
/// its nucleus, so a trailing `d` here is unambiguously the đ modifier, never a
/// letter — letting the user mark `đ` anywhere (`dược`+`d` → `được`) instead of
/// only as the adjacent `dd` at the onset.
fn stroke_target_before_vowel(buffer: &str) -> bool {
    let mut seen_d = false;
    for c in buffer.chars() {
        if matches!(c, 'd' | 'D' | 'đ' | 'Đ') {
            seen_d = true;
        } else if seen_d && is_vowel(c) {
            return true;
        }
    }
    false
}

/// Classify the `d` key into a stroke action. Fires for the adjacent `dd` digraph,
/// the `đ`+`d` revert, and the free-position case (a `d`/`đ` already past the
/// nucleus). The apply layer ([`apply_stroke`]/[`try_revert_stroke`]) then targets
/// the right `d`/`đ` wherever it is.
///
/// [`apply_stroke`]: crate::composition::apply::apply_stroke
/// [`try_revert_stroke`]: crate::composition::revert::try_revert_stroke
fn classify_stroke(buffer: &str, key: char) -> Option<KeyAction> {
    if !key.eq_ignore_ascii_case(&'d') {
        return None;
    }
    if let Some(last) = last_char(buffer)
        && (last.eq_ignore_ascii_case(&'d') || matches!(last, 'đ' | 'Đ'))
    {
        return Some(KeyAction::Stroke);
    }
    if stroke_target_before_vowel(buffer) {
        return Some(KeyAction::Stroke);
    }
    None
}

fn classify_digraph(buffer: &str, key: char) -> Option<KeyAction> {
    let last = last_char(buffer)?;

    for base in ['a', 'e', 'o'] {
        if is_plain_vowel(last, base) && key.eq_ignore_ascii_case(&base) {
            return Some(KeyAction::Shape(VowelShape::Circumflex));
        }
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
    fn classify_remove_tone() {
        assert_eq!(classify_key("á", 'z'), KeyAction::RemoveTone);
        assert_eq!(classify_key("viết", 'z'), KeyAction::RemoveTone);
        assert_eq!(classify_key("", 'z'), KeyAction::RemoveTone);
    }

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
    fn classify_stroke_anywhere_in_syllable() {
        // The `d` key strokes the onset `d`/`đ` even when typed after the rest of
        // the syllable — `dược` + `d` → stroke (→ `được`), not a literal `d`.
        assert_eq!(classify_key("duoc", 'd'), KeyAction::Stroke);
        assert_eq!(classify_key("dược", 'd'), KeyAction::Stroke);
        assert_eq!(classify_key("dang", 'd'), KeyAction::Stroke);
        assert_eq!(classify_key("Dươc", 'd'), KeyAction::Stroke);
        // No `d` in the syllable, or only an onset with no nucleus yet → literal.
        assert_eq!(classify_key("tao", 'd'), KeyAction::Normal);
        assert_eq!(classify_key("nga", 'd'), KeyAction::Normal);
    }

    #[test]
    fn classify_w_anywhere_in_syllable() {
        // `w` typed after the coda still shapes the nucleus — `lam` + `w` → breve
        // (→ `lăm`), `con` + `w` → horn (→ `cơn`).
        assert_eq!(classify_key("lam", 'w'), KeyAction::Shape(VowelShape::Breve));
        assert_eq!(classify_key("an", 'w'), KeyAction::Shape(VowelShape::Breve));
        assert_eq!(classify_key("con", 'w'), KeyAction::Shape(VowelShape::Horn));
        assert_eq!(classify_key("tun", 'w'), KeyAction::Shape(VowelShape::Horn));
        // `uo` compound after a coda already worked and still does.
        assert_eq!(classify_key("nuoc", 'w'), KeyAction::Shape(VowelShape::Horn));
        // No shapeable nucleus → `w` is a literal.
        assert_eq!(classify_key("eng", 'w'), KeyAction::Normal);
        assert_eq!(classify_key("ng", 'w'), KeyAction::Normal);
    }

    #[test]
    fn classify_w_after_trailing_vowel() {
        // `w` typed after a trailing vowel that can't take the shape still horns the
        // earlier nucleus vowel — so the horn can be placed last, `moi` + `w` →
        // `mơi` (the user types `moiwf` for `mời`, not only `mowif`).
        assert_eq!(classify_key("moi", 'w'), KeyAction::Shape(VowelShape::Horn));
        assert_eq!(classify_key("doi", 'w'), KeyAction::Shape(VowelShape::Horn));
        assert_eq!(classify_key("coi", 'w'), KeyAction::Shape(VowelShape::Horn));
        // Breve target wins over horn when present (only `a` takes the breve).
        assert_eq!(classify_key("oa", 'w'), KeyAction::Shape(VowelShape::Breve));
        // A plain vowel typed right before `w` still shapes itself (adjacent rule).
        assert_eq!(classify_key("mo", 'w'), KeyAction::Shape(VowelShape::Horn));
        // `i` alone has no shapeable vowel → literal.
        assert_eq!(classify_key("mi", 'w'), KeyAction::Normal);
    }

    #[test]
    fn classify_w_on_ua_horns_the_u() {
        // Plain `ua` + `w` forms the `ưa` rhyme by horning the `u`, not a breve on
        // the `a` (there is no `uă` rhyme): `nua` + `w` → `nưa` (→ `nữa`),
        // `mua` + `w` → `mưa`, `ngua` + `w` → `ngưa`.
        assert_eq!(classify_key("nua", 'w'), KeyAction::Shape(VowelShape::Horn));
        assert_eq!(classify_key("mua", 'w'), KeyAction::Shape(VowelShape::Horn));
        assert_eq!(classify_key("ngua", 'w'), KeyAction::Shape(VowelShape::Horn));
        assert_eq!(classify_key("ua", 'w'), KeyAction::Shape(VowelShape::Horn));
        // `qu` glide: the `u` is part of the onset, so the `a` takes the breve.
        assert_eq!(classify_key("qua", 'w'), KeyAction::Shape(VowelShape::Breve));
        assert_eq!(classify_key("Qua", 'w'), KeyAction::Shape(VowelShape::Breve));
        // `oa` still breves the `a` (`xoăn`), unaffected by the `ua` rule.
        assert_eq!(classify_key("hoa", 'w'), KeyAction::Shape(VowelShape::Breve));
        // Toned `u` still classifies as horn (the apply layer keeps the tone → `ứa`).
        assert_eq!(classify_key("úa", 'w'), KeyAction::Shape(VowelShape::Horn));
        // Already-horned `ưa` classifies as horn too — the transform layer sees no
        // apply target and reverts it (`nưa` + `w` → `nua`).
        assert_eq!(classify_key("nưa", 'w'), KeyAction::Shape(VowelShape::Horn));
        assert_eq!(classify_key("ưa", 'w'), KeyAction::Shape(VowelShape::Horn));
    }

    #[test]
    fn classify_revert_triggers() {
        assert_eq!(classify_key("â", 'a'), KeyAction::Shape(VowelShape::Circumflex));
        assert_eq!(classify_key("ơ", 'w'), KeyAction::Shape(VowelShape::Horn));
        assert_eq!(classify_key("đ", 'd'), KeyAction::Stroke);
        assert_eq!(classify_key("á", 's'), KeyAction::Tone(Tone::Sac));
    }
}
