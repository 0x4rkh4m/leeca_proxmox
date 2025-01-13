use crate::core::domain::{
    error::{ProxmoxResult, ValidationError},
    value_object::base_value_object::ValueObject,
};
use async_trait::async_trait;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Represents the configuration for a Proxmox realm value object
///
/// This configuration object encapsulates the constraints and settings
/// for realm validation according to:
/// - Proxmox VE Authentication Realms
/// - PAM Authentication standards
/// - LDAP/Active Directory requirements
///
/// # Examples
///
/// ```
/// use leeca_proxmox::core::domain::value_object::proxmox_realm::ProxmoxRealmConfig;
///
/// let config = ProxmoxRealmConfig::default();
/// ```
#[derive(Debug, Clone)]
pub struct ProxmoxRealmConfig {
    min_length: usize,
    max_length: usize,
    allowed_realms: HashSet<String>,
    allowed_chars: HashSet<char>,
}

impl ProxmoxRealmConfig {
    const DEFAULT_REALMS: [&'static str; 3] = ["pam", "pve", "ldap"];

    pub(super) async fn validate_realm(&self, realm: &str) -> Result<(), ValidationError> {
        if realm.is_empty() {
            return Err(ValidationError::FieldError {
                field: "realm".to_string(),
                message: "Realm cannot be empty".to_string(),
            });
        }

        if realm.len() < self.min_length || realm.len() > self.max_length {
            return Err(ValidationError::FormatError(format!(
                "Realm length must be between {} and {} characters",
                self.min_length, self.max_length
            )));
        }

        if !realm.chars().all(|c| self.allowed_chars.contains(&c)) {
            return Err(ValidationError::FormatError(
                "Realm contains invalid characters".to_string(),
            ));
        }

        if !self.allowed_realms.contains(realm) {
            return Err(ValidationError::ConstraintViolation(format!(
                "Invalid realm. Allowed realms are: {}",
                self.allowed_realms
                    .iter()
                    .cloned()
                    .collect::<Vec<String>>()
                    .join(", ")
            )));
        }

        Ok(())
    }
}

impl Default for ProxmoxRealmConfig {
    fn default() -> Self {
        let mut allowed_chars = HashSet::new();
        allowed_chars.extend('a'..='z');
        allowed_chars.extend('0'..='9');
        allowed_chars.extend(['-', '_']);

        let allowed_realms = Self::DEFAULT_REALMS.iter().map(|s| s.to_string()).collect();

        Self {
            min_length: 2,
            max_length: 32,
            allowed_realms,
            allowed_chars,
        }
    }
}

/// Represents a validated Proxmox authentication realm
///
/// This value object ensures realms comply with:
/// - Proxmox VE authentication standards
/// - System authentication requirements
/// - Security best practices
///
/// # Examples
///
/// ```
/// use leeca_proxmox::core::domain::value_object::proxmox_realm::ProxmoxRealm;
///
/// #[tokio::main]
/// async fn main() {
///     let realm = ProxmoxRealm::new("pve".to_string()).await.unwrap();
///     assert_eq!(realm.as_inner().await, "pve");
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ProxmoxRealm {
    value: Arc<RwLock<String>>,
}

impl ProxmoxRealm {
    pub async fn new(realm: String) -> ProxmoxResult<Self> {
        <Self as ValueObject>::new(realm).await
    }
}

#[async_trait]
impl ValueObject for ProxmoxRealm {
    type Value = String;
    type ValidationConfig = ProxmoxRealmConfig;

    fn value(&self) -> &Arc<RwLock<Self::Value>> {
        &self.value
    }

    fn validation_config() -> Self::ValidationConfig {
        ProxmoxRealmConfig::default()
    }

    async fn validate(
        value: &Self::Value,
        config: &Self::ValidationConfig,
    ) -> Result<(), ValidationError> {
        config.validate_realm(value).await
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
    async fn test_valid_realms() {
        let valid_realms = vec!["pam", "pve", "ldap"];

        for realm in valid_realms {
            let result = ProxmoxRealm::new(realm.to_string()).await;
            assert!(result.is_ok(), "Realm {} should be valid", realm);
        }
    }

    #[tokio::test]
    async fn test_invalid_realms() {
        let test_cases = vec![
            ("", "empty realm"),
            ("a", "too short"),
            ("invalid_realm", "unknown realm"),
            ("PAM", "wrong case"),
            ("pve@domain", "invalid characters"),
        ];

        for (realm, case) in test_cases {
            let result = ProxmoxRealm::new(realm.to_string()).await;
            assert!(
                matches!(result, Err(ProxmoxError::ValidationError { .. })),
                "Case '{}' should fail validation: {}",
                case,
                realm
            );
        }
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let realm = ProxmoxRealm::new("pve".to_string()).await.unwrap();

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let realm_clone = realm.clone();
                tokio::spawn(async move { realm_clone.as_inner().await })
            })
            .collect();

        for handle in handles {
            let result = handle.await.unwrap();
            assert_eq!(result, "pve");
        }
    }
}
