//! Shared helpers for engine integration tests.

#![allow(dead_code)]

use funput_core::InputMethod;
use funput_engine::{Action, Engine, ImeResult};

/// Type keys through a fresh [`Engine`]; returns final buffer + per-step [`ImeResult`].
pub fn type_keys_with_results(method: InputMethod, keys: &str) -> (String, Vec<ImeResult>) {
    let mut engine = Engine::new();
    engine.set_method(method);
    let mut results = Vec::new();
    for key in keys.chars() {
        results.push(engine.process_char(key));
    }
    (engine.buffer().to_owned(), results)
}

/// Short alias for [`type_keys_with_results`].
pub fn type_keys(method: InputMethod, keys: &str) -> (String, Vec<ImeResult>) {
    type_keys_with_results(method, keys)
}

pub fn type_keys_buffer(method: InputMethod, keys: &str) -> String {
    type_keys_with_results(method, keys).0
}

/// Type space-separated words; simulates engine word-boundary clear between words.
pub fn type_words(method: InputMethod, text: &str) -> String {
    let mut engine = Engine::new();
    engine.set_method(method);
    let mut words = Vec::new();
    for (i, word) in text.split(' ').enumerate() {
        if i > 0 {
            engine.process_char(' ');
        }
        for key in word.chars() {
            engine.process_char(key);
        }
        words.push(engine.buffer().to_owned());
    }
    words.join(" ")
}

/// Reconstruct the app text from the inject stream (None → append key,
/// Send → delete `backspace` chars then append `output`).
pub fn app_text(method: InputMethod, keys: &str) -> String {
    let mut engine = Engine::new();
    engine.set_method(method);
    let mut app = String::new();
    for key in keys.chars() {
        let r = engine.process_char(key);
        match r.action {
            Action::None => app.push(key),
            Action::Send => {
                for _ in 0..r.backspace {
                    app.pop();
                }
                app.push_str(&r.output);
            }
            Action::Restore => unreachable!("Restore not implemented yet"),
        }
    }
    app
}
