use crate::core::domain::error::ValidationError;

/// A validated Proxmox authentication realm.
#[derive(Debug, Clone)]
pub struct ProxmoxRealm(String);

impl ProxmoxRealm {
    /// Creates a new realm without validation.
    pub(crate) fn new_unchecked(realm: String) -> Self {
        Self(realm)
    }

    /// Returns the realm as a string slice.
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

/// Validates a realm string.
pub(crate) fn validate_realm(realm: &str) -> Result<(), ValidationError> {
    if realm.is_empty() {
        return Err(ValidationError::Field {
            field: "realm".to_string(),
            message: "Realm cannot be empty".to_string(),
        });
    }
    if realm.len() > 32 {
        return Err(ValidationError::Format(
            "Realm cannot exceed 32 characters".to_string(),
        ));
    }
    if !realm
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(ValidationError::Format(
            "Realm can only contain alphanumeric characters, hyphens, and underscores".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_realm_valid() {
        assert!(validate_realm("pam").is_ok());
        assert!(validate_realm("pve").is_ok());
        assert!(validate_realm("my-realm_123").is_ok());
    }

    #[test]
    fn test_validate_realm_invalid() {
        assert!(validate_realm("").is_err());
        assert!(validate_realm(&"a".repeat(33)).is_err());
        assert!(validate_realm("pam!").is_err());
        assert!(validate_realm("realm@domain").is_err());
    }

    #[test]
    fn test_realm_new_unchecked() {
        let realm = ProxmoxRealm::new_unchecked("pve".to_string());
        assert_eq!(realm.as_str(), "pve");
    }
}
