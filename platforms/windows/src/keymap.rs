//! Translate a Windows low-level keyboard event into the host-neutral
//! [`funput_desktop::KeyEvent`] the classifier understands.

use funput_desktop::{KeyEvent, Mods};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, GetKeyState, GetKeyboardLayout, ToUnicodeEx, VIRTUAL_KEY, VK_BACK, VK_CAPITAL,
    VK_CONTROL, VK_DELETE, VK_DOWN, VK_END, VK_ESCAPE, VK_HOME, VK_INSERT, VK_LEFT, VK_LWIN,
    VK_MENU, VK_NEXT, VK_OEM_3, VK_PRIOR, VK_RETURN, VK_RIGHT, VK_RWIN, VK_SHIFT, VK_SPACE, VK_TAB,
    VK_UP,
};
use windows::Win32::UI::WindowsAndMessaging::KBDLLHOOKSTRUCT;

use crate::settings::Hotkey;

fn async_down(vk: VIRTUAL_KEY) -> bool {
    (unsafe { GetAsyncKeyState(vk.0 as i32) } as u16 & 0x8000) != 0
}

/// Physical modifier state. Shift is reported but is not a "shortcut" by itself.
pub fn read_mods() -> Mods {
    Mods {
        ctrl: async_down(VK_CONTROL),
        alt: async_down(VK_MENU),
        win: async_down(VK_LWIN) || async_down(VK_RWIN),
        shift: async_down(VK_SHIFT),
    }
}

/// Whether this keydown matches the configured VI/EN toggle hotkey.
pub fn is_toggle(vk: VIRTUAL_KEY, mods: Mods, hotkey: Hotkey) -> bool {
    match hotkey {
        Hotkey::CtrlBacktick => mods.ctrl && !mods.alt && !mods.win && vk == VK_OEM_3,
        Hotkey::CtrlSpace => mods.ctrl && !mods.alt && !mods.win && vk == VK_SPACE,
        // Modifier-only combo: fires on the second modifier's keydown.
        Hotkey::AltShift => mods.alt && mods.shift && (vk == VK_SHIFT || vk == VK_MENU),
    }
}

fn is_navigation(vk: VIRTUAL_KEY) -> bool {
    if matches!(
        vk,
        VK_RETURN | VK_TAB | VK_ESCAPE | VK_LEFT | VK_RIGHT | VK_UP | VK_DOWN | VK_HOME | VK_END
            | VK_PRIOR | VK_NEXT | VK_DELETE | VK_INSERT
    ) {
        return true;
    }
    // F1..F24 (0x70..=0x87).
    (0x70..=0x87).contains(&vk.0)
}

/// The character this key would produce, ignoring Ctrl/Alt (those keys never
/// compose). Shift and CapsLock are honored so uppercase letters reach the engine.
fn translate_char(kbd: &KBDLLHOOKSTRUCT) -> Option<char> {
    let mut state = [0u8; 256];
    if async_down(VK_SHIFT) {
        state[VK_SHIFT.0 as usize] = 0x80;
    }
    if (unsafe { GetKeyState(VK_CAPITAL.0 as i32) } & 0x0001) != 0 {
        state[VK_CAPITAL.0 as usize] = 0x01;
    }
    let layout = unsafe { GetKeyboardLayout(0) };
    let mut buf = [0u16; 8];
    let n = unsafe { ToUnicodeEx(kbd.vkCode, kbd.scanCode, &state, &mut buf, 0, layout) };
    if n == 1 {
        char::from_u32(buf[0] as u32)
    } else {
        None // dead key (<0), no mapping (0), or multi-char (>1): not composable
    }
}

pub fn to_key_event(kbd: &KBDLLHOOKSTRUCT) -> KeyEvent {
    let vk = VIRTUAL_KEY(kbd.vkCode as u16);
    KeyEvent {
        mods: read_mods(),
        ch: translate_char(kbd),
        is_backspace: vk == VK_BACK,
        is_navigation: is_navigation(vk),
    }
}
