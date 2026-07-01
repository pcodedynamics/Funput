//! `funput-term` — type Vietnamese inside terminal apps via a transparent PTY
//! wrapper.
//!
//! This crate is the reusable library behind the `funput term` subcommand: run a
//! program through it (`app::run`) and ASCII keystrokes are composed into
//! Vietnamese before reaching the child; everything else is forwarded untouched.
//! Toggle with `Ctrl-\`. Not an IME — no system hooks, no permissions; works in any
//! terminal emulator.
//!
//! Settings come from the shared `Funput/settings.json`, overridable by env vars
//! and CLI flags (see [`config`]). [`install`] wires it into the user's shell.
//!
//! The umbrella `funput` binary ([crates/funput-cli]) depends on this crate and
//! dispatches to [`app`], [`config`], and [`install`]; the PTY-heavy plumbing
//! (`inject`, `input`, `output`, `state`, `term`) stays crate-internal.

pub mod app;
pub mod config;
pub mod install;

mod inject;
mod input;
mod output;
mod state;
mod term;
