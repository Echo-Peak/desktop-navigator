use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::config::Environment;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub version: String,
    #[serde(default)]
    pub release_date: String,
    pub artifacts: BTreeMap<String, String>,
}

impl Manifest {
    pub fn artifact_for(&self, key: &str) -> Option<&str> {
        self.artifacts.get(key).map(|s| s.as_str())
    }
}

pub fn current_artifact_key() -> &'static str {
    if cfg!(target_os = "windows") {
        "win-nsis"
    } else if cfg!(target_os = "macos") {
        "mac-pkg"
    } else {
        "linux-deb"
    }
}

pub fn manifest_url(worker_url: &str, env: Environment) -> String {
    format!("{worker_url}?artifact=manifest.json&env={}", env.as_str())
}

pub fn checksum_url(worker_url: &str, env: Environment, version: &str) -> String {
    format!(
        "{worker_url}?artifact=sha256Checksum.txt&env={}&version={version}",
        env.as_str()
    )
}

pub fn artifact_url(
    worker_url: &str,
    env: Environment,
    version: &str,
    artifact_name: &str,
) -> String {
    format!(
        "{worker_url}?artifact={artifact_name}&env={}&version={version}",
        env.as_str()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_manifest() {
        let json = r##"{
            "version": "1.2.3",
            "releaseDate": "2026-06-11T00:00:00Z",
            "artifacts": {
                "linux-deb": "browser-navigator_1.2.3_amd64.deb",
                "win-nsis": "BrowserNavigator-1.2.3-setup.exe"
            }
        }"##;
        let m: Manifest = serde_json::from_str(json).unwrap();
        assert_eq!(m.version, "1.2.3");
        assert_eq!(
            m.artifact_for("linux-deb"),
            Some("browser-navigator_1.2.3_amd64.deb")
        );
        assert_eq!(m.artifact_for("mac-pkg"), None);
    }

    #[test]
    fn builds_urls() {
        let w = "https://worker.example";
        assert_eq!(
            manifest_url(w, Environment::Production),
            "https://worker.example?artifact=manifest.json&env=production"
        );
        assert_eq!(
            checksum_url(w, Environment::Development, "1.2.3"),
            "https://worker.example?artifact=sha256Checksum.txt&env=development&version=1.2.3"
        );
        assert_eq!(
            artifact_url(w, Environment::Production, "1.2.3", "app.zip"),
            "https://worker.example?artifact=app.zip&env=production&version=1.2.3"
        );
    }

    #[test]
    fn artifact_key_is_known() {
        let key = current_artifact_key();
        assert!(["win-nsis", "mac-pkg", "linux-deb"].contains(&key));
    }
}
