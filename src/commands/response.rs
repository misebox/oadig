use serde_json::Value;

use crate::commands::operations;
use crate::error::OadigError;
use crate::resolver::ResolveOptions;

pub fn run(
    spec: &Value,
    id: Option<&str>,
    method: Option<&str>,
    path: Option<&str>,
    status: Option<&str>,
    opts: ResolveOptions,
) -> Result<Value, OadigError> {
    let located = operations::resolve_lookup(spec, id, method, path)?;
    let responses = located.op.get("responses").cloned().unwrap_or(Value::Null);
    let selected = match status {
        Some(code) => responses.get(code).cloned().unwrap_or(Value::Null),
        None => responses,
    };
    let origin = format!("#response/{}/{}", located.method, located.path);
    Ok(operations::resolve_in_place(selected, spec, opts, &origin))
}
