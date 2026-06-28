//! Tone mark placement — modern Vietnamese reposition rules.

use crate::unicode::marks::{apply_tone_to_vowel, is_vowel, tone_on_vowel, vowel_stem};
use crate::unicode::shapes::{apply_shape, shape_on_vowel, VowelShape};
use crate::ToneStyle;

struct VowelCluster {
    indices: Vec<usize>,
}

/// True if the vowel at `idx` is the onset glide of `qu`/`gi` (the `u`/`i` belongs
/// to the leading consonant, not the tonal nucleus).
fn is_onset_glide(chars: &[char], idx: usize) -> bool {
    if idx == 0 {
        return false;
    }
    let vowel = chars[idx];
    let prev = chars[idx - 1];
    (vowel.eq_ignore_ascii_case(&'u') && prev.eq_ignore_ascii_case(&'q'))
        || (vowel.eq_ignore_ascii_case(&'i') && prev.eq_ignore_ascii_case(&'g'))
}

fn vowel_cluster(buffer: &str) -> Option<VowelCluster> {
    let chars: Vec<char> = buffer.chars().collect();
    let start = chars.iter().position(|c| is_vowel(*c))?;
    let mut indices = Vec::new();

    for (i, ch) in chars.iter().enumerate().skip(start) {
        if is_vowel(*ch) {
            indices.push(i);
        } else {
            break;
        }
    }

    if indices.is_empty() {
        return None;
    }

    // Drop the onset glide of `qu`/`gi` when a real nucleus vowel follows
    // (e.g. `qua` → tone on `a`, `gia` → tone on `a`), but keep it when it is
    // the only vowel (e.g. `gì`, `gìn`).
    if indices.len() >= 2 && is_onset_glide(&chars, indices[0]) {
        indices.remove(0);
    }

    Some(VowelCluster { indices })
}

/// Which vowel in the cluster receives the tone (0-based within cluster) — the
/// **traditional** ("kiểu cũ") rule. Only consulted for clusters with no shaped
/// vowel (those take priority and are handled in [`tone_vowel_index`]):
///
/// - 1 vowel → on it.
/// - 2 vowels, **open** (no final consonant) → first vowel: `hòa`, `ùy`, `mía`.
/// - 2 vowels **+ final consonant**, or 3 vowels → second vowel: `toán`, `ngoài`.
fn tone_offset_in_cluster(cluster_len: usize, has_coda: bool) -> usize {
    match cluster_len {
        1 => 0,
        2 if has_coda => 1,
        2 => 0,
        _ => 1, // triphthong → middle vowel
    }
}

/// True for the open glide-initial diphthongs where "kiểu mới" moves the tone onto
/// the second (main) vowel: `oa`, `oe`, `uy` — the only syllables on which the
/// modern and traditional styles disagree. Compared on the stem so a vowel that
/// already carries a tone (reposition) still matches (`hoà` → `o`, `à`→stem `a`).
fn modern_open_pair_takes_second(first: char, second: char) -> bool {
    let f = vowel_stem(first).unwrap_or(first).to_ascii_lowercase();
    let s = vowel_stem(second).unwrap_or(second).to_ascii_lowercase();
    matches!((f, s), ('o', 'a') | ('o', 'e') | ('u', 'y'))
}

/// Char index where a tone mark should be placed, for the given placement `style`.
pub fn tone_vowel_index(buffer: &str, style: ToneStyle) -> Option<usize> {
    let cluster = vowel_cluster(buffer)?;
    let chars: Vec<char> = buffer.chars().collect();

    // A vowel carrying mũ/móc/trần (â ê ô ơ ư ă) takes the tone. For `ươ` (two
    // horned vowels) the tone sits on the second one (`ơ`): trường, được, rượu.
    let last_shaped = cluster
        .indices
        .iter()
        .copied()
        .rfind(|&i| shape_on_vowel(chars[i]).is_some());
    if let Some(i) = last_shaped {
        return Some(i);
    }

    // No shaped vowel: structural rule. A coda exists when a (consonant) char
    // follows the last vowel of the cluster.
    let last_vowel = *cluster.indices.last().expect("cluster is non-empty");
    let has_coda = last_vowel + 1 < chars.len();

    // "Kiểu mới": an open `oa`/`oe`/`uy` takes the tone on the second vowel.
    if style == ToneStyle::Modern
        && cluster.indices.len() == 2
        && !has_coda
        && modern_open_pair_takes_second(chars[cluster.indices[0]], chars[cluster.indices[1]])
    {
        return Some(cluster.indices[1]);
    }

    let offset = tone_offset_in_cluster(cluster.indices.len(), has_coda);
    Some(cluster.indices[offset.min(cluster.indices.len() - 1)])
}

/// Vowel character used when applying a tone (handles `ie`/`gie` → tonal base `ê`).
///
/// Plain `e` preceded by `i` forms the rising diphthong written with a circumflex
/// (`viết`, `giết`), so the tone lands on `ê`. This covers both the `i` nucleus of
/// `viet` and the `i` that is part of the `gi` onset of `giet`.
pub fn tone_target_vowel(buffer: &str, vowel_idx: usize) -> Option<char> {
    let chars: Vec<char> = buffer.chars().collect();
    let vowel = *chars.get(vowel_idx)?;
    let stem = vowel_stem(vowel)?;
    if !stem.eq_ignore_ascii_case(&'e') {
        return Some(vowel);
    }
    // The preceding `i` may already carry a tone (`vịe` + `t` → `việt`), so compare
    // its stem, not the raw char.
    let prev_is_i = vowel_idx > 0
        && vowel_stem(chars[vowel_idx - 1]).is_some_and(|s| s.eq_ignore_ascii_case(&'i'));
    if !prev_is_i {
        return Some(vowel);
    }

    // `apply_shape` already preserves case (`e` → `ê`, `E` → `Ê`).
    apply_shape(stem, VowelShape::Circumflex)
}

/// If a tone exists on the wrong vowel, move it to the correct position for `style`.
pub fn reposition_existing_tone(buffer: &str, style: ToneStyle) -> Option<String> {
    let desired = tone_vowel_index(buffer, style)?;

    let mut toned_index: Option<(usize, crate::unicode::marks::Tone)> = None;
    for (i, ch) in buffer.chars().enumerate() {
        if is_vowel(ch) && let Some(tone) = tone_on_vowel(ch) {
            toned_index = Some((i, tone));
            break;
        }
    }

    let (current, tone) = toned_index?;
    if current == desired {
        return None;
    }

    let mut chars: Vec<char> = buffer.chars().collect();
    let old_stem = vowel_stem(chars[current])?;
    chars[current] = old_stem;

    let new_vowel = chars[desired];
    let tone_base = tone_target_vowel(buffer, desired).unwrap_or(new_vowel);
    let new_stem = vowel_stem(tone_base)?;
    let toned = apply_tone_to_vowel(new_stem, tone)?;
    chars[desired] = toned;

    Some(chars.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn char_at(buffer: &str, index: usize) -> char {
        buffer.chars().nth(index).expect("char at index")
    }

    /// `tone_vowel_index` with the default (traditional) style.
    fn trad(buffer: &str) -> usize {
        tone_vowel_index(buffer, ToneStyle::Traditional).unwrap()
    }

    /// `tone_vowel_index` with the modern ("kiểu mới") style.
    fn modern(buffer: &str) -> usize {
        tone_vowel_index(buffer, ToneStyle::Modern).unwrap()
    }

    #[test]
    fn tone_vowel_index_open_diphthong_first_vowel() {
        // Traditional rule: open 2-vowel cluster → tone on the first vowel.
        assert_eq!(char_at("hoa", trad("hoa")), 'o'); // hòa
        assert_eq!(char_at("chao", trad("chao")), 'a'); // chào
        assert_eq!(char_at("hoe", trad("hoe")), 'o'); // hòe
    }

    #[test]
    fn tone_vowel_index_uy_open_is_first_vowel() {
        // Open `uy` → tone on `u` (ùy/úy), traditional style.
        assert_eq!(char_at("thuy", trad("thuy")), 'u');
    }

    #[test]
    fn tone_vowel_index_oa_with_coda_is_second_vowel() {
        // 2 vowels + final consonant → second vowel: hoàn, toán.
        assert_eq!(char_at("hoan", trad("hoan")), 'a');
        assert_eq!(char_at("toan", trad("toan")), 'a');
    }

    #[test]
    fn tone_vowel_index_single_vowel() {
        assert_eq!(char_at("ma", trad("ma")), 'a');
        assert_eq!(char_at("ho", trad("ho")), 'o');
    }

    #[test]
    fn tone_vowel_index_uo_horn_cluster() {
        assert_eq!(char_at("trương", trad("trương")), 'ơ');
        assert_eq!(char_at("thuơ", trad("thuơ")), 'ơ');
    }

    #[test]
    fn tone_vowel_index_open_diphthongs_ia_ua() {
        assert_eq!(char_at("mia", trad("mia")), 'i');
        assert_eq!(char_at("mua", trad("mua")), 'u');
        assert_eq!(char_at("cua", trad("cua")), 'u');
        assert_eq!(char_at("lua", trad("lua")), 'u');
    }

    #[test]
    fn tone_vowel_index_uoi_cluster() {
        assert_eq!(char_at("ngươi", trad("ngươi")), 'ơ');
    }

    #[test]
    fn tone_vowel_index_plain_triphthong_is_middle() {
        // Plain triphthongs take the tone on the middle vowel, not the last.
        assert_eq!(char_at("ngoai", trad("ngoai")), 'a'); // ngoài
        assert_eq!(char_at("xoay", trad("xoay")), 'a'); // xoáy
        assert_eq!(char_at("khuyu", trad("khuyu")), 'y'); // khuỷu
    }

    #[test]
    fn tone_vowel_index_modern_moves_oa_oe_uy_to_second() {
        // "Kiểu mới": open oa/oe/uy → tone on the second (main) vowel.
        assert_eq!(char_at("hoa", modern("hoa")), 'a'); // hoà
        assert_eq!(char_at("hoe", modern("hoe")), 'e'); // hoè
        assert_eq!(char_at("thuy", modern("thuy")), 'y'); // thuý
    }

    #[test]
    fn tone_vowel_index_modern_leaves_other_clusters_unchanged() {
        // Only oa/oe/uy differ — everything else matches the traditional rule.
        assert_eq!(char_at("mia", modern("mia")), 'i'); // mía
        assert_eq!(char_at("mua", modern("mua")), 'u'); // múa
        assert_eq!(char_at("chao", modern("chao")), 'a'); // chào
        assert_eq!(char_at("hoan", modern("hoan")), 'a'); // hoàn (coda)
        assert_eq!(char_at("ngoai", modern("ngoai")), 'a'); // ngoài (triphthong)
        assert_eq!(char_at("trương", modern("trương")), 'ơ'); // shaped vowel wins
    }

    #[test]
    fn tone_target_vowel_ie_uses_circumflex_e() {
        assert_eq!(tone_target_vowel("viet", 2), Some('ê'));
        assert_eq!(tone_target_vowel("lien", 2), Some('ê'));
    }

    #[test]
    fn reposition_moves_tone_to_first_vowel_of_open_diphthong() {
        // Traditional: an open `oa` takes the tone on `o`, so `hoà` → `hòa`.
        assert_eq!(
            reposition_existing_tone("hoà", ToneStyle::Traditional).as_deref(),
            Some("hòa")
        );
    }

    #[test]
    fn reposition_modern_moves_tone_to_second_vowel() {
        // Modern: an open `oa` takes the tone on `a`, so `hòa` → `hoà`.
        assert_eq!(
            reposition_existing_tone("hòa", ToneStyle::Modern).as_deref(),
            Some("hoà")
        );
        // Already correct for the style → no change.
        assert_eq!(reposition_existing_tone("hoà", ToneStyle::Modern), None);
    }
}
