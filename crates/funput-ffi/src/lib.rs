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
