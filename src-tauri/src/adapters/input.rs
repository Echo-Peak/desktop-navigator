use enigo::{
    Axis, Button, Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Settings,
};
use navigator_core::engine::{EngineError, EngineResult, InputSink};

pub struct EnigoSink {
    enigo: Enigo,
}

impl EnigoSink {
    pub fn new() -> EngineResult<Self> {
        let enigo = Enigo::new(&Settings::default())
            .map_err(|e| EngineError::Input(format!("enigo init failed: {e}")))?;
        Ok(Self { enigo })
    }
}

fn map_input_err(e: impl std::fmt::Display) -> EngineError {
    EngineError::Input(e.to_string())
}

fn parse_key(token: &str) -> Key {
    match token {
        "Enter" | "Return" => Key::Return,
        "Tab" => Key::Tab,
        "Escape" | "Esc" => Key::Escape,
        "Backspace" => Key::Backspace,
        "Delete" => Key::Delete,
        "Space" => Key::Space,
        "ArrowUp" | "Up" => Key::UpArrow,
        "ArrowDown" | "Down" => Key::DownArrow,
        "ArrowLeft" | "Left" => Key::LeftArrow,
        "ArrowRight" | "Right" => Key::RightArrow,
        "Control" | "Ctrl" => Key::Control,
        "Shift" => Key::Shift,
        "Alt" | "Option" => Key::Alt,
        "Meta" | "Cmd" | "Super" | "Win" => Key::Meta,
        "F1" => Key::F1,
        "F2" => Key::F2,
        "F3" => Key::F3,
        "F4" => Key::F4,
        "F5" => Key::F5,
        "F6" => Key::F6,
        "F7" => Key::F7,
        "F8" => Key::F8,
        "F9" => Key::F9,
        "F10" => Key::F10,
        "F11" => Key::F11,
        "F12" => Key::F12,
        other => {
            let ch = other.chars().next().unwrap_or(' ');
            Key::Unicode(ch)
        }
    }
}

fn is_modifier(token: &str) -> bool {
    matches!(
        token,
        "Control" | "Ctrl" | "Shift" | "Alt" | "Option" | "Meta" | "Cmd" | "Super" | "Win"
    )
}

impl InputSink for EnigoSink {
    fn move_to(&mut self, x: i32, y: i32) -> EngineResult<()> {
        self.enigo
            .move_mouse(x, y, Coordinate::Abs)
            .map_err(map_input_err)
    }

    fn left_click(&mut self) -> EngineResult<()> {
        self.enigo
            .button(Button::Left, Direction::Click)
            .map_err(map_input_err)
    }

    fn double_click(&mut self) -> EngineResult<()> {
        self.enigo
            .button(Button::Left, Direction::Click)
            .map_err(map_input_err)?;
        self.enigo
            .button(Button::Left, Direction::Click)
            .map_err(map_input_err)
    }

    fn right_click(&mut self) -> EngineResult<()> {
        self.enigo
            .button(Button::Right, Direction::Click)
            .map_err(map_input_err)
    }

    fn scroll(&mut self, delta_x: i32, delta_y: i32) -> EngineResult<()> {
        if delta_x != 0 {
            self.enigo
                .scroll(delta_x, Axis::Horizontal)
                .map_err(map_input_err)?;
        }
        if delta_y != 0 {
            self.enigo
                .scroll(delta_y, Axis::Vertical)
                .map_err(map_input_err)?;
        }
        Ok(())
    }

    fn key_combo(&mut self, keys: &[String]) -> EngineResult<()> {
        let modifiers: Vec<&String> = keys.iter().filter(|k| is_modifier(k)).collect();
        let mains: Vec<&String> = keys.iter().filter(|k| !is_modifier(k)).collect();

        for m in &modifiers {
            self.enigo
                .key(parse_key(m), Direction::Press)
                .map_err(map_input_err)?;
        }
        let result = (|| {
            if mains.is_empty() {
                for m in &modifiers {
                    self.enigo
                        .key(parse_key(m), Direction::Click)
                        .map_err(map_input_err)?;
                }
            } else {
                for k in &mains {
                    self.enigo
                        .key(parse_key(k), Direction::Click)
                        .map_err(map_input_err)?;
                }
            }
            Ok(())
        })();
        for m in modifiers.iter().rev() {
            let _ = self.enigo.key(parse_key(m), Direction::Release);
        }
        result
    }

    fn type_text(&mut self, text: &str) -> EngineResult<()> {
        self.enigo.text(text).map_err(map_input_err)
    }

    fn sleep_ms(&mut self, ms: u64) {
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }
}
