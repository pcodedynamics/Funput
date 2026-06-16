//! C ABI result type and the marshalling from [`funput_engine::ImeResult`].

use funput_engine::{Action, ImeResult};

/// Max output codepoints carried inline. Generous enough for English-restore of
/// long words; longer output is truncated (practically never happens).
pub const CHARS_CAP: usize = 64;

/// Platform action: `0 = None`, `1 = Send`, `2 = Restore`.
pub const ACTION_NONE: u8 = 0;
pub const ACTION_SEND: u8 = 1;
pub const ACTION_RESTORE: u8 = 2;

/// Result of one keystroke, returned by value (POD, no allocation, no free).
///
/// `chars[0..count]` are the output codepoints (UTF-32) to inject after deleting
/// `backspace` characters. For `action == ACTION_NONE`, `count == 0`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FunputResult {
    pub action: u8,
    pub backspace: u32,
    pub count: u32,
    pub chars: [u32; CHARS_CAP],
}

impl FunputResult {
    /// No-op result — pass the key through.
    pub fn none() -> Self {
        Self {
            action: ACTION_NONE,
            backspace: 0,
            count: 0,
            chars: [0; CHARS_CAP],
        }
    }

    /// Marshal an engine [`ImeResult`] into the C struct.
    pub fn from_ime(result: &ImeResult) -> Self {
        let action = match result.action {
            Action::None => ACTION_NONE,
            Action::Send => ACTION_SEND,
            Action::Restore => ACTION_RESTORE,
        };

        let mut chars = [0u32; CHARS_CAP];
        let mut count = 0usize;
        for ch in result.output.chars() {
            if count >= CHARS_CAP {
                break;
            }
            chars[count] = ch as u32;
            count += 1;
        }

        Self {
            action,
            backspace: result.backspace as u32,
            count: count as u32,
            chars,
        }
    }

    /// Output codepoints as a `String` — test helper.
    #[cfg(test)]
    pub(crate) fn output_string(&self) -> String {
        self.chars[..self.count as usize]
            .iter()
            .filter_map(|&c| char::from_u32(c))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn none_is_empty() {
        let r = FunputResult::none();
        assert_eq!(r.action, ACTION_NONE);
        assert_eq!(r.backspace, 0);
        assert_eq!(r.count, 0);
    }

    #[test]
    fn from_ime_send() {
        let ime = ImeResult {
            action: Action::Send,
            backspace: 1,
            output: "á".into(),
        };
        let r = FunputResult::from_ime(&ime);
        assert_eq!(r.action, ACTION_SEND);
        assert_eq!(r.backspace, 1);
        assert_eq!(r.count, 1);
        assert_eq!(r.chars[0], 'á' as u32);
        assert_eq!(r.output_string(), "á");
    }

    #[test]
    fn from_ime_none_and_restore() {
        let none = FunputResult::from_ime(&ImeResult {
            action: Action::None,
            backspace: 0,
            output: String::new(),
        });
        assert_eq!(none.action, ACTION_NONE);
        assert_eq!(none.count, 0);

        let restore = FunputResult::from_ime(&ImeResult {
            action: Action::Restore,
            backspace: 3,
            output: "abc".into(),
        });
        assert_eq!(restore.action, ACTION_RESTORE);
        assert_eq!(restore.output_string(), "abc");
    }

    #[test]
    fn output_longer_than_cap_truncates() {
        let long: String = "a".repeat(CHARS_CAP + 10);
        let r = FunputResult::from_ime(&ImeResult {
            action: Action::Send,
            backspace: 0,
            output: long,
        });
        assert_eq!(r.count as usize, CHARS_CAP);
    }
}
