use serde_json::{Map, Value};

pub fn run(spec: &Value, show_null: bool) -> Value {
    let mut out = Map::new();

    // Spec version: OpenAPI 3.x uses `openapi`, Swagger 2.0 uses `swagger`.
    // Emit whichever is present. When neither exists and `show_null` is on,
    // still surface the expected `openapi` key as null to make the absence
    // explicit.
    let openapi = spec.get("openapi").filter(|v| !v.is_null()).cloned();
    let swagger = spec.get("swagger").filter(|v| !v.is_null()).cloned();
    match (openapi, swagger) {
        (Some(v), _) => {
            out.insert("openapi".into(), v);
        }
        (None, Some(v)) => {
            out.insert("swagger".into(), v);
        }
        (None, None) if show_null => {
            out.insert("openapi".into(), Value::Null);
        }
        _ => {}
    }

    let info_obj = match spec.get("info") {
        Some(Value::Object(obj)) => Some(obj),
        _ => None,
    };
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

    let servers = spec.get("servers").filter(|v| !v.is_null()).cloned();
    match servers {
        Some(v) => {
            out.insert("servers".into(), v);
        }
        None if show_null => {
            out.insert("servers".into(), Value::Null);
        }
        _ => {}
    }

    Value::Object(out)
}
