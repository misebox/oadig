use serde_json::Value;

use crate::error::OadigError;

#[derive(Debug, Clone)]
enum Match {
    Exact(String),
    Contains(String),
    Prefix(String),
    Suffix(String),
}

impl Match {
    fn parse(value: &str) -> Self {
        let starts_glob = value.starts_with('*');
        let ends_glob = value.ends_with('*') && value.len() > 1;
        match (starts_glob, ends_glob) {
            (true, true) => Match::Contains(value[1..value.len() - 1].to_string()),
            (false, true) => Match::Prefix(value[..value.len() - 1].to_string()),
            (true, false) => Match::Suffix(value[1..].to_string()),
            (false, false) => Match::Exact(value.to_string()),
        }
    }

    fn matches(&self, s: &str) -> bool {
        match self {
            Match::Exact(v) => s == v,
            Match::Contains(v) => s.contains(v.as_str()),
            Match::Prefix(v) => s.starts_with(v.as_str()),
            Match::Suffix(v) => s.ends_with(v.as_str()),
        }
    }
}

#[derive(Debug, Clone)]
enum Predicate {
    Method(Vec<String>), // lowercase, OR within
    Path(Match),
    Tag(Vec<String>), // OR within
    OperationId(Match),
    Summary(Match),
    Description(Match),
    Deprecated(bool),
}

impl Predicate {
    fn parse(raw: &str) -> Result<Self, OadigError> {
        let (key, value) = raw
            .split_once('=')
            .ok_or_else(|| OadigError::Other(format!("filter must be key=value, got: {raw:?}")))?;
        let key = key.trim();
        let value = value.trim();
        match key {
            "method" => Ok(Predicate::Method(
                value
                    .split(',')
                    .map(|s| s.trim().to_ascii_lowercase())
                    .collect(),
            )),
            "tag" => Ok(Predicate::Tag(
                value.split(',').map(|s| s.trim().to_string()).collect(),
            )),
            "path" => Ok(Predicate::Path(Match::parse(value))),
            "operationId" => Ok(Predicate::OperationId(Match::parse(value))),
            "summary" => Ok(Predicate::Summary(Match::parse(value))),
            "description" => Ok(Predicate::Description(Match::parse(value))),
            "deprecated" => match value {
                "true" => Ok(Predicate::Deprecated(true)),
                "false" => Ok(Predicate::Deprecated(false)),
                other => Err(OadigError::Other(format!(
                    "deprecated expects true or false, got: {other:?}"
                ))),
            },
            other => Err(OadigError::Other(format!("unknown filter key: {other:?}"))),
        }
    }

    fn accepts_op(&self, method: &str, path: &str, op: &Value) -> bool {
        match self {
            Predicate::Method(ms) => ms.iter().any(|m| m == method),
            Predicate::Path(m) => m.matches(path),
            Predicate::Tag(tags) => op
                .get("tags")
                .and_then(Value::as_array)
                .map(|arr| {
                    arr.iter()
                        .filter_map(Value::as_str)
                        .any(|t| tags.iter().any(|wanted| wanted == t))
                })
                .unwrap_or(false),
            Predicate::OperationId(m) => op
                .get("operationId")
                .and_then(Value::as_str)
                .map(|s| m.matches(s))
                .unwrap_or(false),
            Predicate::Summary(m) => op
                .get("summary")
                .and_then(Value::as_str)
                .map(|s| m.matches(s))
                .unwrap_or(false),
            Predicate::Description(m) => op
                .get("description")
                .and_then(Value::as_str)
                .map(|s| m.matches(s))
                .unwrap_or(false),
            Predicate::Deprecated(want) => {
                op.get("deprecated")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                    == *want
            }
        }
    }

    fn accepts_path(&self, path: &str) -> Option<bool> {
        match self {
            Predicate::Path(m) => Some(m.matches(path)),
            // Other predicates reference operation-level fields; not applicable
            // to the bare `paths` listing.
            _ => None,
        }
    }
}

pub struct OpFilter {
    predicates: Vec<Predicate>,
}

impl OpFilter {
    pub fn from_strings(filters: &[String]) -> Result<Self, OadigError> {
        Ok(Self {
            predicates: filters
                .iter()
                .map(|s| Predicate::parse(s))
                .collect::<Result<_, _>>()?,
        })
    }

    pub fn accepts(&self, method: &str, path: &str, op: &Value) -> bool {
        self.predicates
            .iter()
            .all(|p| p.accepts_op(method, path, op))
    }
}

pub struct PathFilter {
    predicates: Vec<Predicate>,
}

impl PathFilter {
    pub fn from_strings(filters: &[String]) -> Result<Self, OadigError> {
        let predicates: Vec<_> = filters
            .iter()
            .map(|s| Predicate::parse(s))
            .collect::<Result<_, _>>()?;
        // Reject predicates that don't apply to bare paths.
        for p in &predicates {
            if p.accepts_path("").is_none() {
                return Err(OadigError::Other(
                    "`paths` only supports filters on `path`".to_string(),
                ));
            }
        }
        Ok(Self { predicates })
    }

    pub fn accepts(&self, path: &str) -> bool {
        self.predicates
            .iter()
            .all(|p| p.accepts_path(path).unwrap_or(true))
    }
}
