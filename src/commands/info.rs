use serde_json::{Map, Value};

pub fn run(spec: &Value) -> Value {
    let mut out = Map::new();

    // Spec version: OpenAPI 3.x uses `openapi`, Swagger 2.0 uses `swagger`.
    // Emit whichever is present; omit the key entirely if neither exists.
    if let Some(v) = spec.get("openapi").filter(|v| !v.is_null()) {
        out.insert("openapi".into(), v.clone());
    } else if let Some(v) = spec.get("swagger").filter(|v| !v.is_null()) {
        out.insert("swagger".into(), v.clone());
    }

    if let Some(Value::Object(info_obj)) = spec.get("info") {
        for key in ["title", "version", "description", "contact", "license"] {
            if let Some(v) = info_obj.get(key).filter(|v| !v.is_null()) {
                out.insert(key.into(), v.clone());
            }
        }
    }

    if let Some(servers) = spec.get("servers").filter(|v| !v.is_null()) {
        out.insert("servers".into(), servers.clone());
    }

    Value::Object(out)
}
