#![feature(error_generic_member_access)] // Sadly, required for error_chain for now (unstable) https://github.com/rust-lang/rust/issues/99301

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

/// A Client for interacting with the Proxmox VE API
///
/// This client provides a safe, ergonomic interface for:
/// - Authentication and session management
/// - Resource operations (nodes, VMs, containers)
/// - Task monitoring and management
///
/// # Examples
///
/// ```no_run
/// use leeca_proxmox::{ProxmoxClient, ProxmoxResult};
///
/// #[tokio::main]
/// async fn main() -> ProxmoxResult<()> {
///     let mut client = ProxmoxClient::builder()
///         .host("proxmox.example.com")?
///         .port(8006)?
///         .credentials("user", "password", "pve")?
///         .secure(true)
///         .build()
///         .await?;
///
///     client.login().await?;
///     Ok(())
/// }
/// ```
pub struct ProxmoxClient {
    connection: ProxmoxConnection,
    auth: Option<ProxmoxAuth>,
}

/// Builder for ProxmoxClient configuration
#[derive(Debug, Default)]
pub struct ProxmoxClientBuilder {
    host: Option<String>,
    port: Option<u16>,
    username: Option<String>,
    password: Option<String>,
    realm: Option<String>,
    secure: bool,
}

impl ProxmoxClientBuilder {
    pub fn host(mut self, host: impl Into<String>) -> ProxmoxResult<Self> {
        self.host = Some(host.into());
        Ok(self)
    }

    pub fn port(mut self, port: u16) -> ProxmoxResult<Self> {
        self.port = Some(port);
        Ok(self)
    }

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

    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

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

        let connection =
            ProxmoxConnection::new(host, port, username, password, realm, self.secure).await?;

        Ok(ProxmoxClient {
            connection,
            auth: None,
        })
    }
}

impl ProxmoxClient {
    /// Creates a new builder for ProxmoxClient configuration
    pub fn builder() -> ProxmoxClientBuilder {
        ProxmoxClientBuilder::default()
    }

    /// Authenticates with the Proxmox server
    ///
    /// This method:
    /// - Creates a login request with the client's credentials
    /// - Sends the request to the Proxmox server
    /// - Handles the authentication response
    /// - Stores the authentication tokens for future requests
    ///
    /// # Returns
    ///
    /// * `Ok(())` if authentication succeeds
    /// * `Err(ProxmoxError)` if authentication fails
    ///
    /// # Errors
    ///
    /// This method will return an error if:
    /// - The credentials are invalid
    /// - The server is unreachable
    /// - The response format is invalid
    /// - The server returns an unexpected status code
    pub async fn login(&mut self) -> ProxmoxResult<()> {
        let service = LoginService::new();
        self.auth = Some(service.execute(&self.connection).await?);
        Ok(())
    }

    /// Returns true if the client is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.auth.is_some()
    }

    /// Returns the current authentication token if authenticated
    pub fn auth_token(&self) -> Option<&ProxmoxTicket> {
        self.auth.as_ref().map(|auth| auth.ticket())
    }

    /// Returns the current CSRF token if authenticated
    pub fn csrf_token(&self) -> Option<&ProxmoxCSRFToken> {
        self.auth.as_ref().and_then(|auth| auth.csrf_token())
    }
}

// TBD: See how to test this without actually connecting to a Proxmox server

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[tokio::test]
//     async fn test_client_builder() {
//         let client = ProxmoxClient::builder()
//             .host("localhost")
//             .unwrap()
//             .port(8006)
//             .unwrap()
//             .credentials("user", "password", "pve")
//             .unwrap()
//             .secure(false)
//             .build()
//             .await;

//         assert!(client.is_ok());
//     }

//     #[tokio::test]
//     async fn test_client_authentication() {
//         let mut client = ProxmoxClient::builder()
//             .host("proxmox.example.com")
//             .unwrap()
//             .credentials("user", "password", "pve")
//             .unwrap()
//             .build()
//             .await
//             .unwrap();

//         assert!(!client.is_authenticated());

//         let login_result = client.login().await;
//         assert!(login_result.is_ok());
//         assert!(client.is_authenticated());
//     }
// }
