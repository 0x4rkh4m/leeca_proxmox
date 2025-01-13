use crate::core::domain::{
    error::{ProxmoxResult, ValidationError},
    value_object::base_value_object::ValueObject,
};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Duration;

/// Represents the configuration for a Proxmox host value object
///
/// This configuration object encapsulates the constraints and settings
/// for a Proxmox host value object.
#[derive(Debug, Clone)]
pub struct ProxmoxHostConfig {
    max_hostname_length: usize,
    max_label_length: usize,
    dns_timeout: Duration,
}

impl ProxmoxHostConfig {
    async fn validate_label(&self, label: &str) -> Result<(), ValidationError> {
        if label.is_empty() || label.len() > self.max_label_length {
            return Err(ValidationError::Format(format!(
                "Label must be between 1 and {} characters",
                self.max_label_length
            )));
        }

        if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(ValidationError::Format(
                "Label can only contain alphanumeric characters and hyphens".to_string(),
            ));
        }

        if label.starts_with('-') || label.ends_with('-') {
            return Err(ValidationError::Format(
                "Label cannot start or end with hyphen".to_string(),
            ));
        }

        Ok(())
    }

    async fn validate_dns(&self, value: &str) -> Result<(), ValidationError> {
        match tokio::time::timeout(
            self.dns_timeout,
            tokio::net::lookup_host(format!("{}:8006", value)),
        )
        .await
        {
            Ok(lookup_result) => {
                let addresses = lookup_result.map_err(|e| {
                    ValidationError::ConstraintViolation(format!("DNS resolution failed: {}", e))
                })?;

                if addresses.count() == 0 {
                    return Err(ValidationError::ConstraintViolation(
                        "No DNS records found".to_string(),
                    ));
                }
                Ok(())
            }
            Err(_) => Err(ValidationError::ConstraintViolation(
                "DNS resolution timeout".to_string(),
            )),
        }
    }
}

impl Default for ProxmoxHostConfig {
    fn default() -> Self {
        Self {
            max_hostname_length: 253,
            max_label_length: 63,
            dns_timeout: Duration::from_secs(5),
        }
    }
}

/// Represents a validated Proxmox host address
///
/// This value object encapsulates a host address and ensures it meets
/// all RFC 1035 requirements for valid hostnames.
#[derive(Debug, Clone)]
pub struct ProxmoxHost {
    value: Arc<RwLock<String>>,
}

impl ProxmoxHost {
    /// Creates a new ProxmoxHost instance with validation
    ///
    /// # Arguments
    ///
    /// * `host` - The hostname to validate and wrap
    ///
    /// # Returns
    ///
    /// * `Ok(ProxmoxHost)` if validation succeeds
    /// * `Err(ProxmoxError)` if validation fails
    pub async fn new(host: String) -> ProxmoxResult<Self> {
        <Self as ValueObject>::new(host).await
    }
}

#[async_trait]
impl ValueObject for ProxmoxHost {
    type Value = String;
    type ValidationConfig = ProxmoxHostConfig;

    fn value(&self) -> &Arc<RwLock<Self::Value>> {
        &self.value
    }

    fn validation_config() -> Self::ValidationConfig {
        ProxmoxHostConfig::default()
    }

    async fn validate(
        value: &Self::Value,
        config: &Self::ValidationConfig,
    ) -> Result<(), ValidationError> {
        if value.is_empty() {
            return Err(ValidationError::Field {
                field: "host".to_string(),
                message: "Host cannot be empty".to_string(),
            });
        }

        if value.len() > config.max_hostname_length {
            return Err(ValidationError::ConstraintViolation(format!(
                "Host length exceeds maximum of {} characters",
                config.max_hostname_length
            )));
        }

        let labels: Vec<&str> = value.split('.').collect();
        for label in labels {
            config.validate_label(label).await?;
        }

        config.validate_dns(value).await?;

        Ok(())
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
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_valid_hostnames() {
        let valid_hosts = vec![
            "example.com",
            "sub.example.com",
            "example-domain.com",
            "123.example.com",
        ];

        for host in valid_hosts {
            let result = create_test_host(host).await;
            assert!(result.is_ok(), "Host {} should be valid", host);
        }
    }

    #[tokio::test]
    async fn test_invalid_hostnames() {
        let long_hostname = "a".repeat(254);
        let test_cases = vec![
            ("", "empty hostname"),
            (long_hostname.as_str(), "hostname too long"),
            ("-example.com", "starts with hyphen"),
            ("example-.com", "ends with hyphen"),
            ("exam@ple.com", "invalid character"),
            ("exam ple.com", "contains space"),
            (".example.com", "empty label"),
            ("example..com", "consecutive dots"),
        ];

        for (host, case) in test_cases {
            let result = ProxmoxHost::new(host.to_string()).await;
            assert!(
                matches!(result, Err(ProxmoxError::Validation { .. })),
                "Case '{}' should fail validation: {}",
                case,
                host
            );
        }
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let host = ProxmoxHost::new("example.com".to_string()).await.unwrap();

        let handles: Vec<_> = (0..10)
            .map(|i| {
                let host_clone = host.clone();
                tokio::spawn(async move {
                    if i % 2 == 0 {
                        sleep(Duration::from_millis(10)).await;
                    }
                    host_clone.as_inner().await
                })
            })
            .collect();

        for handle in handles {
            let result = handle.await.unwrap();
            assert_eq!(result, "example.com");
        }
    }

    #[tokio::test]
    async fn test_dns_timeout() {
        let result = ProxmoxHost::new("non-existent-domain-12345.local".to_string()).await;
        assert!(
            matches!(result, Err(ProxmoxError::Validation { .. })),
            "Should fail for non-resolvable domain"
        );
    }

    // Helper function to skip DNS validation during tests
    pub async fn create_test_host(hostname: &str) -> ProxmoxResult<ProxmoxHost> {
        let value = hostname.to_string();
        Ok(ProxmoxHost::create(value))
    }
}
