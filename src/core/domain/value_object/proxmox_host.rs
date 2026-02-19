use crate::core::domain::error::ValidationError;

/// A validated Proxmox hostname or IP address.
#[derive(Debug, Clone)]
pub struct ProxmoxHost(String);

impl ProxmoxHost {
    /// Creates a new host without validation (use builder for validation).
    pub(crate) fn new_unchecked(host: String) -> Self {
        Self(host)
    }

    /// Returns the host as a string slice.
    #[allow(unused)]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the object and returns the inner string.
    #[allow(unused)]
    pub fn into_inner(self) -> String {
        self.0
    }
}

/// Validates a host string according to the configuration.
pub(crate) fn validate_host(host: &str, _resolve_dns: bool) -> Result<(), ValidationError> {
    if host.is_empty() {
        return Err(ValidationError::Field {
            field: "host".to_string(),
            message: "Host cannot be empty".to_string(),
        });
    }
    if host.len() > 253 {
        return Err(ValidationError::Format(
            "Hostname exceeds maximum length of 253 characters".to_string(),
        ));
    }
    for label in host.split('.') {
        if label.is_empty() || label.len() > 63 {
            return Err(ValidationError::Format(
                "Each hostname label must be 1-63 characters".to_string(),
            ));
        }
        if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(ValidationError::Format(
                "Labels can only contain alphanumeric characters and hyphens".to_string(),
            ));
        }
        if label.starts_with('-') || label.ends_with('-') {
            return Err(ValidationError::Format(
                "Labels cannot start or end with hyphen".to_string(),
            ));
        }
    }
    // TODO: DNS resolution would go here if enabled, but it's async; we skip for now.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_host_valid() {
        assert!(validate_host("example.com", false).is_ok());
        assert!(validate_host("sub.domain.co", false).is_ok());
        assert!(validate_host("my-host123.org", false).is_ok());
    }

    #[test]
    fn test_validate_host_invalid() {
        assert!(validate_host("", false).is_err());
        assert!(validate_host(&"a".repeat(254), false).is_err());
        assert!(validate_host("-example.com", false).is_err());
        assert!(validate_host("example-.com", false).is_err());
        assert!(validate_host("exam ple.com", false).is_err());
        assert!(validate_host("exam@ple.com", false).is_err());
        assert!(validate_host(".example.com", false).is_err());
        assert!(validate_host("example..com", false).is_err());
    }

    #[test]
    fn test_host_new_unchecked() {
        let host = ProxmoxHost::new_unchecked("test.com".to_string());
        assert_eq!(host.as_str(), "test.com");
    }
}
