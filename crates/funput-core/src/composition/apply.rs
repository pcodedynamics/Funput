//! Per-action buffer transforms (stroke, tone, shape).
//!
//! Method-agnostic: these operate on a [`crate::input_method::KeyAction`] already
//! resolved by a classifier, so VNI and Telex share them unchanged.

use crate::composition::replace_char_at;
use crate::unicode::marks::{apply_tone_to_vowel, is_vowel, stroke_d, tone_on_vowel, vowel_stem, Tone};
use crate::unicode::shapes::{apply_shape, apply_shape_to_vowel, shape_target_index, VowelShape};
use crate::unicode::tone_position::{tone_target_vowel, tone_vowel_index};
use crate::{ToneStyle, TransformKind, TransformResult};

fn ignored(buffer: &str) -> TransformResult {
    TransformResult {
        kind: TransformKind::Ignored,
        text: buffer.to_owned(),
    }
}

/// Turn a `d`/`D` into `đ`/`Đ` so the key works wherever it is typed: `dang` + `9`
/// → `đang`, not only `d` + `9`. Targets the **last** `d` in the buffer — a
/// Vietnamese syllable has at most one `d` (always the onset), and in an
/// abbreviation run the last one is the most recent onset (`GD` + `9` → `GĐ`,
/// `GDD` → `GĐ`).
pub(crate) fn apply_stroke(buffer: &str) -> TransformResult {
    let mut chars: Vec<char> = buffer.chars().collect();
    let Some(idx) = chars.iter().rposition(|c| matches!(c, 'd' | 'D')) else {
        return ignored(buffer);
    };
    chars[idx] = stroke_d(chars[idx]).expect("d/D always strokes");
    TransformResult {
        kind: TransformKind::Applied,
        text: chars.into_iter().collect(),
    }
}

/// Place `tone` on the nucleus vowel (handles reposition and `ie`/`gie` → `ê`).
pub(crate) fn apply_tone_key(buffer: &str, tone: Tone, style: ToneStyle) -> TransformResult {
    let Some(vowel_idx) = tone_vowel_index(buffer, style) else {
        return ignored(buffer);
    };

    let vowel = buffer.chars().nth(vowel_idx).expect("vowel index in bounds");
    let tone_target = tone_target_vowel(buffer, vowel_idx).unwrap_or(vowel);
    let Some(toned) = apply_tone_to_vowel(tone_target, tone) else {
        return ignored(buffer);
    };

    TransformResult {
        kind: TransformKind::Applied,
        text: replace_char_at(buffer, vowel_idx, toned),
    }
}

/// Remove the tone mark from the syllable, keeping any shape (`việt` → `viêt`,
/// `toán` → `toan`, `được` → `đươc`). Returns `None` when there is no tone.
pub(crate) fn remove_tone(buffer: &str) -> Option<String> {
    let mut chars: Vec<char> = buffer.chars().collect();
    let idx = chars.iter().position(|&c| tone_on_vowel(c).is_some())?;
    chars[idx] = vowel_stem(chars[idx])?; // strips tone, keeps shape
    Some(chars.into_iter().collect())
}

/// True if `shape` can still be applied to some vowel in `buffer` (the horn
/// `uo` compound, or a single vowel that can receive the shape).
pub(crate) fn shape_apply_target_exists(buffer: &str, shape: VowelShape) -> bool {
    if shape == VowelShape::Horn && uo_pair_in_vowel_cluster(buffer).is_some() {
        return true;
    }
    shape_target_index(buffer, shape).is_some()
}

/// Apply a vowel shape. For an adjacent plain `uo`, initially horn only the `o`
/// (`uơ`): that is the completed open rhyme in `thuở`/`huơ`/`quơ`. If another
/// vowel or coda arrives, [`complete_uo_horn_for_continuation`] turns it into
/// `ươ` (`thương`, `hươu`).
pub(crate) fn apply_shape_key(buffer: &str, shape: VowelShape) -> TransformResult {
    if shape == VowelShape::Horn
        && let Some(text) = apply_uo_compound(buffer)
    {
        return TransformResult {
            kind: TransformKind::Applied,
            text,
        };
    }

    let Some(vowel_idx) = shape_target_index(buffer, shape) else {
        return ignored(buffer);
    };

    let vowel = buffer.chars().nth(vowel_idx).expect("vowel index in bounds");
    let Some(shaped) = apply_shape_to_vowel(vowel, shape) else {
        return ignored(buffer);
    };

    TransformResult {
        kind: TransformKind::Applied,
        text: replace_char_at(buffer, vowel_idx, shaped),
    }
}

/// Char indices of an adjacent plain `uo` pair inside the vowel cluster.
pub(crate) fn uo_pair_in_vowel_cluster(buffer: &str) -> Option<(usize, usize)> {
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

    // A plain (untoned, unshaped) `u` followed by a plain `o`. The ASCII match
    // already excludes `ư`, `ơ`, and any toned variants.
    for pair in indices.windows(2) {
        let (u_idx, o_idx) = (pair[0], pair[1]);
        if chars[u_idx].eq_ignore_ascii_case(&'u') && chars[o_idx].eq_ignore_ascii_case(&'o') {
            return Some((u_idx, o_idx));
        }
    }
    None
}

fn apply_uo_compound(buffer: &str) -> Option<String> {
    let (u_idx, o_idx) = uo_pair_in_vowel_cluster(buffer)?;
    let mut chars: Vec<char> = buffer.chars().collect();
    let has_continuation = o_idx + 1 < chars.len();
    let is_qu_glide = u_idx > 0 && chars[u_idx - 1].eq_ignore_ascii_case(&'q');
    if has_continuation && !is_qu_glide {
        chars[u_idx] = apply_shape(chars[u_idx], VowelShape::Horn)?;
    }
    chars[o_idx] = apply_shape(chars[o_idx], VowelShape::Horn)?;
    Some(chars.into_iter().collect())
}

/// Return the byte offset and character of the plain `u` when `buffer` ends in
/// ambiguous `uơ` (including a tone on `ơ`), plus the character before `u`.
/// The ASCII-byte guard makes the overwhelmingly common path constant-time.
fn open_uo_horn_suffix(buffer: &str) -> Option<(usize, char, Option<char>)> {
    if buffer.as_bytes().last().is_none_or(u8::is_ascii) {
        return None;
    }

    let mut chars = buffer.char_indices().rev();
    let (_, o) = chars.next()?;
    let (u_offset, u) = chars.next()?;
    let before_u = chars.next().map(|(_, ch)| ch);
    let is_plain_u = vowel_stem(u).is_some_and(|stem| stem.eq_ignore_ascii_case(&'u'))
        && crate::unicode::shapes::shape_on_vowel(u).is_none();
    let is_horned_o = vowel_stem(o).is_some_and(|stem| stem.eq_ignore_ascii_case(&'ơ'));
    (is_plain_u && is_horned_o).then_some((u_offset, u, before_u))
}

/// Complete an ambiguous open `uơ` as `ươ` once another character proves that
/// the rhyme continues (`thuơ` + `n` → `thươn`, `huơ` + `u` → `hươu`). The `u`
/// in a `qu` onset is a glide, not part of the nucleus, so `quơi` stays `quơi`.
pub(crate) fn complete_uo_horn_for_continuation(buffer: &str, key: char) -> Option<String> {
    let (u_offset, u, before_u) = open_uo_horn_suffix(buffer)?;
    if before_u.is_some_and(|ch| ch.eq_ignore_ascii_case(&'q')) {
        return None;
    }

    let shaped_u = apply_shape(u, VowelShape::Horn)?;
    let after_u = u_offset + u.len_utf8();
    let mut completed = String::with_capacity(buffer.len() + key.len_utf8() + 1);
    completed.push_str(&buffer[..u_offset]);
    completed.push(shaped_u);
    completed.push_str(&buffer[after_u..]);
    completed.push(key);
    Some(completed)
}

/// Whether the buffer ends in the ambiguous open `uơ` form. A repeated horn key
/// must revert this form (`uo77` → `uo7`) rather than horn the remaining `u`.
pub(crate) fn ends_with_open_uo_horn(buffer: &str) -> bool {
    open_uo_horn_suffix(buffer).is_some()
}
