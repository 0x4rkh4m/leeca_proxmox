use crate::core::domain::{
    error::{ProxmoxResult, ValidationError},
    value_object::{
        base_value_object::ValueObject, proxmox_realm::ProxmoxRealm,
        proxmox_realm::ProxmoxRealmConfig, proxmox_username::ProxmoxUsername,
        proxmox_username::ProxmoxUsernameConfig,
    },
};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use tokio::time::Duration;

/// Represents the configuration for a Proxmox ticket value object
///
/// This configuration object encapsulates the constraints and settings
/// for ticket validation according to:
/// - Proxmox VE Authentication Standards
/// - RFC 6749 (OAuth 2.0)
/// - Security best practices for session management
///
/// # Examples
///
/// ```
/// use leeca_proxmox::core::domain::value_object::proxmox_ticket::ProxmoxTicketConfig;
///
/// let config = ProxmoxTicketConfig::default();
/// ```
#[derive(Debug, Clone)]
pub struct ProxmoxTicketConfig {
    ticket_lifetime: Duration,
    min_length: usize,
    max_length: usize,
    required_parts: usize,
    allowed_realms: ProxmoxRealmConfig,
    allowed_usernames: ProxmoxUsernameConfig,
}

impl ProxmoxTicketConfig {
    async fn validate_ticket_format(&self, ticket: &str) -> Result<(), ValidationError> {
        if ticket.is_empty() {
            return Err(ValidationError::FieldError {
                field: "ticket".to_string(),
                message: "Ticket cannot be empty".to_string(),
            });
        }

        if ticket.len() < self.min_length || ticket.len() > self.max_length {
            return Err(ValidationError::FormatError(format!(
                "Ticket length must be between {} and {} characters",
                self.min_length, self.max_length
            )));
        }

        let parts: Vec<&str> = ticket.split(':').collect();
        if parts.len() < self.required_parts {
            return Err(ValidationError::FormatError(
                "Invalid ticket format: missing required parts".to_string(),
            ));
        }

        if let Some(user_realm) = parts.get(1) {
            let ur_parts: Vec<&str> = user_realm.split('@').collect();
            if ur_parts.len() != 2 {
                return Err(ValidationError::FormatError(
                    "Invalid user@realm format".to_string(),
                ));
            }

            self.allowed_usernames
                .validate_username(ur_parts[0])
                .await?;

            self.allowed_realms.validate_realm(ur_parts[1]).await?;
        }

        if let Some(ticket_id) = parts.get(2) {
            if ticket_id.len() != 8 || !ticket_id.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(ValidationError::FormatError(
                    "Invalid ticket ID format".to_string(),
                ));
            }
        }

        Ok(())
    }

    pub async fn extract_credentials(
        &self,
        ticket: &str,
    ) -> Result<(ProxmoxUsername, ProxmoxRealm), ValidationError> {
        let parts: Vec<&str> = ticket.split(':').collect();
        if let Some(user_realm) = parts.get(1) {
            let ur_parts: Vec<&str> = user_realm.split('@').collect();
            if ur_parts.len() == 2 {
                let username = ProxmoxUsername::new(ur_parts[0].to_string()).await;
                let realm = ProxmoxRealm::new(ur_parts[1].to_string()).await;
                return Ok((username.unwrap(), realm.unwrap()));
            }
        }
        Err(ValidationError::FormatError(
            "Invalid user@realm format".to_string(),
        ))
    }

    fn is_expired(&self, created_at: SystemTime) -> bool {
        created_at
            .elapsed()
            .map(|elapsed| elapsed > self.ticket_lifetime)
            .unwrap_or(true)
    }
}

impl Default for ProxmoxTicketConfig {
    fn default() -> Self {
        Self {
            ticket_lifetime: Duration::from_secs(7200), // 2 hours as per Proxmox VE spec
            min_length: 32,
            max_length: 512,
            required_parts: 5, // PVE:user@realm:ticketid::signature
            allowed_realms: ProxmoxRealmConfig::default(),
            allowed_usernames: ProxmoxUsernameConfig::default(),
        }
    }
}

/// Represents a validated Proxmox authentication ticket
///
/// This value object ensures tickets comply with:
/// - Proxmox VE ticket format requirements
/// - Security standards for authentication tokens
/// - Proper lifetime management
///
/// Ticket Format: PVE:USER@REALM:TICKETID::SIGNATURE
///
/// # Examples
///
/// ```
/// use leeca_proxmox::core::domain::value_object::proxmox_ticket::ProxmoxTicket;
///
/// #[tokio::main]
/// async fn main() {
///    let ticket = ProxmoxTicket::new("PVE:root@pam:4EEC61E2::validticket".to_string())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ProxmoxTicket {
    value: Arc<RwLock<String>>,
    created_at: SystemTime,
}

impl ProxmoxTicket {
    pub async fn new(ticket: String) -> ProxmoxResult<Self> {
        let ticket = Self {
            value: Arc::new(RwLock::new(ticket)),
            created_at: SystemTime::now(),
        };

        let config = Self::validation_config();
        Self::validate(&ticket.as_inner().await, &config).await;

        Ok(ticket)
    }

    pub async fn is_expired(&self) -> bool {
        Self::validation_config().is_expired(self.created_at)
    }

    pub async fn as_cookie_header(&self) -> String {
        format!("PVEAuthCookie={}", self.as_inner().await)
    }

    pub async fn extract_user_realm(&self) -> Option<(String, String)> {
        let value = self.as_inner().await;
        let parts: Vec<&str> = value.split(':').collect();

        parts.get(1).and_then(|user_realm| {
            let ur_parts: Vec<&str> = user_realm.split('@').collect();
            if ur_parts.len() == 2 {
                Some((ur_parts[0].to_string(), ur_parts[1].to_string()))
            } else {
                None
            }
        })
    }
}

#[async_trait]
impl ValueObject for ProxmoxTicket {
    type Value = String;
    type ValidationConfig = ProxmoxTicketConfig;

    fn value(&self) -> &Arc<RwLock<Self::Value>> {
        &self.value
    }

    fn validation_config() -> Self::ValidationConfig {
        ProxmoxTicketConfig::default()
    }

    async fn validate(
        value: &Self::Value,
        config: &Self::ValidationConfig,
    ) -> Result<(), ValidationError> {
        config.validate_ticket_format(value).await
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
    use crate::core::domain::error::ProxmoxError::ValidationError;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_valid_tickets() {
        let valid_tickets = vec![
            "PVE:root@pam:4EEC61E2::rsKoApxDTLYPn6H3NNT6iP2mv",
            "PVE:admin@pve:8ABC1234::validticketstring",
        ];

        for ticket in valid_tickets {
            let result = ProxmoxTicket::new(ticket.to_string()).await;
            assert!(result.is_ok(), "Ticket should be valid: {}", ticket);
        }
    }

    #[tokio::test]
    async fn test_invalid_tickets() {
        let test_cases = vec![
            ("", "empty ticket"),
            ("invalid", "missing format"),
            ("WRONG:root@pam:ticket", "wrong prefix"),
            ("PVE:root:4EEC61E2::sig", "missing realm"),
            ("PVE:root@pam:SHORT::sig", "invalid ticket id"),
        ];

        for (ticket, case) in test_cases {
            let result = ProxmoxTicket::new(ticket.to_string()).await;
            assert!(
                matches!(result, Err(ValidationError { .. })),
                "Case '{}' should fail validation: {}",
                case,
                ticket
            );
        }
    }

    #[tokio::test]
    async fn test_ticket_expiration() {
        let config = ProxmoxTicketConfig {
            ticket_lifetime: Duration::from_millis(100),
            ..Default::default()
        };

        let ticket = ProxmoxTicket::new("PVE:root@pam:4EEC61E2::validticket".to_string())
            .await
            .unwrap();

        assert!(!ticket.is_expired().await);
        sleep(Duration::from_millis(150)).await;
        assert!(ticket.is_expired().await);
    }

    #[tokio::test]
    async fn test_user_realm_extraction() {
        let ticket = ProxmoxTicket::new("PVE:root@pam:4EEC61E2::validticket".to_string())
            .await
            .unwrap();

        let (user, realm) = ticket.extract_user_realm().await.unwrap();
        assert_eq!(user, "root");
        assert_eq!(realm, "pam");
    }

    #[tokio::test]
    async fn test_cookie_header() {
        let ticket = ProxmoxTicket::new("PVE:root@pam:4EEC61E2::validticket".to_string())
            .await
            .unwrap();

        let header = ticket.as_cookie_header().await;
        assert!(header.starts_with("PVEAuthCookie="));
        assert!(header.contains("PVE:root@pam"));
    }
}
