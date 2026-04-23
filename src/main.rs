mod cli;
mod commands;
mod error;
mod loader;
mod output;
mod resolver;

use anyhow::Result;
use clap::Parser;

use crate::cli::{Cli, Command};
use crate::resolver::ResolveOptions;

fn main() -> Result<()> {
    let args = Cli::parse();
    let opts = ResolveOptions {
        resolve: args.should_resolve_refs(),
        max_depth: args.max_depth,
    };

    let output = match &args.command {
        Command::Info { file } => {
            let loaded = loader::load(file)?;
            commands::info::run(&loaded.value)
        }
        Command::Overview { file } => {
            let loaded = loader::load(file)?;
            commands::overview::run(&loaded.value)
        }
        Command::Stats { file } => {
            let loaded = loader::load(file)?;
            commands::stats::run(&loaded.value)
        }
        Command::Paths { file } => {
            let loaded = loader::load(file)?;
            commands::paths::run(&loaded.value)
        }
        Command::Schemas { file } => {
            let loaded = loader::load(file)?;
            commands::schemas::run(&loaded.value)
        }
        Command::Schema { name, file } => {
            let loaded = loader::load(file)?;
            commands::schema::run(&loaded.value, name, opts)?
        }
    };

    let text = output::render(&output, args.format, args.compact)?;
    println!("{}", text);
    Ok(())
}
