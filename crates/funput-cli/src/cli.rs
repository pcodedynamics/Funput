//! Command-line surface (clap derive) for the `funput` umbrella binary.
//!
//! Top level splits into the user-facing `term` (the terminal input wrapper) and
//! `dev` (engine dev/CI tools). Adding a future product is another top-level variant.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

use funput_core::InputMethod;

use crate::sim::Method;

#[derive(Debug, Parser)]
#[command(name = "funput", version, about = "Funput — Vietnamese input, from the terminal")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Type Vietnamese (Telex/VNI) inside terminal apps via a PTY wrapper.
    Term(TermArgs),
    /// Developer & CI tools that drive funput-engine directly.
    Dev(DevArgs),
}

// --- `funput term` ----------------------------------------------------------

/// `funput term`: run a program through the wrapper, or manage shell integration.
///
/// Mirrors the old standalone `funput-term` CLI: a default run action plus an
/// `install` subcommand. `args_conflicts_with_subcommands` keeps `funput term
/// install …` from being parsed as a program to run.
#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct TermArgs {
    #[command(subcommand)]
    pub command: Option<TermCommand>,

    #[command(flatten)]
    pub run: TermRunArgs,
}

/// Arguments for the default action: run a program through the wrapper.
#[derive(Debug, Args)]
pub struct TermRunArgs {
    /// Input method; overrides the config file and `$FUNPUT_METHOD`.
    #[arg(short, long, value_enum)]
    pub method: Option<TermMethod>,

    /// Program to run (defaults to `$SHELL`). Pass after `--`, e.g. `funput term -- claude`.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub program: Vec<String>,
}

#[derive(Debug, Subcommand)]
pub enum TermCommand {
    /// Print (or write) shell integration so Funput Terminal is always on.
    Install {
        /// Target shell (bash/zsh/fish); defaults to `$SHELL`.
        #[arg(long)]
        shell: Option<String>,

        /// Alias to add, `name` or `name=command`; repeatable.
        #[arg(long = "alias", value_name = "NAME[=CMD]")]
        alias: Vec<String>,

        /// Append the snippet to your shell rc file instead of just printing it.
        #[arg(long)]
        write: bool,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TermMethod {
    Telex,
    Vni,
}

impl From<TermMethod> for InputMethod {
    fn from(m: TermMethod) -> Self {
        match m {
            TermMethod::Telex => InputMethod::Telex,
            TermMethod::Vni => InputMethod::Vni,
        }
    }
}

// --- `funput dev` -----------------------------------------------------------

/// `funput dev`: drive funput-engine from the terminal for quick checks, debugging,
/// and CI. Not a real IME — no keyboard hooks, no injecting into other apps.
#[derive(Debug, Args)]
pub struct DevArgs {
    #[command(subcommand)]
    pub command: DevCommand,
}

#[derive(Debug, Subcommand)]
pub enum DevCommand {
    /// Transform an input string and print the resulting app text.
    Run {
        /// Keys to type. A literal string — spaces and punctuation are word boundaries.
        input: String,
        #[command(flatten)]
        opts: CommonOpts,
    },
    /// Interactive REPL: type a line, see the result, repeat (Ctrl-D or `:q` to quit).
    Repl {
        #[command(flatten)]
        opts: CommonOpts,
    },
    /// Round-trip coverage check over a Vietnamese corpus (Telex & VNI).
    Coverage {
        /// Corpus file (one word per line). Defaults to `benchmarks/sample.txt`.
        corpus: Option<PathBuf>,
        /// Emit machine-readable JSON instead of a human report.
        #[arg(long)]
        json: bool,
        /// Print up to N sample mismatches per method.
        #[arg(long, default_value_t = 0)]
        show_mismatches: usize,
        /// Cap the number of syllables evaluated (for a quick run).
        #[arg(long)]
        limit: Option<usize>,
    },
}

#[derive(Debug, Args)]
pub struct CommonOpts {
    /// Input method.
    #[arg(short, long, value_enum, default_value_t = MethodArg::Vni)]
    pub method: MethodArg,
    /// Print per-keystroke detail instead of just the final app text.
    #[arg(long)]
    pub steps: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum MethodArg {
    Telex,
    Vni,
}

impl From<MethodArg> for Method {
    fn from(m: MethodArg) -> Self {
        match m {
            MethodArg::Telex => Method::Telex,
            MethodArg::Vni => Method::Vni,
        }
    }
}
