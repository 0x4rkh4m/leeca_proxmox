//! Domain models for cluster-wide resources.
//!
//! This module defines the structures returned by the `/cluster/resources` endpoint.
//! The response contains a heterogeneous list of resources (VMs, containers, storage, etc.),
//! each identified by a `type` field. We model this as an enum to provide type safety.

use serde::{Deserialize, Serialize};

/// A resource discovered in the Proxmox cluster.
///
/// The `type` field determines which variant this is. All variants share common fields
/// like `node`, `id`, and `status`, but may have additional type‑specific fields.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ClusterResource {
    /// A QEMU virtual machine.
    Qemu(QemuResource),
    /// An LXC container.
    Lxc(LxcResource),
    /// A storage entity.
    Storage(StorageResource),
    /// A node in the cluster.
    Node(NodeResource),
    // Other resource types (e.g., `pool`, `sdn`) can be added as needed.
}

/// Common fields present in every resource.
///
/// These are extracted into a separate struct to avoid duplication.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CommonResourceFields {
    /// The Proxmox node where this resource resides.
    pub node: String,
    /// Unique resource identifier (e.g., `qemu/100`).
    pub id: String,
    /// Human‑readable name (may be absent).
    #[serde(default)]
    pub name: Option<String>,
    /// Resource status (e.g., `running`, `stopped`, `available`).
    pub status: String,
    /// Uptime in seconds (if applicable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uptime: Option<u64>,
}

/// A QEMU virtual machine resource.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct QemuResource {
    /// Common fields.
    #[serde(flatten)]
    pub common: CommonResourceFields,
    /// The VM identifier (unique per cluster).
    pub vmid: u32,
    /// Number of allocated virtual CPUs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maxcpu: Option<u32>,
    /// Maximum memory in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maxmem: Option<u64>,
    /// Disk usage in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disk: Option<u64>,
    /// Network usage statistics (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub netin: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub netout: Option<u64>,
}

/// An LXC container resource.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct LxcResource {
    /// Common fields.
    #[serde(flatten)]
    pub common: CommonResourceFields,
    /// The container identifier.
    pub vmid: u32,
    /// Number of allocated virtual CPUs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maxcpu: Option<u32>,
    /// Maximum memory in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maxmem: Option<u64>,
    /// Disk usage in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disk: Option<u64>,
    /// Swap usage in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub swap: Option<u64>,
}

/// A storage resource (e.g., directory, ZFS, LVM).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct StorageResource {
    /// Common fields.
    #[serde(flatten)]
    pub common: CommonResourceFields,
    /// Storage identifier (e.g., `local`, `nfs-storage`).
    pub storage: String,
    /// Storage plugin type (e.g., `dir`, `zfspool`, `lvm`).
    #[serde(rename = "plugintype")]
    pub storage_type: String,
    /// Total capacity in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
    /// Used space in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub used: Option<u64>,
    /// Available space in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avail: Option<u64>,
}

/// A node resource (the node itself).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct NodeResource {
    /// Common fields.
    #[serde(flatten)]
    pub common: CommonResourceFields,
    /// Node CPU usage percentage.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu: Option<f64>,
    /// Node memory usage percentage.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mem: Option<f64>,
    /// Node load average (1,5,15 minutes).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loadavg: Option<[f64; 3]>,
    /// Kernel version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kversion: Option<String>,
    /// Current power state (e.g., `online`, `offline`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub power: Option<String>,
}
