use serde_json::{Map, Value};

use crate::commands::operations;
use crate::error::OadigError;
use crate::resolver::ResolveOptions;

pub fn run(
    spec: &Value,
    id: Option<&str>,
    method: Option<&str>,
    path: Option<&str>,
    opts: ResolveOptions,
) -> Result<Value, OadigError> {
    let located = operations::resolve_lookup(spec, id, method, path)?;
    let origin = format!("#operation/{}/{}", located.method, located.path);
    let resolved = operations::resolve_in_place(located.op.clone(), spec, opts, &origin);

    let mut entry = Map::new();
    entry.insert(
        "method".into(),
        Value::String(located.method.to_uppercase()),
    );
    entry.insert("path".into(), Value::String(located.path.clone()));
    if let Value::Object(obj) = resolved {
        for (k, v) in obj {
            entry.insert(k, v);
        }
    }
    Ok(Value::Object(entry))
}
