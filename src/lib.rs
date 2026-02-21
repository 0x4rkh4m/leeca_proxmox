#![feature(error_generic_member_access)]

//! A safe and ergonomic Rust client for the Proxmox VE API.
//!
//! This crate provides a strongly-typed interface for interacting with Proxmox Virtual Environment.
//! Validation rules (like password strength, DNS resolution) are **opt-in** via a [`ValidationConfig`].
//! By default, only basic format checks are performed to ensure the values can be used in HTTP requests.
//!
//! # Example
//! ```no_run
//! use leeca_proxmox::{ProxmoxClient, ProxmoxResult};
//!
//! #[tokio::main]
//! async fn main() -> ProxmoxResult<()> {
//!     let mut client = ProxmoxClient::builder()
//!         .host("192.168.1.182")
//!         .port(8006)
//!         .secure(false) // HTTP for local development
//!         .accept_invalid_certs(true) // Testing & Self signed certs
//!         .enable_password_strength(3) // Optional validation
//!         .block_reserved_usernames() // Optional validation
//!         .build()
//!         .await?;
//!
//!     client.login().await?;
//!     println!("Authenticated: {}", client.is_authenticated().await);
//!     Ok(())
//! }
//! ```

mod auth;
mod core;

pub use crate::core::domain::error::{ProxmoxError, ProxmoxResult, ValidationError};

use crate::{
    auth::application::service::login_service::LoginService,
    core::{
        domain::{
            model::{proxmox_auth::ProxmoxAuth, proxmox_connection::ProxmoxConnection},
            value_object::{
                ProxmoxCSRFToken, ProxmoxHost, ProxmoxPassword, ProxmoxPort, ProxmoxRealm,
                ProxmoxTicket, ProxmoxUrl, ProxmoxUsername, validate_host, validate_password,
                validate_port, validate_realm, validate_url, validate_username,
            },
        },
        infrastructure::api_client::ApiClient,
    },
};

use std::backtrace::Backtrace;
use std::io::Read;
use std::time::Duration;

/// Configuration for rate limiting.
#[derive(Debug, Clone, Copy)]
pub struct RateLimitConfig {
    /// Number of requests allowed per second (steady state).
    pub requests_per_second: u32,
    /// Maximum burst size (how many requests can be sent in a short burst).
    pub burst_size: u32,
}

/// Configuration for validating client inputs.
///
/// By default, all extra checks are disabled, meaning only basic format validation is performed.
/// You can enable specific checks by calling the corresponding builder methods.
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Minimum password strength (zxcvbn score 0-4). If `None`, password strength is not checked.
    pub password_min_score: Option<zxcvbn::Score>,
    /// If true, DNS resolution is performed for hostnames.
    pub resolve_dns: bool,
    /// If true, reserved usernames (root, admin, etc.) are rejected.
    pub block_reserved_usernames: bool,
    /// Ticket lifetime for expiration checks (default 2 hours).
    pub ticket_lifetime: Duration,
    /// CSRF token lifetime (default 5 minutes).
    pub csrf_lifetime: Duration,
    /// Optional rate limiting configuration. If `None`, no rate limiting is applied.
    pub rate_limit: Option<RateLimitConfig>,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            password_min_score: None,
            resolve_dns: false,
            block_reserved_usernames: false,
            ticket_lifetime: Duration::from_secs(7200),
            csrf_lifetime: Duration::from_secs(300),
            rate_limit: None, // default: no limiting
        }
    }
}

/// A strongly-typed client for the Proxmox VE API.
///
/// Use the builder to configure connection settings and validation rules.
#[derive(Debug)]
pub struct ProxmoxClient {
    api_client: ApiClient,
    config: ValidationConfig,
}

/// Builder for [`ProxmoxClient`].
#[derive(Debug)]
pub struct ProxmoxClientBuilder {
    host: Option<String>,
    port: Option<u16>,
    username: Option<String>,
    password: Option<String>,
    realm: Option<String>,
    secure: bool,
    accept_invalid_certs: bool,
    config: ValidationConfig,
    initial_auth: Option<ProxmoxAuth>,
}

impl Default for ProxmoxClientBuilder {
    fn default() -> Self {
        Self {
            host: None,
            port: None,
            username: None,
            password: None,
            realm: None,
            secure: true, // Default to HTTPS
            accept_invalid_certs: false,
            config: ValidationConfig::default(),
            initial_auth: None,
        }
    }
}

impl ProxmoxClientBuilder {
    /// Sets the Proxmox VE host address.
    #[must_use]
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    /// Sets the Proxmox VE API port (default 8006).
    #[must_use]
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Sets the authentication credentials.
    #[must_use]
    pub fn credentials(
        mut self,
        username: impl Into<String>,
        password: impl Into<String>,
        realm: impl Into<String>,
    ) -> Self {
        self.username = Some(username.into());
        self.password = Some(password.into());
        self.realm = Some(realm.into());
        self
    }

    /// Configures TLS security: `true` for HTTPS, `false` for HTTP.
    ///
    /// When `false`, certificate validation is also disabled for convenience.
    #[must_use]
    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        if !secure {
            self.accept_invalid_certs = true;
        }
        self
    }

    /// Configures certificate validation: `true` accepts invalid/self-signed certificates.
    #[must_use]
    pub fn accept_invalid_certs(mut self, accept: bool) -> Self {
        self.accept_invalid_certs = accept;
        self
    }

    /// Replaces the validation configuration with a custom one.
    #[must_use]
    pub fn with_validation_config(mut self, config: ValidationConfig) -> Self {
        self.config = config;
        self
    }

    /// Enables password strength checking with a minimum score (0-4).
    #[must_use]
    pub fn enable_password_strength(mut self, min_score: u8) -> Self {
        self.config.password_min_score = Some(match min_score {
            0 => zxcvbn::Score::Zero,
            1 => zxcvbn::Score::One,
            2 => zxcvbn::Score::Two,
            3 => zxcvbn::Score::Three,
            4 => zxcvbn::Score::Four,
            _ => zxcvbn::Score::Three,
        });
        self
    }

    /// Enables DNS resolution for hostnames (may block during build).
    #[must_use]
    pub fn enable_dns_resolution(mut self) -> Self {
        self.config.resolve_dns = true;
        self
    }

    /// Blocks reserved usernames (root, admin, etc.).
    #[must_use]
    pub fn block_reserved_usernames(mut self) -> Self {
        self.config.block_reserved_usernames = true;
        self
    }

    /// Sets client‑side rate limiting: `requests_per_second` and `burst_size`.
    #[must_use]
    pub fn rate_limit(mut self, requests_per_second: u32, burst_size: u32) -> Self {
        self.config.rate_limit = Some(RateLimitConfig {
            requests_per_second,
            burst_size,
        });
        self
    }

    /// Load an authentication state from a reader and use it as the initial auth.
    /// The tokens will be validated for expiration. Returns an error if the data is malformed
    /// or if the tokens are already expired according to the client's validation config.
    pub async fn with_session<R: Read>(mut self, mut reader: R) -> ProxmoxResult<Self> {
        let mut data = String::new();
        reader.read_to_string(&mut data)?;
        let auth: ProxmoxAuth = serde_json::from_str(&data)?;
        // Validate expiration
        if let Some(csrf) = auth.csrf_token()
            && csrf.is_expired(self.config.csrf_lifetime)
        {
            return Err(ProxmoxError::Session(
                "Loaded CSRF token is expired".to_string(),
            ));
        }
        if auth.ticket().is_expired(self.config.ticket_lifetime) {
            return Err(ProxmoxError::Session(
                "Loaded ticket is expired".to_string(),
            ));
        }
        self.initial_auth = Some(auth);
        Ok(self)
    }

    /// Constructs a [`ProxmoxClient`] after validating all inputs according to the configuration.
    pub async fn build(self) -> ProxmoxResult<ProxmoxClient> {
        // Extract required fields
        let host_str = self.host.ok_or_else(|| ProxmoxError::Validation {
            source: ValidationError::Field {
                field: "host".to_string(),
                message: "Host is required".to_string(),
            },
            backtrace: Backtrace::capture(),
        })?;
        let port_num = self.port.unwrap_or(8006);
        let username_str = self.username.ok_or_else(|| ProxmoxError::Validation {
            source: ValidationError::Field {
                field: "username".to_string(),
                message: "Username is required".to_string(),
            },
            backtrace: Backtrace::capture(),
        })?;
        let password_str = self.password.ok_or_else(|| ProxmoxError::Validation {
            source: ValidationError::Field {
                field: "password".to_string(),
                message: "Password is required".to_string(),
            },
            backtrace: Backtrace::capture(),
        })?;
        let realm_str = self.realm.ok_or_else(|| ProxmoxError::Validation {
            source: ValidationError::Field {
                field: "realm".to_string(),
                message: "Realm is required".to_string(),
            },
            backtrace: Backtrace::capture(),
        })?;

        // Perform validation
        validate_host(&host_str, self.config.resolve_dns).map_err(|e| {
            ProxmoxError::Validation {
                source: e,
                backtrace: Backtrace::capture(),
            }
        })?;
        validate_port(port_num).map_err(|e| ProxmoxError::Validation {
            source: e,
            backtrace: Backtrace::capture(),
        })?;
        validate_username(&username_str, self.config.block_reserved_usernames).map_err(|e| {
            ProxmoxError::Validation {
                source: e,
                backtrace: Backtrace::capture(),
            }
        })?;
        validate_password(&password_str, self.config.password_min_score).map_err(|e| {
            ProxmoxError::Validation {
                source: e,
                backtrace: Backtrace::capture(),
            }
        })?;
        validate_realm(&realm_str).map_err(|e| ProxmoxError::Validation {
            source: e,
            backtrace: Backtrace::capture(),
        })?;

        // Construct URL
        let scheme = if self.secure { "https" } else { "http" };
        let url_str = format!("{}://{}:{}/", scheme, host_str, port_num);
        validate_url(&url_str).map_err(|e| ProxmoxError::Validation {
            source: e,
            backtrace: Backtrace::capture(),
        })?;

        // Create value objects (unchecked, already validated)
        let host = ProxmoxHost::new_unchecked(host_str);
        let port = ProxmoxPort::new_unchecked(port_num);
        let username = ProxmoxUsername::new_unchecked(username_str);
        let password = ProxmoxPassword::new_unchecked(password_str);
        let realm = ProxmoxRealm::new_unchecked(realm_str);
        let url = ProxmoxUrl::new_unchecked(url_str);

        let connection = ProxmoxConnection::new(
            host,
            port,
            username,
            password,
            realm,
            self.secure,
            self.accept_invalid_certs,
            url,
        );

        let api_client = ApiClient::new(connection, self.config.clone())?;
        if let Some(auth) = self.initial_auth {
            api_client.set_auth(auth).await;
        }

        Ok(ProxmoxClient {
            api_client,
            config: self.config,
        })
    }
}

impl ProxmoxClient {
    /// Creates a new builder with default settings.
    #[must_use]
    pub fn builder() -> ProxmoxClientBuilder {
        ProxmoxClientBuilder::default()
    }

    /// Authenticates with the Proxmox VE server.
    ///
    /// This method performs a login using the credentials provided during builder construction
    /// and stores the obtained ticket and CSRF token inside the client.
    pub async fn login(&mut self) -> ProxmoxResult<()> {
        let service = LoginService::new();
        let auth = service.execute(self.api_client.connection()).await?;
        self.api_client.set_auth(auth).await;
        Ok(())
    }

    /// Returns `true` if the client has a valid (non‑expired) authentication ticket.
    pub async fn is_authenticated(&self) -> bool {
        self.api_client.is_authenticated().await
    }

    /// Returns the authentication ticket, if any.
    pub async fn auth_token(&self) -> Option<ProxmoxTicket> {
        self.api_client.auth().await.map(|a| a.ticket().clone())
    }

    /// Returns the CSRF token, if any.
    pub async fn csrf_token(&self) -> Option<ProxmoxCSRFToken> {
        self.api_client
            .auth()
            .await
            .and_then(|a| a.csrf_token().cloned())
    }

    /// Checks if the stored ticket is expired according to the configured lifetime.
    pub async fn is_ticket_expired(&self) -> bool {
        if let Some(auth) = self.api_client.auth().await {
            auth.ticket().is_expired(self.config.ticket_lifetime)
        } else {
            true
        }
    }

    /// Checks if the stored CSRF token is expired according to the configured lifetime.
    pub async fn is_csrf_expired(&self) -> bool {
        if let Some(auth) = self.api_client.auth().await {
            auth.csrf_token()
                .map(|c| c.is_expired(self.config.csrf_lifetime))
                .unwrap_or(true)
        } else {
            true
        }
    }

    /// Serializes the current authentication state (if any) and saves it to a file.
    /// Returns the number of bytes written.
    pub async fn save_session_to_file<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> ProxmoxResult<usize> {
        let auth = match self.api_client.auth().await {
            Some(auth) => auth,
            None => return Ok(0), // no auth to save
        };
        let json = serde_json::to_string(&auth)?;
        tokio::fs::write(path, &json).await?;
        Ok(json.len())
    }

    /// Loads an authentication state from a file and sets it as the current auth.
    /// Returns an error if the data is malformed or if the tokens are already expired
    /// (according to the client's validation config).
    pub async fn load_session_from_file<P: AsRef<std::path::Path>>(
        &mut self,
        path: P,
    ) -> ProxmoxResult<()> {
        let data = tokio::fs::read_to_string(path).await?;
        let auth: ProxmoxAuth = serde_json::from_str(&data)?;
        // Validate expiration
        if let Some(csrf) = auth.csrf_token()
            && csrf.is_expired(self.config.csrf_lifetime)
        {
            return Err(ProxmoxError::Session(
                "Loaded CSRF token is expired".to_string(),
            ));
        }
        if auth.ticket().is_expired(self.config.ticket_lifetime) {
            return Err(ProxmoxError::Session(
                "Loaded ticket is expired".to_string(),
            ));
        }
        self.api_client.set_auth(auth).await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    mod integration;
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_builder_default_secure() {
        let builder = ProxmoxClientBuilder::default();
        assert!(builder.secure);
        assert!(!builder.accept_invalid_certs);
    }

    #[tokio::test]
    async fn test_builder_missing_host() {
        let builder = ProxmoxClientBuilder::default()
            .port(8006)
            .credentials("user", "pass", "pam");
        let err = builder.build().await.unwrap_err();
        assert!(
            matches!(err, ProxmoxError::Validation { source: ValidationError::Field { field, .. }, .. } if field == "host")
        );
    }

    #[tokio::test]
    async fn test_builder_missing_username() {
        let builder = ProxmoxClientBuilder::default()
            .host("example.com")
            .port(8006);
        let err = builder.build().await.unwrap_err();
        assert!(
            matches!(err, ProxmoxError::Validation { source: ValidationError::Field { field, .. }, .. } if field == "username")
        );
    }

    #[tokio::test]
    async fn test_builder_valid_minimal() {
        let client = ProxmoxClientBuilder::default()
            .host("example.com")
            .port(8006)
            .credentials("user", "password123", "pam")
            .build()
            .await
            .unwrap();
        assert!(!client.is_authenticated().await);
        assert!(client.auth_token().await.is_none());
        assert!(client.csrf_token().await.is_none());
        // No ticket/CSRF => they are considered expired
        assert!(client.is_ticket_expired().await);
        assert!(client.is_csrf_expired().await);
    }

    #[tokio::test]
    async fn test_builder_with_validation_config() {
        let config = ValidationConfig {
            password_min_score: Some(zxcvbn::Score::Three),
            resolve_dns: true,
            block_reserved_usernames: true,
            ..Default::default()
        };
        // Use a password that meets length but is weak
        let builder = ProxmoxClientBuilder::default()
            .host("example.com")
            .port(8006)
            .credentials("user", "password", "pam") // length 8, weak
            .with_validation_config(config.clone());
        let err = builder.build().await.unwrap_err();
        assert!(matches!(
            err,
            ProxmoxError::Validation {
                source: ValidationError::ConstraintViolation(_),
                ..
            }
        ));

        // Now with strong password
        let builder = ProxmoxClientBuilder::default()
            .host("example.com")
            .port(8006)
            .credentials("user", "Str0ng!P@ss", "pam")
            .with_validation_config(config);
        assert!(builder.build().await.is_ok());
    }

    #[tokio::test]
    async fn test_client_login_no_auth() {
        let client = ProxmoxClientBuilder::default()
            .host("example.com")
            .port(8006)
            .credentials("user", "password123", "pam")
            .build()
            .await
            .unwrap();
        assert!(!client.is_authenticated().await);
    }

    #[tokio::test]
    async fn test_builder_enable_methods() {
        let builder = ProxmoxClientBuilder::default()
            .host("example.com")
            .port(8006)
            .credentials("root", "password", "pam") // length 8, weak password
            .enable_password_strength(3)
            .enable_dns_resolution()
            .block_reserved_usernames();
        // Should fail because password weak and username reserved
        let err = builder.build().await.unwrap_err();
        assert!(matches!(err, ProxmoxError::Validation { .. }));
    }

    #[test]
    fn test_validation_config_default() {
        let config = ValidationConfig::default();
        assert_eq!(config.password_min_score, None);
        assert!(!config.resolve_dns);
        assert!(!config.block_reserved_usernames);
        assert_eq!(config.ticket_lifetime, Duration::from_secs(7200));
        assert_eq!(config.csrf_lifetime, Duration::from_secs(300));
    }

    // Test expiration methods with mocked auth
    #[tokio::test]
    async fn test_expiration_checks() {
        use crate::core::domain::model::proxmox_auth::ProxmoxAuth;
        use crate::core::domain::value_object::{ProxmoxCSRFToken, ProxmoxTicket};

        let ticket = ProxmoxTicket::new_unchecked("PVE:ticket".to_string());
        let csrf = ProxmoxCSRFToken::new_unchecked("id:val".to_string());
        let auth = ProxmoxAuth::new(ticket, Some(csrf));

        // Build a client with a dummy connection (won't be used)
        let connection = ProxmoxConnection::new(
            ProxmoxHost::new_unchecked("host".to_string()),
            ProxmoxPort::new_unchecked(8006),
            ProxmoxUsername::new_unchecked("user".to_string()),
            ProxmoxPassword::new_unchecked("pass".to_string()),
            ProxmoxRealm::new_unchecked("pam".to_string()),
            true,
            false,
            ProxmoxUrl::new_unchecked("https://host:8006/".to_string()),
        );
        let api_client = ApiClient::new(connection, ValidationConfig::default()).unwrap();
        api_client.set_auth(auth).await;

        let client = ProxmoxClient {
            api_client,
            config: ValidationConfig::default(),
        };

        assert!(!client.is_ticket_expired().await);
        assert!(!client.is_csrf_expired().await);
    }

    #[tokio::test]
    async fn test_session_save_load() {
        use crate::core::domain::model::proxmox_auth::ProxmoxAuth;
        use crate::core::domain::value_object::{ProxmoxCSRFToken, ProxmoxTicket};
        use tempfile::NamedTempFile;

        let ticket = ProxmoxTicket::new_unchecked("PVE:ticket".to_string());
        let csrf = ProxmoxCSRFToken::new_unchecked("id:val".to_string());
        let auth = ProxmoxAuth::new(ticket.clone(), Some(csrf.clone()));

        // Build a client with dummy connection
        let connection = ProxmoxConnection::new(
            ProxmoxHost::new_unchecked("host".to_string()),
            ProxmoxPort::new_unchecked(8006),
            ProxmoxUsername::new_unchecked("user".to_string()),
            ProxmoxPassword::new_unchecked("pass".to_string()),
            ProxmoxRealm::new_unchecked("pam".to_string()),
            true,
            false,
            ProxmoxUrl::new_unchecked("https://host:8006/".to_string()),
        );
        let api_client = ApiClient::new(connection, ValidationConfig::default()).unwrap();
        api_client.set_auth(auth).await;

        let client = ProxmoxClient {
            api_client,
            config: ValidationConfig::default(),
        };

        // Save to temp file
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();
        let written = client.save_session_to_file(&path).await.unwrap();
        assert!(written > 0);

        // Create a new client and load session
        let new_client_builder = ProxmoxClient::builder()
            .host("host")
            .port(8006)
            .credentials("user", "password", "pam")
            .secure(false)
            .accept_invalid_certs(false);
        let new_client = new_client_builder
            .with_session(std::fs::File::open(&path).unwrap())
            .await
            .unwrap()
            .build()
            .await
            .unwrap();

        assert!(new_client.is_authenticated().await);
        assert_eq!(
            client.auth_token().await.unwrap().as_str(),
            new_client.auth_token().await.unwrap().as_str()
        );
    }
}
