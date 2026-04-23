use serde_json::Value;

use crate::cli::Format;
use crate::error::OadigError;

#[derive(Debug, Clone, Copy)]
pub enum Display {
    Pretty,
    Compact,
    Lines,
}

pub fn render(value: &Value, format: Format, display: Display) -> Result<String, OadigError> {
    let map_err = |e: serde_json::Error| OadigError::Other(e.to_string());
    match (format, display) {
        (Format::Json, Display::Compact) => serde_json::to_string(value).map_err(map_err),
        (Format::Json, Display::Pretty) => serde_json::to_string_pretty(value).map_err(map_err),
        (Format::Json, Display::Lines) => render_lines(value).map_err(map_err),
        (Format::Yaml, _) => {
            serde_yaml::to_string(value).map_err(|e| OadigError::Other(e.to_string()))
        }
    }
}

// Top-level array multi-line, each element compact on its own line.
// Non-array values fall back to pretty so the flag is safe to apply globally.
fn render_lines(value: &Value) -> Result<String, serde_json::Error> {
    let Value::Array(items) = value else {
        return serde_json::to_string_pretty(value);
    };
    if items.is_empty() {
        return Ok("[]".to_string());
    }
    let parts: Result<Vec<_>, _> = items.iter().map(serde_json::to_string).collect();
    Ok(format!("[\n  {}\n]", parts?.join(",\n  ")))
}
