use std::{
    fs,
    io::{self, Read},
    path::Path,
};

use serde_json::Value;

use crate::error::OadigError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Json,
    Yaml,
}

pub struct Loaded {
    pub value: Value,
}

pub fn load(input: &str) -> Result<Loaded, OadigError> {
    let (text, hint) = read_input(input)?;
    let format = detect_format(&text, hint);
    let value = parse(&text, format)?;
    Ok(Loaded { value })
}

fn read_input(input: &str) -> Result<(String, Option<Format>), OadigError> {
    if input == "-" {
        let mut buf = String::new();
        io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| OadigError::read("<stdin>", e))?;
        return Ok((buf, None));
    }
    let path = Path::new(input);
    let text = fs::read_to_string(path).map_err(|e| OadigError::read(path, e))?;
    let hint = path.extension().and_then(|e| e.to_str()).and_then(|e| {
        match e.to_ascii_lowercase().as_str() {
            "json" => Some(Format::Json),
            "yaml" | "yml" => Some(Format::Yaml),
            _ => None,
        }
    });
    Ok((text, hint))
}

fn detect_format(text: &str, hint: Option<Format>) -> Format {
    if let Some(f) = hint {
        return f;
    }
    // Content sniffing: first non-whitespace char is `{` or `[` → JSON.
    for c in text.chars() {
        if c.is_whitespace() {
            continue;
        }
        return if c == '{' || c == '[' {
            Format::Json
        } else {
            Format::Yaml
        };
    }
    Format::Yaml
}

fn parse(text: &str, format: Format) -> Result<Value, OadigError> {
    match format {
        Format::Json => {
            let de = &mut serde_json::Deserializer::from_str(text);
            serde_path_to_error::deserialize(de).map_err(|e| OadigError::ParseJson {
                pointer: e.path().to_string(),
                message: e.inner().to_string(),
            })
        }
        Format::Yaml => {
            let yaml_value: serde_yaml::Value =
                serde_yaml::from_str(text).map_err(|e| OadigError::ParseYaml(e.to_string()))?;
            serde_json::to_value(yaml_value).map_err(|e| OadigError::ParseYaml(e.to_string()))
        }
    }
}
