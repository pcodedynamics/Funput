//! Vietnamese tone marks and shared vowel helpers (tone lives here; shape lives
//! in [`crate::unicode::shapes`]).

use crate::unicode::vowels;

/// Vietnamese tone marks (thanh điệu).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Sac,
    Huyen,
    Hoi,
    Nga,
    Nang,
}

impl Tone {
    fn index(self) -> usize {
        match self {
            Tone::Sac => 0,
            Tone::Huyen => 1,
            Tone::Hoi => 2,
            Tone::Nga => 3,
            Tone::Nang => 4,
        }
    }

    fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Tone::Sac),
            1 => Some(Tone::Huyen),
            2 => Some(Tone::Hoi),
            3 => Some(Tone::Nga),
            4 => Some(Tone::Nang),
            _ => None,
        }
    }
}

/// Apply a tone mark to a vowel character.
pub fn apply_tone(base: char, tone: Tone) -> Option<char> {
    vowels::toned_vowel(base, tone.index())
}

/// Apply a tone to any vowel, replacing an existing tone if present.
pub fn apply_tone_to_vowel(vowel: char, tone: Tone) -> Option<char> {
    let stem = vowel_stem(vowel)?;
    apply_tone(stem, tone)
}

/// Convert d/D to đ/Đ stroke letter.
pub fn stroke_d(c: char) -> Option<char> {
    match c {
        'd' => Some('đ'),
        'D' => Some('Đ'),
        _ => None,
    }
}

/// Returns true if `c` is a vowel (ASCII or precomposed Vietnamese).
pub fn is_vowel(c: char) -> bool {
    vowels::is_vowel(c)
}

/// Index of the vowel where a tone mark should be placed (modern Vietnamese rules).
#[allow(dead_code)] // Public API — used by engine and external callers.
pub fn main_vowel_index(syllable: &str) -> Option<usize> {
    crate::unicode::tone_position::tone_vowel_index(syllable)
}

/// Detect tone on a vowel character, if any.
pub(crate) fn tone_on_vowel(c: char) -> Option<Tone> {
    let index = vowels::tone_index_on_vowel(c)?;
    Tone::from_index(index)
}

/// Strip tone from a vowel, keeping shape (e.g. `á` → `a`, `ấ` → `â`).
pub(crate) fn vowel_stem(c: char) -> Option<char> {
    vowels::vowel_stem(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_tone_basic_vowels() {
        for (base, sac, huyen, hoi, nga, nang) in [
            ('a', 'á', 'à', 'ả', 'ã', 'ạ'),
            ('e', 'é', 'è', 'ẻ', 'ẽ', 'ẹ'),
            ('i', 'í', 'ì', 'ỉ', 'ĩ', 'ị'),
            ('o', 'ó', 'ò', 'ỏ', 'õ', 'ọ'),
            ('u', 'ú', 'ù', 'ủ', 'ũ', 'ụ'),
            ('y', 'ý', 'ỳ', 'ỷ', 'ỹ', 'ỵ'),
        ] {
            assert_eq!(apply_tone(base, Tone::Sac), Some(sac));
            assert_eq!(apply_tone(base, Tone::Huyen), Some(huyen));
            assert_eq!(apply_tone(base, Tone::Hoi), Some(hoi));
            assert_eq!(apply_tone(base, Tone::Nga), Some(nga));
            assert_eq!(apply_tone(base, Tone::Nang), Some(nang));
        }
    }

    #[test]
    fn apply_tone_on_shaped_vowels() {
        assert_eq!(apply_tone('â', Tone::Sac), Some('ấ'));
        assert_eq!(apply_tone('ơ', Tone::Huyen), Some('ờ'));
        assert_eq!(apply_tone('ư', Tone::Nang), Some('ự'));
    }

    #[test]
    fn stroke_d_maps_correctly() {
        assert_eq!(stroke_d('d'), Some('đ'));
        assert_eq!(stroke_d('D'), Some('Đ'));
        assert_eq!(stroke_d('x'), None);
    }

    #[test]
    fn main_vowel_index_delegates_to_tone_rules() {
        assert_eq!(main_vowel_index("hoa"), Some(1)); // hòa — tone on `o`
        assert_eq!(main_vowel_index("chao"), Some(2));
        assert_eq!(main_vowel_index("ma"), Some(1));
        assert_eq!(main_vowel_index("ng"), None);
    }
}
