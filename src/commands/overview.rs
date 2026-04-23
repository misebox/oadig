use serde_json::{Value, json};

use crate::commands::{info, paths, stats};

pub fn run(spec: &Value) -> Value {
    json!({
        "info": info::run(spec),
        "stats": stats::run(spec),
        "paths": paths::run(spec),
    })
}
