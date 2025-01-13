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
#[derive(Debug, Clone)]
pub struct ProxmoxUrlConfig {
    allowed_schemes: HashSet<String>,
    max_length: usize,
    allowed_paths: HashSet<String>,
}

impl ProxmoxUrlConfig {
    async fn validate_url(&self, url: &str) -> Result<(), ValidationError> {
        // Reject empty URLs
        if url.is_empty() {
            return Err(ValidationError::Field {
                field: "url".to_string(),
                message: "URL cannot be empty".to_string(),
            });
        }

        // Reject bare double slashes
        if url == "//" {
            return Err(ValidationError::Format(
                "URL cannot be just double slashes".to_string(),
            ));
        }

        // Length validation
        if url.len() > self.max_length {
            return Err(ValidationError::Format(format!(
                "URL exceeds maximum length of {} characters",
                self.max_length
            )));
        }

        let url_parts = url::Url::parse(url)
            .map_err(|e| ValidationError::Format(format!("Invalid URL format: {}", e)))?;

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

        // Validate path
        let path = url_parts.path();

        // Check for consecutive slashes in the path
        if path.contains("//") {
            return Err(ValidationError::Format(
                "Path cannot contain consecutive slashes".to_string(),
            ));
        }

        // Allow base URL with just "/"
        if path == "/" {
            return Ok(());
        }

        // For non-base URLs, validate against allowed paths
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
    use crate::core::domain::error::ProxmoxError;

    #[tokio::test]
    async fn test_valid_urls() {
        let base_url = create_test_url("proxmox.example.com", 8006, true).await;
        assert!(base_url.is_ok(), "Base URL should be valid");

        let url = base_url.unwrap();
        let api_url = url.with_path("/api2/json").await;
        assert!(api_url.is_ok(), "API URL should be valid");
    }

    #[tokio::test]
    async fn test_invalid_urls() {
        let test_host = create_test_host("invalid.com").await.unwrap();
        let port = ProxmoxPort::new(8006).await.unwrap();
        let long_path = &"/a".repeat(2084);
        let test_cases = vec![
            (true, "/api2//json", "consecutive slashes"),
            (true, "/api3/json", "invalid api version"),
            (false, "/invalid/path", "invalid api path"),
            (true, long_path, "path too long"),
        ];

        for (secure, path, case) in test_cases {
            let url = ProxmoxUrl::new(&test_host, &port, &secure).await.unwrap();
            let result = url.with_path(path).await;
            assert!(
                matches!(result, Err(ProxmoxError::Validation { .. })),
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

    // Helper function to create a test host without DNS validation
    pub async fn create_test_host(hostname: &str) -> ProxmoxResult<ProxmoxHost> {
        Ok(ProxmoxHost::create(hostname.to_string()))
    }

    // Helper function to create a test URL without DNS validation
    async fn create_test_url(host: &str, port: u16, secure: bool) -> ProxmoxResult<ProxmoxUrl> {
        let test_host = create_test_host(host).await?;
        let test_port = ProxmoxPort::new(port).await?;
        ProxmoxUrl::new(&test_host, &test_port, &secure).await
    }
}
