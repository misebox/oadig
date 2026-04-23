use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::commands::operations::METHODS;

pub fn run(spec: &Value) -> Value {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    if let Some(paths) = spec.get("paths").and_then(Value::as_object) {
        for (_, item) in paths {
            let Some(item_obj) = item.as_object() else {
                continue;
            };
            for method in METHODS {
                let Some(op) = item_obj.get(*method) else {
                    continue;
                };
                if let Some(tags) = op.get("tags").and_then(Value::as_array) {
                    for tag in tags {
                        if let Some(name) = tag.as_str() {
                            *counts.entry(name.to_string()).or_default() += 1;
                        }
                    }
                }
            }
        }
    }

    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut out = Vec::new();

    // Declared tags first, in spec order.
    if let Some(declared) = spec.get("tags").and_then(Value::as_array) {
        for tag in declared {
            let Some(obj) = tag.as_object() else { continue };
            let Some(name) = obj.get("name").and_then(Value::as_str) else {
                continue;
            };
            let mut entry = Map::new();
            entry.insert("name".into(), Value::String(name.to_string()));
            if let Some(desc) = obj.get("description").filter(|v| !v.is_null()) {
                entry.insert("description".into(), desc.clone());
            }
            let count = counts.remove(name).unwrap_or(0);
            entry.insert("operationCount".into(), Value::from(count));
            seen.insert(name.to_string());
            out.push(Value::Object(entry));
        }
    }

    // Tags referenced by operations but not declared, sorted by name.
    for (name, count) in counts {
        if seen.contains(&name) {
            continue;
        }
        let mut entry = Map::new();
        entry.insert("name".into(), Value::String(name));
        entry.insert("operationCount".into(), Value::from(count));
        out.push(Value::Object(entry));
    }

    Value::Array(out)
}
