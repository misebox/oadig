use serde_json::{Map, Value};

use crate::commands::filter::OpFilter;
use crate::commands::{info, operations, stats};
use crate::resolver::ResolveOptions;

// `raw` is the source spec (used for the version key); `canonical` is the
// same spec normalized to OpenAPI 3.x (used for stats + operations so
// Swagger 2.0 specs produce the same shape).
pub fn run(raw: &Value, canonical: &Value, show_null: bool) -> Value {
    let filter = OpFilter::from_strings(&[]).expect("empty filter list never fails");

    let mut out = Map::new();

    if let Some(v) = raw.get("openapi").filter(|v| !v.is_null()) {
        out.insert("openapi".into(), v.clone());
    } else if let Some(v) = raw.get("swagger").filter(|v| !v.is_null()) {
        out.insert("swagger".into(), v.clone());
    } else if show_null {
        out.insert("openapi".into(), Value::Null);
    }

    out.insert("info".into(), info::run(canonical, show_null));

    if let Some(v) = canonical.get("servers").filter(|v| !v.is_null()) {
        out.insert("servers".into(), v.clone());
    } else if show_null {
        out.insert("servers".into(), Value::Null);
    }

    out.insert("stats".into(), stats::run(canonical));
    out.insert(
        "operations".into(),
        operations::run(canonical, &[], &[], &filter, ResolveOptions::default()),
    );

    Value::Object(out)
}
