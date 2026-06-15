//! Pure Vietnamese input transform — one keystroke at a time.
//!
//! `funput-core` answers: given the current syllable buffer and a key, what is the
//! new text according to VNI or Telex?
//!
//! # API FROZEN (Phase 8)
//!
//! The public surface is intentionally minimal for `funput-engine`:
//! [`InputMethod`], [`TransformKind`], [`TransformResult`], and [`apply`].
//! Breaking changes require semver coordination with the engine.
//!
//! # Contract
//!
//! - **Stateless:** no session, no backspace count — the engine diffs `buffer` vs
//!   `result.text`.
//! - **Syllable chunk:** the engine passes one syllable buffer per word; core does not
//!   split words on spaces.
//! - **No I/O:** no platform hooks, config files, or English auto-restore (engine).

mod composition;
mod input_method;
mod unicode;
mod validation;

/// Input method selector — the engine passes this on each [`apply`] call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMethod {
    /// VNI digit modifiers (`1`–`9`).
    Vni,
    /// Telex letter modifiers (`s`/`f`/`r`/`x`/`j`, `aa`/`dd`/`w`, …).
    Telex,
}

/// Result kind for a single keystroke transform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformKind {
    /// Normal key appended; no Vietnamese transform yet (or pass-through modifier on
    /// non-Vietnamese text — e.g. `text` + `1` → `"text1"`).
    Pending,
    /// Tone, shape, stroke, or reposition produced new composed text in `text`.
    Applied,
    /// Double modifier removed one layer (e.g. `a11` → `a`).
    Reverted,
    /// Modifier rejected — `text` unchanged (e.g. `ng` + `1`, stroke on non-`d`).
    Ignored,
}

/// Output of applying one keystroke to the current buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformResult {
    /// How the engine should interpret this step.
    pub kind: TransformKind,
    /// Buffer after this keystroke (compare with the previous buffer for backspace).
    pub text: String,
}

/// Apply one keystroke to the current syllable buffer.
///
/// # Examples
///
/// ```
/// use funput_core::{apply, InputMethod, TransformKind, TransformResult};
///
/// let r = apply("a", '1', InputMethod::Vni);
/// assert_eq!(
///     r,
///     TransformResult {
///         kind: TransformKind::Applied,
///         text: "á".into(),
///     }
/// );
/// ```
pub fn apply(buffer: &str, key: char, method: InputMethod) -> TransformResult {
    match method {
        InputMethod::Vni => composition::transform::apply_vni(buffer, key),
        InputMethod::Telex => composition::transform::apply_telex(buffer, key),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_vni_appends_normal_key() {
        let result = apply("", 'a', InputMethod::Vni);
        assert_eq!(
            result,
            TransformResult {
                kind: TransformKind::Pending,
                text: "a".into(),
            }
        );
    }

    #[test]
    fn apply_telex_tone() {
        let result = apply("a", 's', InputMethod::Telex);
        assert_eq!(
            result,
            TransformResult {
                kind: TransformKind::Applied,
                text: "á".into(),
            }
        );
    }
}
