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
    #[must_use]
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
