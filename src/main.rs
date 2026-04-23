mod cli;
mod commands;
mod error;
mod loader;
mod output;
mod resolver;

use anyhow::Result;
use clap::Parser;

use crate::cli::Cli;
use crate::output::Display;
use crate::resolver::ResolveOptions;

fn main() -> Result<()> {
    let args = Cli::parse();
    let opts = ResolveOptions {
        resolve: args.should_resolve_refs(),
        max_depth: args.max_depth,
    };
    let display = if args.compact {
        Display::Compact
    } else if args.lines {
        Display::Lines
    } else {
        Display::Pretty
    };
    let value = commands::dispatch(&args.command, opts)?;
    println!("{}", output::render(&value, args.format, display)?);
    Ok(())
}
