use crate::core::domain::{
    error::{ProxmoxResult, ValidationError},
    value_object::base_value_object::ValueObject,
};
use async_trait::async_trait;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use zxcvbn::{zxcvbn, Score};

/// Represents the configuration for a Proxmox password value object
///
/// This configuration object encapsulates the constraints and settings
/// for password validation according to:
/// - NIST Special Publication 800-63B
/// - OWASP Password Security Guidelines
/// - Proxmox VE security requirements
///
/// # Examples
///
/// ```
/// use leeca_proxmox::core::domain::value_object::proxmox_password::ProxmoxPasswordConfig;
///
/// let config = ProxmoxPasswordConfig::default();
/// ```
#[derive(Debug, Clone)]
pub struct ProxmoxPasswordConfig {
    min_length: usize,
    max_length: usize,
    min_entropy_score: Score,
    required_char_types: HashSet<CharacterType>,
    special_chars: &'static str,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
enum CharacterType {
    Lowercase,
    Uppercase,
    Digit,
    Special,
}

impl ProxmoxPasswordConfig {
    const DEFAULT_SPECIAL_CHARS: &'static str = "~!@#$%^&*()_+-=`{}[]|\\:;\"'<>,.?/";
    fn is_special_char(&self, c: char) -> bool {
        self.special_chars.contains(c)
    }

    async fn validate_password(&self, password: &str) -> Result<(), ValidationError> {
        if password.is_empty() {
            return Err(ValidationError::FieldError {
                field: "password".to_string(),
                message: "Password cannot be empty".to_string(),
            });
        }

        if password.len() < self.min_length || password.len() > self.max_length {
            return Err(ValidationError::FormatError(format!(
                "Password length must be between {} and {} characters",
                self.min_length, self.max_length
            )));
        }

        let mut found_types = HashSet::new();
        for c in password.chars() {
            if c.is_ascii_lowercase() {
                found_types.insert(CharacterType::Lowercase);
            } else if c.is_ascii_uppercase() {
                found_types.insert(CharacterType::Uppercase);
            } else if c.is_ascii_digit() {
                found_types.insert(CharacterType::Digit);
            } else if self.is_special_char(c) {
                found_types.insert(CharacterType::Special);
            }
        }

        let missing_types: HashSet<_> = self.required_char_types.difference(&found_types).collect();

        if !missing_types.is_empty() {
            return Err(ValidationError::ConstraintViolation(
                "Password must contain at least one character from each required type".to_string(),
            ));
        }

        // Evaluate password strength using zxcvbn
        let entropy = zxcvbn(password, &[]).score();

        if entropy < self.min_entropy_score {
            return Err(ValidationError::ConstraintViolation(
                "Password is too weak".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for ProxmoxPasswordConfig {
    fn default() -> Self {
        let mut required_types = HashSet::new();
        required_types.insert(CharacterType::Lowercase);
        required_types.insert(CharacterType::Uppercase);
        required_types.insert(CharacterType::Digit);
        required_types.insert(CharacterType::Special);

        Self {
            min_length: 8,
            max_length: 64,
            min_entropy_score: Score::Three,
            required_char_types: required_types,
            special_chars: Self::DEFAULT_SPECIAL_CHARS,
        }
    }
}

/// Represents a validated Proxmox password
///
/// This value object ensures passwords comply with:
/// - NIST security guidelines
/// - OWASP password requirements
/// - Proxmox VE security standards
///
/// # Examples
///
/// ```
/// use leeca_proxmox::core::domain::value_object::proxmox_password::ProxmoxPassword;
///
/// #[tokio::main]
/// async fn main() {
///     let password = ProxmoxPassword::new("Str0ng!P@ssw0rd".to_string()).await.unwrap();
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ProxmoxPassword {
    value: Arc<RwLock<String>>,
}

impl ProxmoxPassword {
    pub async fn new(password: String) -> ProxmoxResult<Self> {
        <Self as ValueObject>::new(password).await
    }
}

#[async_trait]
impl ValueObject for ProxmoxPassword {
    type Value = String;
    type ValidationConfig = ProxmoxPasswordConfig;

    fn value(&self) -> &Arc<RwLock<Self::Value>> {
        &self.value
    }

    fn validation_config() -> Self::ValidationConfig {
        ProxmoxPasswordConfig::default()
    }

    async fn validate(
        value: &Self::Value,
        config: &Self::ValidationConfig,
    ) -> Result<(), ValidationError> {
        config.validate_password(value).await
    }

    fn create(value: Self::Value) -> Self {
        Self {
            value: Arc::new(RwLock::new(value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::domain::error::ProxmoxError;

    #[tokio::test]
    async fn test_valid_passwords() {
        let valid_passwords = vec![
            "Str0ng!P@ssw0rd",
            "C0mpl3x#P@ss",
            "Sup3r.S3cur3!2024",
            "P@ssw0rd.With-Symb0ls",
        ];

        for password in valid_passwords {
            let result = ProxmoxPassword::new(password.to_string()).await;
            assert!(result.is_ok(), "Password should be valid");
        }
    }

    #[tokio::test]
    async fn test_invalid_passwords() {
        let long_password = "a".repeat(65);
        let test_cases = vec![
            ("", "empty password"),
            ("short", "too short"),
            (long_password.as_str(), "too long"),
            ("password123", "no uppercase or special"),
            ("Password", "no numbers or special"),
            ("12345678", "only numbers"),
            ("abcdefgh", "only lowercase"),
        ];

        for (password, case) in test_cases {
            let result = ProxmoxPassword::new(password.to_string()).await;
            assert!(
                matches!(result, Err(ProxmoxError::ValidationError { .. })),
                "Case '{}' should fail validation: {}",
                case,
                password
            );
        }
    }

    #[tokio::test]
    async fn test_password_entropy() {
        let weak_passwords = vec!["Password123!", "Qwerty123!", "Admin123!"];

        for password in weak_passwords {
            let result = ProxmoxPassword::new(password.to_string()).await;
            assert!(
                matches!(result, Err(ProxmoxError::ValidationError { .. })),
                "Common password '{}' should be rejected",
                password
            );
        }
    }
}
