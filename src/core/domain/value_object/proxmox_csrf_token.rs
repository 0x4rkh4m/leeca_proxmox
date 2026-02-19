use crate::core::domain::error::ValidationError;
use std::time::{Duration, SystemTime};

/// A Proxmox CSRF protection token.
#[derive(Debug, Clone)]
pub struct ProxmoxCSRFToken {
    value: String,
    created_at: SystemTime,
}

impl ProxmoxCSRFToken {
    /// Creates a new CSRF token without validation.
    pub(crate) fn new_unchecked(value: String) -> Self {
        Self {
            value,
            created_at: SystemTime::now(),
        }
    }

    /// Returns the token value as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Returns the creation time.
    #[must_use]
    pub fn created_at(&self) -> SystemTime {
        self.created_at
    }

    /// Checks if the token is expired based on a given lifetime.
    #[must_use]
    pub fn is_expired(&self, lifetime: Duration) -> bool {
        self.created_at
            .elapsed()
            .map(|age| age > lifetime)
            .unwrap_or(true)
    }

    /// Formats the token as an HTTP header.
    #[must_use]
    pub fn as_header(&self) -> String {
        format!("CSRFPreventionToken: {}", self.value)
    }
}

/// Validates the format of a CSRF token string.
pub(crate) fn validate_csrf_token(token: &str) -> Result<(), ValidationError> {
    if token.is_empty() {
        return Err(ValidationError::Field {
            field: "csrf_token".to_string(),
            message: "CSRF token cannot be empty".to_string(),
        });
    }
    let parts: Vec<&str> = token.split(':').collect();
    if parts.len() != 2 {
        return Err(ValidationError::Format(
            "CSRF token must be in format TOKENID:VALUE".to_string(),
        ));
    }
    if parts[0].len() != 8 || !parts[0].chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ValidationError::Format(
            "Token ID must be 8 hexadecimal characters".to_string(),
        ));
    }
    Ok(())
}
