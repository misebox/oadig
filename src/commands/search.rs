use std::collections::HashSet;

use regex::RegexBuilder;
use serde_json::{Map, Value, json};

use crate::cli::SearchField;
use crate::commands::operations::METHODS;
use crate::error::OadigError;

const ALL_FIELDS: &[SearchField] = &[
    SearchField::Pointer,
    SearchField::JsonPath,
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

// Typed token: array index vs object key. Kept typed because JSONPath
// renders `[0]` for indexes but `.name` / `["complex"]` for keys.
#[derive(Debug, Clone)]
enum Token {
    Key(String),
    Index(usize),
}

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
    let mut tokens: Vec<Token> = Vec::new();
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
    tokens: &mut Vec<Token>,
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
                tokens.push(Token::Key(k.clone()));
                walk(v, tokens, pattern, fields, hits);
                tokens.pop();
            }
        }
        Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                tokens.push(Token::Index(i));
                walk(v, tokens, pattern, fields, hits);
                tokens.pop();
            }
        }
        _ => {}
    }
}

fn build_hit(tokens: &[Token], value: &str, fields: &HashSet<SearchField>) -> Value {
    let mut out = Map::new();
    let op_context = operation_context(tokens);

    if fields.contains(&SearchField::Pointer) {
        out.insert("pointer".into(), Value::String(to_pointer(tokens)));
    }
    if fields.contains(&SearchField::JsonPath) {
        out.insert("jsonPath".into(), Value::String(to_jsonpath(tokens)));
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

// RFC 6901 JSON Pointer. Keys escape `~` → `~0`, `/` → `~1`; indexes
// render as their decimal string.
fn to_pointer(tokens: &[Token]) -> String {
    let mut s = String::new();
    for t in tokens {
        s.push('/');
        match t {
            Token::Key(k) => {
                for ch in k.chars() {
                    match ch {
                        '~' => s.push_str("~0"),
                        '/' => s.push_str("~1"),
                        c => s.push(c),
                    }
                }
            }
            Token::Index(i) => {
                s.push_str(&i.to_string());
            }
        }
    }
    s
}

// Best-effort RFC 9535 JSONPath expression. Plain identifier keys use
// dot notation; everything else falls back to quoted bracket notation
// so the expression survives spec paths like `/pets/{id}`.
fn to_jsonpath(tokens: &[Token]) -> String {
    let mut s = String::from("$");
    for t in tokens {
        match t {
            Token::Index(i) => {
                s.push_str(&format!("[{}]", i));
            }
            Token::Key(k) if is_plain_ident(k) => {
                s.push('.');
                s.push_str(k);
            }
            Token::Key(k) => {
                s.push_str(&format!("[\"{}\"]", escape_jsonpath_key(k)));
            }
        }
    }
    s
}

fn is_plain_ident(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn escape_jsonpath_key(k: &str) -> String {
    let mut out = String::with_capacity(k.len());
    for ch in k.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            c => out.push(c),
        }
    }
    out
}

// Decide whether the hit is inside an operation. Expected token prefix:
// [Key("paths"), Key(<path>), Key(<method>), ...]. Returns the operation
// reference and the JSONPath-style suffix after `<method>`.
fn operation_context(tokens: &[Token]) -> Option<(Value, Value)> {
    if tokens.len() < 3 {
        return None;
    }
    let Token::Key(head) = &tokens[0] else {
        return None;
    };
    if head != "paths" {
        return None;
    }
    let Token::Key(op_path) = &tokens[1] else {
        return None;
    };
    let Token::Key(method) = &tokens[2] else {
        return None;
    };
    if !METHODS.contains(&method.as_str()) {
        return None;
    }
    let op_ref = json!({
        "method": method.to_uppercase(),
        "path": op_path.clone(),
    });
    let at = to_at_expr(&tokens[3..]);
    Some((op_ref, Value::String(at)))
}

// JSONPath-style relative expression (no leading `$`). Used for the tail
// portion after an operation prefix.
fn to_at_expr(tokens: &[Token]) -> String {
    let mut s = String::new();
    for t in tokens {
        match t {
            Token::Index(i) => s.push_str(&format!("[{}]", i)),
            Token::Key(k) if is_plain_ident(k) => {
                if !s.is_empty() {
                    s.push('.');
                }
                s.push_str(k);
            }
            Token::Key(k) => {
                s.push_str(&format!("[\"{}\"]", escape_jsonpath_key(k)));
            }
        }
    }
    s
}
