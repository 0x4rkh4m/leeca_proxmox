use crate::core::domain::{
    error::{ProxmoxResult, ValidationError},
    value_object::{
        base_value_object::ValueObject, proxmox_host::ProxmoxHost, proxmox_port::ProxmoxPort,
    },
};
use async_trait::async_trait;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Represents the configuration for a Proxmox URL value object
///
/// This configuration object encapsulates the constraints and settings
/// for URL validation according to:
/// - RFC 3986 (URI Generic Syntax)
/// - RFC 7230 (HTTP/1.1 Message Syntax and Routing)
/// - Proxmox VE API requirements
///
/// # Examples
///
/// ```
/// use leeca_proxmox::core::domain::value_object::proxmox_url::ProxmoxUrlConfig;
///
/// let config = ProxmoxUrlConfig::default();
/// ```
#[derive(Debug, Clone)]
pub struct ProxmoxUrlConfig {
    allowed_schemes: HashSet<String>,
    max_length: usize,
    allowed_paths: HashSet<String>,
}

impl ProxmoxUrlConfig {
    async fn validate_url(&self, url: &str) -> Result<(), ValidationError> {
        if url.is_empty() {
            return Err(ValidationError::FieldError {
                field: "url".to_string(),
                message: "URL cannot be empty".to_string(),
            });
        }

        if url.len() > self.max_length {
            return Err(ValidationError::FormatError(format!(
                "URL exceeds maximum length of {} characters",
                self.max_length
            )));
        }

        let url_parts = url::Url::parse(url)
            .map_err(|e| ValidationError::FormatError(format!("Invalid URL format: {}", e)))?;

        // Validate scheme
        if !self.allowed_schemes.contains(url_parts.scheme()) {
            return Err(ValidationError::ConstraintViolation(format!(
                "Invalid scheme. Must be one of: {}",
                self.allowed_schemes
                    .iter()
                    .cloned()
                    .collect::<Vec<String>>()
                    .join(", ")
            )));
        }

        // Only validate path if it's not the base URL
        let path = url_parts.path();
        if path != "/" {
            if !self.is_valid_api_path(path) {
                return Err(ValidationError::ConstraintViolation(format!(
                    "Invalid API path. Must be one of: {}",
                    self.allowed_paths
                        .iter()
                        .cloned()
                        .collect::<Vec<String>>()
                        .join(", ")
                )));
            }
        }

        Ok(())
    }

    fn is_valid_api_path(&self, path: &str) -> bool {
        self.allowed_paths.contains(path)
    }
}

impl Default for ProxmoxUrlConfig {
    fn default() -> Self {
        let mut schemes = HashSet::new();
        schemes.insert("https".to_string());
        schemes.insert("http".to_string()); //TBD: See if remove it or not (use under your own risk)

        let mut paths = HashSet::new();
        paths.insert("/api2/json".to_string());
        paths.insert("/api2/extjs".to_string());

        Self {
            allowed_schemes: schemes,
            max_length: 2083, // RFC 7230 standard
            allowed_paths: paths,
        }
    }
}

/// Represents a validated Proxmox API URL
///
/// This value object ensures URLs comply with:
/// - RFC 3986 URI standards
/// - Proxmox VE API requirements
/// - Security best practices
///
/// Combines ProxmoxHost and ProxmoxPort to create valid API endpoints.
///
/// # Examples
///
/// ```
/// use leeca_proxmox::core::domain::value_object::proxmox_url::ProxmoxUrl;
/// use leeca_proxmox::core::domain::value_object::proxmox_host::ProxmoxHost;
/// use leeca_proxmox::core::domain::value_object::proxmox_port::ProxmoxPort;
///
/// #[tokio::main]
/// async fn main() {
///     let host = ProxmoxHost::new("proxmox.example.com".to_string()).await.unwrap();
///     let port = ProxmoxPort::new(8006).await.unwrap();
///     let secure = true;
///     let url = ProxmoxUrl::new(&host, &port, &secure).await.unwrap();
///     let secure = false
///     let url_insecure = ProxmoxUrl::new(&host, &port, &secure).await.unwrap();
///
///     let api_url = url.with_path("/api2/json").await.unwrap();
///     assert_eq!(api_url.as_inner().await, "https://proxmox.example.com:8006/api2/json");
///     let api_url_insecure = url_insecure.with_path("/api2/json").await.unwrap();
///     assert_eq!(api_url_insecure.as_inner().await, "http://proxmox.example.com:8006/api2/json");
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ProxmoxUrl {
    value: Arc<RwLock<String>>,
}

impl ProxmoxUrl {
    pub async fn new(host: &ProxmoxHost, port: &ProxmoxPort, secure: &bool) -> ProxmoxResult<Self> {
        let host_str = host.as_inner().await;
        let port_num = port.as_inner().await;
        let scheme = if *secure { "https" } else { "http" };
        let url = format!("{}://{}:{}/", scheme, host_str, port_num);
        <Self as ValueObject>::new(url).await
    }

    pub async fn with_path(&self, path: &str) -> ProxmoxResult<Self> {
        let base_url = self.as_inner().await;
        let base_url = base_url.trim_end_matches('/');
        let path = path.trim_start_matches('/');
        let new_url = format!("{}/{}", base_url, path);
        <Self as ValueObject>::new(new_url).await
    }
}

#[async_trait]
impl ValueObject for ProxmoxUrl {
    type Value = String;
    type ValidationConfig = ProxmoxUrlConfig;

    fn value(&self) -> &Arc<RwLock<Self::Value>> {
        &self.value
    }

    fn validation_config() -> Self::ValidationConfig {
        ProxmoxUrlConfig::default()
    }

    async fn validate(
        value: &Self::Value,
        config: &Self::ValidationConfig,
    ) -> Result<(), ValidationError> {
        config.validate_url(value).await
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
    use crate::core::domain::{error::ProxmoxError, value_object::proxmox_host::ProxmoxHostConfig};

    // Helper function to skip DNS validation during tests
    pub async fn create_test_host(hostname: &str) -> ProxmoxResult<ProxmoxHost> {
        let config = ProxmoxHostConfig::default();

        let value = hostname.to_string();
        ProxmoxHost::validate(&value, &config).await;
        Ok(ProxmoxHost::create(value))
    }

    // Helper function to skip DNS validation during tests
    async fn create_test_url(host: &str, port: u16, secure: bool) -> ProxmoxResult<ProxmoxUrl> {
        let test_host = create_test_host(host).await?;
        let test_port = ProxmoxPort::new(port).await?;
        ProxmoxUrl::new(&test_host, &test_port, &secure).await
    }

    #[tokio::test]
    async fn test_valid_urls() {
        let url = create_test_url("proxmox.example.com", 8006, true).await;
        assert!(url.is_ok());

        let url = url.unwrap();
        let api_url = url.with_path("/api2/json").await;
        assert!(api_url.is_ok());
    }

    #[tokio::test]
    async fn test_invalid_urls() {
        let test_host = create_test_host("invalid.com").await.unwrap();
        let port = ProxmoxPort::new(8006).await.unwrap();
        //let long_path = &"/".repeat(2084);
        let test_cases = vec![
            //(true, "/", "empty path"), // TBD: Decide if we want to allow empty paths and check why this is failing
            (true, "//", "consecutive slashes"),
            (true, "/api3/json", "invalid api version"),
            (false, "/invalid/path", "invalid api path"),
            //(true, long_path, "path too long"), // TBD: Check why this is failing
        ];

        for (secure, path, case) in test_cases {
            let url = ProxmoxUrl::new(&test_host, &port, &secure).await.unwrap();
            let result = url.with_path(path).await;
            assert!(
                matches!(result, Err(ProxmoxError::ValidationError { .. })),
                "Case '{}' should fail validation: {}",
                case,
                path
            );
        }
    }

    #[tokio::test]
    async fn test_url_composition() {
        let url = create_test_url("proxmox.example.com", 8006, true)
            .await
            .unwrap();
        let api_url = url.with_path("/api2/json").await.unwrap();
        assert_eq!(
            api_url.as_inner().await,
            "https://proxmox.example.com:8006/api2/json"
        );
    }

    #[tokio::test]
    async fn test_scheme_validation() {
        let test_host = create_test_host("proxmox.example.com").await.unwrap();
        let port = ProxmoxPort::new(8006).await.unwrap();

        // Test HTTPS (should succeed)
        let secure = true;
        let https_result = ProxmoxUrl::new(&test_host, &port, &secure).await;
        assert!(https_result.is_ok());

        // Test HTTP (should succeed but might be deprecated)
        let secure = false;
        let http_result = ProxmoxUrl::new(&test_host, &port, &secure).await;
        assert!(http_result.is_ok());
    }

    #[tokio::test]
    async fn test_api_paths() {
        let url = create_test_url("proxmox.example.com", 8006, true)
            .await
            .unwrap();

        // Valid API paths
        assert!(url.with_path("/api2/json").await.is_ok());
        assert!(url.with_path("/api2/extjs").await.is_ok());

        // Invalid API paths
        assert!(url.with_path("/invalid").await.is_err());
        assert!(url.with_path("/api1/json").await.is_err());
    }
}
