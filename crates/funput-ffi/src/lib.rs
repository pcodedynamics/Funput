//! C ABI boundary for `funput-engine`.
//!
//! Lets a non-Rust shell (Swift IMKit, C# on Windows, a C terminal interposer…)
//! drive the Vietnamese IME engine. A Rust consumer should link `funput-engine`
//! directly instead.
//!
//! # Model
//!
//! Handle-based: [`funput_engine_new`] returns an opaque `*mut Engine`; pass it
//! to every call and release it with [`funput_engine_free`]. Results come back
//! by value as [`FunputResult`] (POD, no allocation, no free).
//!
//! # Safety
//!
//! All functions are null-safe: a null handle (or invalid codepoint) yields a
//! no-op [`FunputResult::none`] / does nothing. The caller must not use a handle
//! after freeing it, and must not free the same handle twice.

mod types;

use funput_core::{InputMethod, ToneStyle};
use funput_engine::Engine;

pub use types::{ACTION_NONE, ACTION_RESTORE, ACTION_SEND, CHARS_CAP, FunputResult};

const METHOD_VNI: u8 = 1;
const TONE_STYLE_MODERN: u8 = 1;

/// Opaque IME engine handle for C callers. Create with [`funput_engine_new`],
/// release with [`funput_engine_free`]. cbindgen emits this as an opaque struct.
pub struct FunputEngine {
    inner: Engine,
}

/// Create a new engine. Release it with [`funput_engine_free`].
#[unsafe(no_mangle)]
pub extern "C" fn funput_engine_new() -> *mut FunputEngine {
    Box::into_raw(Box::new(FunputEngine {
        inner: Engine::new(),
    }))
}

/// Free an engine created by [`funput_engine_new`]. Null is ignored.
///
/// # Safety
/// `engine` must come from [`funput_engine_new`] and not be freed already.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn funput_engine_free(engine: *mut FunputEngine) {
    if !engine.is_null() {
        drop(unsafe { Box::from_raw(engine) });
    }
}

/// Set the input method: `0 = Telex`, `1 = VNI` (any other value = Telex).
///
/// # Safety
/// `engine` must be a valid handle or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn funput_set_method(engine: *mut FunputEngine, method: u8) {
    if let Some(engine) = unsafe { engine.as_mut() } {
        let method = if method == METHOD_VNI {
            InputMethod::Vni
        } else {
            InputMethod::Telex
        };
        engine.inner.set_method(method);
    }
}

/// Set the tone-mark placement style: `0 = Traditional` (`hòa`), `1 = Modern`
/// (`hoà`) — any other value = Traditional.
///
/// # Safety
/// `engine` must be a valid handle or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn funput_set_tone_style(engine: *mut FunputEngine, style: u8) {
    if let Some(engine) = unsafe { engine.as_mut() } {
        let style = if style == TONE_STYLE_MODERN {
            ToneStyle::Modern
        } else {
            ToneStyle::Traditional
        };
        engine.inner.set_tone_style(style);
    }
}

/// Enable or disable Vietnamese composition.
///
/// # Safety
/// `engine` must be a valid handle or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn funput_set_enabled(engine: *mut FunputEngine, enabled: bool) {
    if let Some(engine) = unsafe { engine.as_mut() } {
        engine.inner.set_enabled(enabled);
    }
}

/// Toggle auto-restore of non-Vietnamese words to their raw Latin keystrokes
/// (`card` stays `card`). When off, the composed buffer is always kept.
///
/// # Safety
/// `engine` must be a valid handle or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn funput_set_smart_restore(engine: *mut FunputEngine, on: bool) {
    if let Some(engine) = unsafe { engine.as_mut() } {
        engine.inner.set_smart_restore(on);
    }
}

/// Toggle eager restore — flip to raw keys the instant a word dead-ends instead of
/// waiting for a word boundary. Only applies while smart restore is on.
///
/// # Safety
/// `engine` must be a valid handle or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn funput_set_eager_restore(engine: *mut FunputEngine, on: bool) {
    if let Some(engine) = unsafe { engine.as_mut() } {
        engine.inner.set_eager_restore(on);
    }
}

/// Toggle spell-check ("Kiểm tra chính tả") — only place a diacritic when the result
/// can still become a real Vietnamese syllable, otherwise keep the modifier key as a
/// literal. Off by default.
///
/// # Safety
/// `engine` must be a valid handle or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn funput_set_spell_check(engine: *mut FunputEngine, on: bool) {
    if let Some(engine) = unsafe { engine.as_mut() } {
        engine.inner.set_spell_check(on);
    }
}

/// Toggle auto-capitalize ("Tự động viết hoa") — uppercase the first letter of a word
/// that starts a sentence. Off by default; a no-op while off.
///
/// # Safety
/// `engine` must be a valid handle or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn funput_set_auto_capitalize(engine: *mut FunputEngine, on: bool) {
    if let Some(engine) = unsafe { engine.as_mut() } {
        engine.inner.set_auto_capitalize(on);
    }
}

/// Arm capitalization for the next word — call on text-field focus so the first
/// letter typed (start of input) is capitalized. A no-op unless auto-capitalize is on.
///
/// # Safety
/// `engine` must be a valid handle or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn funput_arm_capitalization(engine: *mut FunputEngine) {
    if let Some(engine) = unsafe { engine.as_mut() } {
        engine.inner.arm_capitalization();
    }
}

/// Reset composition state (buffer + raw keys), e.g. on focus change.
///
/// # Safety
/// `engine` must be a valid handle or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn funput_clear(engine: *mut FunputEngine) {
    if let Some(engine) = unsafe { engine.as_mut() } {
        engine.inner.clear();
    }
}

/// Process one Unicode scalar. Returns the platform instruction by value.
///
/// A null handle or invalid `codepoint` yields [`FunputResult::none`].
///
/// # Safety
/// `engine` must be a valid handle or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn funput_process_char(
    engine: *mut FunputEngine,
    codepoint: u32,
) -> FunputResult {
    let Some(engine) = (unsafe { engine.as_mut() }) else {
        return FunputResult::none();
    };
    let Some(ch) = char::from_u32(codepoint) else {
        return FunputResult::none();
    };
    FunputResult::from_ime(&engine.inner.process_char(ch))
}

/// Copy the current composed buffer (the text the host shows as marked/underlined
/// composition) as UTF-32 into `out`, up to `cap` codepoints. Returns the number
/// of codepoints written.
///
/// Null-safe: a null handle or null `out` yields `0`.
///
/// # Safety
/// `engine` must be a valid handle or null. `out` must point to writable storage
/// for at least `cap` `u32` values, or be null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn funput_buffer(
    engine: *const FunputEngine,
    out: *mut u32,
    cap: usize,
) -> usize {
    let Some(engine) = (unsafe { engine.as_ref() }) else {
        return 0;
    };
    if out.is_null() {
        return 0;
    }
    let mut count = 0;
    for ch in engine.inner.buffer().chars() {
        if count >= cap {
            break;
        }
        unsafe { *out.add(count) = ch as u32 };
        count += 1;
    }
    count
}

/// Define a text-expansion shortcut (gõ tắt): typing `trigger` then a word boundary
/// injects `expansion` (`vn` → `Việt Nam`). Both strings are passed as UTF-32
/// (`*const u32` + length), matching [`funput_buffer`]. An empty trigger is ignored
/// by the engine; re-adding a trigger overwrites it.
///
/// Hosts sync the whole table by calling [`funput_clear_shortcuts`] then adding each
/// entry (the engine is a runtime mirror of the host's config).
///
/// Null-safe: a null handle does nothing. A null `trigger`/`expansion` pointer is
/// treated as an empty string.
///
/// # Safety
/// `engine` must be a valid handle or null. `trigger` must point to at least
/// `trigger_len` `u32` values (or be null), and `expansion` to at least
/// `expansion_len` `u32` values (or be null).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn funput_add_shortcut(
    engine: *mut FunputEngine,
    trigger: *const u32,
    trigger_len: usize,
    expansion: *const u32,
    expansion_len: usize,
) {
    if let Some(engine) = unsafe { engine.as_mut() } {
        let trigger = unsafe { string_from_utf32(trigger, trigger_len) };
        let expansion = unsafe { string_from_utf32(expansion, expansion_len) };
        engine.inner.add_shortcut(trigger, expansion);
    }
}

/// Remove every text-expansion shortcut. Combine with [`funput_add_shortcut`] to
/// replace the whole table when syncing from config.
///
/// # Safety
/// `engine` must be a valid handle or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn funput_clear_shortcuts(engine: *mut FunputEngine) {
    if let Some(engine) = unsafe { engine.as_mut() } {
        engine.inner.clear_shortcuts();
    }
}

/// Decode `len` UTF-32 codepoints at `ptr` into a `String`, skipping invalid
/// scalars. A null pointer yields an empty string.
///
/// # Safety
/// `ptr` must point to at least `len` `u32` values, or be null.
unsafe fn string_from_utf32(ptr: *const u32, len: usize) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    slice.iter().filter_map(|&c| char::from_u32(c)).collect()
}

/// Backspace inside the current composition: drop the last composed character so
/// the next keystroke composes against the corrected text (`Phua` ⌫ `s` → `Phú`).
///
/// Returns a no-op result — the host passes the Backspace through to delete one
/// character in the app.
///
/// # Safety
/// `engine` must be a valid handle or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn funput_backspace(engine: *mut FunputEngine) -> FunputResult {
    let Some(engine) = (unsafe { engine.as_mut() }) else {
        return FunputResult::none();
    };
    FunputResult::from_ime(&engine.inner.on_backspace())
}

/// Flip the word being composed between its Vietnamese form and its raw keystrokes
/// (`card` ⇄ `cải`), and back on a second call. Returns the delete+inject the host
/// should apply (`ACTION_SEND`), or [`FunputResult::none`] when there is nothing to
/// flip. Hosts that show marked text can ignore the payload and re-render
/// [`funput_buffer`] after a non-`ACTION_NONE` result.
///
/// # Safety
/// `engine` must be a valid handle or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn funput_flip_composing(engine: *mut FunputEngine) -> FunputResult {
    let Some(engine) = (unsafe { engine.as_mut() }) else {
        return FunputResult::none();
    };
    FunputResult::from_ime(&engine.inner.flip_composing())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flip_composing_on_fresh_engine_is_noop() {
        let engine = funput_engine_new();
        let result = unsafe { funput_flip_composing(engine) };
        assert_eq!(result.action, ACTION_NONE); // nothing composing
        unsafe { funput_engine_free(engine) };
    }
}
