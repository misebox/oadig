use std::collections::HashSet;

use regex::RegexBuilder;
use serde_json::{Map, Value, json};

use crate::cli::SearchField;
use crate::commands::operations::METHODS;
use crate::error::OadigError;

const ALL_FIELDS: &[SearchField] = &[
    SearchField::Pointer,
    SearchField::Path,
    SearchField::OperationRef,
    SearchField::At,
    SearchField::Value,
];

const DEFAULT_FIELDS: &[SearchField] = &[
    SearchField::Pointer,
    SearchField::OperationRef,
    SearchField::At,
    SearchField::Value,
];

enum Pattern {
    Substring {
        needle: String,
        case_sensitive: bool,
    },
    Regex(regex::Regex),
}

impl Pattern {
    fn build(keyword: &str, regex: bool, case_sensitive: bool) -> Result<Self, OadigError> {
        if regex {
            let re = RegexBuilder::new(keyword)
                .case_insensitive(!case_sensitive)
                .build()
                .map_err(|e| OadigError::Other(format!("invalid regex: {e}")))?;
            Ok(Pattern::Regex(re))
        } else {
            let needle = if case_sensitive {
                keyword.to_string()
            } else {
                keyword.to_ascii_lowercase()
            };
            Ok(Pattern::Substring {
                needle,
                case_sensitive,
            })
        }
    }

    fn matches(&self, haystack: &str) -> bool {
        match self {
            Pattern::Substring {
                needle,
                case_sensitive,
            } => {
                if *case_sensitive {
                    haystack.contains(needle.as_str())
                } else {
                    haystack.to_ascii_lowercase().contains(needle.as_str())
                }
            }
            Pattern::Regex(re) => re.is_match(haystack),
        }
    }
}

pub fn run(
    spec: &Value,
    keyword: &str,
    regex: bool,
    case_sensitive: bool,
    include: &[SearchField],
    exclude: &[SearchField],
) -> Result<Value, OadigError> {
    let pattern = Pattern::build(keyword, regex, case_sensitive)?;
    let fields = resolve_fields(include, exclude);
    let mut tokens: Vec<String> = Vec::new();
    let mut hits = Vec::new();
    walk(spec, &mut tokens, &pattern, &fields, &mut hits);
    Ok(Value::Array(hits))
}

fn resolve_fields(include: &[SearchField], exclude: &[SearchField]) -> HashSet<SearchField> {
    let mut fields: HashSet<SearchField> = DEFAULT_FIELDS.iter().copied().collect();
    for f in include {
        if *f == SearchField::All {
            fields.extend(ALL_FIELDS.iter().copied());
        } else {
            fields.insert(*f);
        }
    }
    for f in exclude {
        if *f == SearchField::All {
            fields.clear();
        } else {
            fields.remove(f);
        }
    }
    fields
}

fn walk(
    value: &Value,
    tokens: &mut Vec<String>,
    pattern: &Pattern,
    fields: &HashSet<SearchField>,
    hits: &mut Vec<Value>,
) {
    match value {
        Value::String(s) if pattern.matches(s) => {
            hits.push(build_hit(tokens, s, fields));
        }
        Value::Object(obj) => {
            for (k, v) in obj {
                tokens.push(k.clone());
                walk(v, tokens, pattern, fields, hits);
                tokens.pop();
            }
        }
        Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                tokens.push(i.to_string());
                walk(v, tokens, pattern, fields, hits);
                tokens.pop();
            }
        }
        _ => {}
    }
}

fn build_hit(tokens: &[String], value: &str, fields: &HashSet<SearchField>) -> Value {
    let mut out = Map::new();

    // operationRef + at are derived from the prefix `paths/<path>/<method>/...`.
    let op_context = operation_context(tokens);

    if fields.contains(&SearchField::Pointer) {
        out.insert("pointer".into(), Value::String(to_pointer(tokens)));
    }
    if fields.contains(&SearchField::Path) {
        let path: Vec<Value> = tokens.iter().cloned().map(Value::String).collect();
        out.insert("path".into(), Value::Array(path));
    }
    if fields.contains(&SearchField::OperationRef)
        && let Some((ref op_ref, _)) = op_context
    {
        out.insert("operationRef".into(), op_ref.clone());
    }
    if fields.contains(&SearchField::At)
        && let Some((_, ref at)) = op_context
    {
        out.insert("at".into(), at.clone());
    }
    if fields.contains(&SearchField::Value) {
        out.insert("value".into(), Value::String(value.to_string()));
    }
    Value::Object(out)
}

// Build the RFC 6901 JSON Pointer from raw tokens. Escape `~` → `~0`, `/` → `~1`.
fn to_pointer(tokens: &[String]) -> String {
    let mut s = String::new();
    for t in tokens {
        s.push('/');
        for ch in t.chars() {
            match ch {
                '~' => s.push_str("~0"),
                '/' => s.push_str("~1"),
                c => s.push(c),
            }
        }
    }
    s
}

// Inspect the token prefix to decide whether the hit is inside an operation.
// Expected shape: tokens = ["paths", "<path>", "<method>", ...rest].
fn operation_context(tokens: &[String]) -> Option<(Value, Value)> {
    if tokens.len() < 3 || tokens[0] != "paths" {
        return None;
    }
    let method = tokens[2].as_str();
    if !METHODS.contains(&method) {
        return None;
    }
    let op_ref = json!({
        "method": method.to_uppercase(),
        "path": tokens[1].clone(),
    });
    let at: Vec<Value> = tokens.iter().skip(3).cloned().map(Value::String).collect();
    Some((op_ref, Value::Array(at)))
}
