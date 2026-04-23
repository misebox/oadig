use serde_json::{Map, Value};

use crate::commands::operations::{METHODS, resolve_in_place};
use crate::resolver::ResolveOptions;

pub fn run(spec: &Value, status: Option<&str>, opts: ResolveOptions) -> Value {
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
            let Some(raw) = op.get("responses").filter(|v| !v.is_null()) else {
                continue;
            };
            let selected = match status {
                Some(code) => match raw.get(code) {
                    Some(v) => {
                        let mut map = Map::new();
                        map.insert(code.to_string(), v.clone());
                        Value::Object(map)
                    }
                    None => continue, // skip ops that don't have this status
                },
                None => raw.clone(),
            };
            let origin = format!("#responses/{}/{}", method, path);
            let resolved = resolve_in_place(selected, spec, opts, &origin);
            let mut entry = Map::new();
            entry.insert("method".into(), Value::String(method.to_uppercase()));
            entry.insert("path".into(), Value::String(path.clone()));
            entry.insert("responses".into(), resolved);
            out.push(Value::Object(entry));
        }
    }
    Value::Array(out)
}
