use keyring::Entry;
use navigator_core::engine::{EngineError, EngineResult, SecretResolver};

pub struct KeyringResolver {
    service: String,
}

impl KeyringResolver {
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
        }
    }

    pub fn store(&self, key: &str, value: &str) -> EngineResult<()> {
        let entry = Entry::new(&self.service, key)
            .map_err(|e| EngineError::Secret(format!("keyring open failed: {e}")))?;
        entry
            .set_password(value)
            .map_err(|e| EngineError::Secret(format!("keyring write failed: {e}")))
    }
}

impl SecretResolver for KeyringResolver {
    fn resolve(&self, key: &str) -> EngineResult<String> {
        let entry = Entry::new(&self.service, key)
            .map_err(|e| EngineError::Secret(format!("keyring open failed: {e}")))?;
        entry
            .get_password()
            .map_err(|e| EngineError::Secret(format!("missing secret '{key}': {e}")))
    }
}
