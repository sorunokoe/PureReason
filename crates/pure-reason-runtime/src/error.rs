use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("SQLite error: {0}")]
    Sqlite(String),
    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("State configuration error: {0}")]
    Configuration(String),
    #[error("Mutex poisoned: {0}")]
    Poisoned(String),
    #[error("Task not found: {0}")]
    TaskNotFound(String),
    #[error("Invalid state transition from {from} to {to}")]
    InvalidTransition { from: String, to: String },
    #[error("Trace error: {0}")]
    Trace(#[from] pure_reason_trace::TraceError),
}

impl From<rusqlite::Error> for RuntimeError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Sqlite(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, RuntimeError>;
