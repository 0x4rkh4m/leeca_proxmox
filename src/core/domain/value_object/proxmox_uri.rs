use crate::core::domain::error::ValidationError;
use url::Url;

/// A validated Proxmox API URL.
#[derive(Debug, Clone)]
pub struct ProxmoxUrl(String);

impl ProxmoxUrl {
    /// Creates a new URL without validation.
    pub(crate) fn new_unchecked(url: String) -> Self {
        Self(url)
    }

    /// Returns the URL as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the object and returns the inner string.
    #[allow(unused)]
    pub fn into_inner(self) -> String {
        self.0
    }
}

/// Validates a URL string.
pub(crate) fn validate_url(url: &str) -> Result<(), ValidationError> {
    if url.is_empty() {
        return Err(ValidationError::Field {
            field: "url".to_string(),
            message: "URL cannot be empty".to_string(),
        });
    }
    Url::parse(url).map_err(|e| ValidationError::Format(format!("Invalid URL: {}", e)))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_url_valid() {
        assert!(validate_url("https://example.com:8006/").is_ok());
        assert!(validate_url("http://localhost/").is_ok());
    }

    #[test]
    fn test_validate_url_invalid() {
        assert!(validate_url("").is_err());
        assert!(validate_url("not a url").is_err());
    }

    #[test]
    fn test_url_new_unchecked() {
        let url = ProxmoxUrl::new_unchecked("https://pve:8006/".to_string());
        assert_eq!(url.as_str(), "https://pve:8006/");
    }
}
