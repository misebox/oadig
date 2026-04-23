use regex::Regex;
use serde_json::Value;

use crate::error::OadigError;

pub struct OpFilter {
    methods: Vec<String>, // lowercase; empty = accept any
    path_re: Option<Regex>,
    tag: Option<String>,
}

impl OpFilter {
    pub fn new(
        methods: &[String],
        path_filter: Option<&str>,
        tag: Option<&str>,
    ) -> Result<Self, OadigError> {
        let methods = methods.iter().map(|s| s.to_ascii_lowercase()).collect();
        let path_re = match path_filter {
            Some(p) => {
                Some(Regex::new(p).map_err(|e| OadigError::Other(format!("invalid regex: {e}")))?)
            }
            None => None,
        };
        Ok(Self {
            methods,
            path_re,
            tag: tag.map(str::to_string),
        })
    }

    pub fn accepts(&self, method: &str, path: &str, op: &Value) -> bool {
        if !self.methods.is_empty() && !self.methods.iter().any(|m| m == method) {
            return false;
        }
        if let Some(re) = &self.path_re
            && !re.is_match(path)
        {
            return false;
        }
        if let Some(tag) = &self.tag
            && !op
                .get("tags")
                .and_then(Value::as_array)
                .map(|arr| arr.iter().any(|v| v.as_str() == Some(tag.as_str())))
                .unwrap_or(false)
        {
            return false;
        }
        true
    }
}

pub struct PathFilter {
    re: Option<Regex>,
}

impl PathFilter {
    pub fn new(path_filter: Option<&str>) -> Result<Self, OadigError> {
        let re = match path_filter {
            Some(p) => {
                Some(Regex::new(p).map_err(|e| OadigError::Other(format!("invalid regex: {e}")))?)
            }
            None => None,
        };
        Ok(Self { re })
    }

    pub fn accepts(&self, path: &str) -> bool {
        match &self.re {
            Some(re) => re.is_match(path),
            None => true,
        }
    }
}
