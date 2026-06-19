//! Revert helpers — detect a double modifier key and strip the diacritic it
//! would re-apply, returning the bare stem. The caller ([`apply_action`]) then
//! appends the literal key so the user gets back their raw keystrokes
//! (`á` + `1` → `a` here → `a1` in the pipeline).
//!
//! [`apply_action`]: crate::composition::transform::apply_action

use crate::unicode::marks::{tone_on_vowel, vowel_stem, Tone};
use crate::unicode::shapes::{shape_on_vowel, shaped_vowel_index, strip_shape, VowelShape};
use crate::unicode::tone_position::tone_vowel_index;

pub(crate) fn replace_char_at(buffer: &str, char_idx: usize, new_ch: char) -> String {
    let mut chars: Vec<char> = buffer.chars().collect();
    chars[char_idx] = new_ch;
    chars.into_iter().collect()
}

/// Revert `đ`/`Đ` back to `d`/`D` when stroke key `9` is pressed again.
pub fn try_revert_stroke(buffer: &str) -> Option<String> {
    let mut chars: Vec<char> = buffer.chars().collect();
    let last = *chars.last()?;
    let reverted = match last {
        'đ' => 'd',
        'Đ' => 'D',
        _ => return None,
    };
    let len = chars.len();
    chars[len - 1] = reverted;
    Some(chars.into_iter().collect())
}

/// Revert tone when the same tone key is pressed on the toned vowel.
pub fn try_revert_tone(buffer: &str, tone: Tone) -> Option<String> {
    let vowel_idx = tone_vowel_index(buffer)?;
    let vowel = buffer.chars().nth(vowel_idx)?;
    if tone_on_vowel(vowel) != Some(tone) {
        return None;
    }
    let unstemmed = vowel_stem(vowel)?;
    Some(replace_char_at(buffer, vowel_idx, unstemmed))
}

fn ends_with_plain_shaped_uo(buffer: &str) -> bool {
    let chars: Vec<char> = buffer.chars().collect();
    if chars.len() < 2 {
        return false;
    }
    let u = chars[chars.len() - 2];
    let o = chars[chars.len() - 1];
    shape_on_vowel(u) == Some(VowelShape::Horn)
        && shape_on_vowel(o) == Some(VowelShape::Horn)
        && tone_on_vowel(u).is_none()
        && tone_on_vowel(o).is_none()
        && strip_shape(u).is_some_and(|c| c.eq_ignore_ascii_case(&'u'))
        && strip_shape(o).is_some_and(|c| c.eq_ignore_ascii_case(&'o'))
}

fn try_revert_uo_compound(buffer: &str) -> Option<String> {
    if !ends_with_plain_shaped_uo(buffer) {
        return None;
    }

    let mut chars: Vec<char> = buffer.chars().collect();
    let len = chars.len();
    let u = chars[len - 2];
    let o = chars[len - 1];
    chars[len - 2] = strip_shape(u)?;
    chars[len - 1] = strip_shape(o)?;
    Some(chars.into_iter().collect())
}

/// Revert shape when the same shape key is pressed on the shaped vowel.
pub fn try_revert_shape(buffer: &str, shape: VowelShape) -> Option<String> {
    if shape == VowelShape::Horn && let Some(text) = try_revert_uo_compound(buffer) {
        return Some(text);
    }

    let vowel_idx = shaped_vowel_index(buffer, shape)?;
    let vowel = buffer.chars().nth(vowel_idx)?;
    let unstemmed = strip_shape(vowel)?;
    Some(replace_char_at(buffer, vowel_idx, unstemmed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revert_stroke() {
        assert_eq!(try_revert_stroke("đ"), Some("d".into()));
        assert_eq!(try_revert_stroke("Đ"), Some("D".into()));
        assert_eq!(try_revert_stroke("d"), None);
    }

    #[test]
    fn revert_tone() {
        assert_eq!(try_revert_tone("á", Tone::Sac), Some("a".into()));
        assert_eq!(try_revert_tone("hòa", Tone::Huyen), Some("hoa".into()));
        assert_eq!(try_revert_tone("a", Tone::Sac), None);
        assert_eq!(try_revert_tone("à", Tone::Sac), None);
    }

    #[test]
    fn revert_shape() {
        assert_eq!(try_revert_shape("â", VowelShape::Circumflex), Some("a".into()));
        assert_eq!(try_revert_shape("ă", VowelShape::Breve), Some("a".into()));
        assert_eq!(try_revert_shape("ươ", VowelShape::Horn), Some("uo".into()));
        assert_eq!(try_revert_shape("a", VowelShape::Circumflex), None);
    }

    #[test]
    fn revert_tone_keeps_shape() {
        assert_eq!(try_revert_tone("ấ", Tone::Sac), Some("â".into()));
    }
}
