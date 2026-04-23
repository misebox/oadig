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
pub mod statuses;
pub mod tags;

use serde_json::Value;

use crate::cli::Command;
use crate::error::OadigError;
use crate::loader;
use crate::resolver::ResolveOptions;

// `operation`/`request`/`response` accept either `<ID> <FILE>` or `<FILE>`
// plus `-m/-p`. Split the positional list into (id, file) so the caller
// doesn't re-derive the rules.
fn split_positionals<'a>(
    args: &'a [String],
    method: &Option<String>,
    path: &Option<String>,
) -> Result<(Option<&'a str>, &'a str), OadigError> {
    match args.len() {
        2 => Ok((Some(args[0].as_str()), args[1].as_str())),
        1 => {
            if method.is_some() && path.is_some() {
                Ok((None, args[0].as_str()))
            } else {
                Err(OadigError::Other(
                    "specify either `<ID> <FILE>` or `<FILE> -m METHOD -p PATH`".to_string(),
                ))
            }
        }
        _ => Err(OadigError::Other(
            "too many positional arguments".to_string(),
        )),
    }
}

pub fn dispatch(
    command: &Command,
    opts: ResolveOptions,
    show_null: bool,
) -> Result<Value, OadigError> {
    Ok(match command {
        Command::Info { file } => info::run(&loader::load(file)?.value, show_null),
        Command::Spec { file } => spec_version::run(&loader::load(file)?.value),
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
        Command::Operation { args, method, path } => {
            let (id, file) = split_positionals(args, method, path)?;
            operation::run(
                &loader::load(file)?.value,
                id,
                method.as_deref(),
                path.as_deref(),
                opts,
            )?
        }
        Command::Statuses {
            file,
            include,
            exclude,
        } => statuses::run(&loader::load(file)?.value, include, exclude, opts),
        Command::Requests { file } => requests::run(&loader::load(file)?.value, opts),
        Command::Responses { file, status } => {
            responses::run(&loader::load(file)?.value, status.as_deref(), opts)
        }
        Command::Request { args, method, path } => {
            let (id, file) = split_positionals(args, method, path)?;
            request::run(
                &loader::load(file)?.value,
                id,
                method.as_deref(),
                path.as_deref(),
                opts,
            )?
        }
        Command::Response {
            args,
            method,
            path,
            status,
        } => {
            let (id, file) = split_positionals(args, method, path)?;
            response::run(
                &loader::load(file)?.value,
                id,
                method.as_deref(),
                path.as_deref(),
                status.as_deref(),
                opts,
            )?
        }
        Command::Search {
            keyword,
            file,
            regex,
            case_sensitive,
            include,
            exclude,
        } => search::run(
            &loader::load(file)?.value,
            keyword,
            *regex,
            *case_sensitive,
            include,
            exclude,
        )?,
        Command::Tags { file } => tags::run(&loader::load(file)?.value),
        Command::Components { file } => components::run(&loader::load(file)?.value, show_null),
        Command::Schemas { file } => schemas::run(&loader::load(file)?.value),
        Command::Schema { name, file } => schema::run(&loader::load(file)?.value, name, opts)?,
    })
}
