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

use funput_core::InputMethod;
use funput_engine::Engine;

pub use types::{FunputResult, ACTION_NONE, ACTION_RESTORE, ACTION_SEND, CHARS_CAP};

const METHOD_VNI: u8 = 1;

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
