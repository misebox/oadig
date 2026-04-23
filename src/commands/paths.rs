use serde_json::Value;

pub fn run(spec: &Value) -> Value {
    let names: Vec<Value> = spec
        .get("paths")
        .and_then(Value::as_object)
        .map(|m| m.keys().cloned().map(Value::String).collect())
        .unwrap_or_default();
    Value::Array(names)
}
