use serde_json::Value;

use crate::commands::filter::PathFilter;

pub fn run(spec: &Value, filter: &PathFilter) -> Value {
    let Some(paths) = spec.get("paths").and_then(Value::as_object) else {
        return Value::Array(vec![]);
    };
    let names: Vec<Value> = paths
        .keys()
        .filter(|k| filter.accepts(k))
        .cloned()
        .map(Value::String)
        .collect();
    Value::Array(names)
}
