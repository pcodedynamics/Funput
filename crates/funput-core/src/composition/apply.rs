//! Per-action buffer transforms (stroke, tone, shape).
//!
//! Method-agnostic: these operate on a [`crate::input_method::KeyAction`] already
//! resolved by a classifier, so VNI and Telex share them unchanged.

use crate::composition::replace_char_at;
use crate::unicode::marks::{apply_tone_to_vowel, is_vowel, stroke_d, Tone};
use crate::unicode::shapes::{apply_shape, apply_shape_to_vowel, shape_target_index, VowelShape};
use crate::unicode::tone_position::{tone_target_vowel, tone_vowel_index};
use crate::{TransformKind, TransformResult};

fn ignored(buffer: &str) -> TransformResult {
    TransformResult {
        kind: TransformKind::Ignored,
        text: buffer.to_owned(),
    }
}

/// Turn the trailing `d`/`D` into `đ`/`Đ`.
pub(crate) fn apply_stroke(buffer: &str) -> TransformResult {
    let mut chars: Vec<char> = buffer.chars().collect();
    let Some(last) = chars.last().copied() else {
        return ignored(buffer);
    };
    let Some(stroked) = stroke_d(last) else {
        return ignored(buffer);
    };

    let len = chars.len();
    chars[len - 1] = stroked;
    TransformResult {
        kind: TransformKind::Applied,
        text: chars.into_iter().collect(),
    }
}

/// Place `tone` on the nucleus vowel (handles reposition and `ie`/`gie` → `ê`).
pub(crate) fn apply_tone_key(buffer: &str, tone: Tone) -> TransformResult {
    let Some(vowel_idx) = tone_vowel_index(buffer) else {
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

/// True if `shape` can still be applied to some vowel in `buffer` (the horn
/// `uo` compound, or a single vowel that can receive the shape).
pub(crate) fn shape_apply_target_exists(buffer: &str, shape: VowelShape) -> bool {
    if shape == VowelShape::Horn && uo_pair_in_vowel_cluster(buffer).is_some() {
        return true;
    }
    shape_target_index(buffer, shape).is_some()
}

/// Apply a vowel shape; horn turns an adjacent `uo` into the `ươ` compound.
pub(crate) fn apply_shape_key(buffer: &str, shape: VowelShape) -> TransformResult {
    if shape == VowelShape::Horn && let Some(text) = apply_uo_compound(buffer) {
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
    chars[u_idx] = apply_shape(chars[u_idx], VowelShape::Horn)?;
    chars[o_idx] = apply_shape(chars[o_idx], VowelShape::Horn)?;
    Some(chars.into_iter().collect())
}
