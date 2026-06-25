//! The global low-level keyboard hook: intercepts keys, drives the engine, and
//! injects composed text. Runs on its own thread with a message loop (required for
//! `WH_KEYBOARD_LL`). The hook callback is a bare C function, so it reaches the
//! engine through [`crate::shell`]'s process-global state.

use std::sync::OnceLock;

use funput_desktop::{classify, plan_inject, KeyKind};
use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, HINSTANCE, HMODULE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Accessibility::{SetWinEventHook, HWINEVENTHOOK};
use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetMessageW, GetWindowThreadProcessId, SetWindowsHookExW,
    TranslateMessage, EVENT_SYSTEM_FOREGROUND, HC_ACTION, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL,
    WINEVENT_OUTOFCONTEXT, WM_KEYDOWN, WM_SYSKEYDOWN,
};

use crate::{inject, keymap, shell, tray};

/// Called after a Ctrl+` toggle so the tray can refresh its checkmark/icon.
type ToggleCb = Box<dyn Fn(bool) + Send + Sync>;
static ON_TOGGLE: OnceLock<ToggleCb> = OnceLock::new();

pub fn set_on_toggle(f: impl Fn(bool) + Send + Sync + 'static) {
    let _ = ON_TOGGLE.set(Box::new(f));
}

/// Install the hook on a dedicated thread with its own message pump.
pub fn spawn() {
    std::thread::spawn(|| unsafe {
        let hmod = GetModuleHandleW(None).unwrap_or_default();
        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), HINSTANCE(hmod.0), 0);
        if hook.is_err() {
            eprintln!("Funput: failed to install keyboard hook: {hook:?}");
            return;
        }
        // Also watch foreground changes for per-app VI/EN auto-switch. An OUT_OF_CONTEXT
        // WinEvent hook is delivered to this thread's message queue (same pump below),
        // so its callback shares the engine via `shell` with no extra synchronization.
        let _win_event = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            HMODULE(std::ptr::null_mut()), // OUT_OF_CONTEXT: no DLL module
            Some(win_event_proc),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        );

        // The tray lives on this thread too, so its menu/click messages flow through
        // the same pump and its events can be drained right after each dispatch.
        tray::install();

        // LL keyboard + WinEvent hooks (and the tray) are delivered through this
        // thread's message queue.
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
            tray::drain_events();
        }
    });
}

/// Foreground-window changed: record the app and apply its per-app VI/EN default.
unsafe extern "system" fn win_event_proc(
    _hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _thread: u32,
    _time: u32,
) {
    if event != EVENT_SYSTEM_FOREGROUND {
        return;
    }
    let Some((id, name)) = exe_of_window(hwnd) else {
        return;
    };
    shell::note_foreground(id.clone(), name);
    if let Some(on) = shell::apply_for_app(&id) {
        if let Some(cb) = ON_TOGGLE.get() {
            cb(on); // keep tray checkmark / tooltip in sync with the auto-switch
        }
    }
}

/// Resolve a window's owning process to `(id, name)` where `id` is the lowercased
/// exe file name (e.g. "code.exe") and `name` strips the extension (e.g. "code").
unsafe fn exe_of_window(hwnd: HWND) -> Option<(String, String)> {
    if hwnd.0.is_null() {
        return None;
    }
    let mut pid = 0u32;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    if pid == 0 {
        return None;
    }
    let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;

    let mut buf = [0u16; 260];
    let mut len = buf.len() as u32;
    let res = QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut len);
    let _ = CloseHandle(handle);
    res.ok()?;

    let full = String::from_utf16_lossy(&buf[..len as usize]);
    let file = full.rsplit(['\\', '/']).next().unwrap_or("").to_string();
    if file.is_empty() {
        return None;
    }
    let id = file.to_lowercase();
    let name = id.strip_suffix(".exe").unwrap_or(&id).to_string();
    Some((id, name))
}

unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        let kbd = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
        // Skip the events we ourselves synthesized via SendInput (no re-entrancy).
        if kbd.dwExtraInfo == shell::INJECT_TAG {
            return CallNextHookEx(None, code, wparam, lparam);
        }
        let msg = wparam.0 as u32;
        if (msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN) && handle_keydown(kbd) {
            return LRESULT(1); // swallow: do not pass the key to the focused app
        }
    }
    CallNextHookEx(None, code, wparam, lparam)
}

/// Returns true if the key should be swallowed (we injected a replacement), false
/// to let it reach the app.
fn handle_keydown(kbd: &KBDLLHOOKSTRUCT) -> bool {
    let vk = VIRTUAL_KEY(kbd.vkCode as u16);
    let mods = keymap::read_mods();

    if keymap::is_toggle(vk, mods, shell::toggle_hotkey()) {
        let on = shell::toggle_enabled_hotkey();
        if let Some(cb) = ON_TOGGLE.get() {
            cb(on);
        }
        return true;
    }

    if !shell::enabled() {
        return false; // English mode: hands off
    }

    match classify(&keymap::to_key_event(kbd)) {
        KeyKind::Compose(c) => {
            let plan = plan_inject(&shell::process_char(c));
            if plan.is_noop() {
                false // Action::None — the literal key reaches the app
            } else {
                // Chrome's omnibox eats a Backspace to clear its autocomplete
                // selection, so it gets a Delete primer first (see
                // inject::send_plan_chrome); everything else takes the direct path.
                if shell::foreground_is_chrome() {
                    inject::send_plan_chrome(&plan);
                } else {
                    inject::send_plan(&plan); // delete + retype the composed text
                }
                true
            }
        }
        KeyKind::Backspace => {
            if shell::is_composing() {
                shell::on_backspace(); // sync engine; app deletes its own char
            }
            false
        }
        KeyKind::Flush => {
            shell::clear(); // commit what is shown; nav/Enter/Tab/shortcut passes
            false
        }
        KeyKind::PassThrough => false,
    }
}
