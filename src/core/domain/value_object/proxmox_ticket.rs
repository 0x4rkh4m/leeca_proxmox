use crate::core::domain::error::ValidationError;
use std::time::{Duration, SystemTime};

/// A Proxmox authentication ticket.
#[derive(Debug, Clone)]
pub struct ProxmoxTicket {
    value: String,
    created_at: SystemTime,
}

impl ProxmoxTicket {
    /// Creates a new ticket without validation.
    pub(crate) fn new_unchecked(value: String) -> Self {
        Self {
            value,
            created_at: SystemTime::now(),
        }
    }

    /// Returns the ticket value as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Returns the creation time.
    #[must_use]
    pub fn created_at(&self) -> SystemTime {
        self.created_at
    }

    /// Checks if the ticket is expired based on a given lifetime.
    #[must_use]
    pub fn is_expired(&self, lifetime: Duration) -> bool {
        self.created_at
            .elapsed()
            .map(|age| age > lifetime)
            .unwrap_or(true)
    }

    /// Formats the ticket as a cookie header.
    #[must_use]
    pub fn as_cookie_header(&self) -> String {
        format!("PVEAuthCookie={}", self.value)
    }
}

/// Validates the format of a ticket string.
pub(crate) fn validate_ticket(ticket: &str) -> Result<(), ValidationError> {
    if ticket.is_empty() {
        return Err(ValidationError::Field {
            field: "ticket".to_string(),
            message: "Ticket cannot be empty".to_string(),
        });
    }
    let parts: Vec<&str> = ticket.split(':').collect();
    if parts.len() < 5 || parts[0] != "PVE" {
        return Err(ValidationError::Format(
            "Invalid ticket format: must start with 'PVE:' and have at least 5 parts".to_string(),
        ));
    }
    // Validate user@realm part
    if let Some(user_realm) = parts.get(1) {
        let ur_parts: Vec<&str> = user_realm.split('@').collect();
        if ur_parts.len() != 2 || ur_parts[0].is_empty() || ur_parts[1].is_empty() {
            return Err(ValidationError::Format(
                "Invalid user@realm format".to_string(),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, SystemTime};

    #[test]
    fn test_validate_ticket_valid() {
        let valid = "PVE:user@pam:4EEC61E2::rsKoApxDTLYPn6H3NNT6iP2mv";
        assert!(validate_ticket(valid).is_ok());
    }

    #[test]
    fn test_validate_ticket_invalid() {
        assert!(validate_ticket("").is_err());
        assert!(validate_ticket("invalid").is_err());
        assert!(validate_ticket("WRONG:user@pam:ticket").is_err()); // wrong prefix
        assert!(validate_ticket("PVE:user:4EEC61E2::sig").is_err()); // missing realm after @
    }

    #[test]
    fn test_ticket_expiration() {
        let ticket = ProxmoxTicket::new_unchecked("PVE:user@pam:4EEC61E2::sig".to_string());
        assert!(!ticket.is_expired(Duration::from_secs(7200))); // fresh

        let old_ticket = ProxmoxTicket {
            value: "PVE:user@pam:4EEC61E2::sig".to_string(),
            created_at: SystemTime::now() - Duration::from_secs(7201),
        };
        assert!(old_ticket.is_expired(Duration::from_secs(7200)));
    }

    #[test]
    fn test_ticket_as_cookie_header() {
        let ticket = ProxmoxTicket::new_unchecked("PVE:ticket".to_string());
        assert_eq!(ticket.as_cookie_header(), "PVEAuthCookie=PVE:ticket");
    }
}
