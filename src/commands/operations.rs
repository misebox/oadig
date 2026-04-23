use serde_json::{Map, Value};

const METHODS: &[&str] = &[
    "get", "put", "post", "delete", "options", "head", "patch", "trace",
];

pub fn run(spec: &Value) -> Value {
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
            let mut entry = Map::new();
            entry.insert("method".into(), Value::String(method.to_uppercase()));
            entry.insert("path".into(), Value::String(path.clone()));
            if let Some(s) = op.get("summary").filter(|v| v.is_string()) {
                entry.insert("summary".into(), s.clone());
            }
            out.push(Value::Object(entry));
        }
    }
    Value::Array(out)
}
