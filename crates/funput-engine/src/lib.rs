//! IME orchestration — session, buffer, and platform inject instructions.
//!
//! `funput-core` answers: given buffer + key, what is the new composed text?
//! `funput-engine` answers: after this key, what should the platform do?
//!
//! # API FROZEN (Phase E4)
//!
//! Public surface: [`Engine`], [`Action`], [`ImeResult`], and their methods.
//! Breaking changes require semver coordination with `funput-ffi` and platform shells.
//!
//! # Contract
//!
//! - **Stateful:** holds composition buffer across keystrokes.
//! - **Delegates transform:** all Telex/VNI rules live in `funput-core`.
//! - **No I/O:** no keyboard hooks, no inject — platform reads [`ImeResult`].

mod boundary;
mod diff;
mod pipeline;
mod result;
mod session;

pub use result::{Action, ImeResult};

use session::Session;

/// Vietnamese IME engine — single source of truth for composition state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Engine {
    session: Session,
}

impl Engine {
    /// New engine with IME enabled and Telex as the default input method.
    pub fn new() -> Self {
        Self {
            session: Session::new(),
        }
    }

    /// Enable or disable Vietnamese composition. When disabled, [`Self::process_char`]
    /// returns [`Action::None`] and does not update buffer or keys.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.session.enabled = enabled;
    }

    /// Whether Vietnamese composition is active.
    pub fn is_enabled(&self) -> bool {
        self.session.enabled
    }

    /// Switch between Telex and VNI digit modifiers.
    pub fn set_method(&mut self, method: funput_core::InputMethod) {
        self.session.method = method;
    }

    /// Current input method.
    pub fn method(&self) -> funput_core::InputMethod {
        self.session.method
    }

    /// Reset composition state (buffer and raw keys) without changing enabled/method.
    pub fn clear(&mut self) {
        self.session.clear();
    }

    /// Composed syllable buffer — text the app should show for the current word.
    pub fn buffer(&self) -> &str {
        &self.session.buffer
    }

    /// Raw keystrokes since the last word boundary (used for English restore).
    pub fn keys(&self) -> &str {
        &self.session.keys
    }

    /// Process one Unicode scalar (platform maps keycode → char).
    ///
    /// # Behavior
    ///
    /// - **Disabled:** [`Action::None`], state unchanged.
    /// - **Word boundary** (whitespace / ASCII punctuation): optionally restore Latin
    ///   via [`Action::Send`] when `keys != buffer` and buffer is not a complete
    ///   Vietnamese syllable; then clear session. Otherwise pass the boundary key.
    /// - **Normal key:** append to `keys`, call `funput-core`, map
    ///   `TransformKind` → `ImeResult` (see crate docs / IMPLEMENTATION.md).
    pub fn process_char(&mut self, key: char) -> ImeResult {
        if !self.session.enabled {
            return ImeResult::none();
        }
        if boundary::is_word_boundary(key) {
            return boundary::on_word_boundary(&mut self.session, key);
        }
        self.session.keys.push(key);
        pipeline::process(&mut self.session, key)
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use funput_core::InputMethod;

    #[test]
    fn engine_new_defaults() {
        let engine = Engine::new();
        assert!(engine.is_enabled());
        assert_eq!(engine.method(), InputMethod::Telex);
        assert_eq!(engine.buffer(), "");
    }

    #[test]
    fn set_method_vni() {
        let mut engine = Engine::new();
        engine.set_method(InputMethod::Vni);
        assert_eq!(engine.method(), InputMethod::Vni);
    }

    #[test]
    fn set_enabled_false() {
        let mut engine = Engine::new();
        engine.set_enabled(false);
        assert!(!engine.is_enabled());
    }

    #[test]
    fn clear_smoke() {
        let mut engine = Engine::new();
        engine.clear();
        assert_eq!(engine.buffer(), "");
    }

    #[test]
    fn process_char_pending_updates_buffer_and_keys() {
        let mut engine = Engine::new();
        let result = engine.process_char('a');
        assert_eq!(result.action, Action::None);
        assert_eq!(result.backspace, 0);
        assert!(result.output.is_empty());
        assert_eq!(engine.buffer(), "a");
        assert_eq!(engine.keys(), "a");
    }

    #[test]
    fn disabled_does_not_touch_buffer_or_keys() {
        let mut engine = Engine::new();
        engine.set_enabled(false);
        let result = engine.process_char('a');
        assert_eq!(result.action, Action::None);
        assert_eq!(engine.buffer(), "");
        assert_eq!(engine.keys(), "");
    }

    #[test]
    fn word_boundary_clears_after_word() {
        let mut engine = Engine::new();
        engine.process_char('m');
        engine.process_char('a');
        let tone = engine.process_char('s');
        assert_eq!(tone.action, Action::Send);
        assert_eq!(engine.buffer(), "má");
        assert_eq!(engine.keys(), "mas");

        let space = engine.process_char(' ');
        assert_eq!(space.action, Action::None);
        assert_eq!(engine.buffer(), "");
        assert_eq!(engine.keys(), "");
    }

    #[test]
    fn word_boundary_on_empty_buffer() {
        let mut engine = Engine::new();
        let result = engine.process_char(' ');
        assert_eq!(result.action, Action::None);
        assert_eq!(engine.buffer(), "");
        assert_eq!(engine.keys(), "");
    }

    #[test]
    fn word_boundary_does_not_append_keys() {
        let mut engine = Engine::new();
        engine.process_char('a');
        assert_eq!(engine.keys(), "a");
        engine.process_char(' ');
        assert_eq!(engine.keys(), "");
    }
}
