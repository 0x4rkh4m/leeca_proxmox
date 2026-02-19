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
    Ok(())
}
