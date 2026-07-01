//! `funput-term install` — wire the wrapper into the user's shell so Vietnamese
//! input is "always on".
//!
//! Emits an idempotent, marked block of shell aliases (each `name` runs
//! `funput-term -- cmd`). By default it just prints the block (safe to inspect);
//! `--write` appends it to the detected shell rc file, skipping if the marker is
//! already present. The snippet builder is pure, so it is unit-tested directly.

use std::io::{self, Write};
use std::path::PathBuf;

/// Start/end markers delimiting the block we own in the rc file, so `--write`
/// stays idempotent and the block is easy to find or remove by hand.
const MARKER_START: &str = "# >>> funput term >>>";
const MARKER_END: &str = "# <<< funput term <<<";

/// Supported shells, distinguished only by alias syntax and rc-file location.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
}

impl Shell {
    /// Match a shell by name or `$SHELL` path basename (`/bin/zsh` → `Zsh`).
    pub fn from_name(name: &str) -> Option<Self> {
        match name.rsplit('/').next().unwrap_or(name) {
            "bash" => Some(Shell::Bash),
            "zsh" => Some(Shell::Zsh),
            "fish" => Some(Shell::Fish),
            _ => None,
        }
    }

    /// Detect from `$SHELL`, defaulting to bash when unknown.
    pub fn detect() -> Self {
        std::env::var("SHELL")
            .ok()
            .and_then(|s| Shell::from_name(&s))
            .unwrap_or(Shell::Bash)
    }

    fn alias_line(self, name: &str, command: &str) -> String {
        match self {
            // POSIX-style quoting for bash/zsh; fish uses a space, not `=`.
            Shell::Bash | Shell::Zsh => format!("alias {name}='funput term -- {command}'"),
            Shell::Fish => format!("alias {name} 'funput term -- {command}'"),
        }
    }
}

/// Parse an `name=command` argument; a bare `name` aliases the command of the same
/// name (`claude` → `claude`→`funput term -- claude`).
pub fn parse_alias(arg: &str) -> (String, String) {
    match arg.split_once('=') {
        Some((name, command)) => (name.to_string(), command.to_string()),
        None => (arg.to_string(), arg.to_string()),
    }
}

/// Build the marked, idempotent shell block for `aliases`. With no aliases it
/// emits a commented example so the user can see the shape.
pub fn snippet(shell: Shell, aliases: &[(String, String)]) -> String {
    let mut out = String::new();
    out.push_str(MARKER_START);
    out.push('\n');
    out.push_str("# Funput Terminal — type Vietnamese inside terminal apps.\n");
    if aliases.is_empty() {
        out.push_str("# Example: add `--alias claude` to wrap a command, e.g.\n");
        out.push_str(&format!("#   {}\n", shell.alias_line("claude", "claude")));
        out.push_str("# Or wrap your whole shell from your terminal emulator:\n");
        out.push_str("#   funput term -- $SHELL\n");
    } else {
        for (name, command) in aliases {
            out.push_str(&shell.alias_line(name, command));
            out.push('\n');
        }
    }
    out.push_str(MARKER_END);
    out.push('\n');
    out
}

/// The rc file a `--write` should append to for `shell`.
pub fn rc_path(shell: Shell) -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    Some(match shell {
        Shell::Bash => home.join(".bashrc"),
        Shell::Zsh => home.join(".zshrc"),
        Shell::Fish => home.join(".config").join("fish").join("config.fish"),
    })
}

/// Run the `install` subcommand: print the snippet, and when `write` is set append
/// it to the shell rc file unless our marker is already there.
pub fn run(shell: Shell, aliases: &[(String, String)], write: bool) -> io::Result<()> {
    let block = snippet(shell, aliases);

    if !write {
        print!("{block}");
        return Ok(());
    }

    let path = rc_path(shell)
        .ok_or_else(|| io::Error::other("cannot determine home directory for rc file"))?;
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    if existing.contains(MARKER_START) {
        println!("funput term: already installed in {}", path.display());
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    // Separate from any preceding content.
    writeln!(file, "\n{block}")?;
    println!("funput term: installed in {}", path.display());
    println!("Restart your shell or run `source {}`.", path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn aliases() -> Vec<(String, String)> {
        vec![("claude".to_string(), "claude".to_string())]
    }

    #[test]
    fn bash_zsh_use_equals_quoting() {
        let s = snippet(Shell::Zsh, &aliases());
        assert!(s.contains("alias claude='funput term -- claude'"));
        assert!(s.contains(MARKER_START) && s.contains(MARKER_END));
    }

    #[test]
    fn fish_uses_space_syntax() {
        let s = snippet(Shell::Fish, &aliases());
        assert!(s.contains("alias claude 'funput term -- claude'"));
    }

    #[test]
    fn empty_aliases_emit_commented_example() {
        let s = snippet(Shell::Bash, &[]);
        assert!(s.contains(MARKER_START) && s.contains(MARKER_END));
        // The example is commented out, so sourcing the block is a no-op.
        assert!(!s.lines().any(|l| l.starts_with("alias ")));
    }

    #[test]
    fn shell_detection_from_path() {
        assert_eq!(Shell::from_name("/bin/zsh"), Some(Shell::Zsh));
        assert_eq!(Shell::from_name("fish"), Some(Shell::Fish));
        assert_eq!(Shell::from_name("/usr/bin/tcsh"), None);
    }

    #[test]
    fn alias_arg_parsing() {
        assert_eq!(parse_alias("claude"), ("claude".into(), "claude".into()));
        assert_eq!(parse_alias("cc=claude"), ("cc".into(), "claude".into()));
    }
}
