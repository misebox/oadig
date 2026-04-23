use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum OadigError {
    #[error("failed to read {path}: {source}")]
    Read {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse JSON at {pointer}: {message}")]
    ParseJson { pointer: String, message: String },

    #[error("failed to parse YAML: {0}")]
    ParseYaml(String),

    #[error("schema not found: {0}")]
    SchemaNotFound(String),

    #[error("{0}")]
    Other(String),
}

impl OadigError {
    pub fn read(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Read {
            path: path.into().display().to_string(),
            source,
        }
    }
}
