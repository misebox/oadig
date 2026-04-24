pub mod components;
pub mod convert;
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
pub mod validate;

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

// Load the spec file as-is. Used by commands that must see the original
// structure (spec, search, validate, convert).
fn load_raw(file: &str) -> Result<Value, OadigError> {
    Ok(loader::load(file)?.value)
}

// Load the spec file and, when it is Swagger 2.0, transparently convert
// it to OpenAPI 3.0 so the rest of the command set can use a single
// code path. OpenAPI 3.x specs pass through unchanged.
fn load_canonical(file: &str) -> Result<Value, OadigError> {
    Ok(load_raw_and_canonical(file)?.1)
}

// Load the raw spec and the canonical (OpenAPI 3.x) form. Used by
// `overview`, which reports the original version alongside data derived
// from the converted spec.
fn load_raw_and_canonical(file: &str) -> Result<(Value, Value), OadigError> {
    let raw = load_raw(file)?;
    let canonical = if raw.get("swagger").and_then(Value::as_str) == Some("2.0") {
        convert::run(&raw, "3.0")?
    } else {
        raw.clone()
    };
    Ok((raw, canonical))
}

pub fn dispatch(
    command: &Command,
    opts: ResolveOptions,
    show_null: bool,
) -> Result<Value, OadigError> {
    Ok(match command {
        // Raw-only: the command's whole purpose depends on the source
        // shape (version report, full-tree search, structural validation,
        // the converter itself).
        Command::Spec { file } => spec_version::run(&load_raw(file)?),
        Command::Validate { file } => validate::run(&load_raw(file)?)?,
        Command::Convert { target, file } => convert::run(&load_raw(file)?, target)?,
        Command::Search {
            keyword,
            file,
            regex,
            case_sensitive,
            include,
            exclude,
        } => search::run(
            &load_raw(file)?,
            keyword,
            *regex,
            *case_sensitive,
            include,
            exclude,
        )?,

        // Overview reports the original version alongside stats/operations,
        // so it needs both.
        Command::Overview { file } => {
            let (raw, canonical) = load_raw_and_canonical(file)?;
            overview::run(&raw, &canonical, show_null)
        }

        // Everything else runs against the canonical (OpenAPI 3.x) form.
        Command::Info { file } => info::run(&load_canonical(file)?, show_null),
        Command::Stats { file } => stats::run(&load_canonical(file)?),
        Command::Paths { file, filters } => {
            let pf = filter::PathFilter::from_strings(filters)?;
            paths::run(&load_canonical(file)?, &pf)
        }
        Command::Operations {
            file,
            filters,
            include,
            exclude,
        } => {
            let of = filter::OpFilter::from_strings(filters)?;
            operations::run(&load_canonical(file)?, include, exclude, &of, opts)
        }
        Command::Operation { args, method, path } => {
            let (id, file) = split_positionals(args, method, path)?;
            operation::run(
                &load_canonical(file)?,
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
        } => statuses::run(&load_canonical(file)?, include, exclude, opts),
        Command::Requests { file } => requests::run(&load_canonical(file)?, opts),
        Command::Responses { file, status } => {
            responses::run(&load_canonical(file)?, status.as_deref(), opts)
        }
        Command::Request { args, method, path } => {
            let (id, file) = split_positionals(args, method, path)?;
            request::run(
                &load_canonical(file)?,
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
                &load_canonical(file)?,
                id,
                method.as_deref(),
                path.as_deref(),
                status.as_deref(),
                opts,
            )?
        }
        Command::Tags { file } => tags::run(&load_canonical(file)?),
        Command::Components { file } => components::run(&load_canonical(file)?, show_null),
        Command::Schemas { file } => schemas::run(&load_canonical(file)?),
        Command::Schema { name, file } => schema::run(&load_canonical(file)?, name, opts)?,
    })
}
