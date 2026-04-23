use serde_json::Value;

pub fn run(spec: &Value) -> Value {
    if let Some(v) = spec.get("openapi").filter(|v| !v.is_null()) {
        return v.clone();
    }
    if let Some(v) = spec.get("swagger").filter(|v| !v.is_null()) {
        return v.clone();
    }
    Value::Null
}
