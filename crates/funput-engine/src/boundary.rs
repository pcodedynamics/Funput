//! Word-boundary handling — end-of-word clears composition state.

use funput_core::is_complete_syllable;

use crate::result::ImeResult;
use crate::session::Session;

/// A word boundary ends the current composition: any whitespace or ASCII
/// punctuation. Digits are *not* boundaries — VNI uses `1`–`9` as modifiers.
///
/// v1 ignores non-ASCII punctuation (em dash, smart quotes, guillemets).
pub(crate) fn is_word_boundary(key: char) -> bool {
    key.is_whitespace() || key.is_ascii_punctuation()
}

/// True when the composed buffer should be replaced with raw keystrokes on boundary.
///
/// The word is finished, so we use the **strict** [`is_complete_syllable`]: a word
/// ending in a non-Vietnamese final (`cảd` from `card`, `côl` from `cool`) is not
/// a real syllable and is restored to the raw Latin keystrokes. A valid syllable
/// (`má`, `tét`) is kept — typing it was intentional.
///
/// Exception: keystrokes that deliberately request a Vietnamese character are never
/// reverted (see [`keystrokes_intend_vietnamese`]) — that keeps abbreviations like
/// `GĐ`, `QĐ`, `đc` instead of exposing `GDD` / `d9c`.
pub(crate) fn should_restore(session: &Session) -> bool {
    session.smart_restore
        && !session.buffer.is_empty()
        && session.keys != session.buffer
        && !is_complete_syllable(&session.buffer)
        && !keystrokes_intend_vietnamese(session)
}

/// The user deliberately asked for a Vietnamese character, so reverting to raw Latin
/// would be wrong. Two method-independent signals:
/// - a `đ`/`Đ` in the composed buffer — English has no `đ`, so `GĐ`, `QĐ`, `đc` are
///   intentional (`dd` in Telex, `d9` in VNI both reach it);
/// - a digit in the raw keys — a VNI tone/shape modifier (`d9c` → `đc`, `to1` → `tó`)
///   that a revert would surface.
fn keystrokes_intend_vietnamese(session: &Session) -> bool {
    session.buffer.contains(['đ', 'Đ'])
        || session.keys.contains(|c: char| c.is_ascii_digit())
}

fn english_restore_result(session: &Session, boundary_key: char) -> ImeResult {
    let backspace = session.buffer.chars().count();
    let output = format!("{}{}", session.keys, boundary_key);
    ImeResult::send(backspace, output)
}

/// Text expansion (gõ tắt): if the raw keystrokes since the last boundary match a
/// defined trigger, replace the composed buffer with the expansion. Matched
/// case-sensitively against `keys`, so `vn` → `Việt Nam` regardless of how Telex/VNI
/// composed it. `backspace` counts the *displayed* buffer (`á` is one char even if
/// typed `as`), so we delete exactly what is on screen before injecting.
fn shortcut_expansion(session: &Session, boundary_key: char) -> Option<ImeResult> {
    if session.keys.is_empty() {
        return None;
    }
    let expansion = session.shortcuts.get(&session.keys)?;
    let backspace = session.buffer.chars().count();
    let output = format!("{expansion}{boundary_key}");
    Some(ImeResult::send(backspace, output))
}

/// Update auto-capitalize state from a boundary key. No-op unless the feature is on.
/// A sentence-ender (`.`/`!`/`?`) only *arms* once whitespace confirms the break, so
/// `google.com` is left alone while `End. Next` is capitalized; a newline arms
/// directly; quotes/brackets are transparent (`He said "Hi." Next`); any other
/// punctuation cancels a pending capitalization.
fn update_caps_on_boundary(session: &mut Session, key: char) {
    if !session.auto_capitalize {
        return;
    }
    match key {
        '.' | '!' | '?' => session.cap_sentence_ended = true,
        '\n' | '\r' => {
            session.cap_armed = true;
            session.cap_sentence_ended = false;
        }
        ' ' | '\t' => {
            if session.cap_sentence_ended {
                session.cap_armed = true;
            }
        }
        // Quotes and brackets are transparent — they don't open or end a sentence.
        '"' | '\'' | '(' | ')' | '[' | ']' | '{' | '}' => {}
        _ => {
            session.cap_sentence_ended = false;
            session.cap_armed = false;
        }
    }
}

/// End-of-word: expand a gõ tắt trigger, else optionally restore English Latin text,
/// then reset composition state. Text expansion takes priority over English restore.
pub(crate) fn on_word_boundary(session: &mut Session, boundary_key: char) -> ImeResult {
    let result = if let Some(expansion) = shortcut_expansion(session, boundary_key) {
        expansion
    } else if should_restore(session) {
        english_restore_result(session, boundary_key)
    } else {
        ImeResult::none()
    };
    update_caps_on_boundary(session, boundary_key);
    session.clear();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::result::Action;
    use funput_core::{InputMethod, ToneStyle};
    use std::collections::HashMap;

    #[test]
    fn word_boundary_chars() {
        // Whitespace and ASCII punctuation are boundaries.
        for c in [' ', '\n', '\t', ',', '.', '!', '?', ')', '-', '"'] {
            assert!(is_word_boundary(c), "{c:?} should be a boundary");
        }
        // Letters and digits are not (digits are VNI modifiers).
        for c in ['a', 'z', 'A', '1', '9'] {
            assert!(!is_word_boundary(c), "{c:?} should not be a boundary");
        }
    }

    #[test]
    fn should_restore_when_buffer_invalid_and_keys_differ() {
        let session = Session {
            enabled: true,
            method: InputMethod::Telex,
            buffer: "ábc".into(),
            keys: "absc".into(),
            tone_style: ToneStyle::Traditional,
            smart_restore: true,
            eager_restore: true,
            spell_check: false,
            auto_capitalize: false,
            cap_sentence_ended: false,
            cap_armed: false,
            shortcuts: HashMap::new(),
        };
        assert!(should_restore(&session));
    }

    #[test]
    fn should_not_restore_when_buffer_valid_vietnamese() {
        let session = Session {
            enabled: true,
            method: InputMethod::Telex,
            buffer: "má".into(),
            keys: "mas".into(),
            tone_style: ToneStyle::Traditional,
            smart_restore: true,
            eager_restore: true,
            spell_check: false,
            auto_capitalize: false,
            cap_sentence_ended: false,
            cap_armed: false,
            shortcuts: HashMap::new(),
        };
        assert!(!should_restore(&session));
    }

    #[test]
    fn should_not_restore_when_keys_match_buffer() {
        let session = Session {
            enabled: true,
            method: InputMethod::Telex,
            buffer: "text".into(),
            keys: "text".into(),
            tone_style: ToneStyle::Traditional,
            smart_restore: true,
            eager_restore: true,
            spell_check: false,
            auto_capitalize: false,
            cap_sentence_ended: false,
            cap_armed: false,
            shortcuts: HashMap::new(),
        };
        assert!(!should_restore(&session));
    }

    #[test]
    fn should_not_restore_when_buffer_empty() {
        let session = Session::new();
        assert!(!should_restore(&session));
    }

    #[test]
    fn should_not_restore_when_keys_contain_a_vni_digit() {
        // VNI `d9c` → `đc`: reverting would expose the `9`, so keep the composed word.
        let session = Session {
            enabled: true,
            method: InputMethod::Vni,
            buffer: "đc".into(),
            keys: "d9c".into(),
            tone_style: ToneStyle::Traditional,
            smart_restore: true,
            eager_restore: true,
            spell_check: false,
            auto_capitalize: false,
            cap_sentence_ended: false,
            cap_armed: false,
            shortcuts: HashMap::new(),
        };
        assert!(!should_restore(&session));
    }

    #[test]
    fn should_not_restore_when_buffer_has_d_stroke_in_telex() {
        // Telex `GDD` → `GĐ`: đ marks intentional Vietnamese, so keep it (no digits).
        let session = Session {
            enabled: true,
            method: InputMethod::Telex,
            buffer: "GĐ".into(),
            keys: "GDD".into(),
            tone_style: ToneStyle::Traditional,
            smart_restore: true,
            eager_restore: true,
            spell_check: false,
            auto_capitalize: false,
            cap_sentence_ended: false,
            cap_armed: false,
            shortcuts: HashMap::new(),
        };
        assert!(!should_restore(&session));
    }

    #[test]
    fn on_word_boundary_restores_english() {
        let mut session = Session {
            enabled: true,
            method: InputMethod::Telex,
            buffer: "ábc".into(),
            keys: "absc".into(),
            tone_style: ToneStyle::Traditional,
            smart_restore: true,
            eager_restore: true,
            spell_check: false,
            auto_capitalize: false,
            cap_sentence_ended: false,
            cap_armed: false,
            shortcuts: HashMap::new(),
        };
        let result = on_word_boundary(&mut session, ' ');
        assert_eq!(result.action, Action::Send);
        assert_eq!(result.backspace, 3);
        assert_eq!(result.output, "absc ");
        assert!(session.buffer.is_empty());
        assert!(session.keys.is_empty());
    }

    #[test]
    fn on_word_boundary_valid_vn_no_restore() {
        let mut session = Session {
            enabled: true,
            method: InputMethod::Telex,
            buffer: "má".into(),
            keys: "mas".into(),
            tone_style: ToneStyle::Traditional,
            smart_restore: true,
            eager_restore: true,
            spell_check: false,
            auto_capitalize: false,
            cap_sentence_ended: false,
            cap_armed: false,
            shortcuts: HashMap::new(),
        };
        let result = on_word_boundary(&mut session, ' ');
        assert_eq!(result.action, Action::None);
        assert_eq!(result.backspace, 0);
        assert!(result.output.is_empty());
        assert!(session.buffer.is_empty());
        assert!(session.keys.is_empty());
    }

    #[test]
    fn on_word_boundary_clears_session() {
        let mut session = Session::new();
        session.buffer.push('á');
        session.keys.push_str("as");
        on_word_boundary(&mut session, ' ');
        assert!(session.buffer.is_empty());
        assert!(session.keys.is_empty());
    }

    #[test]
    fn shortcut_expands_on_boundary() {
        // `vn` is not a complete syllable, so it would normally English-restore to
        // `vn`; the trigger wins and expands to the full text instead.
        let mut session = Session::new();
        session.buffer.push_str("vn");
        session.keys.push_str("vn");
        session.shortcuts.insert("vn".into(), "Việt Nam".into());
        let result = on_word_boundary(&mut session, ' ');
        assert_eq!(result.action, Action::Send);
        assert_eq!(result.backspace, 2); // deletes the displayed "vn"
        assert_eq!(result.output, "Việt Nam ");
        assert!(session.buffer.is_empty());
        assert!(session.keys.is_empty());
    }

    #[test]
    fn shortcut_backspace_counts_displayed_buffer() {
        // Telex `as` displays `á` (one char) but keys are `as`; deleting must match
        // what is on screen, so backspace is 1, not 2.
        let mut session = Session::new();
        session.buffer.push('á');
        session.keys.push_str("as");
        session.shortcuts.insert("as".into(), "address".into());
        let result = on_word_boundary(&mut session, ' ');
        assert_eq!(result.backspace, 1);
        assert_eq!(result.output, "address ");
    }

    #[test]
    fn shortcut_is_case_sensitive() {
        // Only `vn` is defined; typing `VN` must not expand.
        let mut session = Session::new();
        session.buffer.push_str("VN");
        session.keys.push_str("VN");
        session.shortcuts.insert("vn".into(), "Việt Nam".into());
        let result = on_word_boundary(&mut session, ' ');
        assert_eq!(result.action, Action::None);
    }

    #[test]
    fn shortcut_keeps_boundary_punctuation() {
        let mut session = Session::new();
        session.buffer.push_str("kg");
        session.keys.push_str("kg");
        session.shortcuts.insert("kg".into(), "không".into());
        let result = on_word_boundary(&mut session, ',');
        assert_eq!(result.output, "không,");
    }
}
