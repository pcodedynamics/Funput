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
    !session.buffer.is_empty()
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

/// End-of-word: optionally restore English Latin text, then reset composition state.
pub(crate) fn on_word_boundary(session: &mut Session, boundary_key: char) -> ImeResult {
    let result = if should_restore(session) {
        english_restore_result(session, boundary_key)
    } else {
        ImeResult::none()
    };
    session.clear();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::result::Action;
    use funput_core::InputMethod;

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
}
