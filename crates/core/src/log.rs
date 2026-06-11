use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::data::now_ms;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

impl LogLevel {
    fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        }
    }
}

pub struct FileLogger {
    path: PathBuf,
    lock: Mutex<()>,
}

impl FileLogger {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            lock: Mutex::new(()),
        }
    }

    pub fn log(&self, level: LogLevel, msg: &str) {
        let line = format!("{} {} {}\n", iso8601_ms(now_ms()), level.as_str(), msg);
        let _guard = self.lock.lock();
        if let Some(parent) = self.path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&self.path) {
            let _ = f.write_all(line.as_bytes());
        }
    }

    pub fn info(&self, msg: &str) {
        self.log(LogLevel::Info, msg);
    }

    pub fn warn(&self, msg: &str) {
        self.log(LogLevel::Warn, msg);
    }

    pub fn error(&self, msg: &str) {
        self.log(LogLevel::Error, msg);
    }
}

pub fn iso8601_ms(ms: u128) -> String {
    let secs = (ms / 1000) as i64;
    let millis = (ms % 1000) as u64;
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (hour, minute, second) = (rem / 3600, (rem % 3600) / 60, rem % 60);

    let z = days + 719_468;
    let era = (if z >= 0 { z } else { z - 146_096 }) / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { year + 1 } else { year };

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        year, month, day, hour, minute, second, millis
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::AppPaths;

    #[test]
    fn iso_epoch() {
        assert_eq!(iso8601_ms(0), "1970-01-01T00:00:00.000Z");
    }

    #[test]
    fn iso_known_value() {
        assert_eq!(iso8601_ms(1_718_000_000_000), "2024-06-10T06:13:20.000Z");
    }

    #[test]
    fn writes_lines() {
        let dir = std::env::temp_dir().join(format!("dn-log-{}", now_ms()));
        let paths = AppPaths::with_install_dir(&dir);
        let file = paths.log_file(now_ms(), "1.0.0");
        let logger = FileLogger::new(&file);
        logger.info("check triggered");
        logger.error("checksum mismatch");
        let contents = std::fs::read_to_string(&file).unwrap();
        assert!(contents.contains("INFO check triggered"));
        assert!(contents.contains("ERROR checksum mismatch"));
        assert_eq!(contents.lines().count(), 2);
        std::fs::remove_dir_all(&dir).ok();
    }
}
