use serde_json::{json, Value};

const METHODS: &[&str] = &[
    "get", "put", "post", "delete", "options", "head", "patch", "trace",
];

pub fn run(spec: &Value) -> Value {
    let Some(paths) = spec.get("paths").and_then(Value::as_object) else {
        return Value::Array(vec![]);
    };
    let mut out = Vec::new();
    for (path, item) in paths {
        let Some(item_obj) = item.as_object() else { continue };
        for method in METHODS {
            if item_obj.contains_key(*method) {
                out.push(json!({
                    "method": method.to_uppercase(),
                    "path": path,
                }));
            }
        }
    }
    Value::Array(out)
}
