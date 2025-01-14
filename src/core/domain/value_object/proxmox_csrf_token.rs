use crate::core::domain::{
    error::{ProxmoxResult, ValidationError},
    value_object::base_value_object::ValueObject,
};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::{sync::RwLock, time::Duration};

/// Represents the configuration for a Proxmox CSRF token value object
///
/// This configuration object encapsulates the constraints and settings
/// for CSRF token validation according to:
/// - Proxmox VE Security Standards
/// - OWASP CSRF Prevention Guidelines
/// - RFC 6749 (OAuth 2.0)
#[derive(Debug, Clone)]
pub struct ProxmoxCSRFTokenConfig {
    min_length: usize,
    max_length: usize,
    required_parts: usize,
    token_id_length: usize,
    header_name: String,
    token_lifetime: Duration,
}

impl ProxmoxCSRFTokenConfig {
    async fn validate_token_format(&self, token: &str) -> Result<(), ValidationError> {
        if token.is_empty() {
            return Err(ValidationError::Field {
                field: "csrf_token".to_string(),
                message: "CSRF token cannot be empty".to_string(),
            });
        }

        if token.len() < self.min_length || token.len() > self.max_length {
            return Err(ValidationError::Format(format!(
                "CSRF token length must be between {} and {} characters",
                self.min_length, self.max_length
            )));
        }

        let parts: Vec<&str> = token.split(':').collect();
        if parts.len() != self.required_parts {
            return Err(ValidationError::Format(
                "Invalid CSRF token format: must contain exactly two parts".to_string(),
            ));
        }

        // Validate token ID (first part)
        if let Some(token_id) = parts.first() {
            if token_id.len() != self.token_id_length
                || !token_id.chars().all(|c| c.is_ascii_hexdigit())
            {
                return Err(ValidationError::Format(format!(
                    "Invalid token ID format. Must be {} hexadecimal characters",
                    self.token_id_length
                )));
            }
        }

        // Validate token value (second part)
        if let Some(token_value) = parts.get(1) {
            if !token_value
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
            {
                return Err(ValidationError::Format(
                    "Token value contains invalid characters".to_string(),
                ));
            }
        }
        Ok(())
    }
}

impl Default for ProxmoxCSRFTokenConfig {
    fn default() -> Self {
        Self {
            min_length: 32,
            max_length: 256,
            required_parts: 2, // TOKENID:VALUE
            token_id_length: 8,
            header_name: "CSRFPreventionToken".to_string(),
            token_lifetime: Duration::from_secs(300),
        }
    }
}

/// Represents a validated Proxmox CSRF prevention token
///
/// This value object ensures CSRF tokens comply with:
/// - Proxmox VE security requirements
/// - OWASP CSRF Prevention Guidelines
/// - Web security best practices
///
/// Token Format: TOKENID:VALUE where:
/// - TOKENID: 8 character hexadecimal identifier
/// - VALUE: Base64 encoded random value
#[derive(Debug, Clone)]
pub struct ProxmoxCSRFToken {
    value: Arc<RwLock<String>>,
    created_at: SystemTime,
}

impl ProxmoxCSRFToken {
    pub async fn new(token: String) -> ProxmoxResult<Self> {
        let token = Self {
            value: Arc::new(RwLock::new(token)),
            created_at: SystemTime::now(),
        };

        let config = Self::validation_config();
        Self::validate(&token.as_inner().await, &config).await?;

        Ok(token)
    }

    pub async fn is_expired(&self) -> bool {
        let config = Self::validation_config();
        SystemTime::now()
            .duration_since(self.created_at)
            .map(|age| age > config.token_lifetime)
            .unwrap_or(true)
    }

    pub async fn as_header(&self) -> String {
        format!(
            "{}: {}",
            Self::validation_config().header_name,
            self.as_inner().await
        )
    }

    pub async fn token_id(&self) -> Option<String> {
        let value = self.as_inner().await;
        value.split(':').next().map(String::from)
    }
}

#[async_trait]
impl ValueObject for ProxmoxCSRFToken {
    type Value = String;
    type ValidationConfig = ProxmoxCSRFTokenConfig;

    fn value(&self) -> &Arc<RwLock<Self::Value>> {
        &self.value
    }

    fn validation_config() -> Self::ValidationConfig {
        ProxmoxCSRFTokenConfig::default()
    }

    async fn validate(
        value: &Self::Value,
        config: &Self::ValidationConfig,
    ) -> Result<(), ValidationError> {
        config.validate_token_format(value).await
    }

    fn create(value: Self::Value) -> Self {
        Self {
            value: Arc::new(RwLock::new(value)),
            created_at: SystemTime::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::domain::error::ProxmoxError;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_valid_tokens() {
        let valid_tokens = vec![
            "4EEC61E2:lwk7od06fa1+DcPUwBTXCcndyAY/3mKxQp5vR8sNjWuBtL9fZg==",
            "8ABC1234:dGhpc2lzYXZhbGlkdG9rZW5mb3J0ZXN0aW5nYmFzZTY0ZW5jb2Rpbmc=",
        ];

        for token in valid_tokens {
            let result = ProxmoxCSRFToken::new(token.to_string()).await;
            assert!(result.is_ok(), "Token should be valid: {}", token);
        }
    }

    #[tokio::test]
    async fn test_invalid_tokens() {
        let test_cases = vec![
            ("", "empty token"),
            ("invalid", "missing format"),
            ("SHORT:token", "invalid token id"),
            ("12345678:", "empty value"),
            ("12345678:invalid@chars", "invalid characters"),
        ];

        for (token, case) in test_cases {
            let result = ProxmoxCSRFToken::new(token.to_string()).await;
            assert!(
                matches!(result, Err(ProxmoxError::Validation { .. })),
                "Case '{}' should fail validation: {}",
                case,
                token
            );
        }
    }

    #[tokio::test]
    async fn test_token_id_extraction() {
        let token = ProxmoxCSRFToken::new(
            "4EEC61E2:lwk7od06fa1+DcPUwBTXCcndyAY/3mKxQp5vR8sNjWuBtL9fZg==".to_string(),
        )
        .await
        .unwrap();

        let token_id = token.token_id().await;
        assert_eq!(token_id, Some("4EEC61E2".to_string()));
    }

    #[tokio::test]
    async fn test_header_format() {
        let token = ProxmoxCSRFToken::new(
            "4EEC61E2:lwk7od06fa1+DcPUwBTXCcndyAY/3mKxQp5vR8sNjWuBtL9fZg==".to_string(),
        )
        .await
        .unwrap();

        let header = token.as_header().await;
        assert!(header.starts_with("CSRFPreventionToken: "));
        assert!(header.contains("4EEC61E2:"));
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let token = ProxmoxCSRFToken::new(
            "4EEC61E2:lwk7od06fa1+DcPUwBTXCcndyAY/3mKxQp5vR8sNjWuBtL9fZg==".to_string(),
        )
        .await
        .unwrap();

        let handles: Vec<_> = (0..10)
            .map(|i| {
                let token_clone = token.clone();
                tokio::spawn(async move {
                    if i % 2 == 0 {
                        sleep(Duration::from_millis(10)).await;
                    }
                    token_clone.as_inner().await
                })
            })
            .collect();

        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.contains("4EEC61E2:"));
        }
    }
}
