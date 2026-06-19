//! VNI key classification — digit modifiers map to a shared [`KeyAction`].
//!
//! | Key | Action |
//! |-----|--------|
//! | `1`–`5` | tone: sắc, huyền, hỏi, ngã, nặng |
//! | `6`–`8` | shape: mũ (â/ê/ô), móc (ơ/ư), trần (ă) |
//! | `9` | stroke: `đ` |
//! | `0` | remove tone mark (xóa dấu) |
//! | other | normal character |

use crate::input_method::KeyAction;
use crate::unicode::marks::Tone;
use crate::unicode::shapes::VowelShape;

/// Map digit keys 1–5 to tone marks.
pub fn tone_from_digit(key: char) -> Option<Tone> {
    match key {
        '1' => Some(Tone::Sac),
        '2' => Some(Tone::Huyen),
        '3' => Some(Tone::Hoi),
        '4' => Some(Tone::Nga),
        '5' => Some(Tone::Nang),
        _ => None,
    }
}

/// Map digit keys 6–8 to vowel shape modifiers.
pub fn shape_from_digit(key: char) -> Option<VowelShape> {
    match key {
        '6' => Some(VowelShape::Circumflex),
        '7' => Some(VowelShape::Horn),
        '8' => Some(VowelShape::Breve),
        _ => None,
    }
}

/// Classify a VNI keystroke into a method-agnostic [`KeyAction`].
pub fn classify_key(_buffer: &str, key: char) -> KeyAction {
    match key {
        '0' => KeyAction::RemoveTone,
        '9' => KeyAction::Stroke,
        '1'..='5' => KeyAction::Tone(tone_from_digit(key).expect("digit 1-5")),
        '6'..='8' => KeyAction::Shape(shape_from_digit(key).expect("digit 6-8")),
        _ => KeyAction::Normal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_stroke_and_tones() {
        assert_eq!(classify_key("", '9'), KeyAction::Stroke);
        assert_eq!(classify_key("", '1'), KeyAction::Tone(Tone::Sac));
        assert_eq!(classify_key("", '5'), KeyAction::Tone(Tone::Nang));
        assert_eq!(classify_key("", 'm'), KeyAction::Normal);
    }

    #[test]
    fn classify_remove_tone() {
        assert_eq!(classify_key("", '0'), KeyAction::RemoveTone);
    }

    #[test]
    fn classify_shapes() {
        assert_eq!(classify_key("", '6'), KeyAction::Shape(VowelShape::Circumflex));
        assert_eq!(classify_key("", '7'), KeyAction::Shape(VowelShape::Horn));
        assert_eq!(classify_key("", '8'), KeyAction::Shape(VowelShape::Breve));
    }
}
