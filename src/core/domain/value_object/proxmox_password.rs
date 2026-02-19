use crate::core::domain::error::ValidationError;
use zxcvbn::zxcvbn;

/// A Proxmox password (plaintext, only stored temporarily).
#[derive(Debug, Clone)]
pub struct ProxmoxPassword(String);

impl ProxmoxPassword {
    /// Creates a new password without validation.
    pub(crate) fn new_unchecked(password: String) -> Self {
        Self(password)
    }

    /// Returns the password as a string slice.
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

/// Validates a password according to the configuration.
pub(crate) fn validate_password(
    password: &str,
    min_score: Option<zxcvbn::Score>,
) -> Result<(), ValidationError> {
    if password.is_empty() {
        return Err(ValidationError::Field {
            field: "password".to_string(),
            message: "Password cannot be empty".to_string(),
        });
    }
    if password.len() < 8 {
        return Err(ValidationError::Format(
            "Password must be at least 8 characters long".to_string(),
        ));
    }
    if password.len() > 128 {
        return Err(ValidationError::Format(
            "Password cannot exceed 128 characters".to_string(),
        ));
    }
    if let Some(min_score) = min_score {
        let entropy = zxcvbn(password, &[]);
        if entropy.score() < min_score {
            return Err(ValidationError::ConstraintViolation(
                "Password is too weak (increase complexity)".to_string(),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use zxcvbn::Score;

    #[test]
    fn test_validate_password_valid() {
        assert!(validate_password("Str0ng!P@ss", None).is_ok());
        assert!(validate_password("password123", None).is_ok());
    }

    #[test]
    fn test_validate_password_invalid() {
        assert!(validate_password("", None).is_err());
        assert!(validate_password("short", None).is_err()); // <8
        assert!(validate_password(&"a".repeat(129), None).is_err()); // >128
    }

    #[test]
    fn test_validate_password_with_strength() {
        let min_score = Some(Score::Three);
        // Strong password
        assert!(validate_password("Str0ng!P@ssw0rd", min_score).is_ok());
        // Weak password (meets length but low entropy)
        assert!(validate_password("password", min_score).is_err());
        assert!(validate_password("12345678", min_score).is_err());
    }

    #[test]
    fn test_password_new_unchecked() {
        let pwd = ProxmoxPassword::new_unchecked("secret".to_string());
        assert_eq!(pwd.as_str(), "secret");
    }
}
