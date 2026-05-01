//! Error types for pure-reason-core.

use thiserror::Error;

/// The top-level error type for all PureReason operations.
#[derive(Debug, Error)]
pub enum PureReasonError {
    #[error("Aesthetic error: {0}")]
    Aesthetic(String),

    #[error("Analytic error: {0}")]
    Analytic(String),

    #[error("Dialectic error: {0}")]
    Dialectic(String),

    #[error("Methodology error: {0}")]
    Methodology(String),

    #[error("Wittgenstein error: {0}")]
    Wittgenstein(String),

    #[error("Pipeline error: {0}")]
    Pipeline(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Kantian validation failed: risk level {risk}")]
    HighRisk { risk: String, report_json: String },

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Convenience Result alias.
pub type Result<T> = std::result::Result<T, PureReasonError>;
