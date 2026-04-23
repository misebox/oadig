use serde_json::Value;

use crate::error::OadigError;
use crate::resolver::{ResolveOptions, Resolver};

pub fn run(spec: &Value, name: &str, opts: ResolveOptions) -> Result<Value, OadigError> {
    let pointer = format!("/components/schemas/{}", name);
    let schema = spec
        .pointer(&pointer)
        .ok_or_else(|| OadigError::SchemaNotFound(name.to_string()))?
        .clone();

    if !opts.resolve {
        return Ok(schema);
    }
    let mut resolver = Resolver::new(spec, opts);
    Ok(resolver.resolve(schema, &format!("#/components/schemas/{}", name)))
}
