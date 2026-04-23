use regex::RegexBuilder;
use serde_json::{Map, Value};

use crate::error::OadigError;

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
) -> Result<Value, OadigError> {
    let pattern = Pattern::build(keyword, regex, case_sensitive)?;
    let mut pointer = String::new();
    let mut hits = Vec::new();
    walk(spec, &mut pointer, &pattern, &mut hits);
    Ok(Value::Array(hits))
}

fn walk(value: &Value, pointer: &mut String, pattern: &Pattern, hits: &mut Vec<Value>) {
    match value {
        Value::String(s) if pattern.matches(s) => {
            let mut entry = Map::new();
            entry.insert("pointer".into(), Value::String(pointer.clone()));
            entry.insert("value".into(), Value::String(s.clone()));
            hits.push(Value::Object(entry));
        }
        Value::Object(obj) => {
            for (k, v) in obj {
                let saved = pointer.len();
                pointer.push('/');
                pointer.push_str(&escape_token(k));
                walk(v, pointer, pattern, hits);
                pointer.truncate(saved);
            }
        }
        Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                let saved = pointer.len();
                pointer.push('/');
                pointer.push_str(&i.to_string());
                walk(v, pointer, pattern, hits);
                pointer.truncate(saved);
            }
        }
        _ => {}
    }
}

// RFC 6901: ~ → ~0, / → ~1.
fn escape_token(token: &str) -> String {
    token.replace('~', "~0").replace('/', "~1")
}
