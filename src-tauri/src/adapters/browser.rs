use std::path::{Path, PathBuf};
use std::process::{Child, Command};

use navigator_core::engine::{launch_args, EngineError, EngineResult};
use navigator_core::schema::BrowserConfig;

pub const DEFAULT_DEBUG_PORT: u16 = 9222;

pub struct ChromeLauncher {
    extension_dir: PathBuf,
    profile_dir: PathBuf,
    debug_port: u16,
}

impl ChromeLauncher {
    pub fn new(extension_dir: impl Into<PathBuf>, profile_dir: impl Into<PathBuf>) -> Self {
        Self {
            extension_dir: extension_dir.into(),
            profile_dir: profile_dir.into(),
            debug_port: DEFAULT_DEBUG_PORT,
        }
    }

    pub fn launch(&self, config: &BrowserConfig, url: Option<&str>) -> EngineResult<Child> {
        let exe = resolve_executable(config.executable_path.as_deref())?;
        std::fs::create_dir_all(&self.profile_dir)
            .map_err(|e| EngineError::Browser(format!("profile dir: {e}")))?;

        let mut args: Vec<String> = vec![
            format!("--user-data-dir={}", self.profile_dir.display()),
            format!("--remote-debugging-port={}", self.debug_port),
            format!("--load-extension={}", self.extension_dir.display()),
            format!(
                "--disable-extensions-except={}",
                self.extension_dir.display()
            ),
            "--no-first-run".to_string(),
            "--no-default-browser-check".to_string(),
        ];
        args.extend(launch_args(config, url));

        Command::new(&exe)
            .args(&args)
            .spawn()
            .map_err(|e| EngineError::Browser(format!("failed to spawn {exe}: {e}")))
    }
}

fn resolve_executable(configured: Option<&str>) -> EngineResult<String> {
    if let Some(path) = configured {
        if Path::new(path).exists() {
            return Ok(path.to_string());
        }
    }
    for candidate in default_candidates() {
        if Path::new(candidate).exists() {
            return Ok(candidate.to_string());
        }
    }
    Err(EngineError::Browser(
        "no Chrome/Chromium executable found; set browserConfig.executablePath".into(),
    ))
}

#[cfg(target_os = "windows")]
fn default_candidates() -> Vec<&'static str> {
    vec![
        "C:\\Program Files\\Chromium\\Application\\chrome.exe",
        "C:\\Program Files\\Google\\Chrome for Testing\\chrome.exe",
    ]
}

#[cfg(target_os = "macos")]
fn default_candidates() -> Vec<&'static str> {
    vec![
        "/Applications/Chromium.app/Contents/MacOS/Chromium",
        "/Applications/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing",
    ]
}

#[cfg(all(unix, not(target_os = "macos")))]
fn default_candidates() -> Vec<&'static str> {
    vec![
        "/usr/bin/chromium",
        "/usr/bin/chromium-browser",
        "/snap/bin/chromium",
    ]
}
