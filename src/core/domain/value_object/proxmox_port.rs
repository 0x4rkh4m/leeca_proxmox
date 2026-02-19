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
    #[allow(unused)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_port_valid() {
        assert!(validate_port(8006).is_ok());
        assert!(validate_port(22).is_ok());
        assert!(validate_port(65535).is_ok());
    }

    #[test]
    fn test_validate_port_invalid() {
        assert!(validate_port(0).is_err());
    }

    #[test]
    fn test_port_new_unchecked() {
        let port = ProxmoxPort::new_unchecked(8006);
        assert_eq!(port.get(), 8006);
    }
}
