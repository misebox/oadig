use std::collections::HashSet;

use serde_json::{json, Map, Value};

#[derive(Debug, Clone, Copy)]
pub struct ResolveOptions {
    pub resolve: bool,
    pub max_depth: Option<usize>,
}

impl Default for ResolveOptions {
    fn default() -> Self {
        Self { resolve: true, max_depth: None }
    }
}

pub struct Resolver<'a> {
    root: &'a Value,
    opts: ResolveOptions,
    stack: HashSet<String>,
}

impl<'a> Resolver<'a> {
    pub fn new(root: &'a Value, opts: ResolveOptions) -> Self {
        Self { root, opts, stack: HashSet::new() }
    }

    pub fn resolve(&mut self, value: Value, origin_ref: &str) -> Value {
        self.stack.insert(origin_ref.to_string());
        let out = self.walk(value, 0);
        self.stack.remove(origin_ref);
        out
    }

    fn walk(&mut self, value: Value, depth: usize) -> Value {
        if let Some(max) = self.opts.max_depth
            && depth > max
        {
            return json!({ "$truncated_depth": max });
        }
        match value {
            Value::Object(mut obj) => {
                if let Some(Value::String(ref_str)) = obj.get("$ref").cloned() {
                    return self.expand_ref(&ref_str, depth);
                }
                let mut out = Map::with_capacity(obj.len());
                for (k, v) in obj.iter_mut() {
                    let taken = std::mem::replace(v, Value::Null);
                    out.insert(k.clone(), self.walk(taken, depth + 1));
                }
                Value::Object(out)
            }
            Value::Array(arr) => Value::Array(
                arr.into_iter().map(|v| self.walk(v, depth + 1)).collect(),
            ),
            other => other,
        }
    }

    fn expand_ref(&mut self, ref_str: &str, depth: usize) -> Value {
        if self.stack.contains(ref_str) {
            return json!({ "$circular_ref": ref_str });
        }
        let pointer = match ref_str.strip_prefix('#') {
            Some(p) => p,
            None => return json!({ "$ref": ref_str }),
        };
        let Some(target) = self.root.pointer(pointer) else {
            return json!({ "$unresolved_ref": ref_str });
        };
        self.stack.insert(ref_str.to_string());
        let resolved = self.walk(target.clone(), depth + 1);
        self.stack.remove(ref_str);
        resolved
    }
}
