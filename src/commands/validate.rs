use serde_json::{Map, Value, json};
use serde_path_to_error::Segment;

use crate::error::OadigError;

// Validate a loaded spec as OpenAPI 3.x. Swagger 2.0 specs are detected
// and reported as unsupported (until a dedicated 2.0 validator is added).
pub fn run(spec: &Value) -> Result<Value, OadigError> {
    if spec.get("swagger").is_some() {
        return Ok(json!({
            "valid": null,
            "version": spec.get("swagger").cloned().unwrap_or(Value::Null),
            "errors": [ { "message": "Swagger 2.0 validation is not implemented; only OpenAPI 3.x is supported." } ],
        }));
    }

    let version = spec.get("openapi").cloned().unwrap_or(Value::Null);

    // Re-serialize to JSON bytes so serde_path_to_error can stream over a
    // real Deserializer and give us typed path segments instead of a
    // stringy Value-level report.
    let bytes = serde_json::to_vec(spec)
        .map_err(|e| OadigError::Other(format!("failed to re-serialize spec: {e}")))?;
    let de = &mut serde_json::Deserializer::from_slice(&bytes);
    let result: Result<oas3::Spec, _> = serde_path_to_error::deserialize(de);

    match result {
        Ok(_) => Ok(json!({ "valid": true, "version": version })),
        Err(err) => {
            let mut entry = Map::new();
            entry.insert("pointer".into(), Value::String(to_pointer(err.path())));
            entry.insert("message".into(), Value::String(err.inner().to_string()));
            Ok(json!({
                "valid": false,
                "version": version,
                "errors": [ Value::Object(entry) ],
            }))
        }
    }
}

// Convert a serde_path_to_error Path into an RFC 6901 JSON Pointer.
fn to_pointer(path: &serde_path_to_error::Path) -> String {
    let mut out = String::new();
    for seg in path.iter() {
        match seg {
            Segment::Map { key } => {
                out.push('/');
                for ch in key.chars() {
                    match ch {
                        '~' => out.push_str("~0"),
                        '/' => out.push_str("~1"),
                        c => out.push(c),
                    }
                }
            }
            Segment::Seq { index } => {
                out.push('/');
                out.push_str(&index.to_string());
            }
            Segment::Enum { variant } => {
                out.push('/');
                out.push_str(variant);
            }
            Segment::Unknown => {
                out.push_str("/?");
            }
        }
    }
    out
}
