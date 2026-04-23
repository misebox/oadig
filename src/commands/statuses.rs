use std::collections::{BTreeMap, HashSet};

use serde_json::{Map, Value};

use crate::cli::StatusField;
use crate::commands::operations::{METHODS, resolve_in_place};
use crate::resolver::ResolveOptions;

const ALL_FIELDS: &[StatusField] = &[StatusField::Headers, StatusField::Schema];

struct Entry {
    description: Value,
    response: Value, // full response object for later field extraction
}

// Walk every response map in the spec and return one flat record per
// unique status code: `{status, description, ...}`. First occurrence
// wins when multiple operations give the same status different
// descriptions; the per-operation view is available via `responses`.
pub fn run(spec: &Value, include: &[StatusField], opts: ResolveOptions) -> Value {
    let fields = resolve_fields(include);

    let mut seen: BTreeMap<String, Entry> = BTreeMap::new();
    if let Some(paths) = spec.get("paths").and_then(Value::as_object) {
        for (_, item) in paths {
            let Some(item_obj) = item.as_object() else {
                continue;
            };
            for method in METHODS {
                let Some(op) = item_obj.get(*method) else {
                    continue;
                };
                let Some(responses) = op.get("responses").and_then(Value::as_object) else {
                    continue;
                };
                for (status, resp) in responses {
                    seen.entry(status.clone()).or_insert_with(|| Entry {
                        description: resp.get("description").cloned().unwrap_or(Value::Null),
                        response: resp.clone(),
                    });
                }
            }
        }
    }

    let out: Vec<Value> = seen
        .into_iter()
        .map(|(status, entry)| build_record(status, entry, &fields, spec, opts))
        .collect();
    Value::Array(out)
}

fn resolve_fields(include: &[StatusField]) -> HashSet<StatusField> {
    let mut fields = HashSet::new();
    for f in include {
        if *f == StatusField::All {
            fields.extend(ALL_FIELDS.iter().copied());
        } else {
            fields.insert(*f);
        }
    }
    fields
}

fn build_record(
    status: String,
    entry: Entry,
    fields: &HashSet<StatusField>,
    spec: &Value,
    opts: ResolveOptions,
) -> Value {
    let mut out = Map::new();
    out.insert("status".into(), Value::String(status.clone()));
    out.insert("description".into(), entry.description);

    for field in ALL_FIELDS {
        if !fields.contains(field) {
            continue;
        }
        match field {
            StatusField::Headers => {
                if let Some(headers) = entry.response.get("headers").filter(|v| !v.is_null()) {
                    let origin = format!("#status/{}/headers", status);
                    out.insert(
                        "headers".into(),
                        resolve_in_place(headers.clone(), spec, opts, &origin),
                    );
                }
            }
            StatusField::Schema => {
                let schemas = extract_schemas(&entry.response, spec, opts, &status);
                if !schemas.is_empty() {
                    out.insert("schema".into(), Value::Object(schemas));
                }
            }
            StatusField::All => {}
        }
    }
    Value::Object(out)
}

// Build a `{<mediaType>: <schema>}` map from a response. Prefers
// OpenAPI 3.x `response.content.<mediaType>.schema`; falls back to
// Swagger 2.0's bare `response.schema`, keyed as `*/*` since that form
// carries no media type.
fn extract_schemas(
    response: &Value,
    spec: &Value,
    opts: ResolveOptions,
    status: &str,
) -> Map<String, Value> {
    if let Some(content) = response.get("content").and_then(Value::as_object) {
        let mut out = Map::new();
        for (media_type, media) in content {
            let Some(schema) = media.get("schema").filter(|v| !v.is_null()) else {
                continue;
            };
            let origin = format!("#status/{}/schema/{}", status, media_type);
            out.insert(
                media_type.clone(),
                resolve_in_place(schema.clone(), spec, opts, &origin),
            );
        }
        return out;
    }
    if let Some(schema) = response.get("schema").filter(|v| !v.is_null()) {
        let origin = format!("#status/{}/schema", status);
        let mut out = Map::new();
        out.insert(
            "*/*".into(),
            resolve_in_place(schema.clone(), spec, opts, &origin),
        );
        return out;
    }
    Map::new()
}
