//! Key → funput-core → ImeResult orchestration.

use funput_core::{apply, is_definitely_invalid, TransformKind};

use crate::diff::diff;
use crate::result::ImeResult;
use crate::session::Session;

/// Apply one keystroke to `session` and return platform instructions.
///
/// `session.keys` already includes `key` (pushed by the caller).
pub(crate) fn process(session: &mut Session, key: char) -> ImeResult {
    let result = apply(&session.buffer, key, session.method);

    // Buffer after composing this key (the engine appends literally on Ignored).
    let composed = match result.kind {
        TransformKind::Ignored => format!("{}{key}", session.buffer),
        _ => result.text,
    };

    // Eager English restore: flip to the raw keystrokes the instant the word can
    // no longer be Vietnamese (`tẽt` → `text` on the closing `t`). Gated by the
    // smart + eager toggles. Skip on Reverted (a deliberate user restore) and when
    // nothing was transformed (`keys == composed`, e.g. a literal digit `ng1`).
    let new_buffer = if session.smart_restore
        && session.eager_restore
        && result.kind != TransformKind::Reverted
        && session.keys != composed
        && is_definitely_invalid(&composed)
    {
        session.keys.clone()
    } else {
        composed
    };

    let (backspace, output) = diff(&session.buffer, &new_buffer);
    session.buffer = new_buffer;

    // A revert is itself a restore to raw, so keep `keys` in sync — otherwise a
    // later word boundary sees `keys != buffer` and re-restores the stale original
    // keystrokes (e.g. `mixx` revert → `mix`, then Space → wrongly `mixx`).
    if result.kind == TransformKind::Reverted {
        session.keys = session.buffer.clone();
    }

    // A pure append of the typed key passes through — the app echoes it itself.
    if backspace == 0 && output.chars().eq(std::iter::once(key)) {
        ImeResult::none()
    } else {
        ImeResult::send(backspace, output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::result::Action;
    use funput_core::InputMethod;

    #[test]
    fn pending_appends_literal() {
        let mut session = Session::new();
        let result = process(&mut session, 'a');
        assert_eq!(result.action, Action::None);
        assert_eq!(session.buffer, "a");
    }

    #[test]
    fn applied_tone_telex() {
        let mut session = Session::new();
        process(&mut session, 'a');
        let result = process(&mut session, 's');
        assert_eq!(result.action, Action::Send);
        assert_eq!(result.backspace, 1);
        assert_eq!(result.output, "á");
        assert_eq!(session.buffer, "á");
    }

    #[test]
    fn ignored_appends_literal_vni() {
        let mut session = Session::new();
        session.method = InputMethod::Vni;
        session.buffer.push_str("ng");
        session.keys.push_str("ng1"); // caller pushes the key before pipeline runs
        let result = process(&mut session, '1');
        assert_eq!(result.action, Action::None);
        assert_eq!(session.buffer, "ng1");
    }

    #[test]
    fn eager_restore_on_dead_end() {
        // Telex "text": "tẽ" is still valid, but the closing "t" makes "tẽt" a
        // dead end → restore to the raw keys immediately (no boundary needed).
        let mut session = Session::new();
        for key in "tex".chars() {
            session.keys.push(key);
            process(&mut session, key);
        }
        assert_eq!(session.buffer, "tẽ");

        session.keys.push('t');
        let result = process(&mut session, 't');
        assert_eq!(result.action, Action::Send);
        assert_eq!(session.buffer, "text");
    }
}
