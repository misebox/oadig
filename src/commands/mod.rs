pub mod components;
pub mod filter;
pub mod info;
pub mod operation;
pub mod operations;
pub mod overview;
pub mod paths;
pub mod request;
pub mod requests;
pub mod response;
pub mod responses;
pub mod schema;
pub mod schemas;
pub mod stats;
pub mod tags;

use serde_json::Value;

use crate::cli::Command;
use crate::error::OadigError;
use crate::loader;
use crate::resolver::ResolveOptions;

pub fn dispatch(
    command: &Command,
    opts: ResolveOptions,
    show_null: bool,
) -> Result<Value, OadigError> {
    Ok(match command {
        Command::Info { file } => info::run(&loader::load(file)?.value, show_null),
        Command::Overview { file } => overview::run(&loader::load(file)?.value, show_null),
        Command::Stats { file } => stats::run(&loader::load(file)?.value),
        Command::Paths { file, path_filter } => {
            let pf = filter::PathFilter::new(path_filter.as_deref())?;
            paths::run(&loader::load(file)?.value, &pf)
        }
        Command::Operations {
            file,
            include,
            exclude,
            method,
            path_filter,
            tag,
        } => {
            let of = filter::OpFilter::new(method, path_filter.as_deref(), tag.as_deref())?;
            operations::run(&loader::load(file)?.value, include, exclude, &of, opts)
        }
        Command::Operation {
            file,
            id,
            method,
            path,
        } => operation::run(
            &loader::load(file)?.value,
            id.as_deref(),
            method.as_deref(),
            path.as_deref(),
            opts,
        )?,
        Command::Requests { file } => requests::run(&loader::load(file)?.value, opts),
        Command::Responses { file, status } => {
            responses::run(&loader::load(file)?.value, status.as_deref(), opts)
        }
        Command::Request {
            file,
            id,
            method,
            path,
        } => request::run(
            &loader::load(file)?.value,
            id.as_deref(),
            method.as_deref(),
            path.as_deref(),
            opts,
        )?,
        Command::Response {
            file,
            id,
            method,
            path,
            status,
        } => response::run(
            &loader::load(file)?.value,
            id.as_deref(),
            method.as_deref(),
            path.as_deref(),
            status.as_deref(),
            opts,
        )?,
        Command::Tags { file } => tags::run(&loader::load(file)?.value),
        Command::Components { file } => components::run(&loader::load(file)?.value, show_null),
        Command::Schemas { file } => schemas::run(&loader::load(file)?.value),
        Command::Schema { name, file } => schema::run(&loader::load(file)?.value, name, opts)?,
    })
}
