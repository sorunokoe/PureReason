use thiserror::Error;

#[derive(Debug, Error)]
pub enum TraceError {
    #[error("SQLite error: {0}")]
    Sqlite(String),
    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Mutex poisoned: {0}")]
    Poisoned(String),
}

// rusqlite doesn't implement std::error::Error in a way that derives From cleanly,
// so we map it manually.
impl From<rusqlite::Error> for TraceError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Sqlite(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, TraceError>;
