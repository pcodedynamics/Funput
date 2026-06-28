//! IME session state — enabled flag, input method, composition buffer.

use std::collections::HashMap;

use funput_core::{InputMethod, ToneStyle};

/// Mutable session held by [`crate::Engine`]. Internal — not part of the public API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Session {
    pub(crate) enabled: bool,
    pub(crate) method: InputMethod,
    /// Tone-mark placement style (traditional `hòa` vs modern `hoà`).
    pub(crate) tone_style: ToneStyle,
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
    /// Spell-check ("Kiểm tra chính tả"): only place a diacritic when the result can
    /// still become a real Vietnamese syllable, otherwise keep the modifier key as a
    /// literal (UniKey-style strict diacritics). Off by default.
    pub(crate) spell_check: bool,
    /// Auto-capitalize ("Tự động viết hoa"): uppercase the first letter of a word at
    /// the start of a sentence. Off by default.
    pub(crate) auto_capitalize: bool,
    /// A sentence-ending mark (`.`/`!`/`?`) was just seen; waiting for whitespace to
    /// confirm the next word starts a new sentence. Survives `clear()` — capitalize
    /// state spans word commits.
    pub(crate) cap_sentence_ended: bool,
    /// The next word's first letter should be capitalized. Set by a confirmed
    /// sentence start (whitespace after `.`/`!`/`?`, a newline) or focus; consumed
    /// when a word begins. Survives `clear()`.
    pub(crate) cap_armed: bool,
    /// Text-expansion table (gõ tắt): raw-keystroke trigger → expansion. Matched
    /// case-sensitively against `keys` at a word boundary, before English restore.
    /// Config that lives for the whole session — `clear()` does not touch it.
    pub(crate) shortcuts: HashMap<String, String>,
}

impl Session {
    pub(crate) fn new() -> Self {
        Self {
            enabled: true,
            method: InputMethod::Telex,
            tone_style: ToneStyle::Traditional,
            buffer: String::new(),
            keys: String::new(),
            smart_restore: true,
            eager_restore: true,
            spell_check: false,
            auto_capitalize: false,
            cap_sentence_ended: false,
            cap_armed: false,
            shortcuts: HashMap::new(),
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
        assert!(session.shortcuts.is_empty());
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

    #[test]
    fn clear_keeps_shortcuts() {
        // Shortcuts are session-wide config, not per-word state.
        let mut session = Session::new();
        session.shortcuts.insert("vn".into(), "Việt Nam".into());
        session.buffer.push('á');
        session.clear();
        assert_eq!(session.shortcuts.get("vn").map(String::as_str), Some("Việt Nam"));
    }
}
