use crate::core::domain::{
    error::{ProxmoxResult, ValidationError},
    value_object::base_value_object::ValueObject,
};
use async_trait::async_trait;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Represents the configuration for a Proxmox port value object
///
/// This configuration object encapsulates the constraints and settings
/// for port validation according to IANA standards and Proxmox requirements.
///
/// # References
/// - IANA Port Numbers: https://www.iana.org/assignments/service-names-port-numbers
/// - RFC 6335: Internet Assigned Numbers Authority (IANA) Procedures
#[derive(Debug, Clone)]
pub struct ProxmoxPortConfig {
    min_port: u16,
    max_port: u16,
    reserved_system_ports: HashSet<u16>,
    proxmox_specific_ports: HashSet<u16>,
}

impl ProxmoxPortConfig {
    /// Validates a port number according to configuration rules
    async fn validate_port(&self, port: u16) -> Result<(), ValidationError> {
        // Basic range validation
        if port < self.min_port || port > self.max_port {
            return Err(ValidationError::FormatError(format!(
                "Port must be between {} and {}",
                self.min_port, self.max_port
            )));
        }

        // System ports validation (0-1023)
        if port < 1024 && !self.reserved_system_ports.contains(&port) {
            return Err(ValidationError::ConstraintViolation(
                "Cannot use restricted system ports (0-1023)".to_string(),
            ));
        }

        Ok(())
    }

    /// Checks if a port is a known Proxmox service port
    pub fn is_proxmox_port(&self, port: u16) -> bool {
        self.proxmox_specific_ports.contains(&port)
    }
}

impl Default for ProxmoxPortConfig {
    fn default() -> Self {
        let mut proxmox_ports = HashSet::new();
        // Proxmox specific ports
        proxmox_ports.insert(8006); // Web interface
        proxmox_ports.insert(3128); // SPICE proxy
        proxmox_ports.extend(5900..=5999); // VNC ports
        proxmox_ports.extend(5405..=5412); // Corosync cluster

        let mut system_ports = HashSet::new();
        // Common system ports used by Proxmox
        system_ports.insert(22); // SSH
        system_ports.insert(80); // HTTP
        system_ports.insert(443); // HTTPS
        system_ports.insert(123); // NTP

        Self {
            min_port: 1,
            max_port: 65535,
            reserved_system_ports: system_ports,
            proxmox_specific_ports: proxmox_ports,
        }
    }
}

/// Represents a validated Proxmox port number
///
/// This value object ensures that port numbers are valid according to:
/// - IANA port number assignments
/// - Proxmox VE specific requirements
/// - System port restrictions
///
/// # Examples
///
/// ```
/// use leeca_proxmox::core::domain::value_object::proxmox_port::ProxmoxPort;
///
/// #[tokio::main]
/// async fn main() {
///     let port = ProxmoxPort::new(8006).await.unwrap();
///     assert_eq!(port.as_inner().await, 8006);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ProxmoxPort {
    value: Arc<RwLock<u16>>,
}

impl ProxmoxPort {
    pub async fn new(port: u16) -> ProxmoxResult<Self> {
        <Self as ValueObject>::new(port).await
    }
}

#[async_trait]
impl ValueObject for ProxmoxPort {
    type Value = u16;
    type ValidationConfig = ProxmoxPortConfig;

    fn value(&self) -> &Arc<RwLock<Self::Value>> {
        &self.value
    }

    fn validation_config() -> Self::ValidationConfig {
        ProxmoxPortConfig::default()
    }

    async fn validate(
        value: &Self::Value,
        config: &Self::ValidationConfig,
    ) -> Result<(), ValidationError> {
        config.validate_port(*value).await
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
    async fn test_valid_ports() {
        let valid_ports = vec![8006, 22, 80, 443, 5900, 3128];

        for port in valid_ports {
            let result = ProxmoxPort::new(port).await;
            assert!(result.is_ok(), "Port {} should be valid", port);
        }
    }

    #[tokio::test]
    async fn test_invalid_ports() {
        let test_cases = vec![(0, "port zero"), (7, "restricted system port")]; // u16 already validates top range

        for (port, case) in test_cases {
            let result = ProxmoxPort::new(port).await;
            assert!(
                matches!(result, Err(ProxmoxError::ValidationError { .. })),
                "Case '{}' should fail validation: {}",
                case,
                port
            );
        }
    }

    #[tokio::test]
    async fn test_proxmox_specific_ports() {
        let config = ProxmoxPortConfig::default();
        assert!(
            config.is_proxmox_port(8006),
            "8006 should be recognized as Proxmox port"
        );
        assert!(
            config.is_proxmox_port(5900),
            "5900 should be recognized as Proxmox port"
        );
        assert!(
            !config.is_proxmox_port(1234),
            "1234 should not be recognized as Proxmox port"
        );
    }
}
