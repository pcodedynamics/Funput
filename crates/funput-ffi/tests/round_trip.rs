//! Round-trip tests: drive the `extern "C"` API exactly like a C caller would.

use funput_ffi::{
    funput_add_shortcut, funput_backspace, funput_buffer, funput_clear, funput_clear_shortcuts,
    funput_engine_free, funput_engine_new, funput_process_char, funput_set_method, FunputResult,
    ACTION_NONE, ACTION_SEND,
};

fn output(result: &FunputResult) -> String {
    result.chars[..result.count as usize]
        .iter()
        .filter_map(|&c| char::from_u32(c))
        .collect()
}

/// Act as the platform: feed a string, apply each result to an app-text model.
/// Safe wrapper that manages one engine handle internally.
fn type_str(method: u8, input: &str) -> String {
    unsafe {
        let engine = funput_engine_new();
        funput_set_method(engine, method);

        let mut app = String::new();
        for ch in input.chars() {
            let result = funput_process_char(engine, ch as u32);
            if result.action == ACTION_NONE {
                app.push(ch);
            } else {
                for _ in 0..result.backspace {
                    app.pop();
                }
                app.push_str(&output(&result));
            }
        }

        funput_engine_free(engine);
        app
    }
}

#[test]
fn telex_round_trip() {
    assert_eq!(type_str(0, "as"), "á");
    assert_eq!(type_str(0, "dd"), "đ");
    assert_eq!(type_str(0, "xins chaof"), "xín chào");
}

#[test]
fn vni_round_trip() {
    assert_eq!(type_str(1, "a1"), "á");
    assert_eq!(type_str(1, "d9"), "đ");
    assert_eq!(type_str(1, "ma1 ca2"), "má cà");
}

#[test]
fn english_restore_on_boundary() {
    assert_eq!(type_str(0, "card "), "card ");
    assert_eq!(type_str(0, "cool "), "cool ");
    assert_eq!(type_str(0, "mas "), "má ");
}

#[test]
fn result_fields_for_as() {
    unsafe {
        let engine = funput_engine_new();
        funput_set_method(engine, 0);

        let r1 = funput_process_char(engine, 'a' as u32);
        assert_eq!(r1.action, ACTION_NONE);
        assert_eq!(r1.count, 0);

        let r2 = funput_process_char(engine, 's' as u32);
        assert_eq!(r2.action, ACTION_SEND);
        assert_eq!(r2.backspace, 1);
        assert_eq!(r2.count, 1);
        assert_eq!(r2.chars[0], 'á' as u32);

        funput_engine_free(engine);
    }
}

#[test]
fn backspace_corrects_composition() {
    unsafe {
        let engine = funput_engine_new();
        funput_set_method(engine, 0); // Telex
        for ch in "Phua".chars() {
            funput_process_char(engine, ch as u32);
        }
        let bs = funput_backspace(engine); // delete the 'a'
        assert_eq!(bs.action, ACTION_NONE);

        let toned = funput_process_char(engine, 's' as u32);
        assert_eq!(toned.action, ACTION_SEND);
        assert_eq!(output(&toned), "ú"); // "Phu" + s → "Phú"
        funput_engine_free(engine);
    }
}

#[test]
fn null_handle_is_safe() {
    unsafe {
        let result = funput_process_char(std::ptr::null_mut(), 'a' as u32);
        assert_eq!(result.action, ACTION_NONE);
        // These must not crash.
        funput_clear(std::ptr::null_mut());
        funput_backspace(std::ptr::null_mut());
        funput_engine_free(std::ptr::null_mut());
    }
}

/// Read the composed buffer the way a host renders marked text.
fn read_buffer(engine: *const funput_ffi::FunputEngine) -> String {
    let mut out = [0u32; 64];
    let count = unsafe { funput_buffer(engine, out.as_mut_ptr(), out.len()) };
    out[..count]
        .iter()
        .filter_map(|&c| char::from_u32(c))
        .collect()
}

#[test]
fn buffer_reflects_marked_text() {
    unsafe {
        let engine = funput_engine_new();
        funput_set_method(engine, 1); // VNI

        funput_process_char(engine, 'a' as u32);
        assert_eq!(read_buffer(engine), "a"); // pending, no tone yet

        funput_process_char(engine, '1' as u32);
        assert_eq!(read_buffer(engine), "á"); // tone applied in place

        funput_clear(engine);
        assert_eq!(read_buffer(engine), "");

        funput_engine_free(engine);
    }
}

#[test]
fn buffer_null_safe() {
    let mut out = [0u32; 8];
    unsafe {
        assert_eq!(funput_buffer(std::ptr::null(), out.as_mut_ptr(), out.len()), 0);
        let engine = funput_engine_new();
        assert_eq!(funput_buffer(engine, std::ptr::null_mut(), 8), 0);
        funput_engine_free(engine);
    }
}

#[test]
fn invalid_codepoint_is_noop() {
    unsafe {
        let engine = funput_engine_new();
        // 0xD800 is a UTF-16 surrogate — not a valid Unicode scalar.
        let result = funput_process_char(engine, 0xD800);
        assert_eq!(result.action, ACTION_NONE);
        funput_engine_free(engine);
    }
}

/// Add a shortcut the way a host does: marshal both strings as UTF-32 slices.
unsafe fn add_shortcut(engine: *mut funput_ffi::FunputEngine, trigger: &str, expansion: &str) {
    let t: Vec<u32> = trigger.chars().map(|c| c as u32).collect();
    let e: Vec<u32> = expansion.chars().map(|c| c as u32).collect();
    unsafe { funput_add_shortcut(engine, t.as_ptr(), t.len(), e.as_ptr(), e.len()) };
}

#[test]
fn shortcut_expands_through_ffi() {
    unsafe {
        let engine = funput_engine_new();
        funput_set_method(engine, 0); // Telex
        add_shortcut(engine, "vn", "Việt Nam");
        add_shortcut(engine, "kg", "không");

        let mut app = String::new();
        for ch in "vn ".chars() {
            let r = funput_process_char(engine, ch as u32);
            if r.action == ACTION_NONE {
                app.push(ch);
            } else {
                for _ in 0..r.backspace {
                    app.pop();
                }
                app.push_str(&output(&r));
            }
        }
        assert_eq!(app, "Việt Nam ");

        funput_engine_free(engine);
    }
}

#[test]
fn clear_shortcuts_disables_expansion() {
    unsafe {
        let engine = funput_engine_new();
        funput_set_method(engine, 0);
        add_shortcut(engine, "vn", "Việt Nam");
        funput_clear_shortcuts(engine);

        // After clearing, the boundary returns Action::None (no expansion) — "vn" is a
        // complete-enough syllable to be kept as-is, so the app sees the literal text.
        funput_process_char(engine, 'v' as u32);
        funput_process_char(engine, 'n' as u32);
        let boundary = funput_process_char(engine, ' ' as u32);
        assert_eq!(boundary.action, ACTION_NONE);

        funput_engine_free(engine);
    }
}

#[test]
fn shortcut_null_safe() {
    unsafe {
        // Null handle: must not crash.
        funput_clear_shortcuts(std::ptr::null_mut());
        let t = [b'v' as u32, b'n' as u32];
        funput_add_shortcut(std::ptr::null_mut(), t.as_ptr(), t.len(), t.as_ptr(), t.len());

        // Null string pointers are treated as empty (empty trigger is ignored).
        let engine = funput_engine_new();
        funput_add_shortcut(engine, std::ptr::null(), 0, std::ptr::null(), 0);
        funput_engine_free(engine);
    }
}
