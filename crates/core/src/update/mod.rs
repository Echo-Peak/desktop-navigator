mod config;
mod install;
mod manifest;
mod verify;
mod version;

pub use config::{
    compiled_environment, compiled_worker_url, load, read_file, resolve, write_file, ConfigSources,
    Environment, UpdateConfig,
};
pub use install::{copy_dir, rewrite_shortcut, unzip_file};
pub use manifest::{
    artifact_url, checksum_url, current_artifact_key, manifest_url, Manifest,
};
pub use verify::{parse_checksum, sha256_file, sha256_hex, verify_bytes, verify_file};
pub use version::is_newer;

use std::fmt;

#[derive(Debug)]
pub enum UpdateError {
    Disabled,
    Network(String),
    Parse(String),
    NoArtifact(String),
    Checksum(String),
    Io(String),
    NotNewer,
}

impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UpdateError::Disabled => write!(f, "auto-update disabled (no worker url)"),
            UpdateError::Network(m) => write!(f, "network error: {m}"),
            UpdateError::Parse(m) => write!(f, "parse error: {m}"),
            UpdateError::NoArtifact(m) => write!(f, "no artifact for platform: {m}"),
            UpdateError::Checksum(m) => write!(f, "checksum mismatch: {m}"),
            UpdateError::Io(m) => write!(f, "io error: {m}"),
            UpdateError::NotNewer => write!(f, "already up to date"),
        }
    }
}

impl std::error::Error for UpdateError {}

impl From<std::io::Error> for UpdateError {
    fn from(e: std::io::Error) -> Self {
        UpdateError::Io(e.to_string())
    }
}

pub trait Fetcher {
    fn get_text(&self, url: &str) -> Result<String, UpdateError>;
    fn get_bytes(&self, url: &str) -> Result<Vec<u8>, UpdateError>;
}

pub fn select_target(manifest: &Manifest, current_version: &str) -> Option<String> {
    if is_newer(&manifest.version, current_version) {
        Some(manifest.version.clone())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn manifest(version: &str) -> Manifest {
        let mut artifacts = BTreeMap::new();
        artifacts.insert("linux-deb".to_string(), "app_1.deb".to_string());
        Manifest {
            version: version.to_string(),
            release_date: String::new(),
            artifacts,
        }
    }

    #[test]
    fn target_when_newer() {
        assert_eq!(
            select_target(&manifest("2.0.0"), "1.0.0"),
            Some("2.0.0".to_string())
        );
    }

    #[test]
    fn no_target_when_same() {
        assert_eq!(select_target(&manifest("1.0.0"), "1.0.0"), None);
    }
}
