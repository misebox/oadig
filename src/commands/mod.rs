pub mod info;
pub mod operations;
pub mod overview;
pub mod paths;
pub mod schema;
pub mod schemas;
pub mod stats;

use serde_json::Value;

use crate::cli::Command;
use crate::error::OadigError;
use crate::loader;
use crate::resolver::ResolveOptions;

pub fn dispatch(command: &Command, opts: ResolveOptions) -> Result<Value, OadigError> {
    Ok(match command {
        Command::Info { file } => info::run(&loader::load(file)?.value),
        Command::Overview { file } => overview::run(&loader::load(file)?.value),
        Command::Stats { file } => stats::run(&loader::load(file)?.value),
        Command::Paths { file } => paths::run(&loader::load(file)?.value),
        Command::Operations {
            file,
            include,
            exclude,
        } => operations::run(&loader::load(file)?.value, include, exclude, opts),
        Command::Schemas { file } => schemas::run(&loader::load(file)?.value),
        Command::Schema { name, file } => schema::run(&loader::load(file)?.value, name, opts)?,
    })
}
