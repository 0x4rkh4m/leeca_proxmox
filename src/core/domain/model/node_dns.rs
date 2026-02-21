//! Domain model for node DNS configuration from the `/nodes/{node}/dns` endpoint.
//!
//! This module defines the DNS settings for a specific node.

use serde::{Deserialize, Serialize};

/// DNS configuration for a Proxmox node.
///
/// Returned by the `/api2/json/nodes/{node}/dns` endpoint.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct NodeDnsConfig {
    /// DNS search domain (e.g., "example.com").
    pub domain: String,
    /// List of DNS server IP addresses.
    pub servers: Vec<String>,
    /// DNS options (if any).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,
}
