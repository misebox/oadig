use serde_json::{Map, Value};

use crate::commands::filter::OpFilter;
use crate::commands::{info, operations, stats};
use crate::resolver::ResolveOptions;

pub fn run(spec: &Value, show_null: bool) -> Value {
    let filter = OpFilter::from_strings(&[]).expect("empty filter list never fails");

    let mut out = Map::new();

    // Spec version leads so the shape of everything below is immediately
    // scoped. Prefer `openapi`; fall back to Swagger 2.0's `swagger`.
    if let Some(v) = spec.get("openapi").filter(|v| !v.is_null()) {
        out.insert("openapi".into(), v.clone());
    } else if let Some(v) = spec.get("swagger").filter(|v| !v.is_null()) {
        out.insert("swagger".into(), v.clone());
    } else if show_null {
        out.insert("openapi".into(), Value::Null);
    }

    out.insert("info".into(), info::run(spec, show_null));

    // `servers` is top-level in OpenAPI; include it here alongside info.
    if let Some(v) = spec.get("servers").filter(|v| !v.is_null()) {
        out.insert("servers".into(), v.clone());
    } else if show_null {
        out.insert("servers".into(), Value::Null);
    }

    out.insert("stats".into(), stats::run(spec));
    out.insert(
        "operations".into(),
        operations::run(spec, &[], &[], &filter, ResolveOptions::default()),
    );

    Value::Object(out)
}
