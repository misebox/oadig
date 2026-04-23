use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::commands::operations::METHODS;

// Walk every response map in the spec and return one flat record per
// unique status code: `{status, description}`. First occurrence wins
// when multiple operations give the same status different descriptions;
// the per-operation view is available via `responses`.
pub fn run(spec: &Value) -> Value {
    let mut seen: BTreeMap<String, Value> = BTreeMap::new();

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
                    seen.entry(status.clone())
                        .or_insert_with(|| resp.get("description").cloned().unwrap_or(Value::Null));
                }
            }
        }
    }

    let out: Vec<Value> = seen
        .into_iter()
        .map(|(status, description)| {
            let mut entry = Map::new();
            entry.insert("status".into(), Value::String(status));
            entry.insert("description".into(), description);
            Value::Object(entry)
        })
        .collect();
    Value::Array(out)
}
