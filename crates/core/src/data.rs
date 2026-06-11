use std::fmt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const APP_NAME: &str = "BrowserNavigator";

#[derive(Debug)]
pub enum DataError {
    NoDataDir,
    Io(std::io::Error),
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::NoDataDir => write!(f, "could not resolve a platform data directory"),
            DataError::Io(e) => write!(f, "io error: {e}"),
        }
    }
}

impl std::error::Error for DataError {}

impl From<std::io::Error> for DataError {
    fn from(e: std::io::Error) -> Self {
        DataError::Io(e)
    }
}

#[derive(Debug, Clone)]
pub struct AppPaths {
    install_dir: PathBuf,
}

impl AppPaths {
    pub fn with_install_dir(install_dir: impl Into<PathBuf>) -> Self {
        Self {
            install_dir: install_dir.into(),
        }
    }

    pub fn resolve() -> Result<Self, DataError> {
        let base = dirs::data_local_dir().ok_or(DataError::NoDataDir)?;
        Ok(Self::with_install_dir(base.join(APP_NAME)))
    }

    pub fn install_dir(&self) -> &Path {
        &self.install_dir
    }

    pub fn updates_dir(&self) -> PathBuf {
        let name = format!("{APP_NAME}Updates");
        match self.install_dir.parent() {
            Some(parent) => parent.join(name),
            None => PathBuf::from(name),
        }
    }

    pub fn logs_dir(&self) -> PathBuf {
        self.install_dir.join("logs")
    }

    pub fn pages_dir(&self) -> PathBuf {
        self.install_dir.join("UserData").join("Pages")
    }

    pub fn version_dir(&self, version: &str) -> PathBuf {
        self.install_dir.join(format!("v{version}"))
    }

    pub fn captcha_resolvers_dir(&self, app_version: &str) -> PathBuf {
        self.install_dir.join(app_version).join("captcha-resolvers")
    }

    pub fn log_file(&self, timestamp_ms: u128, version: &str) -> PathBuf {
        self.logs_dir().join(format!("{timestamp_ms}-{version}.log"))
    }

    pub fn ensure_base_dirs(&self) -> Result<(), DataError> {
        std::fs::create_dir_all(self.logs_dir())?;
        std::fs::create_dir_all(self.pages_dir())?;
        Ok(())
    }
}

pub fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths() -> AppPaths {
        AppPaths::with_install_dir("/base/BrowserNavigator")
    }

    #[test]
    fn install_dir_is_preserved() {
        assert_eq!(
            paths().install_dir(),
            Path::new("/base/BrowserNavigator")
        );
    }

    #[test]
    fn updates_dir_is_sibling() {
        assert_eq!(
            paths().updates_dir(),
            Path::new("/base/BrowserNavigatorUpdates")
        );
    }

    #[test]
    fn logs_dir_under_install() {
        assert_eq!(
            paths().logs_dir(),
            Path::new("/base/BrowserNavigator/logs")
        );
    }

    #[test]
    fn pages_dir_under_userdata() {
        assert_eq!(
            paths().pages_dir(),
            Path::new("/base/BrowserNavigator/UserData/Pages")
        );
    }

    #[test]
    fn version_dir_has_v_prefix() {
        assert_eq!(
            paths().version_dir("1.0.0"),
            Path::new("/base/BrowserNavigator/v1.0.0")
        );
    }

    #[test]
    fn captcha_dir_under_version() {
        assert_eq!(
            paths().captcha_resolvers_dir("1.0.0"),
            Path::new("/base/BrowserNavigator/1.0.0/captcha-resolvers")
        );
    }

    #[test]
    fn log_file_name_format() {
        assert_eq!(
            paths().log_file(1718000000000, "1.0.0"),
            Path::new("/base/BrowserNavigator/logs/1718000000000-1.0.0.log")
        );
    }

    #[test]
    fn ensure_base_dirs_creates_logs_and_pages() {
        let dir = std::env::temp_dir().join(format!("dn-test-{}", now_ms()));
        let p = AppPaths::with_install_dir(&dir);
        p.ensure_base_dirs().unwrap();
        assert!(p.logs_dir().is_dir());
        assert!(p.pages_dir().is_dir());
        std::fs::remove_dir_all(&dir).ok();
    }
}
