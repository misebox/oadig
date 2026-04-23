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
pub mod search;
pub mod spec_version;
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
        Command::SpecVersion { file } => spec_version::run(&loader::load(file)?.value),
        Command::Overview { file } => overview::run(&loader::load(file)?.value, show_null),
        Command::Stats { file } => stats::run(&loader::load(file)?.value),
        Command::Paths { file, filters } => {
            let pf = filter::PathFilter::from_strings(filters)?;
            paths::run(&loader::load(file)?.value, &pf)
        }
        Command::Operations {
            file,
            filters,
            include,
            exclude,
        } => {
            let of = filter::OpFilter::from_strings(filters)?;
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
        Command::Search {
            keyword,
            file,
            regex,
            case_sensitive,
        } => search::run(&loader::load(file)?.value, keyword, *regex, *case_sensitive)?,
        Command::Tags { file } => tags::run(&loader::load(file)?.value),
        Command::Components { file } => components::run(&loader::load(file)?.value, show_null),
        Command::Schemas { file } => schemas::run(&loader::load(file)?.value),
        Command::Schema { name, file } => schema::run(&loader::load(file)?.value, name, opts)?,
    })
}
