use serde_json::{Value, json};

use crate::commands::{info, operations, stats};
use crate::resolver::ResolveOptions;

pub fn run(spec: &Value, show_null: bool) -> Value {
    json!({
        "info": info::run(spec, show_null),
        "stats": stats::run(spec),
        "operations": operations::run(spec, &[], &[], ResolveOptions::default()),
    })
}
