//! `funput` — the umbrella binary for Funput's command-line surface.
//!
//! Two families of subcommands:
//! - `funput term …` — type Vietnamese inside terminal apps via a PTY wrapper
//!   (the reusable [`funput_term`] library; not an IME, no permissions).
//! - `funput dev …` — feed a string through `funput-engine` and print what a
//!   platform shell would show, for quick checks, debugging, and CI.

mod cli;
mod coverage;
mod encode;
mod render;
mod repl;
mod sim;

use std::path::PathBuf;

use clap::Parser;

use cli::{Cli, Command, DevCommand, TermArgs, TermCommand};
use render::steps_table;

fn main() {
    match Cli::parse().command {
        Command::Term(args) => run_term(args),
        Command::Dev(dev) => run_dev(dev.command),
    }
}

/// `funput term`: run a program through the wrapper, or manage shell integration.
fn run_term(args: TermArgs) {
    match args.command {
        Some(TermCommand::Install {
            shell,
            alias,
            write,
        }) => {
            let shell = shell
                .as_deref()
                .and_then(funput_term::install::Shell::from_name)
                .unwrap_or_else(funput_term::install::Shell::detect);
            let aliases: Vec<_> = alias
                .iter()
                .map(|a| funput_term::install::parse_alias(a))
                .collect();
            if let Err(err) = funput_term::install::run(shell, &aliases, write) {
                eprintln!("funput term: {err}");
                std::process::exit(1);
            }
        }
        None => run_wrapper(args.run),
    }
}

/// Default `funput term` action: run `program` (or `$SHELL`) through the wrapper.
/// Precedence for the input method: CLI flag > env var > settings.json > default.
fn run_wrapper(run: cli::TermRunArgs) {
    let mut config = funput_term::config::load();
    if let Some(method) = run.method {
        config.method = method.into();
    }

    let command = if run.program.is_empty() {
        vec![std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())]
    } else {
        run.program
    };

    match funput_term::app::run(funput_term::app::Options { config, command }) {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("funput term: {err}");
            std::process::exit(1);
        }
    }
}

/// `funput dev`: drive funput-engine directly (run / repl / coverage).
fn run_dev(command: DevCommand) {
    match command {
        DevCommand::Run { input, opts } => {
            let simulation = sim::simulate(opts.method.into(), &input);
            if opts.steps {
                println!("{}", steps_table(&simulation));
            } else {
                println!("{}", simulation.app_text);
            }
        }
        DevCommand::Repl { opts } => repl::run(opts.method.into(), opts.steps),
        DevCommand::Coverage {
            corpus,
            json,
            show_mismatches,
            limit,
        } => {
            let path = corpus.unwrap_or_else(|| PathBuf::from("benchmarks/sample.txt"));
            if let Err(e) = coverage::run(&path, json, show_mismatches, limit) {
                eprintln!("coverage: cannot read corpus {}: {e}", path.display());
                std::process::exit(1);
            }
        }
    }
}
