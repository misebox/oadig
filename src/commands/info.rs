use serde_json::{Map, Value};

pub fn run(spec: &Value) -> Value {
    let info = spec.get("info").cloned().unwrap_or(Value::Null);
    let servers = spec.get("servers").cloned().unwrap_or(Value::Array(vec![]));
    let openapi = spec.get("openapi").cloned().unwrap_or(Value::Null);

    let mut out = Map::new();
    out.insert("openapi".into(), openapi);
    if let Value::Object(info_obj) = info {
        for key in ["title", "version", "description", "contact", "license"] {
            if let Some(v) = info_obj.get(key) {
                out.insert(key.into(), v.clone());
            }
        }
    }
    out.insert("servers".into(), servers);
    Value::Object(out)
}
