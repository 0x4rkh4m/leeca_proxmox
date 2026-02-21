//! Domain model for node list items from the `/nodes` endpoint.
//!
//! This module defines the structure of a node as returned by the Proxmox API
//! when listing all nodes in the cluster.

use serde::{Deserialize, Serialize};

/// A node in the Proxmox cluster.
///
/// This struct represents a node as returned by the `/api2/json/nodes` endpoint.
/// It contains identifying information, status, and resource usage statistics.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct NodeListItem {
    /// The node name (e.g., "pve1").
    pub node: String,
    /// Current node status (e.g., "online", "offline", "unknown").
    pub status: String,
    /// CPU usage percentage (0.0 to 1.0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu: Option<f64>,
    /// Maximum CPU count (number of cores/threads).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maxcpu: Option<u32>,
    /// Memory usage in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mem: Option<u64>,
    /// Maximum memory in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maxmem: Option<u64>,
    /// Disk usage in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disk: Option<u64>,
    /// Maximum disk space in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maxdisk: Option<u64>,
    /// System uptime in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uptime: Option<u64>,
    /// Unique node identifier (e.g., "node/pve1").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// SSL fingerprint (if available).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ssl_fingerprint: Option<String>,
}
