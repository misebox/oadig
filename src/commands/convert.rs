use serde_json::{Map, Value, json};

use crate::error::OadigError;

// Top-level convert dispatch. Only Swagger 2.0 → OpenAPI 3.0 is implemented
// for the first cut; other directions are future phases.
pub fn run(spec: &Value, target: &str) -> Result<Value, OadigError> {
    let swagger = spec.get("swagger").and_then(Value::as_str);
    let openapi = spec.get("openapi").and_then(Value::as_str);

    match (swagger, openapi, target) {
        (Some("2.0"), _, "3.0") => Ok(swagger2_to_openapi30(spec)),
        (None, Some(v), t) if v.starts_with("3.") && v == t => Ok(spec.clone()),
        (None, Some(_), _) => Err(OadigError::Other(format!(
            "conversion to {target} from OpenAPI 3.x is not implemented yet"
        ))),
        (Some("2.0"), _, _) => Err(OadigError::Other(format!(
            "conversion to {target} from Swagger 2.0 is not implemented (try 3.0)"
        ))),
        _ => Err(OadigError::Other(format!(
            "unable to determine source spec version; target={target}"
        ))),
    }
}

// ---- Swagger 2.0 → OpenAPI 3.0.3 ----

fn swagger2_to_openapi30(spec: &Value) -> Value {
    let mut out = Map::new();
    out.insert("openapi".into(), Value::String("3.0.3".into()));

    if let Some(info) = spec.get("info") {
        out.insert("info".into(), info.clone());
    }

    if let Some(servers) = build_servers(spec) {
        out.insert("servers".into(), servers);
    }

    for key in ["tags", "security", "externalDocs"] {
        if let Some(v) = spec.get(key).filter(|v| !v.is_null()) {
            out.insert(key.into(), v.clone());
        }
    }

    let global_consumes = collect_strings(spec.get("consumes"));
    let global_produces = collect_strings(spec.get("produces"));

    if let Some(paths) = spec.get("paths").and_then(Value::as_object) {
        let mut new_paths = Map::new();
        for (path, item) in paths {
            new_paths.insert(
                path.clone(),
                convert_path_item(item, &global_consumes, &global_produces),
            );
        }
        out.insert("paths".into(), Value::Object(new_paths));
    }

    let components = build_components(spec);
    if !components.is_empty() {
        out.insert("components".into(), Value::Object(components));
    }

    let mut rewritten = Value::Object(out);
    rewrite_refs(&mut rewritten);
    rewritten
}

fn build_servers(spec: &Value) -> Option<Value> {
    let host = spec.get("host").and_then(Value::as_str)?;
    let base = spec.get("basePath").and_then(Value::as_str).unwrap_or("");
    let schemes: Vec<String> = spec
        .get("schemes")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .map(String::from)
                .collect()
        })
        .filter(|v: &Vec<String>| !v.is_empty())
        .unwrap_or_else(|| vec!["https".to_string()]);
    let urls: Vec<Value> = schemes
        .iter()
        .map(|s| json!({ "url": format!("{s}://{host}{base}") }))
        .collect();
    Some(Value::Array(urls))
}

fn build_components(spec: &Value) -> Map<String, Value> {
    let mut components = Map::new();
    if let Some(defs) = spec.get("definitions").and_then(Value::as_object) {
        components.insert("schemas".into(), Value::Object(defs.clone()));
    }
    if let Some(params) = spec.get("parameters").and_then(Value::as_object) {
        let mut out = Map::new();
        for (k, v) in params {
            out.insert(k.clone(), convert_parameter(v));
        }
        components.insert("parameters".into(), Value::Object(out));
    }
    if let Some(resps) = spec.get("responses").and_then(Value::as_object) {
        let mut out = Map::new();
        for (k, v) in resps {
            out.insert(k.clone(), convert_response(v, &[]));
        }
        components.insert("responses".into(), Value::Object(out));
    }
    if let Some(sec) = spec.get("securityDefinitions").and_then(Value::as_object) {
        components.insert("securitySchemes".into(), Value::Object(sec.clone()));
    }
    components
}

fn convert_path_item(item: &Value, g_consumes: &[String], g_produces: &[String]) -> Value {
    let Some(item_obj) = item.as_object() else {
        return item.clone();
    };
    let mut out = Map::new();
    for (key, value) in item_obj {
        let is_method = matches!(
            key.as_str(),
            "get" | "put" | "post" | "delete" | "options" | "head" | "patch" | "trace"
        );
        if is_method {
            out.insert(
                key.clone(),
                convert_operation(value, g_consumes, g_produces),
            );
        } else if key == "parameters" {
            if let Some(arr) = value.as_array() {
                let converted: Vec<Value> = arr.iter().map(convert_parameter).collect();
                out.insert("parameters".into(), Value::Array(converted));
            }
        } else {
            out.insert(key.clone(), value.clone());
        }
    }
    Value::Object(out)
}

fn convert_operation(op: &Value, g_consumes: &[String], g_produces: &[String]) -> Value {
    let Some(op_obj) = op.as_object() else {
        return op.clone();
    };
    let consumes = collect_strings_or(op.get("consumes"), g_consumes);
    let produces = collect_strings_or(op.get("produces"), g_produces);

    let mut out = Map::new();
    let mut new_parameters: Vec<Value> = Vec::new();
    let mut body_param: Option<Value> = None;
    let mut form_params: Vec<Value> = Vec::new();

    for (key, value) in op_obj {
        match key.as_str() {
            "consumes" | "produces" => {
                // Absorbed into requestBody/responses content; drop from op.
            }
            "parameters" => {
                if let Some(arr) = value.as_array() {
                    for p in arr {
                        match p.get("in").and_then(Value::as_str) {
                            Some("body") => body_param = Some(p.clone()),
                            Some("formData") => form_params.push(p.clone()),
                            _ => new_parameters.push(convert_parameter(p)),
                        }
                    }
                }
            }
            "responses" => {
                let responses = convert_responses(value, &produces);
                out.insert("responses".into(), responses);
            }
            "schemes" => {
                // Could become per-operation servers; skipped for MVP.
            }
            _ => {
                out.insert(key.clone(), value.clone());
            }
        }
    }

    if !new_parameters.is_empty() {
        out.insert("parameters".into(), Value::Array(new_parameters));
    }
    if let Some(body) = body_param {
        out.insert(
            "requestBody".into(),
            body_param_to_request_body(&body, &consumes),
        );
    } else if !form_params.is_empty() {
        out.insert(
            "requestBody".into(),
            form_params_to_request_body(&form_params, &consumes),
        );
    }

    Value::Object(out)
}

fn convert_parameter(p: &Value) -> Value {
    let Some(obj) = p.as_object() else {
        return p.clone();
    };
    if obj.get("$ref").is_some() {
        return p.clone();
    }
    let mut out = Map::new();
    let mut schema = Map::new();
    for (k, v) in obj {
        match k.as_str() {
            "name" | "in" | "description" | "required" | "deprecated" | "allowEmptyValue" => {
                out.insert(k.clone(), v.clone());
            }
            "type" | "format" | "items" | "collectionFormat" | "enum" | "default" | "minimum"
            | "maximum" | "exclusiveMinimum" | "exclusiveMaximum" | "minLength" | "maxLength"
            | "pattern" | "uniqueItems" | "multipleOf" => {
                schema.insert(k.clone(), v.clone());
            }
            _ => {
                out.insert(k.clone(), v.clone());
            }
        }
    }
    if !schema.is_empty() {
        out.insert("schema".into(), Value::Object(schema));
    }
    Value::Object(out)
}

fn convert_responses(responses: &Value, produces: &[String]) -> Value {
    let Some(obj) = responses.as_object() else {
        return responses.clone();
    };
    let mut out = Map::new();
    for (code, r) in obj {
        out.insert(code.clone(), convert_response(r, produces));
    }
    Value::Object(out)
}

fn convert_response(r: &Value, produces: &[String]) -> Value {
    let Some(obj) = r.as_object() else {
        return r.clone();
    };
    if obj.get("$ref").is_some() {
        return r.clone();
    }
    let mut out = Map::new();
    let mut content = Map::new();
    for (k, v) in obj {
        match k.as_str() {
            "schema" => {
                let media_types = if produces.is_empty() {
                    vec!["application/json".to_string()]
                } else {
                    produces.to_vec()
                };
                for mt in media_types {
                    content.insert(mt, json!({ "schema": v.clone() }));
                }
            }
            _ => {
                out.insert(k.clone(), v.clone());
            }
        }
    }
    if !content.is_empty() {
        out.insert("content".into(), Value::Object(content));
    }
    if !out.contains_key("description") {
        out.insert("description".into(), Value::String(String::new()));
    }
    Value::Object(out)
}

fn body_param_to_request_body(p: &Value, consumes: &[String]) -> Value {
    let schema = p
        .get("schema")
        .cloned()
        .unwrap_or_else(|| json!({ "type": "object" }));
    let media_types = if consumes.is_empty() {
        vec!["application/json".to_string()]
    } else {
        consumes.to_vec()
    };
    let mut content = Map::new();
    for mt in media_types {
        content.insert(mt, json!({ "schema": schema.clone() }));
    }
    let mut body = Map::new();
    if let Some(desc) = p.get("description").filter(|v| !v.is_null()) {
        body.insert("description".into(), desc.clone());
    }
    if p.get("required").and_then(Value::as_bool) == Some(true) {
        body.insert("required".into(), Value::Bool(true));
    }
    body.insert("content".into(), Value::Object(content));
    Value::Object(body)
}

fn form_params_to_request_body(params: &[Value], consumes: &[String]) -> Value {
    // Build a JSON schema object out of the individual formData parameters.
    let mut properties = Map::new();
    let mut required: Vec<Value> = Vec::new();
    for p in params {
        let Some(obj) = p.as_object() else { continue };
        let Some(name) = obj.get("name").and_then(Value::as_str) else {
            continue;
        };
        let mut schema = Map::new();
        for (k, v) in obj {
            if matches!(
                k.as_str(),
                "type" | "format" | "items" | "enum" | "default" | "minimum" | "maximum"
            ) {
                schema.insert(k.clone(), v.clone());
            }
        }
        properties.insert(name.into(), Value::Object(schema));
        if obj.get("required").and_then(Value::as_bool) == Some(true) {
            required.push(Value::String(name.into()));
        }
    }
    let mut schema_obj = Map::new();
    schema_obj.insert("type".into(), Value::String("object".into()));
    schema_obj.insert("properties".into(), Value::Object(properties));
    if !required.is_empty() {
        schema_obj.insert("required".into(), Value::Array(required));
    }
    let media_types = if consumes.is_empty() {
        vec!["application/x-www-form-urlencoded".to_string()]
    } else {
        consumes.to_vec()
    };
    let mut content = Map::new();
    for mt in media_types {
        content.insert(mt, json!({ "schema": Value::Object(schema_obj.clone()) }));
    }
    json!({ "content": Value::Object(content) })
}

// Rewrite `#/definitions/X` → `#/components/schemas/X`, and similar
// for top-level parameters/responses. Walks the whole tree in place.
fn rewrite_refs(value: &mut Value) {
    match value {
        Value::Object(obj) => {
            if let Some(Value::String(s)) = obj.get("$ref").cloned()
                && let Some(rewritten) = rewrite_ref(&s)
            {
                obj.insert("$ref".into(), Value::String(rewritten));
            }
            for (_, v) in obj.iter_mut() {
                rewrite_refs(v);
            }
        }
        Value::Array(arr) => {
            for v in arr.iter_mut() {
                rewrite_refs(v);
            }
        }
        _ => {}
    }
}

fn rewrite_ref(r: &str) -> Option<String> {
    for (from, to) in [
        ("#/definitions/", "#/components/schemas/"),
        ("#/parameters/", "#/components/parameters/"),
        ("#/responses/", "#/components/responses/"),
        ("#/securityDefinitions/", "#/components/securitySchemes/"),
    ] {
        if let Some(rest) = r.strip_prefix(from) {
            return Some(format!("{to}{rest}"));
        }
    }
    None
}

// ---- helpers ----

fn collect_strings(v: Option<&Value>) -> Vec<String> {
    v.and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .map(String::from)
                .collect()
        })
        .unwrap_or_default()
}

fn collect_strings_or(v: Option<&Value>, fallback: &[String]) -> Vec<String> {
    let local = collect_strings(v);
    if local.is_empty() {
        fallback.to_vec()
    } else {
        local
    }
}
