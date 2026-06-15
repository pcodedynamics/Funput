//! Key → funput-core → ImeResult orchestration.

use funput_core::{apply, TransformKind};

use crate::diff::diff;
use crate::result::ImeResult;
use crate::session::Session;

/// Apply one keystroke to `session` and return platform instructions.
pub(crate) fn process(session: &mut Session, key: char) -> ImeResult {
    let result = apply(&session.buffer, key, session.method);

    match result.kind {
        TransformKind::Pending => {
            session.buffer = result.text;
            ImeResult::none()
        }
        TransformKind::Ignored => {
            session.buffer.push(key);
            ImeResult::none()
        }
        TransformKind::Applied | TransformKind::Reverted => {
            // `session.buffer` is still the old text until we reassign it below.
            let (backspace, output) = diff(&session.buffer, &result.text);
            session.buffer = result.text;
            ImeResult::send(backspace, output)
        }
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
        let result = process(&mut session, '1');
        assert_eq!(result.action, Action::None);
        assert_eq!(session.buffer, "ng1");
    }
}
