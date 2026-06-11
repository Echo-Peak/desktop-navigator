use std::fmt;

#[derive(Debug)]
pub enum EngineError {
    Validation(String),
    Secret(String),
    Browser(String),
    Bridge(String),
    Input(String),
    Timeout(String),
    Variable(String),
    Step { id: String, source: Box<EngineError> },
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineError::Validation(m) => write!(f, "validation error: {m}"),
            EngineError::Secret(m) => write!(f, "secret error: {m}"),
            EngineError::Browser(m) => write!(f, "browser error: {m}"),
            EngineError::Bridge(m) => write!(f, "dom bridge error: {m}"),
            EngineError::Input(m) => write!(f, "input error: {m}"),
            EngineError::Timeout(m) => write!(f, "timeout: {m}"),
            EngineError::Variable(m) => write!(f, "variable error: {m}"),
            EngineError::Step { id, source } => write!(f, "step '{id}' failed: {source}"),
        }
    }
}

impl std::error::Error for EngineError {}

pub type EngineResult<T> = Result<T, EngineError>;
