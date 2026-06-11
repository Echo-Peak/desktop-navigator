use serde_json::{Map, Value};

use super::error::{EngineError, EngineResult};

#[derive(Debug, Default, Clone)]
pub struct StateMap {
    vars: Map<String, Value>,
}

impl StateMap {
    pub fn new() -> Self {
        Self { vars: Map::new() }
    }

    pub fn set(&mut self, key: impl Into<String>, value: Value) {
        self.vars.insert(key.into(), value);
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.vars.get(key)
    }

    pub fn clear(&mut self) {
        self.vars.clear();
    }

    pub fn render_template(&self, template: &str) -> String {
        render_template(template, &self.vars)
    }

    pub fn aggregate(
        &self,
        input_variable: &str,
        template: &str,
        join_with: &str,
    ) -> EngineResult<String> {
        let value = self
            .vars
            .get(input_variable)
            .ok_or_else(|| EngineError::Variable(format!("unknown variable '{input_variable}'")))?;
        let arr = value.as_array().ok_or_else(|| {
            EngineError::Variable(format!("variable '{input_variable}' is not an array"))
        })?;
        let mut parts = Vec::with_capacity(arr.len());
        for item in arr {
            match item.as_object() {
                Some(obj) => parts.push(render_template(template, obj)),
                None => {
                    let mut single = Map::new();
                    single.insert("value".to_string(), item.clone());
                    parts.push(render_template(template, &single));
                }
            }
        }
        Ok(parts.join(join_with))
    }
}

pub fn render_template(template: &str, lookup: &Map<String, Value>) -> String {
    let mut out = String::with_capacity(template.len());
    let bytes = template.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'{' && bytes[i + 1] == b'{' {
            if let Some(close) = template[i + 2..].find("}}") {
                let key = template[i + 2..i + 2 + close].trim();
                match lookup.get(key) {
                    Some(v) => out.push_str(&value_to_string(v)),
                    None => out.push_str(&template[i..i + 2 + close + 2]),
                }
                i = i + 2 + close + 2;
                continue;
            }
        }
        let ch = template[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

pub fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn render_replaces_known_keys() {
        let mut state = StateMap::new();
        state.set("name", json!("Ada"));
        assert_eq!(state.render_template("Hi {{name}}!"), "Hi Ada!");
    }

    #[test]
    fn render_keeps_unknown_keys_verbatim() {
        let state = StateMap::new();
        assert_eq!(state.render_template("Hi {{name}}!"), "Hi {{name}}!");
    }

    #[test]
    fn render_trims_whitespace_in_token() {
        let mut state = StateMap::new();
        state.set("x", json!(7));
        assert_eq!(state.render_template("v={{  x  }}"), "v=7");
    }

    #[test]
    fn aggregate_renders_each_item_and_joins() {
        let mut state = StateMap::new();
        state.set(
            "jobs",
            json!([
                {"title": "A", "link": "http://a"},
                {"title": "B", "link": "http://b"}
            ]),
        );
        let out = state
            .aggregate("jobs", "* {{title}} {{link}}", "\n")
            .unwrap();
        assert_eq!(out, "* A http://a\n* B http://b");
    }

    #[test]
    fn aggregate_errors_on_non_array() {
        let mut state = StateMap::new();
        state.set("x", json!("nope"));
        assert!(state.aggregate("x", "{{value}}", ",").is_err());
    }
}
