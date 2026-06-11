use std::path::{Path, PathBuf};
use std::process::Command;

use navigator_core::data::now_ms;
use navigator_core::log::FileLogger;
use navigator_core::update::{
    self, artifact_url, checksum_url, current_artifact_key, manifest_url, select_target,
    Fetcher, Manifest, UpdateConfig, UpdateError,
};
use navigator_core::AppPaths;
use serde_json::json;
use tauri::{AppHandle, Emitter};

pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

const APP_BIN: &str = if cfg!(windows) {
    "desktop-navigator.exe"
} else {
    "desktop-navigator"
};

pub struct ReqwestFetcher {
    client: reqwest::blocking::Client,
}

impl ReqwestFetcher {
    pub fn new() -> Result<Self, UpdateError> {
        let client = reqwest::blocking::Client::builder()
            .build()
            .map_err(|e| UpdateError::Network(e.to_string()))?;
        Ok(Self { client })
    }
}

impl Fetcher for ReqwestFetcher {
    fn get_text(&self, url: &str) -> Result<String, UpdateError> {
        self.client
            .get(url)
            .send()
            .and_then(|r| r.error_for_status())
            .and_then(|r| r.text())
            .map_err(|e| UpdateError::Network(e.to_string()))
    }

    fn get_bytes(&self, url: &str) -> Result<Vec<u8>, UpdateError> {
        self.client
            .get(url)
            .send()
            .and_then(|r| r.error_for_status())
            .and_then(|r| r.bytes())
            .map(|b| b.to_vec())
            .map_err(|e| UpdateError::Network(e.to_string()))
    }
}

fn config_path(paths: &AppPaths) -> PathBuf {
    paths.install_dir().join("update.json")
}

fn logger(paths: &AppPaths) -> FileLogger {
    FileLogger::new(paths.log_file(now_ms(), CURRENT_VERSION))
}

fn fetch_manifest(fetcher: &ReqwestFetcher, cfg: &UpdateConfig) -> Result<Manifest, UpdateError> {
    let url = manifest_url(&cfg.worker_url, cfg.environment);
    let text = fetcher.get_text(&url)?;
    serde_json::from_str(&text).map_err(|e| UpdateError::Parse(e.to_string()))
}

pub fn parse_update_flag(args: &[String]) -> Option<String> {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "--update" {
            return iter.next().cloned();
        }
        if let Some(rest) = arg.strip_prefix("--update=") {
            return Some(rest.to_string());
        }
    }
    None
}

pub fn run_helper(version: &str) -> Result<String, UpdateError> {
    let paths = AppPaths::resolve().map_err(|e| UpdateError::Io(e.to_string()))?;
    let _ = paths.ensure_base_dirs();
    let log = logger(&paths);
    log.info(&format!("helper started for v{version}"));

    let cfg = update::load(&config_path(&paths));
    if !cfg.is_enabled() {
        log.error("helper aborted: update disabled (no worker url)");
        return Err(UpdateError::Disabled);
    }

    let fetcher = ReqwestFetcher::new()?;
    let manifest = fetch_manifest(&fetcher, &cfg)?;
    let key = current_artifact_key();
    let artifact_name = manifest
        .artifact_for(key)
        .ok_or_else(|| UpdateError::NoArtifact(key.to_string()))?
        .to_string();
    log.info(&format!("artifact for {key}: {artifact_name}"));

    let checksum = fetcher.get_text(&checksum_url(&cfg.worker_url, cfg.environment, version))?;
    log.info("download start");
    let bytes = fetcher.get_bytes(&artifact_url(
        &cfg.worker_url,
        cfg.environment,
        version,
        &artifact_name,
    ))?;
    log.info(&format!("download end ({} bytes)", bytes.len()));

    let updates_dir = paths.updates_dir();
    let staging = updates_dir.join(format!("v{version}"));
    std::fs::create_dir_all(&updates_dir).map_err(|e| UpdateError::Io(e.to_string()))?;
    let zip_path = updates_dir.join(&artifact_name);
    std::fs::write(&zip_path, &bytes).map_err(|e| UpdateError::Io(e.to_string()))?;

    if !update::verify_file(&zip_path, &checksum).map_err(|e| UpdateError::Io(e.to_string()))? {
        let _ = std::fs::remove_file(&zip_path);
        log.error("checksum mismatch: artifact deleted, aborting");
        return Err(UpdateError::Checksum(artifact_name));
    }
    log.info("checksum pass");

    update::unzip_file(&zip_path, &staging).map_err(|e| UpdateError::Io(e.to_string()))?;
    let install_version_dir = paths.version_dir(version);
    update::copy_dir(&staging, &install_version_dir)
        .map_err(|e| UpdateError::Io(e.to_string()))?;
    log.info(&format!(
        "installed to {}",
        install_version_dir.display()
    ));

    let shortcut = paths.install_dir().join("shortcutLauncher");
    let target = install_version_dir.join(APP_BIN);
    update::rewrite_shortcut(&shortcut, &target).map_err(|e| UpdateError::Io(e.to_string()))?;
    log.info(&format!("shortcut -> {}", target.display()));
    log.info(&format!("install success v{version}"));

    Ok(version.to_string())
}

fn copy_self_to_temp() -> Result<PathBuf, UpdateError> {
    let exe = std::env::current_exe().map_err(|e| UpdateError::Io(e.to_string()))?;
    let dir = std::env::temp_dir().join(format!("dn-update-helper-{}", now_ms()));
    std::fs::create_dir_all(&dir).map_err(|e| UpdateError::Io(e.to_string()))?;
    let dest = dir.join(exe.file_name().unwrap_or_else(|| std::ffi::OsStr::new(APP_BIN)));
    std::fs::copy(&exe, &dest).map_err(|e| UpdateError::Io(e.to_string()))?;
    Ok(dest)
}

fn spawn_helper(helper: &Path, version: &str) -> Result<(), UpdateError> {
    let output = Command::new(helper)
        .arg("--update")
        .arg(version)
        .output()
        .map_err(|e| UpdateError::Io(e.to_string()))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(UpdateError::Io(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ))
    }
}

pub fn check_and_update(app: &AppHandle) {
    let paths = match AppPaths::resolve() {
        Ok(p) => p,
        Err(_) => return,
    };
    let _ = paths.ensure_base_dirs();
    let log = logger(&paths);
    log.info("update check triggered");

    let cfg = update::load(&config_path(&paths));
    if !cfg.is_enabled() {
        log.info("update check skipped: no worker url configured");
        return;
    }

    let fetcher = match ReqwestFetcher::new() {
        Ok(f) => f,
        Err(e) => {
            log.error(&format!("fetcher init failed: {e}"));
            return;
        }
    };

    let manifest = match fetch_manifest(&fetcher, &cfg) {
        Ok(m) => m,
        Err(e) => {
            log.error(&format!("manifest fetch failed: {e}"));
            return;
        }
    };

    match select_target(&manifest, CURRENT_VERSION) {
        None => log.info(&format!(
            "up to date (current {CURRENT_VERSION}, latest {})",
            manifest.version
        )),
        Some(version) => {
            log.info(&format!("newer version found: v{version}"));
            let helper = match copy_self_to_temp() {
                Ok(h) => h,
                Err(e) => {
                    log.error(&format!("helper copy failed: {e}"));
                    let _ = app.emit("update:error", json!({ "message": e.to_string() }));
                    return;
                }
            };
            match spawn_helper(&helper, &version) {
                Ok(()) => {
                    log.info(&format!("helper success v{version}"));
                    let _ = app.emit("update:ready", json!({ "version": version }));
                }
                Err(e) => {
                    log.error(&format!("helper failed: {e}"));
                    let _ = app.emit("update:error", json!({ "message": e.to_string() }));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::parse_update_flag;

    #[test]
    fn parses_space_form() {
        let args = vec!["bin".into(), "--update".into(), "1.2.3".into()];
        assert_eq!(parse_update_flag(&args), Some("1.2.3".to_string()));
    }

    #[test]
    fn parses_eq_form() {
        let args = vec!["bin".into(), "--update=1.2.3".into()];
        assert_eq!(parse_update_flag(&args), Some("1.2.3".to_string()));
    }

    #[test]
    fn none_when_absent() {
        let args = vec!["bin".into()];
        assert_eq!(parse_update_flag(&args), None);
    }
}
