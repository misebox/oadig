use serde_json::{Value, json};

use crate::commands::filter::OpFilter;
use crate::commands::{info, operations, stats};
use crate::resolver::ResolveOptions;

pub fn run(spec: &Value, show_null: bool) -> Value {
    let filter = OpFilter::new(&[], None, None, None);
    json!({
        "info": info::run(spec, show_null),
        "stats": stats::run(spec),
        "operations": operations::run(spec, &[], &[], &filter, ResolveOptions::default()),
    })
}
