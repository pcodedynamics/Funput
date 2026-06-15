//! Transform pipeline: classify a key, then orchestrate revert → validate → apply.
//!
//! [`apply_action`] is method-agnostic; only the input-method classifier differs.

use crate::composition::apply::{
    apply_shape_key, apply_stroke, apply_tone_key, shape_apply_target_exists,
};
use crate::composition::revert::{try_revert_shape, try_revert_stroke, try_revert_tone};
use crate::input_method::telex;
use crate::input_method::vni;
use crate::input_method::KeyAction;
use crate::unicode::tone_position::reposition_existing_tone;
use crate::validation::syllable::{
    validate_shape, validate_stroke, validate_tone, ModifierValidation,
};
use crate::{TransformKind, TransformResult};

fn reverted(text: String) -> TransformResult {
    TransformResult {
        kind: TransformKind::Reverted,
        text,
    }
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
pub(crate) fn apply_vni(buffer: &str, key: char) -> TransformResult {
    apply_action(buffer, key, vni::classify_key(buffer, key))
}

/// Apply one Telex keystroke to `buffer`.
pub(crate) fn apply_telex(buffer: &str, key: char) -> TransformResult {
    apply_action(buffer, key, telex::classify_key(buffer, key))
}

/// Apply a classified key action to `buffer`.
///
/// Method-agnostic: VNI and (later) Telex differ only in how a key maps to a
/// [`KeyAction`] — the revert → validate → apply orchestration here is shared.
/// `key` is the literal character to append for [`KeyAction::Normal`] and
/// pass-through.
pub(crate) fn apply_action(buffer: &str, key: char, action: KeyAction) -> TransformResult {
    match action {
        KeyAction::Stroke => {
            if let Some(text) = try_revert_stroke(buffer) {
                return reverted(text);
            }
            validation_gate(buffer, key, validate_stroke(buffer))
                .unwrap_or_else(|| apply_stroke(buffer))
        }
        KeyAction::Tone(tone) => {
            if let Some(text) = try_revert_tone(buffer, tone) {
                return reverted(text);
            }
            validation_gate(buffer, key, validate_tone(buffer))
                .unwrap_or_else(|| apply_tone_key(buffer, tone))
        }
        KeyAction::Shape(shape) => {
            // Apply takes priority when an unshaped target exists, so the second
            // horn in `u7o7` shapes the `o` (→ `ươ`) instead of reverting the
            // earlier `ư`. Revert only fires when there is no target to apply to
            // (e.g. `a66`, `uo77`).
            if shape_apply_target_exists(buffer, shape) {
                return validation_gate(buffer, key, validate_shape(buffer))
                    .unwrap_or_else(|| apply_shape_key(buffer, shape));
            }
            if let Some(text) = try_revert_shape(buffer, shape) {
                return reverted(text);
            }
            validation_gate(buffer, key, validate_shape(buffer))
                .unwrap_or_else(|| apply_shape_key(buffer, shape))
        }
        KeyAction::Normal => {
            let text = format!("{buffer}{key}");
            match reposition_existing_tone(&text) {
                Some(repositioned) => TransformResult {
                    kind: TransformKind::Applied,
                    text: repositioned,
                },
                None => TransformResult {
                    kind: TransformKind::Pending,
                    text,
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{apply, InputMethod};

    fn type_keys(keys: &str) -> String {
        let mut buf = String::new();
        for key in keys.chars() {
            buf = apply_vni(&buf, key).text;
        }
        buf
    }

    #[test]
    fn stroke_and_tone_basics() {
        assert_eq!(apply_vni("d", '9').text, "đ");
        assert_eq!(apply_vni("D", '9').text, "Đ");
        for (key, expected) in [('1', "á"), ('2', "à"), ('3', "ả"), ('4', "ã"), ('5', "ạ")] {
            assert_eq!(apply_vni("a", key).text, expected, "key {key}");
        }
    }

    #[test]
    fn shape_basics_and_compound() {
        assert_eq!(apply_vni("a", '6').text, "â");
        assert_eq!(apply_vni("o", '7').text, "ơ");
        assert_eq!(apply_vni("a", '8').text, "ă");
        assert_eq!(apply_vni("uo", '7').text, "ươ");
    }

    #[test]
    fn shape_then_tone() {
        assert_eq!(type_keys("o71"), "ớ");
        assert_eq!(type_keys("a61"), "ấ");
    }

    #[test]
    fn reposition_and_complex() {
        assert_eq!(type_keys("hoa2"), "hoà");
        assert_eq!(type_keys("thuy3"), "thuỷ");
        assert_eq!(type_keys("hoaf2"), "hoàf");
        assert_eq!(type_keys("tru7o7n2g"), "trường");
        assert_eq!(type_keys("vie5t"), "việt");
        assert_eq!(type_keys("ngu7o7i2"), "người");
    }

    #[test]
    fn revert_cases() {
        assert_eq!(apply_vni("á", '1'), reverted("a".into()));
        assert_eq!(apply_vni("â", '6'), reverted("a".into()));
        assert_eq!(apply_vni("đ", '9'), reverted("d".into()));
        assert_eq!(apply_vni("ấ", '1'), reverted("â".into()));
        assert_eq!(type_keys("a12"), "à");
        assert_eq!(type_keys("a11"), "a");
        assert_eq!(type_keys("a66"), "a");
        assert_eq!(type_keys("uo77"), "uo");
    }

    #[test]
    fn ignored_and_pending() {
        assert_eq!(apply_vni("", '6').kind, TransformKind::Ignored);
        assert_eq!(apply_vni("ng", '7').kind, TransformKind::Ignored);
        assert_eq!(apply_vni("", '1').kind, TransformKind::Ignored);
        assert_eq!(apply_vni("a", 'b'), TransformResult {
            kind: TransformKind::Pending,
            text: "ab".into(),
        });
    }

    #[test]
    fn dispatches_through_public_api() {
        let mut buf = String::new();
        for key in "ma1".chars() {
            buf = apply(&buf, key, InputMethod::Vni).text;
        }
        assert_eq!(buf, "má");
    }
}
