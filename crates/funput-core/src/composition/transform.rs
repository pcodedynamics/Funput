//! Transform pipeline: classify a key, then orchestrate revert → validate → apply.
//!
//! [`apply_action`] is method-agnostic; only the input-method classifier differs.

use crate::composition::apply::{
    apply_shape_key, apply_stroke, apply_tone_key, complete_uo_horn_for_continuation,
    ends_with_open_uo_horn, remove_tone, shape_apply_target_exists,
};
use crate::composition::revert::{try_revert_shape, try_revert_stroke, try_revert_tone};
use crate::input_method::telex;
use crate::input_method::vni;
use crate::input_method::KeyAction;
use crate::unicode::tone_position::reposition_existing_tone;
use crate::validation::syllable::{
    is_definitely_invalid, validate_shape, validate_stroke, validate_tone, ModifierValidation,
};
use crate::{ToneStyle, TransformKind, TransformResult};

fn reverted(text: String) -> TransformResult {
    TransformResult {
        kind: TransformKind::Reverted,
        text,
    }
}

/// Spell-check ("Kiểm tra chính tả") gate. When `spell_check` is on, a composed
/// modifier result that can no longer become a real Vietnamese syllable is rejected
/// and the modifier key is passed through as a literal instead (UniKey-style strict
/// diacritics). A no-op when spell-check is off or the modifier was not applied.
fn spell_check_gate(
    buffer: &str,
    key: char,
    spell_check: bool,
    result: TransformResult,
) -> TransformResult {
    if spell_check && result.kind == TransformKind::Applied && is_definitely_invalid(&result.text) {
        return TransformResult {
            kind: TransformKind::Pending,
            text: format!("{buffer}{key}"),
        };
    }
    result
}

fn validation_gate(buffer: &str, key: char, validation: ModifierValidation) -> Option<TransformResult> {
    match validation {
        ModifierValidation::Allow => None,
        ModifierValidation::Ignored => Some(TransformResult {
            kind: TransformKind::Ignored,
            text: buffer.to_owned(),
        }),
        ModifierValidation::PassThrough => Some(TransformResult {
            kind: TransformKind::Pending,
            text: format!("{buffer}{key}"),
        }),
    }
}

/// Apply one VNI keystroke to `buffer`.
pub(crate) fn apply_vni(
    buffer: &str,
    key: char,
    style: ToneStyle,
    spell_check: bool,
) -> TransformResult {
    apply_action(buffer, key, vni::classify_key(buffer, key), style, spell_check)
}

/// Apply one Telex keystroke to `buffer`.
pub(crate) fn apply_telex(
    buffer: &str,
    key: char,
    style: ToneStyle,
    spell_check: bool,
) -> TransformResult {
    apply_action(
        buffer,
        key,
        telex::classify_key(buffer, key),
        style,
        spell_check,
    )
}

/// Apply a classified key action to `buffer`.
///
/// Method-agnostic: VNI and (later) Telex differ only in how a key maps to a
/// [`KeyAction`] — the revert → validate → apply orchestration here is shared.
/// `key` is the literal character to append for [`KeyAction::Normal`] and
/// pass-through.
pub(crate) fn apply_action(
    buffer: &str,
    key: char,
    action: KeyAction,
    style: ToneStyle,
    spell_check: bool,
) -> TransformResult {
    match action {
        KeyAction::Stroke => {
            if let Some(text) = try_revert_stroke(buffer) {
                return reverted(format!("{text}{key}"));
            }
            let result = validation_gate(buffer, key, validate_stroke(buffer))
                .unwrap_or_else(|| apply_stroke(buffer));
            spell_check_gate(buffer, key, spell_check, result)
        }
        KeyAction::Tone(tone) => {
            if let Some(text) = try_revert_tone(buffer, tone, style) {
                return reverted(format!("{text}{key}"));
            }
            let result = validation_gate(buffer, key, validate_tone(buffer))
                .unwrap_or_else(|| apply_tone_key(buffer, tone, style));
            spell_check_gate(buffer, key, spell_check, result)
        }
        KeyAction::Shape(shape) => {
            if shape == crate::unicode::shapes::VowelShape::Horn
                && ends_with_open_uo_horn(buffer)
                && let Some(text) = try_revert_shape(buffer, shape)
            {
                return reverted(format!("{text}{key}"));
            }
            // Apply takes priority when an unshaped target exists, so the second
            // horn in `u7o7` shapes the `o` (→ `ươ`) instead of reverting the
            // earlier `ư`. Revert only fires when there is no target to apply to
            // (e.g. `a66`, `uo77`).
            if shape_apply_target_exists(buffer, shape) {
                let result = validation_gate(buffer, key, validate_shape(buffer))
                    .unwrap_or_else(|| apply_shape_key(buffer, shape));
                return spell_check_gate(buffer, key, spell_check, result);
            }
            if let Some(text) = try_revert_shape(buffer, shape) {
                return reverted(format!("{text}{key}"));
            }
            let result = validation_gate(buffer, key, validate_shape(buffer))
                .unwrap_or_else(|| apply_shape_key(buffer, shape));
            spell_check_gate(buffer, key, spell_check, result)
        }
        KeyAction::RemoveTone => match remove_tone(buffer) {
            // Tone stripped (keeps shape): `viết` + `z`/`0` → `viêt`.
            Some(text) => TransformResult {
                kind: TransformKind::Applied,
                text,
            },
            // No tone to remove → the key is a literal (digit `0`, letter `z`).
            None => TransformResult {
                kind: TransformKind::Pending,
                text: format!("{buffer}{key}"),
            },
        },
        KeyAction::Normal => {
            if let Some(completed) = complete_uo_horn_for_continuation(buffer, key) {
                return match reposition_existing_tone(&completed, style) {
                    Some(repositioned) => TransformResult {
                        kind: TransformKind::Applied,
                        text: repositioned,
                    },
                    None => TransformResult {
                        kind: TransformKind::Applied,
                        text: completed,
                    },
                };
            }

            let appended = format!("{buffer}{key}");
            match reposition_existing_tone(&appended, style) {
                Some(repositioned) => TransformResult {
                    kind: TransformKind::Applied,
                    text: repositioned,
                },
                None => TransformResult {
                    kind: TransformKind::Pending,
                    text: appended,
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{apply, InputMethod};

    /// VNI keystroke with the default (traditional) tone style.
    fn av(buffer: &str, key: char) -> TransformResult {
        apply_vni(buffer, key, ToneStyle::Traditional, false)
    }

    /// Telex keystroke with the default (traditional) tone style.
    fn at(buffer: &str, key: char) -> TransformResult {
        apply_telex(buffer, key, ToneStyle::Traditional, false)
    }

    fn type_keys(keys: &str) -> String {
        let mut buf = String::new();
        for key in keys.chars() {
            buf = av(&buf, key).text;
        }
        buf
    }

    /// Type a VNI sequence with the modern ("kiểu mới") tone style.
    fn type_keys_modern(keys: &str) -> String {
        let mut buf = String::new();
        for key in keys.chars() {
            buf = apply_vni(&buf, key, ToneStyle::Modern, false).text;
        }
        buf
    }

    /// Type a Telex sequence with the spell-check ("Kiểm tra chính tả") gate set.
    fn type_telex(keys: &str, spell_check: bool) -> String {
        let mut buf = String::new();
        for key in keys.chars() {
            buf = apply_telex(&buf, key, ToneStyle::Traditional, spell_check).text;
        }
        buf
    }

    #[test]
    fn spell_check_blocks_invalid_syllable() {
        // A stop coda (`t`) may only carry sắc / nặng. `tetf` (huyền) → `tèt` is
        // structurally allowed but not a real syllable. With spell-check off the
        // diacritic still lands (legacy behaviour); with it on the modifier key
        // passes through as a literal so the raw word survives.
        assert_eq!(type_telex("tetf", false), "tèt");
        let blocked = apply_telex("tet", 'f', ToneStyle::Traditional, true);
        assert_eq!(blocked.text, "tetf");
        assert_eq!(blocked.kind, TransformKind::Pending);
    }

    #[test]
    fn spell_check_allows_real_syllables() {
        // Real Vietnamese syllables are never blocked, in either mode.
        assert_eq!(type_telex("phus", true), "phú");
        assert_eq!(type_telex("dd", true), "đ");
        assert_eq!(type_telex("vieets", true), "viết");
        // `tét` (sắc on a stop coda) is valid and must still compose with the gate on.
        assert_eq!(type_telex("tets", true), "tét");
    }

    #[test]
    fn spell_check_does_not_block_revert() {
        // Double modifier to revert (`ass` → `as`) is a deliberate restore, not an
        // applied diacritic, so the spell-check gate must leave it alone.
        assert_eq!(apply_telex("á", 's', ToneStyle::Traditional, true).text, "as");
    }

    #[test]
    fn stroke_and_tone_basics() {
        assert_eq!(av("d", '9').text, "đ");
        assert_eq!(av("D", '9').text, "Đ");
        for (key, expected) in [('1', "á"), ('2', "à"), ('3', "ả"), ('4', "ã"), ('5', "ạ")] {
            assert_eq!(av("a", key).text, expected, "key {key}");
        }
    }

    #[test]
    fn shape_basics_and_compound() {
        assert_eq!(av("a", '6').text, "â");
        assert_eq!(av("o", '7').text, "ơ");
        assert_eq!(av("a", '8').text, "ă");
        assert_eq!(av("uo", '7').text, "uơ");
    }

    #[test]
    fn shape_then_tone() {
        assert_eq!(type_keys("o71"), "ớ");
        assert_eq!(type_keys("a61"), "ấ");
    }

    #[test]
    fn remove_tone_vni() {
        assert_eq!(type_keys("a1"), "á");
        assert_eq!(type_keys("a10"), "a"); // 0 removes the tone
        assert_eq!(type_keys("a610"), "â"); // keeps the circumflex shape
        assert_eq!(type_keys("vie65t1"), "viết");
        assert_eq!(type_keys("vie65t10"), "viêt"); // tone gone, ê kept
        assert_eq!(type_keys("a0"), "a0"); // no tone → literal digit
    }

    #[test]
    fn remove_tone_telex() {
        let telex = |keys: &str| {
            let mut buf = String::new();
            for key in keys.chars() {
                buf = at(&buf, key).text;
            }
            buf
        };
        assert_eq!(telex("as"), "á");
        assert_eq!(telex("asz"), "a"); // z removes the tone
        assert_eq!(telex("aasz"), "â"); // keeps â
        assert_eq!(telex("vieetjz"), "viêt"); // việt → remove tone → viêt
        assert_eq!(telex("z"), "z"); // no tone → literal z
    }

    #[test]
    fn telex_horn_placed_after_trailing_vowel() {
        // The horn (`w`) can be placed anywhere, including after the whole rhyme:
        // `moiwf` → `mời` works the same as `mowif`. Likewise the tone key is
        // already position-free, so both orders land the huyền on the right vowel.
        assert_eq!(type_telex("moiwf", false), "mời");
        assert_eq!(type_telex("mowif", false), "mời");
        assert_eq!(type_telex("doiwf", false), "dời");
        assert_eq!(type_telex("nguoiwf", false), "người");
    }

    #[test]
    fn shape_after_tone_keeps_tone() {
        // Applying a shape to an already-toned vowel must keep the tone, so the mark
        // order is free (shape-then-tone and tone-then-shape agree).
        // Telex:
        assert_eq!(type_telex("asw", false), "ắ"); // á + breve → ắ (not ă)
        assert_eq!(type_telex("aws", false), "ắ"); // breve + á → ắ (same result)
        assert_eq!(type_telex("osw", false), "ớ"); // ó + horn → ớ (keeps sắc)
        // VNI (position-free digits) gets the same benefit:
        assert_eq!(type_keys("a18"), "ắ"); // á + 8 (breve) → ắ
        assert_eq!(type_keys("o17"), "ớ"); // ó + 7 (horn) → ớ
        assert_eq!(type_keys("a16"), "ấ"); // á + 6 (circumflex) → ấ
    }

    #[test]
    fn telex_ua_rhyme_horns_u() {
        // `w` after a plain `ua` forms the `ưa` rhyme (horn on `u`), so the horn can
        // be placed last: `nuawx` → `nữa`, same as `nuwax`.
        assert_eq!(type_telex("nuawx", false), "nữa");
        assert_eq!(type_telex("nuwax", false), "nữa");
        assert_eq!(type_telex("muaw", false), "mưa");
        assert_eq!(type_telex("dduaw", false), "đưa");
        // `qu` glide is untouched: `a` takes the breve (`quăng`).
        assert_eq!(type_telex("quawng", false), "quăng");
        // The plain `ua` rhyme (no `w`) still composes normally (`múa`, not `mứa`).
        assert_eq!(type_telex("muas", false), "múa");
    }

    #[test]
    fn telex_ua_horn_keeps_tone_and_reverts() {
        // #2: a tone already on the `u` is kept when the horn lands (`úa` → `ứa`),
        // so `ua` + tone + `w` composes the same as `ua` + `w` + tone.
        assert_eq!(type_telex("uasw", false), "ứa");
        assert_eq!(type_telex("uaws", false), "ứa");
        assert_eq!(type_telex("cuawr", false), "cửa"); // cửa (door)
        // #1: a repeat `w` reverts the horn even when it sits on `ư` (not the last
        // char), giving back `ua` plus the literal `w`, not a stray breve on `a`.
        assert_eq!(type_telex("nuaww", false), "nuaw");
        assert_eq!(type_telex("muaww", false), "muaw");
        // `qu` glide is still breve-and-revert on the `a`, untouched by the `ua` rule.
        assert_eq!(type_telex("quaww", false), "quaw");
    }

    #[test]
    fn reposition_and_complex() {
        assert_eq!(type_keys("hoa2"), "hòa");
        assert_eq!(type_keys("thuy3"), "thủy");
        assert_eq!(type_keys("hoaf2"), "hoàf");
        assert_eq!(type_keys("tru7o7n2g"), "trường");
        assert_eq!(type_keys("vie5t"), "việt");
        assert_eq!(type_keys("ngu7o7i2"), "người");
    }

    #[test]
    fn modern_tone_style_oa_oe_uy() {
        // "Kiểu mới": tone on the second vowel for open oa/oe/uy, regardless of
        // where the tone key is typed.
        assert_eq!(type_keys_modern("hoa2"), "hoà"); // tone after both vowels
        assert_eq!(type_keys_modern("ho2a"), "hoà"); // tone typed before `a` (reposition)
        assert_eq!(type_keys_modern("thuy3"), "thuỷ");
        assert_eq!(type_keys_modern("khoe3"), "khoẻ");
        // Unchanged from traditional: ia/ua, coda, triphthong, shaped vowel.
        assert_eq!(type_keys_modern("mua1"), "múa");
        assert_eq!(type_keys_modern("hoan2"), "hoàn");
        assert_eq!(type_keys_modern("ngoai2"), "ngoài");
        assert_eq!(type_keys_modern("tru7o7n2g"), "trường");
    }

    #[test]
    fn revert_cases() {
        // Double modifier restores raw keystrokes: strip diacritic + append the key.
        assert_eq!(av("á", '1'), reverted("a1".into()));
        assert_eq!(av("â", '6'), reverted("a6".into()));
        assert_eq!(av("đ", '9'), reverted("d9".into()));
        assert_eq!(av("ấ", '1'), reverted("â1".into()));
        assert_eq!(type_keys("a12"), "à"); // different tone key → re-tone, not revert
        assert_eq!(type_keys("a11"), "a1");
        assert_eq!(type_keys("a66"), "a6");
        assert_eq!(type_keys("uo77"), "uo7");
        assert_eq!(type_keys("phu11"), "phu1");
    }

    #[test]
    fn ignored_and_pending() {
        assert_eq!(av("", '6').kind, TransformKind::Ignored);
        assert_eq!(av("ng", '7').kind, TransformKind::Ignored);
        assert_eq!(av("", '1').kind, TransformKind::Ignored);
        assert_eq!(av("a", 'b'), TransformResult {
            kind: TransformKind::Pending,
            text: "ab".into(),
        });
    }

    #[test]
    fn dispatches_through_public_api() {
        let mut buf = String::new();
        for key in "ma1".chars() {
            buf = apply(&buf, key, InputMethod::Vni, ToneStyle::Traditional).text;
        }
        assert_eq!(buf, "má");
    }
}
