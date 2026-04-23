use serde_json::Value;

pub fn run(spec: &Value) -> Value {
    let names: Vec<String> = spec
        .pointer("/components/schemas")
        .and_then(Value::as_object)
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default();
    Value::Array(names.into_iter().map(Value::String).collect())
}
