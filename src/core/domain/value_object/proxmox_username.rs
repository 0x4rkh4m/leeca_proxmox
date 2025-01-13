use crate::core::domain::{
    error::{ProxmoxResult, ValidationError},
    value_object::base_value_object::ValueObject,
};
use async_trait::async_trait;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Represents the configuration for a Proxmox username value object
///
/// This configuration object encapsulates the constraints and settings
/// for username validation according to:
/// - RFC 4513 (LDAP Authentication)
/// - Proxmox VE user management requirements
/// - Common security practices
#[derive(Debug, Clone)]
pub struct ProxmoxUsernameConfig {
    min_length: usize,
    max_length: usize,
    allowed_chars: HashSet<char>,
    reserved_names: HashSet<String>,
}

impl ProxmoxUsernameConfig {
    pub(super) async fn validate_username(&self, username: &str) -> Result<(), ValidationError> {
        if username.is_empty() {
            return Err(ValidationError::Field {
                field: "username".to_string(),
                message: "Username cannot be empty".to_string(),
            });
        }

        if username.len() < self.min_length || username.len() > self.max_length {
            return Err(ValidationError::Format(format!(
                "Username length must be between {} and {} characters",
                self.min_length, self.max_length
            )));
        }

        if !username.chars().all(|c| self.allowed_chars.contains(&c)) {
            return Err(ValidationError::Format(
                "Username contains invalid characters".to_string(),
            ));
        }

        if self.reserved_names.contains(username) {
            return Err(ValidationError::ConstraintViolation(
                "Username is reserved".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for ProxmoxUsernameConfig {
    fn default() -> Self {
        let mut allowed_chars = HashSet::new();
        // Alphanumeric characters
        allowed_chars.extend('a'..='z');
        allowed_chars.extend('A'..='Z');
        allowed_chars.extend('0'..='9');
        // Special characters allowed by Proxmox
        allowed_chars.extend(['-', '_', '.', '@']);

        let mut reserved_names = HashSet::new();
        // System reserved usernames
        reserved_names.extend([
            "root".to_string(),
            "admin".to_string(),
            "administrator".to_string(),
            "nobody".to_string(),
            "guest".to_string(),
            "www-data".to_string(),
        ]);

        Self {
            min_length: 3,
            max_length: 64,
            allowed_chars,
            reserved_names,
        }
    }
}

/// Represents a validated Proxmox username
///
/// This value object ensures usernames comply with:
/// - Proxmox VE naming conventions
/// - Security best practices
/// - System restrictions
#[derive(Debug, Clone)]
pub struct ProxmoxUsername {
    value: Arc<RwLock<String>>,
}

impl ProxmoxUsername {
    pub async fn new(username: String) -> ProxmoxResult<Self> {
        <Self as ValueObject>::new(username).await
    }
}

#[async_trait]
impl ValueObject for ProxmoxUsername {
    type Value = String;
    type ValidationConfig = ProxmoxUsernameConfig;

    fn value(&self) -> &Arc<RwLock<Self::Value>> {
        &self.value
    }

    fn validation_config() -> Self::ValidationConfig {
        ProxmoxUsernameConfig::default()
    }

    async fn validate(
        value: &Self::Value,
        config: &Self::ValidationConfig,
    ) -> Result<(), ValidationError> {
        config.validate_username(value).await
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
    use tokio::time::Duration;

    #[tokio::test]
    async fn test_valid_usernames() {
        let valid_usernames = vec![
            "user@pve",
            "john.doe",
            "admin_user",
            "test-123",
            "user.name@domain",
        ];

        for username in valid_usernames {
            let result = ProxmoxUsername::new(username.to_string()).await;
            assert!(result.is_ok(), "Username {} should be valid", username);
        }
    }

    #[tokio::test]
    async fn test_invalid_usernames() {
        let long_username = "a".repeat(65);
        let test_cases = vec![
            ("", "empty username"),
            ("ab", "too short"),
            (long_username.as_str(), "too long"),
            ("user$name", "invalid characters"),
            ("root", "reserved username"),
            ("user name", "contains space"),
            ("admin", "reserved username"),
        ];

        for (username, case) in test_cases {
            let result = ProxmoxUsername::new(username.to_string()).await;
            assert!(
                matches!(result, Err(ProxmoxError::Validation { .. })),
                "Case '{}' should fail validation: {}",
                case,
                username
            );
        }
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let username = ProxmoxUsername::new("test.user@pve".to_string())
            .await
            .unwrap();

        let handles: Vec<_> = (0..10)
            .map(|i| {
                let username_clone = username.clone();
                tokio::spawn(async move {
                    if i % 2 == 0 {
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }
                    username_clone.as_inner().await
                })
            })
            .collect();

        for handle in handles {
            let result = handle.await.unwrap();
            assert_eq!(result, "test.user@pve");
        }
    }

    #[tokio::test]
    async fn test_update() {
        let username = ProxmoxUsername::new("old.user@pve".to_string())
            .await
            .unwrap();
        let update_result = username.update("new.user@pve".to_string()).await;
        assert!(update_result.is_ok());
        assert_eq!(username.as_inner().await, "new.user@pve");
    }
}
