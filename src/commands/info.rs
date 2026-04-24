use serde_json::{Map, Value};

// Emit only the OpenAPI `info` object (title/version/description/contact/license).
// Spec version (`openapi`/`swagger`) and `servers` are top-level siblings and
// belong elsewhere — `spec` for the version, `overview` for the combined view.
pub fn run(spec: &Value, show_null: bool) -> Value {
    let info_obj = match spec.get("info") {
        Some(Value::Object(obj)) => Some(obj),
        _ => None,
    };

    let mut out = Map::new();
    for key in ["title", "version", "description", "contact", "license"] {
        let value = info_obj
            .and_then(|o| o.get(key))
            .filter(|v| !v.is_null())
            .cloned();
        match value {
            Some(v) => {
                out.insert(key.into(), v);
            }
            None if show_null => {
                out.insert(key.into(), Value::Null);
            }
            _ => {}
        }
    }

    Value::Object(out)
}
