use std::collections::HashSet;

use serde_json::{Map, Value};

use crate::cli::OperationField;
use crate::commands::filter::OpFilter;
use crate::error::OadigError;
use crate::resolver::{ResolveOptions, Resolver};

pub const METHODS: &[&str] = &[
    "get", "put", "post", "delete", "options", "head", "patch", "trace",
];

pub struct Located<'a> {
    pub method: String,
    pub path: String,
    pub op: &'a Value,
}

/// Look up the operation object for a given method and path.
pub fn find<'a>(spec: &'a Value, method: &str, path: &str) -> Result<&'a Value, OadigError> {
    let lower = method.to_ascii_lowercase();
    spec.get("paths")
        .and_then(|p| p.get(path))
        .and_then(|item| item.get(&lower))
        .ok_or_else(|| OadigError::OperationNotFound {
            method: method.to_uppercase(),
            path: path.to_string(),
        })
}

/// Look up the operation object by operationId. Must match exactly one op.
pub fn find_by_id<'a>(spec: &'a Value, id: &str) -> Result<Located<'a>, OadigError> {
    let mut matches: Vec<Located<'a>> = Vec::new();
    if let Some(paths) = spec.get("paths").and_then(Value::as_object) {
        for (path, item) in paths {
            let Some(item_obj) = item.as_object() else {
                continue;
            };
            for method in METHODS {
                let Some(op) = item_obj.get(*method) else {
                    continue;
                };
                if op.get("operationId").and_then(Value::as_str) == Some(id) {
                    matches.push(Located {
                        method: (*method).to_string(),
                        path: path.clone(),
                        op,
                    });
                }
            }
        }
    }
    match matches.len() {
        0 => Err(OadigError::OperationIdNotFound { id: id.to_string() }),
        1 => Ok(matches.into_iter().next().unwrap()),
        _ => {
            let list = matches
                .iter()
                .map(|m| format!("  - {:<6} {}", m.method.to_uppercase(), m.path))
                .collect::<Vec<_>>()
                .join("\n");
            Err(OadigError::OperationIdAmbiguous {
                id: id.to_string(),
                matches: list,
            })
        }
    }
}

/// Resolve an operation lookup that may use either `id` or `(method, path)`.
pub fn resolve_lookup<'a>(
    spec: &'a Value,
    id: Option<&str>,
    method: Option<&str>,
    path: Option<&str>,
) -> Result<Located<'a>, OadigError> {
    match (id, method, path) {
        (Some(id), _, _) => find_by_id(spec, id),
        (None, Some(m), Some(p)) => {
            let op = find(spec, m, p)?;
            Ok(Located {
                method: m.to_ascii_lowercase(),
                path: p.to_string(),
                op,
            })
        }
        _ => Err(OadigError::Other(
            "specify either <id> or both -m/--method and -p/--path".to_string(),
        )),
    }
}

pub fn resolve_in_place(value: Value, spec: &Value, opts: ResolveOptions, origin: &str) -> Value {
    if !opts.resolve {
        return value;
    }
    Resolver::new(spec, opts).resolve(value, origin)
}

// Every concrete field (i.e. excluding the `All` meta-variant), in the order
// they should appear in output. Identifiers first, long prose later.
const ALL_FIELDS: &[OperationField] = &[
    OperationField::OperationId,
    OperationField::Summary,
    OperationField::Description,
    OperationField::Tags,
    OperationField::Parameters,
    OperationField::Request,
    OperationField::Response,
    OperationField::Security,
    OperationField::Deprecated,
];

pub fn run(
    spec: &Value,
    include: &[OperationField],
    exclude: &[OperationField],
    filter: &OpFilter,
    resolve_opts: ResolveOptions,
) -> Value {
    let fields = resolve_fields(include, exclude);

    let Some(paths) = spec.get("paths").and_then(Value::as_object) else {
        return Value::Array(vec![]);
    };

    let mut out = Vec::new();
    for (path, item) in paths {
        let Some(item_obj) = item.as_object() else {
            continue;
        };
        for method in METHODS {
            let Some(op) = item_obj.get(*method) else {
                continue;
            };
            if !filter.accepts(method, path, op) {
                continue;
            }
            out.push(build_entry(method, path, op, &fields, spec, resolve_opts));
        }
    }
    Value::Array(out)
}

fn resolve_fields(
    include: &[OperationField],
    exclude: &[OperationField],
) -> HashSet<OperationField> {
    let mut fields: HashSet<OperationField> =
        [OperationField::OperationId, OperationField::Summary].into();
    for f in include {
        if *f == OperationField::All {
            fields.extend(ALL_FIELDS.iter().copied());
        } else {
            fields.insert(*f);
        }
    }
    for f in exclude {
        if *f == OperationField::All {
            fields.clear();
        } else {
            fields.remove(f);
        }
    }
    fields
}

fn build_entry(
    method: &str,
    path: &str,
    op: &Value,
    fields: &HashSet<OperationField>,
    spec: &Value,
    resolve_opts: ResolveOptions,
) -> Value {
    let mut entry = Map::new();
    entry.insert("method".into(), Value::String(method.to_uppercase()));
    entry.insert("path".into(), Value::String(path.to_string()));

    for field in ALL_FIELDS {
        if !fields.contains(field) {
            continue;
        }
        if let Some((key, value)) = project_field(*field, op, spec, resolve_opts) {
            entry.insert(key.into(), value);
        }
    }

    Value::Object(entry)
}

fn project_field(
    field: OperationField,
    op: &Value,
    spec: &Value,
    resolve_opts: ResolveOptions,
) -> Option<(&'static str, Value)> {
    let (source_key, output_key) = match field {
        OperationField::Summary => ("summary", "summary"),
        OperationField::Description => ("description", "description"),
        OperationField::Tags => ("tags", "tags"),
        OperationField::Parameters => ("parameters", "parameters"),
        OperationField::Request => ("requestBody", "request"),
        OperationField::Response => ("responses", "response"),
        OperationField::Security => ("security", "security"),
        OperationField::Deprecated => ("deprecated", "deprecated"),
        OperationField::OperationId => ("operationId", "operationId"),
        OperationField::All => return None,
    };
    let raw = op.get(source_key)?;
    if raw.is_null() {
        return None;
    }
    let value = match field {
        OperationField::Parameters | OperationField::Request | OperationField::Response => {
            resolve_in_place(
                raw.clone(),
                spec,
                resolve_opts,
                &format!("#op/{}", source_key),
            )
        }
        _ => raw.clone(),
    };
    Some((output_key, value))
}
