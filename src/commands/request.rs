use serde_json::Value;

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
    let body = located
        .op
        .get("requestBody")
        .cloned()
        .unwrap_or(Value::Null);
    let origin = format!("#request/{}/{}", located.method, located.path);
    Ok(operations::resolve_in_place(body, spec, opts, &origin))
}
