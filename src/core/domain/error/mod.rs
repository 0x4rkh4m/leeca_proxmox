use std::backtrace::Backtrace;
use thiserror::Error;

/// Result type alias for Proxmox operations.
pub type ProxmoxResult<T> = Result<T, ProxmoxError>;

/// Enumeration of possible errors.
#[derive(Debug, Error)]
pub enum ProxmoxError {
    /// Authentication error (e.g., invalid credentials, ticket expired)
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// Connection error (network, TLS, etc.)
    #[error("Connection error: {0}")]
    Connection(String),

    /// Validation error with backtrace.
    #[error("Validation error")]
    Validation {
        source: ValidationError,
        backtrace: Backtrace,
    },

    /// Session persistence error (I/O, serialization, etc.)
    #[error("Session error: {0}")]
    Session(String),

    /// Other unexpected errors.
    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

/// Validation-specific errors.
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Field '{field}' is invalid: {message}")]
    Field { field: String, message: String },

    #[error("Format error: {0}")]
    Format(String),

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
}

impl From<std::io::Error> for ProxmoxError {
    fn from(err: std::io::Error) -> Self {
        ProxmoxError::Session(err.to_string())
    }
}

impl From<serde_json::Error> for ProxmoxError {
    fn from(err: serde_json::Error) -> Self {
        ProxmoxError::Session(err.to_string())
    }
}
