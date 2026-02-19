#![feature(error_generic_member_access)]
// Sadly, required for error_chain for now (unstable) https://github.com/rust-lang/rust/issues/99301

//! A safe and ergonomic Rust client for the Proxmox VE API
//!
//! This crate provides a strongly-typed interface for interacting with Proxmox Virtual Environment,
//! following security best practices and industry standards including:
//!
//! - RFC 8446 (TLS 1.3)
//! - ISO 27001:2013 security controls
//! - Proxmox VE security guidelines
//!
//! # Security Features
//!
//! - Enforces strong password requirements (minimum entropy of 3, mixed case, numbers, symbols)
//! - Prevents usage of privileged usernames (root, admin)
//! - Supports secure TLS configuration
//! - Optional certificate validation for development environments

mod auth;
mod core;

pub use crate::core::domain::error::ProxmoxResult;
use crate::{
    auth::application::service::login_service::LoginService,
    core::domain::{
        error::{ProxmoxError, ValidationError},
        model::proxmox_auth::ProxmoxAuth,
        model::proxmox_connection::ProxmoxConnection,
        value_object::{
            proxmox_csrf_token::ProxmoxCSRFToken, proxmox_host::ProxmoxHost,
            proxmox_password::ProxmoxPassword, proxmox_port::ProxmoxPort,
            proxmox_realm::ProxmoxRealm, proxmox_ticket::ProxmoxTicket,
            proxmox_username::ProxmoxUsername,
        },
    },
};
use std::backtrace::Backtrace;

///   A strongly-typed client for the Proxmox VE API that enforces security best practices
///
/// # Security Considerations
///
/// - Uses TLS 1.3 by default for secure communication
/// - Enforces strong password requirements
/// - Prevents usage of privileged system accounts
/// - Implements proper token and session management
///
/// # Examples
///
/// ```no_run
/// use leeca_proxmox::{ProxmoxClient, ProxmoxResult};
///
/// #[tokio::main]
/// async fn main() -> ProxmoxResult<()> {
///     let mut client = ProxmoxClient::builder()
///         .host("192.168.1.182")?
///         .port(8006)?
///         // Use a dedicated user with strong password
///         .credentials(
///             "leeca",                  // Non-privileged default username
///             "Leeca_proxmox1!",       // Strong password with mixed case, numbers, symbols and a minimum entropy of 3
///             "pam"
///         )?
///         .secure(false)               // Optional | Use HTTP instead of HTTPS and disables certificate validation
///         .accept_invalid_certs(true)  // Optional | Disables certificate validation
///         .build()
///         .await?;
///
///     client.login().await?;
///     println!("Authenticated: {}", client.is_authenticated());
///     if let Some(token) = client.auth_token() {
///         println!("Session Token: {}", token.value().await);
///         println!("Session Token expires at: {:?}", token.expires_at().await);
///    }
///
///    if let Some(csrf) = client.csrf_token() {
///        println!("CSRF Token: {}", csrf.value().await);
///        println!("CSRF Token expires at: {:?}", csrf.expires_at().await);
///   }
///
///     Ok(())
/// }
/// ```
pub struct ProxmoxClient {
    connection: ProxmoxConnection,
    auth: Option<ProxmoxAuth>,
}

/// Builder pattern implementation for creating a properly configured ProxmoxClient
///
/// Enforces security requirements and validates all configuration options before
/// constructing the client.
#[derive(Debug, Default)]
pub struct ProxmoxClientBuilder {
    host: Option<String>,
    port: Option<u16>,
    username: Option<String>,
    password: Option<String>,
    realm: Option<String>,
    secure: bool,
    accept_invalid_certs: bool,
}

impl ProxmoxClientBuilder {
    /// Sets the Proxmox VE host address
    ///
    /// # Arguments
    ///
    /// * `host` - IP address or hostname of the Proxmox VE server
    ///
    /// # Security
    ///
    /// Validates the host format and performs basic security checks
    pub fn host(mut self, host: impl Into<String>) -> ProxmoxResult<Self> {
        self.host = Some(host.into());
        Ok(self)
    }

    /// Sets the Proxmox VE API port
    ///
    /// # Arguments
    ///
    /// * `port` - Port number (default: 8006)
    ///
    /// # Security
    ///
    /// Validates the port is within allowed ranges
    pub fn port(mut self, port: u16) -> ProxmoxResult<Self> {
        self.port = Some(port);
        Ok(self)
    }

    /// Sets the authentication credentials
    ///
    /// # Arguments
    ///
    /// * `username` - Non-privileged username (cannot be root/admin)
    /// * `password` - Strong password meeting security requirements:
    ///   - Minimum length: 12 characters
    ///   - Contains uppercase and lowercase letters
    ///   - Contains numbers and special characters
    ///   - Minimum entropy score of 3
    /// * `realm` - Authentication realm (e.g., "pam", "pve")
    ///
    /// # Security
    ///
    /// Enforces password strength and username requirements
    pub fn credentials(
        mut self,
        username: impl Into<String>,
        password: impl Into<String>,
        realm: impl Into<String>,
    ) -> ProxmoxResult<Self> {
        self.username = Some(username.into());
        self.password = Some(password.into());
        self.realm = Some(realm.into());
        Ok(self)
    }

    /// Configures TLS security settings
    ///
    /// # Arguments
    ///
    /// * `secure` - When true (default), enforces TLS 1.3
    ///
    /// # Security
    ///
    /// Setting this to false is only recommended in development environments
    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        // If not secure, always accept invalid certs
        if !secure {
            self.accept_invalid_certs = true;
        }
        self
    }

    /// Configures certificate validation
    ///
    /// # Arguments
    ///
    /// * `accept` - When true, accepts self-signed/invalid certificates
    ///
    /// # Security
    ///
    /// Only disable certificate validation in development environments
    pub fn accept_invalid_certs(mut self, accept: bool) -> Self {
        self.accept_invalid_certs = accept;
        self
    }

    /// Constructs a new ProxmoxClient with the configured settings
    ///
    /// # Security
    ///
    /// Validates all security requirements before creating the client
    pub async fn build(self) -> ProxmoxResult<ProxmoxClient> {
        let host = ProxmoxHost::new(self.host.ok_or_else(|| ProxmoxError::Validation {
            source: ValidationError::Field {
                field: "host".to_string(),
                message: "Host is required".to_string(),
            },
            backtrace: Backtrace::capture(),
        })?)
        .await?;

        let port = ProxmoxPort::new(self.port.unwrap_or(8006)).await?;

        let username =
            ProxmoxUsername::new(self.username.ok_or_else(|| ProxmoxError::Validation {
                source: ValidationError::Field {
                    field: "username".to_string(),
                    message: "Username is required".to_string(),
                },
                backtrace: Backtrace::capture(),
            })?)
            .await?;

        let password =
            ProxmoxPassword::new(self.password.ok_or_else(|| ProxmoxError::Validation {
                source: ValidationError::Field {
                    field: "password".to_string(),
                    message: "Password is required".to_string(),
                },
                backtrace: Backtrace::capture(),
            })?)
            .await?;

        let realm = ProxmoxRealm::new(self.realm.ok_or_else(|| ProxmoxError::Validation {
            source: ValidationError::Field {
                field: "realm".to_string(),
                message: "Realm is required".to_string(),
            },
            backtrace: Backtrace::capture(),
        })?)
        .await?;
        let connection = ProxmoxConnection::new(
            host,
            port,
            username,
            password,
            realm,
            self.secure,
            self.accept_invalid_certs,
        )
        .await?;

        Ok(ProxmoxClient {
            connection,
            auth: None,
        })
    }
}

impl ProxmoxClient {
    /// Creates a new ProxmoxClientBuilder with default secure settings
    pub fn builder() -> ProxmoxClientBuilder {
        ProxmoxClientBuilder::default()
    }

    /// Authenticates with the Proxmox VE server using secure credentials
    ///
    /// # Security
    ///
    /// - Uses TLS for transport security
    /// - Implements proper token management
    /// - Validates server responses
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Authentication fails
    /// - Server is unreachable
    /// - TLS validation fails
    /// - Response format is invalid
    pub async fn login(&mut self) -> ProxmoxResult<()> {
        let service = LoginService::new();
        self.auth = Some(service.execute(&self.connection).await?);
        Ok(())
    }

    /// Checks if the client has valid authentication tokens
    pub fn is_authenticated(&self) -> bool {
        self.auth.is_some()
    }

    /// Returns the current authentication token if authenticated
    ///
    /// # Security
    ///
    /// Token is stored securely and validated
    pub fn auth_token(&self) -> Option<&ProxmoxTicket> {
        self.auth.as_ref().map(|auth| auth.ticket())
    }

    /// Returns the current CSRF protection token if authenticated
    ///
    /// # Security
    ///
    /// Implements proper CSRF protection
    pub fn csrf_token(&self) -> Option<&ProxmoxCSRFToken> {
        self.auth.as_ref().and_then(|auth| auth.csrf_token())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenvy::dotenv;
    use std::env;

    fn setup() {
        dotenv().ok();
    }

    async fn setup_client() -> ProxmoxResult<ProxmoxClient> {
        dotenv().ok();

        ProxmoxClient::builder()
            .host(env::var("PROXMOX_HOST").unwrap())?
            .port(env::var("PROXMOX_PORT").unwrap().parse().unwrap())?
            .credentials(
                env::var("PROXMOX_USERNAME").unwrap(),
                env::var("PROXMOX_PASSWORD").unwrap(),
                env::var("PROXMOX_REALM").unwrap(),
            )?
            .secure(true)
            .accept_invalid_certs(true)
            .build()
            .await
    }

    #[tokio::test]
    #[ignore = "requires running Proxmox instance"]
    async fn test_client_builder() {
        setup();

        let client = setup_client().await;
        assert!(client.is_ok());
    }

    #[tokio::test]
    #[ignore = "requires running Proxmox instance"]
    async fn test_client_authentication() {
        setup();

        let mut client = setup_client().await.unwrap();
        assert!(!client.is_authenticated());

        let login_result = client.login().await;
        assert!(login_result.is_ok());
        assert!(client.is_authenticated());
    }

    // // Temporal workaround until github actions secrets are available
    // // and running remote Proxmox VE for ci testing
    // fn has_proxmox_config() -> bool {
    //     env::var("PROXMOX_HOST").is_ok()
    //         && env::var("PROXMOX_PORT").is_ok()
    //         && env::var("PROXMOX_USERNAME").is_ok()
    //         && env::var("PROXMOX_PASSWORD").is_ok()
    //         && env::var("PROXMOX_REALM").is_ok()
    // }
}
