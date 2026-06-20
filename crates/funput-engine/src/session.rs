//! IME session state — enabled flag, input method, composition buffer.

use funput_core::InputMethod;

/// Mutable session held by [`crate::Engine`]. Internal — not part of the public API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Session {
    pub(crate) enabled: bool,
    pub(crate) method: InputMethod,
    /// Composed text currently shown in the app (the composition span).
    pub(crate) buffer: String,
    /// Raw keystrokes since the last word boundary. Lets English restore
    /// (phase E3) rebuild the original Latin text when the composed buffer is
    /// not a complete Vietnamese syllable (`keys != buffer && !is_complete_syllable(buffer)`).
    pub(crate) keys: String,
    /// Auto-restore words that aren't valid Vietnamese to their raw Latin keys.
    /// When `false`, the composed buffer is always kept (no English restore).
    pub(crate) smart_restore: bool,
    /// Restore the instant a word becomes a dead end, without waiting for a word
    /// boundary. Only meaningful while `smart_restore` is on.
    pub(crate) eager_restore: bool,
}

impl Session {
    pub(crate) fn new() -> Self {
        Self {
            enabled: true,
            method: InputMethod::Telex,
            buffer: String::new(),
            keys: String::new(),
            smart_restore: true,
            eager_restore: true,
        }
    }

    pub(crate) fn clear(&mut self) {
        self.buffer.clear();
        self.keys.clear();
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_defaults() {
        let session = Session::new();
        assert!(session.enabled);
        assert_eq!(session.method, InputMethod::Telex);
        assert!(session.buffer.is_empty());
        assert!(session.keys.is_empty());
    }

    #[test]
    fn clear_resets_buffer_and_keys() {
        let mut session = Session::new();
        session.buffer.push('á');
        session.keys.push_str("as");
        session.clear();
        assert!(session.buffer.is_empty());
        assert!(session.keys.is_empty());
    }
}
