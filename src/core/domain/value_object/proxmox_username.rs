use crate::core::domain::error::ValidationError;

/// A validated Proxmox username.
#[derive(Debug, Clone)]
pub struct ProxmoxUsername(String);

impl ProxmoxUsername {
    /// Creates a new username without validation.
    pub(crate) fn new_unchecked(username: String) -> Self {
        Self(username)
    }

    /// Returns the username as a string slice.
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

/// Validates a username according to the configuration.
pub(crate) fn validate_username(
    username: &str,
    block_reserved: bool,
) -> Result<(), ValidationError> {
    if username.is_empty() {
        return Err(ValidationError::Field {
            field: "username".to_string(),
            message: "Username cannot be empty".to_string(),
        });
    }
    if username.len() < 3 || username.len() > 64 {
        return Err(ValidationError::Format(format!(
            "Username length must be between 3 and 64 characters (got {})",
            username.len()
        )));
    }
    let allowed =
        |c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '@';
    if !username.chars().all(allowed) {
        return Err(ValidationError::Format(
            "Username contains invalid characters. Allowed: alphanumeric, -, _, ., @".to_string(),
        ));
    }
    if block_reserved {
        let reserved = [
            "root",
            "admin",
            "administrator",
            "nobody",
            "guest",
            "www-data",
        ];
        if reserved.contains(&username) {
            return Err(ValidationError::ConstraintViolation(
                "Username is reserved".to_string(),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_username_valid() {
        assert!(validate_username("user@pve", false).is_ok());
        assert!(validate_username("john.doe", false).is_ok());
        assert!(validate_username("admin", false).is_ok()); // not blocked
        assert!(validate_username("root", true).is_err()); // blocked
    }

    #[test]
    fn test_validate_username_invalid() {
        assert!(validate_username("", false).is_err());
        assert!(validate_username("ab", false).is_err()); // too short
        assert!(validate_username(&"a".repeat(65), false).is_err()); // too long
        assert!(validate_username("user$name", false).is_err()); // invalid char
        assert!(validate_username("user name", false).is_err()); // space
    }

    #[test]
    fn test_validate_username_block_reserved() {
        let reserved = [
            "root",
            "admin",
            "administrator",
            "nobody",
            "guest",
            "www-data",
        ];
        for name in reserved {
            assert!(validate_username(name, true).is_err());
            assert!(validate_username(name, false).is_ok()); // not blocked
        }
    }

    #[test]
    fn test_username_new_unchecked() {
        let username = ProxmoxUsername::new_unchecked("test".to_string());
        assert_eq!(username.as_str(), "test");
    }
}
