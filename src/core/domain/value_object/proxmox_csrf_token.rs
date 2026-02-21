use crate::core::domain::error::ValidationError;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

/// A Proxmox CSRF protection token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxmoxCSRFToken {
    value: String,
    #[serde(with = "crate::core::domain::value_object::serde_helpers::system_time")]
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
    // Validate token value (allow base64 chars)
    if !parts[1]
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
    {
        return Err(ValidationError::Format(
            "Token value contains invalid characters".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, SystemTime};

    #[test]
    fn test_validate_csrf_token_valid() {
        let valid = "4EEC61E2:lwk7od06fa1+DcPUwBTXCcndyAY/3mKxQp5vR8sNjWuBtL9fZg==";
        assert!(validate_csrf_token(valid).is_ok());
    }

    #[test]
    fn test_validate_csrf_token_invalid() {
        assert!(validate_csrf_token("").is_err());
        assert!(validate_csrf_token("invalid").is_err()); // no colon
        assert!(validate_csrf_token("1234567:value").is_err()); // ID too short
        // Use a character not allowed (space)
        assert!(validate_csrf_token("12345678:value with space").is_err());
    }

    #[test]
    fn test_csrf_expiration() {
        let token = ProxmoxCSRFToken::new_unchecked("4EEC61E2:value".to_string());
        assert!(!token.is_expired(Duration::from_secs(300)));

        let old_token = ProxmoxCSRFToken {
            value: "4EEC61E2:value".to_string(),
            created_at: SystemTime::now() - Duration::from_secs(301),
        };
        assert!(old_token.is_expired(Duration::from_secs(300)));
    }

    #[test]
    fn test_csrf_as_header() {
        let token = ProxmoxCSRFToken::new_unchecked("id:val".to_string());
        assert_eq!(token.as_header(), "CSRFPreventionToken: id:val");
    }

    #[test]
    fn test_serde_roundtrip() {
        let original = ProxmoxCSRFToken::new_unchecked("id:val".to_string());
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: ProxmoxCSRFToken = serde_json::from_str(&serialized).unwrap();
        assert_eq!(original.as_str(), deserialized.as_str());

        // Because we serialize as seconds (truncating fractional seconds), the deserialized
        // time may be up to 1 second earlier. We check that the difference is less than 1 second.
        let diff = original
            .created_at
            .duration_since(deserialized.created_at)
            .unwrap_or_else(|_| {
                deserialized
                    .created_at
                    .duration_since(original.created_at)
                    .unwrap()
            });
        assert!(diff < Duration::from_secs(1));
    }
}
