//! Platform-agnostic logic for desktop "hook + inject" input shells.
//!
//! A global-hook shell (Windows `WH_KEYBOARD_LL`, or any host that intercepts raw
//! key events and types text back) only differs from the others in *how* it reads
//! keys and *how* it injects text. The decisions in between — what a key means and
//! what to emit for an [`ImeResult`] — are pure and live here so they can be unit
//! tested without any OS APIs. This mirrors the model in `funput-term`'s
//! `result_bytes`, but produces a host-neutral plan instead of terminal bytes.

use funput_engine::{Action, ImeResult};

/// What to emit to the focused app for an [`ImeResult`]: delete `backspaces`
/// characters, then type `units` (the UTF-16 code units of the composed output).
///
/// UTF-16 because that is what Windows `SendInput` (`KEYEVENTF_UNICODE`) consumes;
/// Vietnamese NFC stays in the BMP (one unit per char), but surrogate pairs are
/// handled correctly for any other text.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InjectPlan {
    /// Number of preceding characters to delete (Backspace presses).
    pub backspaces: usize,
    /// UTF-16 code units to type after the deletions.
    pub units: Vec<u16>,
}

impl InjectPlan {
    /// Nothing to inject — the key should pass through to the app unchanged.
    pub fn is_noop(&self) -> bool {
        self.backspaces == 0 && self.units.is_empty()
    }
}

/// Translate an engine result into an [`InjectPlan`].
///
/// - [`Action::None`] → empty plan: let the key reach the app untouched.
/// - [`Action::Send`] / [`Action::Restore`] → delete `backspace` chars, then type
///   `output`. The triggering key is swallowed by the shell.
pub fn plan_inject(result: &ImeResult) -> InjectPlan {
    match result.action {
        Action::None => InjectPlan::default(),
        Action::Send | Action::Restore => InjectPlan {
            backspaces: result.backspace,
            units: result.output.encode_utf16().collect(),
        },
    }
}

/// Modifier keys held when a key is pressed. `shift` is tracked but does **not**
/// by itself mark a system shortcut (Shift is part of normal typing).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Mods {
    pub ctrl: bool,
    pub alt: bool,
    pub win: bool,
    pub shift: bool,
}

impl Mods {
    /// A non-Shift modifier is held, i.e. the key is part of a system shortcut
    /// (Ctrl+A, Alt+Tab, Win+…) and must not be composed.
    pub fn is_shortcut(&self) -> bool {
        self.ctrl || self.alt || self.win
    }
}

/// A normalized key event the shell feeds the classifier. `ch` is the character
/// the key would produce (from `ToUnicodeEx` on Windows), if any.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    pub mods: Mods,
    pub ch: Option<char>,
    /// Backspace / Delete-back.
    pub is_backspace: bool,
    /// Caret-moving or non-text key: arrows, Home/End, PageUp/Down, Esc, Delete,
    /// Insert, F-keys, Enter, Tab.
    pub is_navigation: bool,
}

/// What the shell should do with a key while Vietnamese mode is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyKind {
    /// Feed this character to the engine (printable text key, incl. space/punct —
    /// the engine itself decides word boundaries).
    Compose(char),
    /// Backspace pressed — call `engine.backspace()` and apply its result.
    Backspace,
    /// Flush the composition (commit/clear) and let the key pass through —
    /// navigation, function keys, or a system shortcut.
    Flush,
    /// Irrelevant key (no character, not navigation) — pass through, leave the
    /// composition as-is.
    PassThrough,
}

/// Decide what a key means for composition. Toggle (VI/EN) is handled by the shell
/// *before* this, since the toggle combo is configurable and host-specific.
pub fn classify(ev: &KeyEvent) -> KeyKind {
    if ev.mods.is_shortcut() {
        return KeyKind::Flush;
    }
    if ev.is_backspace {
        return KeyKind::Backspace;
    }
    if ev.is_navigation {
        return KeyKind::Flush;
    }
    match ev.ch {
        Some(c) => KeyKind::Compose(c),
        None => KeyKind::PassThrough,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use funput_engine::Action;

    fn key(ch: Option<char>) -> KeyEvent {
        KeyEvent {
            mods: Mods::default(),
            ch,
            is_backspace: false,
            is_navigation: false,
        }
    }

    #[test]
    fn plan_none_is_noop() {
        let plan = plan_inject(&ImeResult {
            action: Action::None,
            backspace: 0,
            output: String::new(),
        });
        assert!(plan.is_noop());
    }

    #[test]
    fn plan_send_deletes_then_types_utf16() {
        let plan = plan_inject(&ImeResult {
            action: Action::Send,
            backspace: 1,
            output: "á".into(),
        });
        assert_eq!(plan.backspaces, 1);
        assert_eq!(plan.units, "á".encode_utf16().collect::<Vec<_>>());
        assert_eq!(plan.units.len(), 1); // BMP: one unit
    }

    #[test]
    fn plan_restore_word() {
        let plan = plan_inject(&ImeResult {
            action: Action::Restore,
            backspace: 3,
            output: "card ".into(),
        });
        assert_eq!(plan.backspaces, 3);
        assert_eq!(plan.units, "card ".encode_utf16().collect::<Vec<_>>());
    }

    #[test]
    fn classify_printable_composes() {
        assert_eq!(classify(&key(Some('a'))), KeyKind::Compose('a'));
        assert_eq!(classify(&key(Some(' '))), KeyKind::Compose(' ')); // boundary → engine decides
        assert_eq!(classify(&key(Some('1'))), KeyKind::Compose('1'));
    }

    #[test]
    fn classify_shortcut_flushes() {
        let mut ev = key(Some('a'));
        ev.mods.ctrl = true;
        assert_eq!(classify(&ev), KeyKind::Flush); // Ctrl+A must not compose
    }

    #[test]
    fn classify_shift_still_composes() {
        let mut ev = key(Some('A'));
        ev.mods.shift = true;
        assert_eq!(classify(&ev), KeyKind::Compose('A'));
    }

    #[test]
    fn classify_backspace_and_navigation() {
        let mut bs = key(None);
        bs.is_backspace = true;
        assert_eq!(classify(&bs), KeyKind::Backspace);

        let mut nav = key(None);
        nav.is_navigation = true;
        assert_eq!(classify(&nav), KeyKind::Flush);
    }

    #[test]
    fn classify_no_char_passes_through() {
        assert_eq!(classify(&key(None)), KeyKind::PassThrough);
    }
}
