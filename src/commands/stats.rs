use std::collections::BTreeMap;

use serde_json::{json, Value};

const METHODS: &[&str] = &[
    "get", "put", "post", "delete", "options", "head", "patch", "trace",
];

pub fn run(spec: &Value) -> Value {
    let paths = spec.get("paths").and_then(Value::as_object);
    let path_count = paths.map(|p| p.len()).unwrap_or(0);

    let mut operations = 0usize;
    let mut by_method: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_tag: BTreeMap<String, usize> = BTreeMap::new();

    if let Some(paths) = paths {
        for (_, item) in paths {
            let Some(item_obj) = item.as_object() else { continue };
            for method in METHODS {
                let Some(op) = item_obj.get(*method) else { continue };
                operations += 1;
                *by_method.entry(method.to_uppercase()).or_default() += 1;
                if let Some(tags) = op.get("tags").and_then(Value::as_array) {
                    for tag in tags {
                        if let Some(s) = tag.as_str() {
                            *by_tag.entry(s.to_string()).or_default() += 1;
                        }
                    }
                }
            }
        }
    }

    let schemas = spec
        .pointer("/components/schemas")
        .and_then(Value::as_object)
        .map(|m| m.len())
        .unwrap_or(0);
    let tags_declared = spec
        .get("tags")
        .and_then(Value::as_array)
        .map(|a| a.len())
        .unwrap_or(0);

    json!({
        "paths": path_count,
        "operations": operations,
        "schemas": schemas,
        "tags_declared": tags_declared,
        "by_method": by_method,
        "by_tag": by_tag,
    })
}
