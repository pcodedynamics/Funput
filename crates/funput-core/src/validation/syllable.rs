//! Syllable-structure validation for modifier keys (tone / shape / stroke).
//!
//! Decides whether a modifier should apply, be ignored, or pass through as a
//! literal key (non-Vietnamese structure the engine restores later).

use crate::unicode::marks::{tone_on_vowel, vowel_stem, Tone};
use crate::validation::parse::{is_valid_onset, parse_syllable};
use crate::validation::rhyme::{self, is_valid_rhyme};

/// Result of validating a modifier keystroke against the current buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierValidation {
    /// Apply Vietnamese transform.
    Allow,
    /// No valid target — discard key.
    Ignored,
    /// Non-Vietnamese structure — append key literally (engine restores later).
    PassThrough,
}

const VALID_CODAS: &[&str] = &["", "c", "ch", "m", "n", "ng", "nh", "p", "t"];

/// Stop (oral plosive) codas. A syllable ending in one of these may only carry
/// the sắc or nặng tone — a Vietnamese phonotactic rule. This is what tells
/// English `text` (→ `tẽt`, ngã + `t`) apart from real syllables like `tét`.
const STOP_CODAS: &[&str] = &["c", "ch", "p", "t"];

/// The tone carried by the nucleus (at most one toned vowel), if any.
fn nucleus_tone(nucleus: &str) -> Option<Tone> {
    nucleus.chars().find_map(tone_on_vowel)
}

/// Toneless rhyme (vần) = nucleus with tones stripped (shape kept) + coda, lowercased.
/// `tiền` → `iên`, `trường` → `ương`.
fn toneless_rhyme(nucleus: &str, coda: &str) -> String {
    let mut rhyme = String::new();
    for ch in nucleus.chars() {
        let stem = vowel_stem(ch).unwrap_or(ch);
        rhyme.extend(char::to_lowercase(stem));
    }
    rhyme.push_str(&coda.to_lowercase());
    rhyme
}

fn violates_ckg_spelling(onset: &str, nucleus: &str) -> bool {
    let Some(first) = nucleus.chars().next().and_then(vowel_stem) else {
        return false;
    };
    let stem = char::to_lowercase(first).next().unwrap_or(first);

    match onset.to_lowercase().as_str() {
        "c" => !matches!(stem, 'a' | 'ă' | 'â' | 'o' | 'ô' | 'ơ' | 'u' | 'ư'),
        // `k` precedes the front vowels e, ê, i, y (kẻ, kê, kim, kỳ/ký/kỹ).
        "k" => !matches!(stem, 'e' | 'ê' | 'i' | 'y'),
        // `g` + `i` is the valid `gi` digraph (gì, gìn); `g` + e/ê uses `gh`.
        "g" => !matches!(stem, 'a' | 'ă' | 'â' | 'o' | 'ô' | 'ơ' | 'u' | 'ư' | 'i'),
        "gh" => !matches!(stem, 'e' | 'ê' | 'i'),
        "ngh" => !matches!(stem, 'e' | 'ê' | 'i'),
        _ => false,
    }
}

fn validate_modifier(buffer: &str) -> ModifierValidation {
    let parts = parse_syllable(buffer);

    if parts.invalid_onset || (!parts.onset.is_empty() && !is_valid_onset(&parts.onset.to_lowercase()))
    {
        return ModifierValidation::PassThrough;
    }

    if parts.nucleus.is_empty() {
        return ModifierValidation::Ignored;
    }

    if violates_ckg_spelling(&parts.onset, &parts.nucleus) {
        return ModifierValidation::PassThrough;
    }

    // Two or more trailing consonants can't form a Vietnamese coda → likely an
    // English word, pass the key through. A single trailing consonant is allowed
    // (the user may still be typing, e.g. "mix" → "mĩx").
    let coda_lower = parts.coda.to_lowercase();
    if parts.coda.chars().count() >= 2 && !VALID_CODAS.contains(&coda_lower.as_str()) {
        return ModifierValidation::PassThrough;
    }

    ModifierValidation::Allow
}

/// Validate tone key (1–5) against the current buffer.
pub fn validate_tone(buffer: &str) -> ModifierValidation {
    validate_modifier(buffer)
}

/// Validate shape key (6–8) against the current buffer.
pub fn validate_shape(buffer: &str) -> ModifierValidation {
    validate_modifier(buffer)
}

/// Validate stroke key (9) against the current buffer.
pub fn validate_stroke(buffer: &str) -> ModifierValidation {
    match buffer.chars().last() {
        Some('d' | 'D') => ModifierValidation::Allow,
        _ => ModifierValidation::Ignored,
    }
}

/// Returns true if the syllable structure is valid for transform.
///
/// **Lenient** (mid-typing): a single trailing consonant is accepted because the
/// user may still be typing (e.g. `mix` → allow, so `mĩx` can compose). For a
/// finished word use [`is_complete_syllable`].
pub fn is_valid(buffer: &str) -> bool {
    matches!(validate_modifier(buffer), ModifierValidation::Allow)
}

/// Returns true if `buffer` is a *complete* valid Vietnamese syllable.
///
/// **Strict**: the coda must be a real Vietnamese final (`c ch m n ng nh p t`),
/// and a **stop coda** (`p t c ch`) only with the sắc or nặng tone (phonotactics).
/// No "still typing" leniency. Use this at a word boundary — the engine restores
/// the raw word when a finished word is *not* a complete syllable: `cảd` (card),
/// `côl` (cool), `tẽt` (text).
pub fn is_complete_syllable(buffer: &str) -> bool {
    let parts = parse_syllable(buffer);

    let structure_ok = !parts.invalid_onset
        && (parts.onset.is_empty() || is_valid_onset(&parts.onset.to_lowercase()))
        && !parts.nucleus.is_empty()
        && !violates_ckg_spelling(&parts.onset, &parts.nucleus)
        && VALID_CODAS.contains(&parts.coda.to_lowercase().as_str());
    if !structure_ok {
        return false;
    }

    // The nucleus+coda must be a real Vietnamese rhyme (Level 2): keeps `việt`,
    // `trường` … but reverts structurally-ok-but-nonexistent rhymes.
    if !is_valid_rhyme(&toneless_rhyme(&parts.nucleus, &parts.coda)) {
        return false;
    }

    // Phonotactics: a stop coda only allows sắc / nặng. Flags `tẽt` (English
    // "text"), `bèct`, etc. as not-a-syllable so the engine restores the raw word.
    if STOP_CODAS.contains(&parts.coda.to_lowercase().as_str()) {
        return matches!(nucleus_tone(&parts.nucleus), Some(Tone::Sac | Tone::Nang));
    }

    true
}

/// Plain base of a vowel — tone **and** shape stripped (`ớ`→`o`, `ẽ`→`e`, `ư`→`u`).
/// Non-vowels pass through lowercased.
fn plain_base(c: char) -> char {
    let stem = vowel_stem(c).unwrap_or(c);
    match char::to_lowercase(stem).next().unwrap_or(stem) {
        'ă' | 'â' => 'a',
        'ê' => 'e',
        'ô' | 'ơ' => 'o',
        'ư' => 'u',
        other => other,
    }
}

/// Strip tone and shape from every char (`ướng` → `uong`).
fn deshape(s: &str) -> String {
    s.chars().map(plain_base).collect()
}

/// True when `buffer` can **no longer** become a valid Vietnamese syllable by
/// typing more — used for *eager* English restore (flip back to the raw
/// keystrokes the instant a word is unrecoverable, without waiting for a boundary):
/// `tẽt`→`text` on the closing `t`, `caé`→`case` on the `e`, `luuỷ`→`luxury`.
///
/// Conservative. The rhyme so far is compared **deshaped** against the deshaped
/// rhyme inventory, so a plain vowel still awaiting its shape stays alive (`ưo`
/// matches `ươ…`, so typing `nước` via `nuwowcs` is never interrupted). Dead when:
/// 1. The deshaped nucleus+coda is not a prefix of any rhyme: `ae`, `uuy`, `ad`.
/// 2. A **stop** coda already carries a *wrong* tone — huyền / hỏi / ngã: `tẽt`.
///    (A stop coda with no tone yet stays alive — the tone follows the coda.)
pub fn is_definitely_invalid(buffer: &str) -> bool {
    let parts = parse_syllable(buffer);
    if parts.nucleus.is_empty() {
        return false; // still building the onset
    }

    let rhyme_query = format!("{}{}", deshape(&parts.nucleus), deshape(&parts.coda));
    let reachable = rhyme::all()
        .iter()
        .any(|r| deshape(r).starts_with(&rhyme_query));
    if !reachable {
        return true;
    }

    if STOP_CODAS.contains(&parts.coda.to_lowercase().as_str()) {
        return matches!(
            nucleus_tone(&parts.nucleus),
            Some(Tone::Huyen | Tone::Hoi | Tone::Nga)
        );
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_tone_cases() {
        assert_eq!(validate_tone("ng"), ModifierValidation::Ignored);
        assert_eq!(validate_tone("text"), ModifierValidation::PassThrough);
        assert_eq!(validate_tone("mix"), ModifierValidation::Allow);
        assert_eq!(validate_tone("ma"), ModifierValidation::Allow);
        assert_eq!(validate_tone("zt"), ModifierValidation::PassThrough);
    }

    #[test]
    fn validate_stroke_cases() {
        assert_eq!(validate_stroke("d"), ModifierValidation::Allow);
        assert_eq!(validate_stroke("x"), ModifierValidation::Ignored);
    }

    #[test]
    fn is_valid_cases() {
        assert!(is_valid("má"));
        assert!(is_valid("ma"));
        assert!(!is_valid("ábc"));
        assert!(!is_valid("text"));
    }

    #[test]
    fn is_complete_syllable_cases() {
        // Complete Vietnamese syllables. `k` + `y` (kỳ/ký/kỹ) and the triphthong
        // `ngoài` are regression guards for tone-placement / ckg-spelling fixes.
        for ok in [
            "má", "ma", "tét", "việt", "trường", "quá", "ăn", "nhanh", "kỳ", "ký", "kỹ", "ngoài",
        ] {
            assert!(is_complete_syllable(ok), "{ok} should be complete");
        }
        // Invalid finals — a finished word ending in a non-Vietnamese coda.
        for bad in ["cảd", "côl", "máz", "hảd", "ng", "abc", "text"] {
            assert!(!is_complete_syllable(bad), "{bad} should be incomplete");
        }
        // Stricter than `is_valid`: single trailing `d`/`z` is lenient-valid but
        // not a complete syllable.
        assert!(is_valid("cảd"));
        assert!(!is_complete_syllable("cảd"));
    }

    #[test]
    fn real_syllables_are_complete() {
        // Broad battery of real Vietnamese syllables (incl. hard rhymes). A failure
        // means the rhyme table is missing an entry — add it to `rhyme.rs`.
        let words = "\
            a ba cá chè dê đi em gà gh ghê gì hoa khô là mẹ nó ô phở quà rể sữa tô \
            uô(no) việt nghĩa người trường nước được rượu hươu khuya khuỷu quýnh quyên \
            nguyệt khuếch doanh hoạch bâng khuâng ngoằn ngoèo tuềnh toàng xoèn xoẹt \
            muốn muống thuốc nhuộm tuốt cướp lướt mượn đường riêng tiếng chuông \
            anh ánh ách inh tính kịch lệnh xanh sạch huỳnh quỳnh xoong(no) \
            hoàng khoảng nguyên nguyền quyết tuyết duyên xuân xuất bâng khuâng \
            ngoắt ngoéo ngoạm ngoạp ngoạc ngoắc loắt choắt \
            cằn nhằn lẳng lặng phưng phức nưng nửng \
            tay hai cao sau cau mây đây kẹo kêu cừu mưu líu xíu";
        for w in words.split_whitespace() {
            if w.ends_with("(no)") {
                continue;
            }
            // skip onset-only fragments used as spacers
            if w == "gh" {
                continue;
            }
            assert!(
                is_complete_syllable(w),
                "{w} (rhyme {:?}) should be a complete syllable",
                {
                    let p = parse_syllable(w);
                    toneless_rhyme(&p.nucleus, &p.coda)
                }
            );
        }
    }

    #[test]
    fn definitely_invalid_detects_dead_ends() {
        // Dead ends — unreachable rhyme (incl. open clusters), or stop coda +
        // wrong (huyền/hỏi/ngã) tone.
        for dead in ["tẽt", "tèt", "cảd", "máz", "pèect", "ábc", "caé", "luuỷ"] {
            assert!(is_definitely_invalid(dead), "{dead} should be a dead end");
        }
        // Alive: still typing, already valid, OR a stop coda awaiting its tone
        // (`nuoc`/`nươc`/`côt` → user types the tone after the coda).
        for alive in [
            "tẽ", "te", "ng", "ngh", "cả", "cản", "việt", "má", "trươ", "nuoc", "nươc", "côt", "tét",
        ] {
            assert!(!is_definitely_invalid(alive), "{alive} should stay alive");
        }
    }

    #[test]
    fn stop_coda_only_allows_sac_or_nang() {
        // Legal: stop coda with sắc or nặng.
        for ok in ["tét", "tẹt", "sách", "học", "đẹp", "việt", "nước"] {
            assert!(is_complete_syllable(ok), "{ok} should be complete");
        }
        // Illegal: stop coda with ngang / huyền / hỏi / ngã — the signal that
        // catches English words like `text` (→ `tẽt`) or `coot` (→ `côt`).
        for bad in ["tẽt", "tèt", "tẻt", "côt", "sàch", "mãc"] {
            assert!(!is_complete_syllable(bad), "{bad} should be incomplete");
        }
        // Sonorant codas keep all tones legal.
        for ok in ["làng", "mển", "ngã", "cũng"] {
            assert!(is_complete_syllable(ok), "{ok} should be complete");
        }
    }

    #[test]
    fn ckg_spelling() {
        assert_eq!(validate_tone("ke"), ModifierValidation::Allow);
        assert_eq!(validate_tone("ka"), ModifierValidation::PassThrough);
        assert_eq!(validate_tone("ca"), ModifierValidation::Allow);
        // `gi` digraph stays valid, `ge` would need `gh`.
        assert_eq!(validate_tone("gi"), ModifierValidation::Allow);
    }
}
