//! Buffer diff → backspace count + output suffix for platform inject.

/// Compare composed text before and after a transform.
///
/// Returns `(backspace, output)` where the platform deletes `backspace` chars
/// then injects `output`.
pub(crate) fn diff(old: &str, new: &str) -> (usize, String) {
    let old_chars: Vec<char> = old.chars().collect();
    let new_chars: Vec<char> = new.chars().collect();

    let prefix_len = old_chars
        .iter()
        .zip(&new_chars)
        .take_while(|(a, b)| a == b)
        .count();

    let backspace = old_chars.len() - prefix_len;
    let output: String = new_chars.into_iter().skip(prefix_len).collect();
    (backspace, output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diff_single_vowel_tone() {
        assert_eq!(diff("a", "á"), (1, "á".to_string()));
    }

    #[test]
    fn diff_reposition_suffix() {
        assert_eq!(diff("hoa", "hoà"), (1, "à".to_string()));
    }

    #[test]
    fn diff_revert_shape() {
        assert_eq!(diff("â", "a"), (1, "a".to_string()));
    }

    #[test]
    fn diff_unchanged() {
        assert_eq!(diff("x", "x"), (0, String::new()));
    }
}
