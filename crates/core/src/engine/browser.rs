use crate::schema::BrowserConfig;

use super::error::EngineResult;

pub fn launch_args(config: &BrowserConfig, url: Option<&str>) -> Vec<String> {
    let mut args = vec![
        format!("--window-size={},{}", config.width, config.height),
        format!("--window-position={},{}", config.x_position, config.y_position),
        "--force-device-scale-factor=1".to_string(),
    ];
    if config.incognito {
        args.push("--incognito".to_string());
    }
    if !config.had_audio {
        args.push("--mute-audio".to_string());
    }
    if let Some(flags) = &config.flags {
        args.extend(flags.iter().cloned());
    }
    if let Some(url) = url {
        args.push(url.to_string());
    }
    args
}

pub trait BrowserManager {
    fn navigate(&mut self, url: &str) -> EngineResult<()>;
    fn reload(&mut self) -> EngineResult<()>;
    fn go_back(&mut self) -> EngineResult<()>;
    fn go_forward(&mut self) -> EngineResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> BrowserConfig {
        BrowserConfig {
            incognito: false,
            had_audio: true,
            is_fullscreen: true,
            executable_path: None,
            width: 1280,
            height: 720,
            x_position: 0,
            y_position: 0,
            flags: None,
        }
    }

    #[test]
    fn enforces_deterministic_window_args() {
        let args = launch_args(&config(), Some("https://example.com"));
        assert!(args.contains(&"--window-size=1280,720".to_string()));
        assert!(args.contains(&"--window-position=0,0".to_string()));
        assert!(args.contains(&"--force-device-scale-factor=1".to_string()));
        assert_eq!(args.last().unwrap(), "https://example.com");
    }

    #[test]
    fn incognito_and_mute_flags_applied() {
        let mut cfg = config();
        cfg.incognito = true;
        cfg.had_audio = false;
        let args = launch_args(&cfg, None);
        assert!(args.contains(&"--incognito".to_string()));
        assert!(args.contains(&"--mute-audio".to_string()));
    }

    #[test]
    fn extra_flags_are_appended() {
        let mut cfg = config();
        cfg.flags = Some(vec!["--lang=en-US".to_string()]);
        let args = launch_args(&cfg, None);
        assert!(args.contains(&"--lang=en-US".to_string()));
    }
}
