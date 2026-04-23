use serde_json::{Map, Value};

// OpenAPI 3.x component section names.
const SECTIONS: &[&str] = &[
    "schemas",
    "responses",
    "parameters",
    "examples",
    "requestBodies",
    "headers",
    "securitySchemes",
    "links",
    "callbacks",
    "pathItems",
];

pub fn run(spec: &Value, show_null: bool) -> Value {
    let components = spec.get("components").and_then(Value::as_object);

    let mut out = Map::new();
    for section in SECTIONS {
        let names: Vec<Value> = components
            .and_then(|c| c.get(*section))
            .and_then(Value::as_object)
            .map(|m| m.keys().cloned().map(Value::String).collect())
            .unwrap_or_default();

        if !names.is_empty() || show_null {
            out.insert((*section).into(), Value::Array(names));
        }
    }
    Value::Object(out)
}
