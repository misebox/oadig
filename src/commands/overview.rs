use serde_json::{Value, json};

use crate::commands::{info, operations, stats};

pub fn run(spec: &Value) -> Value {
    json!({
        "info": info::run(spec),
        "stats": stats::run(spec),
        "operations": operations::run(spec),
    })
}
