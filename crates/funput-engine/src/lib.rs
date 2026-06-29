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
mod flip;
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

    /// Set the tone-mark placement style (traditional `hòa` vs modern `hoà`).
    pub fn set_tone_style(&mut self, style: funput_core::ToneStyle) {
        self.session.tone_style = style;
    }

    /// Current tone-mark placement style.
    pub fn tone_style(&self) -> funput_core::ToneStyle {
        self.session.tone_style
    }

    /// Toggle auto-restore of non-Vietnamese words to their raw Latin keystrokes
    /// (`card` stays `card` instead of composing `cảd`). When off, the composed
    /// buffer is always kept.
    pub fn set_smart_restore(&mut self, on: bool) {
        self.session.smart_restore = on;
    }

    /// Toggle eager restore — flip to raw keys the instant a word becomes a dead
    /// end, instead of waiting for a word boundary. Only applies while smart
    /// restore is on.
    pub fn set_eager_restore(&mut self, on: bool) {
        self.session.eager_restore = on;
    }

    /// Toggle spell-check ("Kiểm tra chính tả"): when on, a tone / shape / stroke is
    /// only placed if the result can still become a real Vietnamese syllable — an
    /// invalid one (`mix` + ngã → `mĩx`) keeps the modifier key as a literal instead.
    pub fn set_spell_check(&mut self, on: bool) {
        self.session.spell_check = on;
    }

    /// Toggle auto-capitalize ("Tự động viết hoa"): uppercase the first letter of a
    /// word that starts a sentence (sentence start, after `.`/`!`/`?` + space, after a
    /// newline). Off by default; when off this is a complete no-op.
    pub fn set_auto_capitalize(&mut self, on: bool) {
        self.session.auto_capitalize = on;
        if !on {
            self.session.cap_armed = false;
            self.session.cap_sentence_ended = false;
        }
    }

    /// Arm capitalization for the next word — the platform calls this when a text
    /// field gains focus, so the first letter typed (start of input) is capitalized.
    /// No-op unless auto-capitalize is on.
    pub fn arm_capitalization(&mut self) {
        if self.session.auto_capitalize {
            self.session.cap_armed = true;
        }
    }

    /// Reset composition state (buffer and raw keys) without changing enabled/method.
    pub fn clear(&mut self) {
        self.session.clear();
    }

    /// Define a text-expansion shortcut (gõ tắt): typing `trigger` then a word
    /// boundary injects `expansion` (`add_shortcut("vn", "Việt Nam")`). Triggers
    /// match the raw keystrokes case-sensitively and take priority over English
    /// restore. An empty `trigger` is ignored. Re-adding a trigger overwrites it.
    pub fn add_shortcut(&mut self, trigger: impl Into<String>, expansion: impl Into<String>) {
        let trigger = trigger.into();
        if trigger.is_empty() {
            return;
        }
        self.session.shortcuts.insert(trigger, expansion.into());
    }

    /// Remove a single shortcut by its trigger. No-op if it is not defined.
    pub fn remove_shortcut(&mut self, trigger: &str) {
        self.session.shortcuts.remove(trigger);
    }

    /// Remove every shortcut. Combine with [`Self::add_shortcut`] to replace the
    /// whole table when syncing from a config file.
    pub fn clear_shortcuts(&mut self) {
        self.session.shortcuts.clear();
    }

    /// The current shortcut table — trigger → expansion.
    pub fn shortcuts(&self) -> &std::collections::HashMap<String, String> {
        &self.session.shortcuts
    }

    /// Composed syllable buffer — text the app should show for the current word.
    pub fn buffer(&self) -> &str {
        &self.session.buffer
    }

    /// Raw keystrokes since the last word boundary (used for English restore).
    pub fn keys(&self) -> &str {
        &self.session.keys
    }

    /// The user pressed Backspace inside the current composition: drop the last
    /// character so the next keystroke composes against the corrected text
    /// (`Phua` → ⌫ → `Phu` → `s` → `Phú`, instead of losing the context).
    ///
    /// Returns [`Action::None`] — the Backspace key passes through so the app
    /// deletes its own last character, keeping app and engine in sync.
    pub fn on_backspace(&mut self) -> ImeResult {
        if !self.session.enabled {
            return ImeResult::none();
        }
        self.session.buffer.pop();
        // The remaining buffer is the corrected raw text for this word.
        self.session.keys = self.session.buffer.clone();
        ImeResult::none()
    }

    /// Flip the word being composed between its Vietnamese form and its raw
    /// keystrokes (`card` ⇄ `cải`), and back on a second call. Returns the
    /// delete+inject instruction ([`Action::Send`]) for hosts that type real text,
    /// or [`Action::None`] when there is nothing to flip: no live composition, or
    /// the word has no Vietnamese/raw distinction (`the`). Hosts that show marked
    /// text can ignore the payload and re-render [`Self::buffer`].
    ///
    /// The choice is sticky: further keystrokes keep the chosen form and the word
    /// boundary won't English-restore it back.
    pub fn flip_composing(&mut self) -> ImeResult {
        match flip::flip(
            &self.session.buffer,
            &self.session.keys,
            &self.session.vn_form,
        ) {
            Some((new_buffer, override_)) => {
                let (backspace, output) = diff::diff(&self.session.buffer, &new_buffer);
                self.session.buffer = new_buffer;
                self.session.restore_override = Some(override_);
                ImeResult::send(backspace, output)
            }
            None => ImeResult::none(),
        }
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
    ///   `TransformKind` → `ImeResult` (see the README).
    pub fn process_char(&mut self, key: char) -> ImeResult {
        if !self.session.enabled {
            return ImeResult::none();
        }
        if boundary::is_word_boundary(key) {
            return boundary::on_word_boundary(&mut self.session, key);
        }
        let key = self.maybe_capitalize(key);
        self.session.keys.push(key);
        pipeline::process(&mut self.session, key)
    }

    /// Auto-capitalize the first letter of a new word when armed. Consumes the armed
    /// state at the start of any word (letter or not), so it only ever affects the
    /// first keystroke. The first keystroke of a Telex/VNI word is an ASCII Latin
    /// letter, so `to_ascii_uppercase` is sufficient.
    fn maybe_capitalize(&mut self, key: char) -> char {
        if !self.session.auto_capitalize || !self.session.buffer.is_empty() {
            return key;
        }
        let armed = self.session.cap_armed;
        self.session.cap_armed = false;
        self.session.cap_sentence_ended = false;
        if armed && key.is_alphabetic() {
            key.to_ascii_uppercase()
        } else {
            key
        }
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

    /// Footprint headline for benchmarks: the engine's stack size stays tiny (run
    /// `cargo test -p funput-engine -- --nocapture engine_struct_size` to print it).
    #[test]
    fn engine_struct_size() {
        let bytes = std::mem::size_of::<Engine>();
        println!("size_of::<Engine>() = {bytes} bytes");
        assert!(
            bytes < 1024,
            "engine struct unexpectedly large: {bytes} bytes"
        );
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

    /// Type a word with smart restore off so the spell-check gate is the only thing
    /// that can alter the diacritic (eager restore would otherwise mask it).
    fn type_word(engine: &mut Engine, word: &str) -> String {
        engine.clear();
        for key in word.chars() {
            engine.process_char(key);
        }
        engine.buffer().to_string()
    }

    #[test]
    fn spell_check_off_keeps_legacy_diacritic() {
        let mut engine = Engine::new();
        engine.set_smart_restore(false);
        // Default: spell-check off → `tetf` composes `tèt` (huyền) as before, even
        // though a stop coda may only carry sắc / nặng.
        assert_eq!(type_word(&mut engine, "tetf"), "tèt");
    }

    #[test]
    fn spell_check_on_blocks_invalid_syllable() {
        let mut engine = Engine::new();
        engine.set_smart_restore(false);
        engine.set_spell_check(true);
        // `tèt` is not a real syllable → the huyền key stays a literal: `tetf`.
        assert_eq!(type_word(&mut engine, "tetf"), "tetf");
        // Real syllables are unaffected.
        assert_eq!(type_word(&mut engine, "mas"), "má");
        assert_eq!(type_word(&mut engine, "tets"), "tét");
    }

    // ---- Auto-capitalize ("Tự động viết hoa") ----

    fn engine_autocap() -> Engine {
        let mut e = Engine::new();
        e.set_auto_capitalize(true);
        e
    }

    /// Feed a string keystroke-by-keystroke; return the current composition buffer.
    fn feed(engine: &mut Engine, s: &str) -> String {
        for k in s.chars() {
            engine.process_char(k);
        }
        engine.buffer().to_string()
    }

    #[test]
    fn autocap_focus_capitalizes_first_word() {
        let mut e = engine_autocap();
        e.arm_capitalization();
        assert_eq!(feed(&mut e, "viet"), "Viet");
    }

    #[test]
    fn autocap_first_letter_composes_vietnamese() {
        let mut e = engine_autocap();
        e.arm_capitalization();
        assert_eq!(feed(&mut e, "chaof"), "Chào"); // Telex
        e.clear();
        e.arm_capitalization();
        assert_eq!(feed(&mut e, "dd"), "Đ"); // capital đ
    }

    #[test]
    fn autocap_after_sentence_end_and_space() {
        let mut e = engine_autocap();
        feed(&mut e, "ok");
        e.process_char('.'); // commits "ok", marks sentence end
        e.process_char(' '); // whitespace confirms → arm
        assert_eq!(feed(&mut e, "lam"), "Lam");
    }

    #[test]
    fn autocap_requires_whitespace_after_period() {
        let mut e = engine_autocap();
        feed(&mut e, "google");
        e.process_char('.'); // no whitespace follows
        assert_eq!(feed(&mut e, "com"), "com"); // google.com stays lower
    }

    #[test]
    fn autocap_newline_arms() {
        let mut e = engine_autocap();
        feed(&mut e, "ok");
        e.process_char('\n');
        assert_eq!(feed(&mut e, "lam"), "Lam");
    }

    #[test]
    fn autocap_comma_does_not_arm() {
        let mut e = engine_autocap();
        feed(&mut e, "ok");
        e.process_char(',');
        e.process_char(' ');
        assert_eq!(feed(&mut e, "lam"), "lam");
    }

    #[test]
    fn autocap_closing_quote_is_transparent() {
        let mut e = engine_autocap();
        feed(&mut e, "di");
        e.process_char('.'); // sentence end
        e.process_char('"'); // transparent closer
        e.process_char(' '); // arm
        assert_eq!(feed(&mut e, "roi"), "Roi");
    }

    #[test]
    fn autocap_off_is_noop() {
        let mut e = Engine::new(); // default off
        e.arm_capitalization(); // no-op while off
        assert_eq!(feed(&mut e, "viet"), "viet");
        e.process_char('.');
        e.process_char(' ');
        assert_eq!(feed(&mut e, "lam"), "lam");
    }

    #[test]
    fn backspace_keeps_composition_context() {
        // Typo "Phua", backspace the "a", then "s" → "Phú" (tone applies on "Phu").
        let mut engine = Engine::new();
        for key in "Phua".chars() {
            engine.process_char(key);
        }
        assert_eq!(engine.buffer(), "Phua");

        let bs = engine.on_backspace();
        assert_eq!(bs.action, Action::None);
        assert_eq!(engine.buffer(), "Phu");
        assert_eq!(engine.keys(), "Phu");

        let result = engine.process_char('s');
        assert_eq!(result.action, Action::Send);
        assert_eq!(engine.buffer(), "Phú");
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
    fn vni_keeps_composed_word_instead_of_exposing_digits() {
        // VNI: d-9-c composes "đc". It is not a complete syllable, but reverting
        // would surface the modifier digit ("d9c"), so the composed "đc" is kept.
        let mut engine = Engine::new();
        engine.set_method(InputMethod::Vni);
        for key in "d9c".chars() {
            engine.process_char(key);
        }
        assert_eq!(engine.buffer(), "đc");

        let space = engine.process_char(' ');
        assert_eq!(space.action, Action::None); // no restore → "đc" committed as-is
        assert_eq!(engine.buffer(), "");
    }

    #[test]
    fn telex_keeps_abbreviation_with_d_stroke() {
        // Telex: G-D-D composes "GĐ" (Giám đốc). Reverting would give "GDD"; the đ
        // marks it as intentional Vietnamese, so it is kept across methods.
        let mut engine = Engine::new(); // Telex by default
        for key in "GDD".chars() {
            engine.process_char(key);
        }
        assert_eq!(engine.buffer(), "GĐ");

        let space = engine.process_char(' ');
        assert_eq!(space.action, Action::None);
        assert_eq!(engine.buffer(), "");
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

    #[test]
    fn flip_eager_restored_word_to_vietnamese_and_back() {
        // "card" eager-restores to English mid-word (shown as "card"). Flipping
        // recovers the Vietnamese composition; flipping again returns to raw.
        let mut engine = Engine::new();
        type_word(&mut engine, "card");
        assert_eq!(engine.buffer(), "card"); // shown as raw English

        // Flip → a Send that rewrites the visible word to the Vietnamese form.
        let to_vn = engine.flip_composing();
        assert_eq!(to_vn.action, Action::Send);
        let vn = engine.buffer().to_string();
        assert_ne!(vn, "card");
        // The diff applied to "card" must reproduce the new buffer.
        assert_eq!(apply_diff("card", &to_vn), vn);

        let to_raw = engine.flip_composing();
        assert_eq!(to_raw.action, Action::Send);
        assert_eq!(engine.buffer(), "card"); // back to raw
        assert_eq!(apply_diff(&vn, &to_raw), "card");
    }

    /// Apply a `Send` result's `backspace`/`output` to `shown` — what a host that
    /// types real text (Windows) would end up displaying.
    fn apply_diff(shown: &str, result: &ImeResult) -> String {
        let mut chars: Vec<char> = shown.chars().collect();
        chars.truncate(chars.len() - result.backspace);
        chars.into_iter().chain(result.output.chars()).collect()
    }

    #[test]
    fn flip_to_vietnamese_sticks_across_word_boundary() {
        // After flipping "card" to Vietnamese, Space must not restore it to English.
        let mut engine = Engine::new();
        type_word(&mut engine, "card");
        engine.flip_composing();
        let vn = engine.buffer().to_string();

        let boundary = engine.process_char(' ');
        // No restore fired (the flip is sticky), so the boundary just passes through.
        assert_eq!(boundary.action, Action::None);
        // The next composition starts clean.
        assert_eq!(engine.buffer(), "");
        assert_ne!(vn, "card");
    }

    #[test]
    fn flip_kept_vietnamese_word_to_raw() {
        // "má" is valid Vietnamese; flipping shows the raw keys "mas".
        let mut engine = Engine::new();
        type_word(&mut engine, "mas");
        assert_eq!(engine.buffer(), "má");
        assert_eq!(engine.flip_composing().action, Action::Send);
        assert_eq!(engine.buffer(), "mas");
    }

    #[test]
    fn flip_is_noop_without_a_flippable_word() {
        let mut engine = Engine::new();
        assert_eq!(engine.flip_composing().action, Action::None); // nothing composing
        type_word(&mut engine, "the"); // composes to itself — no VN/raw distinction
        assert_eq!(engine.flip_composing().action, Action::None);
        assert_eq!(engine.buffer(), "the");
    }

    #[test]
    fn flip_choice_resets_after_the_word_commits() {
        // The override is per-word: a fresh word restores normally again.
        let mut engine = Engine::new();
        type_word(&mut engine, "card");
        engine.flip_composing(); // force Vietnamese
        engine.process_char(' '); // commit + clear
        type_word(&mut engine, "card"); // a new word
        assert_eq!(engine.buffer(), "card"); // eager-restored again, override gone
    }
}
