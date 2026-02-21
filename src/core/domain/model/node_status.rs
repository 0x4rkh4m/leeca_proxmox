//! Domain model for node status from the `/nodes/{node}/status` endpoint.
//!
//! This module defines the detailed status information for a specific node.

use serde::{Deserialize, Serialize};

/// Detailed status information for a Proxmox node.
///
/// Returned by the `/api2/json/nodes/{node}/status` endpoint.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct NodeStatus {
    /// CPU usage percentage (0.0 to 1.0).
    pub cpu: f64,
    /// Memory usage in bytes.
    pub memory: MemoryInfo,
    /// Swap usage in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub swap: Option<MemoryInfo>,
    /// System uptime in seconds.
    pub uptime: u64,
    /// Kernel version string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kversion: Option<String>,
    /// Load average over 1, 5, and 15 minutes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loadavg: Option<[f64; 3]>,
    /// Current power state (e.g., "running").
    pub current_kernel: Option<String>,
    /// Node description (if set).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// IO delay percentage (0.0 to 1.0) - time spent waiting for I/O operations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wait: Option<f64>,
    /// CPU information model string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpuinfo: Option<String>,
    /// Hardware model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pve_version: Option<String>,
}

/// Memory usage information.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct MemoryInfo {
    /// Total memory in bytes.
    pub total: u64,
    /// Used memory in bytes.
    pub used: u64,
    /// Free memory in bytes.
    pub free: u64,
}
