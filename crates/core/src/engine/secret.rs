use std::collections::HashMap;

use super::error::{EngineError, EngineResult};

pub const ENV_PREFIX: &str = "env:";

pub trait SecretResolver {
    fn resolve(&self, key: &str) -> EngineResult<String>;
}

pub fn is_secret_ref(value: &str) -> bool {
    value.starts_with(ENV_PREFIX)
}

pub fn resolve_value(value: &str, resolver: &dyn SecretResolver) -> EngineResult<String> {
    match value.strip_prefix(ENV_PREFIX) {
        Some(key) => resolver.resolve(key.trim()),
        None => Ok(value.to_string()),
    }
}

pub fn redact(value: &str, is_secret: bool) -> String {
    if is_secret || is_secret_ref(value) {
        "***".to_string()
    } else {
        value.to_string()
    }
}

#[derive(Debug, Default, Clone)]
pub struct InMemorySecrets {
    secrets: HashMap<String, String>,
}

impl InMemorySecrets {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.secrets.insert(key.into(), value.into());
        self
    }
}

impl SecretResolver for InMemorySecrets {
    fn resolve(&self, key: &str) -> EngineResult<String> {
        self.secrets
            .get(key)
            .cloned()
            .ok_or_else(|| EngineError::Secret(format!("missing secret '{key}'")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_env_prefixed_value() {
        let vault = InMemorySecrets::new().with("PW", "hunter2");
        assert_eq!(resolve_value("env:PW", &vault).unwrap(), "hunter2");
    }

    #[test]
    fn passes_through_literal_value() {
        let vault = InMemorySecrets::new();
        assert_eq!(resolve_value("plain", &vault).unwrap(), "plain");
    }

    #[test]
    fn missing_secret_errors() {
        let vault = InMemorySecrets::new();
        assert!(resolve_value("env:NOPE", &vault).is_err());
    }

    #[test]
    fn redacts_secret_values() {
        assert_eq!(redact("env:PW", false), "***");
        assert_eq!(redact("plain", true), "***");
        assert_eq!(redact("plain", false), "plain");
    }
}
