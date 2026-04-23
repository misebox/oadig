use serde_json::Value;

pub struct OpFilter {
    methods: Vec<String>, // lowercase; empty = accept any
    contains: Option<String>,
    prefix: Option<String>,
    tag: Option<String>,
}

impl OpFilter {
    pub fn new(
        methods: &[String],
        filter: Option<&str>,
        prefix: Option<&str>,
        tag: Option<&str>,
    ) -> Self {
        Self {
            methods: methods.iter().map(|s| s.to_ascii_lowercase()).collect(),
            contains: filter.map(str::to_string),
            prefix: prefix.map(str::to_string),
            tag: tag.map(str::to_string),
        }
    }

    pub fn accepts(&self, method: &str, path: &str, op: &Value) -> bool {
        if !self.methods.is_empty() && !self.methods.iter().any(|m| m == method) {
            return false;
        }
        if let Some(s) = &self.contains
            && !path.contains(s.as_str())
        {
            return false;
        }
        if let Some(p) = &self.prefix
            && !path.starts_with(p.as_str())
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
    contains: Option<String>,
    prefix: Option<String>,
}

impl PathFilter {
    pub fn new(filter: Option<&str>, prefix: Option<&str>) -> Self {
        Self {
            contains: filter.map(str::to_string),
            prefix: prefix.map(str::to_string),
        }
    }

    pub fn accepts(&self, path: &str) -> bool {
        if let Some(s) = &self.contains
            && !path.contains(s.as_str())
        {
            return false;
        }
        if let Some(p) = &self.prefix
            && !path.starts_with(p.as_str())
        {
            return false;
        }
        true
    }
}
