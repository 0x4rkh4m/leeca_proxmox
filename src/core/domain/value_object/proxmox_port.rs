use crate::core::domain::error::ValidationError;

/// A validated Proxmox port number.
#[derive(Debug, Clone, Copy)]
pub struct ProxmoxPort(u16);

impl ProxmoxPort {
    /// Creates a new port without validation.
    pub(crate) fn new_unchecked(port: u16) -> Self {
        Self(port)
    }

    /// Returns the port number.
    #[must_use]
    pub fn get(&self) -> u16 {
        self.0
    }
}

/// Validates a port number.
pub(crate) fn validate_port(port: u16) -> Result<(), ValidationError> {
    if port == 0 {
        return Err(ValidationError::Field {
            field: "port".to_string(),
            message: "Port cannot be 0".to_string(),
        });
    }
    // All ports 1-65535 are valid.
    Ok(())
}
