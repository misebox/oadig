use serde::Serialize;

use crate::cli::Format;
use crate::error::OadigError;

pub fn render<T: Serialize>(
    value: &T,
    format: Format,
    compact: bool,
) -> Result<String, OadigError> {
    match format {
        Format::Json => {
            if compact {
                serde_json::to_string(value).map_err(|e| OadigError::Other(e.to_string()))
            } else {
                serde_json::to_string_pretty(value)
                    .map_err(|e| OadigError::Other(e.to_string()))
            }
        }
        Format::Yaml => {
            serde_yaml::to_string(value).map_err(|e| OadigError::Other(e.to_string()))
        }
    }
}
