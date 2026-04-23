use serde_json::{Map, Value};

use crate::commands::operations::{METHODS, resolve_in_place};
use crate::resolver::ResolveOptions;

pub fn run(spec: &Value, opts: ResolveOptions) -> Value {
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
            let Some(raw) = op.get("requestBody").filter(|v| !v.is_null()) else {
                continue;
            };
            let origin = format!("#request/{}/{}", method, path);
            let body = resolve_in_place(raw.clone(), spec, opts, &origin);
            let mut entry = Map::new();
            entry.insert("method".into(), Value::String(method.to_uppercase()));
            entry.insert("path".into(), Value::String(path.clone()));
            entry.insert("request".into(), body);
            out.push(Value::Object(entry));
        }
    }
    Value::Array(out)
}
