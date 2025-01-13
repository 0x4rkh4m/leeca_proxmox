use std::backtrace::Backtrace;
use thiserror::Error;

/// The main error type for ProxmoxVE operations.
///
/// This enum represents all possible errors that can occur during
/// ProxmoxVE operations, including connection, authentication,
/// validation, and concurrent operation failures.
///
/// # Examples
///
/// ```
/// use leeca_proxmox::core::domain::error::ProxmoxError;
///
/// fn example_operation() -> Result<(), ProxmoxError> {
///     Err(ProxmoxError::ConnectionError("Failed to connect".to_string()))
/// }
/// ```
#[derive(Error, Debug)]
pub enum ProxmoxError {
    /// Represents errors that occur during connection attempts
    ///
    /// # Fields
    /// * `0` - A description of what went wrong during the connection attempt
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// Represents authentication failures
    ///
    /// # Fields
    /// * `0` - A description of the authentication failure
    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    /// Represents validation failures with detailed context
    ///
    /// # Fields
    /// * `source` - The underlying validation error
    /// * `backtrace` - Stack trace where the error occurred
    #[error("Validation error: {source}")]
    ValidationError {
        source: ValidationError,
        #[backtrace]
        backtrace: Backtrace,
    },
}

/// Specialized error type for validation failures.
///
/// This enum provides detailed context about why a validation
/// failed, including field-specific errors and format violations.
#[derive(Error, Debug)]
pub enum ValidationError {
    /// Represents a validation failure for a specific field
    ///
    /// # Fields
    /// * `field` - The name of the field that failed validation
    /// * `message` - A detailed message about why validation failed
    #[error("Field '{field}' validation failed: {message}")]
    FieldError { field: String, message: String },

    /// Represents format/syntax validation failures
    ///
    /// # Fields
    /// * `0` - Description of the format violation
    #[error("Format error: {0}")]
    FormatError(String),

    /// Represents violations of domain constraints
    ///
    /// # Fields
    /// * `0` - Description of the constraint violation
    #[error("Domain constraint violation: {0}")]
    ConstraintViolation(String),
}

/// Type alias for Results that may fail with a ProxmoxError
pub type ProxmoxResult<T> = Result<T, ProxmoxError>;
