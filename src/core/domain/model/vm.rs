//! Domain models for QEMU virtual machine operations.
//!
//! This module defines the structures used when interacting with VMs via the Proxmox API.

use serde::{Deserialize, Serialize};

/// A virtual machine as returned by the `/nodes/{node}/qemu` endpoint.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VmListItem {
    /// The VM identifier (unique per cluster).
    pub vmid: u32,
    /// Human-readable name.
    pub name: String,
    /// Current status (e.g., "running", "stopped").
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
    /// Uptime in seconds (if running).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uptime: Option<u64>,
    /// The Proxmox node where this VM resides.
    pub node: String,
    /// Unique resource identifier (e.g., "qemu/100").
    pub id: String,
    /// Additional tags (if any).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,
}

/// Detailed runtime status of a VM from `/nodes/{node}/qemu/{vmid}/status/current`.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct VmStatusCurrent {
    /// Current VM status (e.g., "running", "stopped", "paused").
    pub status: String,
    /// VM name.
    pub name: String,
    /// CPU usage percentage (0.0 to 1.0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu: Option<f64>,
    /// Memory usage in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mem: Option<u64>,
    /// Uptime in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uptime: Option<u64>,
    /// QEMU process status (e.g., "running", "stopped").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub qmpstatus: Option<String>,
    /// Balloon info (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub balloon: Option<BalloonInfo>,
    /// Network interfaces (if available).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub net: Option<serde_json::Value>,
    /// Block device statistics.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blockstat: Option<serde_json::Value>,
    /// NUMA node memory info.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub numa: Option<serde_json::Value>,
    /// Proxmox configuration digest.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,
    /// Number of CPU sockets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sockets: Option<u32>,
    /// Number of cores per socket.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cores: Option<u32>,
    /// CPU type (e.g., "kvm64", "host").
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "cputype")]
    pub cpu_type: Option<String>,
    /// Balloon device minimum memory.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub balloon_min: Option<u64>,
    /// Maximum memory in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maxmem: Option<u64>,
    /// Free CPU memory in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub freemem: Option<u64>,
    /// Total memory in bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub totalmem: Option<u64>,
}

/// Balloon device information.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BalloonInfo {
    /// Current balloon size in bytes.
    pub current: u64,
    /// Maximum balloon size in bytes.
    pub maximum: u64,
    /// Minimum balloon size in bytes.
    pub minimum: u64,
}

/// VM configuration from `/nodes/{node}/qemu/{vmid}/config`.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VmConfig {
    /// VM identifier.
    pub vmid: u32,
    /// VM name.
    pub name: String,
    /// Description (if set).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Memory in MB.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory: Option<u32>,
    /// Balloon device minimum memory.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub balloon: Option<u32>,
    /// Number of CPU sockets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sockets: Option<u32>,
    /// Number of cores per socket.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cores: Option<u32>,
    /// Number of threads per core.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threads: Option<u32>,
    /// CPU type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu: Option<String>,
    /// Network devices.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub net: Option<serde_json::Value>,
    /// Storage devices.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scsi: Option<serde_json::Value>,
    /// IDE devices.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ide: Option<serde_json::Value>,
    /// SATA devices.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sata: Option<serde_json::Value>,
    /// VirtIO devices.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub virtio: Option<serde_json::Value>,
    /// Boot order.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boot: Option<String>,
    /// OS type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ostype: Option<String>,
    /// Agent enabled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<u8>,
    /// KVM hardware virtualization enabled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kvm: Option<u8>,
    /// NUMA enabled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub numa: Option<u8>,
    /// Tags.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,
    /// Proxmox configuration digest (for updates).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,
    /// Start at boot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub onboot: Option<u8>,
    /// Protection from accidental removal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protection: Option<u8>,
    /// Tablet USB pointer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tablet: Option<u8>,
    /// VGA configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vga: Option<String>,
    /// SCSI controller type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scsihw: Option<String>,
    /// BIOS type (seabios, ovmf).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bios: Option<String>,
    /// EFI disk (for OVMF).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub efidisk: Option<String>,
    /// TPM state.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tpmstate: Option<String>,
}

/// Parameters for creating a new VM.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVmParams {
    /// VM identifier (required, must be unique in the cluster).
    pub vmid: u32,
    /// VM name (required).
    pub name: String,
    /// Memory in MB (optional, default 512).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory: Option<u32>,
    /// Number of CPU sockets (optional, default 1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sockets: Option<u32>,
    /// Number of cores per socket (optional, default 1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cores: Option<u32>,
    /// Number of threads per core (optional, default 1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threads: Option<u32>,
    /// CPU type (optional, default "kvm64").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu: Option<String>,
    /// OS type (optional, e.g., "l26", "win10").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ostype: Option<String>,
    /// Enable/disable KVM hardware virtualization (optional, default 1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kvm: Option<u8>,
    /// Enable/disable NUMA (optional, default 0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub numa: Option<u8>,
    /// Network configuration (optional, e.g., "virtio,bridge=vmbr0").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub net: Option<String>,
    /// SCSI controller type (optional, e.g., "virtio-scsi-pci").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scsihw: Option<String>,
    /// Boot order (optional, e.g., "order=scsi0;net0").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boot: Option<String>,
    /// Start after creation (optional, default 0).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<u8>,
    /// Tags (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,
    /// Description (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Protection from accidental removal (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protection: Option<u8>,
    /// Tablet USB pointer (optional, default 1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tablet: Option<u8>,
    /// VGA configuration (optional, e.g., "virtio").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vga: Option<String>,
    /// BIOS type (optional, "seabios" or "ovmf").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bios: Option<String>,
    /// EFI disk (optional, for OVMF).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub efidisk: Option<String>,
    /// TPM state (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tpmstate: Option<String>,
    /// Agent enabled (optional, 1 to enable QEMU Guest Agent).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<u8>,
}
