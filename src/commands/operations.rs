use std::collections::HashSet;

use serde_json::{Map, Value};

use crate::cli::OperationField;
use crate::resolver::{ResolveOptions, Resolver};

const METHODS: &[&str] = &[
    "get", "put", "post", "delete", "options", "head", "patch", "trace",
];

// Every concrete field (i.e. excluding the `All` meta-variant).
const ALL_FIELDS: &[OperationField] = &[
    OperationField::Summary,
    OperationField::Description,
    OperationField::Tags,
    OperationField::Parameters,
    OperationField::Request,
    OperationField::Response,
    OperationField::Security,
    OperationField::Deprecated,
    OperationField::OperationId,
];

pub fn run(
    spec: &Value,
    include: &[OperationField],
    exclude: &[OperationField],
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
            out.push(build_entry(method, path, op, &fields, spec, resolve_opts));
        }
    }
    Value::Array(out)
}

fn resolve_fields(
    include: &[OperationField],
    exclude: &[OperationField],
) -> HashSet<OperationField> {
    let mut fields: HashSet<OperationField> = [OperationField::Summary].into();
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
            resolve_refs(
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

fn resolve_refs(value: Value, spec: &Value, opts: ResolveOptions, origin: &str) -> Value {
    if !opts.resolve {
        return value;
    }
    Resolver::new(spec, opts).resolve(value, origin)
}
