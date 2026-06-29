//! Flip the word being composed between its Vietnamese form and its raw keystrokes.
//!
//! When `smart_restore` shows the wrong form (e.g. eager-restored `card` while the
//! user wanted `cải`), the flip hotkey swaps the *in-progress* composition in place.
//! Operating on the still-composing word means the platform just re-renders its
//! marked text — no editing of already-committed text, so it works in every app.
//!
//! The choice is **sticky** via [`RestoreOverride`]: once flipped, further keystrokes
//! keep the chosen form and the word boundary won't English-restore it back.

/// Per-word override of the English-restore decision, set by the flip hotkey.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RestoreOverride {
    /// Keep the Vietnamese composition — suppress restore.
    ForceVietnamese,
    /// Keep the raw keystrokes.
    ForceRaw,
}

/// Decide the flipped display form and the override to pin it, or `None` when there
/// is nothing to flip: no live composition, or the Vietnamese form equals the raw
/// keys (`the` → `the`, so there is no VN/raw distinction).
///
/// `buffer` is what's currently shown; it equals `keys` exactly when the raw form is
/// on screen (typed plainly or after an English restore), so that drives the toggle.
pub(crate) fn flip(buffer: &str, keys: &str, vn_form: &str) -> Option<(String, RestoreOverride)> {
    if vn_form.is_empty() || vn_form == keys {
        return None;
    }
    if buffer == keys {
        Some((vn_form.to_string(), RestoreOverride::ForceVietnamese))
    } else {
        Some((keys.to_string(), RestoreOverride::ForceRaw))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flip_raw_display_to_vietnamese() {
        // "card" shown raw (buffer == keys) → show the composed form, pin VN.
        assert_eq!(
            flip("card", "card", "cải"),
            Some(("cải".to_string(), RestoreOverride::ForceVietnamese))
        );
    }

    #[test]
    fn flip_vietnamese_display_to_raw() {
        // "má" shown (buffer != keys) → show the raw keys, pin raw.
        assert_eq!(
            flip("má", "mas", "má"),
            Some(("mas".to_string(), RestoreOverride::ForceRaw))
        );
    }

    #[test]
    fn nothing_to_flip() {
        assert_eq!(flip("", "", ""), None); // no composition
        assert_eq!(flip("the", "the", "the"), None); // VN form == raw keys
    }
}
