use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Production,
    Development,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Production => "production",
            Environment::Development => "development",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "production" | "prod" => Some(Environment::Production),
            "development" | "dev" => Some(Environment::Development),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateConfig {
    pub worker_url: String,
    pub environment: Environment,
}

impl UpdateConfig {
    pub fn is_enabled(&self) -> bool {
        !self.worker_url.trim().is_empty()
    }
}

pub fn compiled_worker_url() -> String {
    option_env!("DN_UPDATE_WORKER_URL").unwrap_or("").to_string()
}

pub fn compiled_environment() -> Environment {
    match option_env!("DN_UPDATE_ENV") {
        Some(v) => Environment::parse(v).unwrap_or_else(default_environment),
        None => default_environment(),
    }
}

fn default_environment() -> Environment {
    if cfg!(debug_assertions) {
        Environment::Development
    } else {
        Environment::Production
    }
}

pub struct ConfigSources {
    pub env_worker_url: Option<String>,
    pub env_environment: Option<String>,
    pub file: Option<UpdateConfig>,
    pub compiled_worker_url: String,
    pub compiled_environment: Environment,
}

pub fn resolve(sources: &ConfigSources) -> UpdateConfig {
    let worker_url = sources
        .env_worker_url
        .clone()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| sources.file.as_ref().map(|f| f.worker_url.clone()))
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| sources.compiled_worker_url.clone());

    let environment = sources
        .env_environment
        .as_deref()
        .and_then(Environment::parse)
        .or_else(|| sources.file.as_ref().map(|f| f.environment))
        .unwrap_or(sources.compiled_environment);

    UpdateConfig {
        worker_url,
        environment,
    }
}

pub fn read_file(path: &Path) -> Option<UpdateConfig> {
    let raw = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

pub fn write_file(path: &Path, config: &UpdateConfig) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    std::fs::write(path, json)
}

pub fn load(config_path: &Path) -> UpdateConfig {
    if read_file(config_path).is_none() {
        let defaults = UpdateConfig {
            worker_url: compiled_worker_url(),
            environment: compiled_environment(),
        };
        let _ = write_file(config_path, &defaults);
    }

    let sources = ConfigSources {
        env_worker_url: std::env::var("DN_UPDATE_WORKER_URL").ok(),
        env_environment: std::env::var("DN_UPDATE_ENV").ok(),
        file: read_file(config_path),
        compiled_worker_url: compiled_worker_url(),
        compiled_environment: compiled_environment(),
    };
    resolve(&sources)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sources() -> ConfigSources {
        ConfigSources {
            env_worker_url: None,
            env_environment: None,
            file: None,
            compiled_worker_url: "https://compiled.example".into(),
            compiled_environment: Environment::Production,
        }
    }

    #[test]
    fn env_overrides_file_and_compiled() {
        let mut s = sources();
        s.file = Some(UpdateConfig {
            worker_url: "https://file.example".into(),
            environment: Environment::Development,
        });
        s.env_worker_url = Some("https://env.example".into());
        s.env_environment = Some("development".into());
        let cfg = resolve(&s);
        assert_eq!(cfg.worker_url, "https://env.example");
        assert_eq!(cfg.environment, Environment::Development);
    }

    #[test]
    fn file_overrides_compiled() {
        let mut s = sources();
        s.file = Some(UpdateConfig {
            worker_url: "https://file.example".into(),
            environment: Environment::Development,
        });
        let cfg = resolve(&s);
        assert_eq!(cfg.worker_url, "https://file.example");
        assert_eq!(cfg.environment, Environment::Development);
    }

    #[test]
    fn falls_back_to_compiled() {
        let cfg = resolve(&sources());
        assert_eq!(cfg.worker_url, "https://compiled.example");
        assert_eq!(cfg.environment, Environment::Production);
    }

    #[test]
    fn blank_env_ignored() {
        let mut s = sources();
        s.env_worker_url = Some("   ".into());
        let cfg = resolve(&s);
        assert_eq!(cfg.worker_url, "https://compiled.example");
    }

    #[test]
    fn round_trip_file() {
        let dir = std::env::temp_dir().join(format!("dn-cfg-{}", crate::data::now_ms()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("update.json");
        let cfg = UpdateConfig {
            worker_url: "https://w.example".into(),
            environment: Environment::Development,
        };
        write_file(&path, &cfg).unwrap();
        assert_eq!(read_file(&path), Some(cfg));
        std::fs::remove_dir_all(&dir).ok();
    }
}
