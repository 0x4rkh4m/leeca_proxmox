# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

---

## [0.3.0] - 2026-02-23

### Added
- **VM operations** – Complete set of methods for managing QEMU virtual machines:
  - `vms(node)` – List all VMs on a node.
  - `vm_status(node, vmid)` – Get detailed current status.
  - `start_vm`, `stop_vm`, `shutdown_vm`, `reboot_vm`, `reset_vm` – Control VM state.
  - `delete_vm(node, vmid, purge)` – Remove a VM.
  - `create_vm(node, params)` – Create a new VM with full configuration.
  - `vm_config(node, vmid)` – Get current configuration.
  - `update_vm_config(node, vmid, params)` – Update configuration.

- **Domain models** – Added:
  - `VmListItem`
  - `VmStatusCurrent`
  - `VmConfig`
  - `CreateVmParams`

- **Example** – `examples/vm_operations.rs` demonstrates listing, inspecting, and creating VMs.

- **Node management operations** – New methods for node discovery and inspection:
  - `ProxmoxClient::nodes()` – Lists all nodes in the cluster with basic information and resource usage.
  - `ProxmoxClient::node_status(node)` – Retrieves detailed status for a specific node, including CPU, memory, swap, load average, and IO delay (`wait`).
  - `ProxmoxClient::node_dns(node)` – Fetches DNS configuration (search domain and servers) for a node.

- **Domain models** – Added:
  - `NodeListItem`
  - `NodeStatus`
  - `MemoryInfo`
  - `NodeDnsConfig`

- **Example** – `examples/node_management.rs` demonstrates listing nodes and retrieving status/DNS.

- **Cluster resource discovery** – New method:
  - `ProxmoxClient::cluster_resources()` – Returns a list of all resources (VMs, containers, storage, nodes) in the cluster.
  - Strongly typed via the `ClusterResource` enum.

- **Domain models** – Added:
  - `ClusterResource` enum
  - Variants: `QemuResource`, `LxcResource`, `StorageResource`, `NodeResource`

- **Example** – `examples/cluster_resources.rs` demonstrates how to list and categorize cluster resources.

- **Client-side rate limiting** – Configurable requests per second and burst size (via `.rate_limit()` builder method).
  - Uses a token bucket algorithm.
  - Asynchronously blocks when limits are exceeded.

- **Internal HTTP client (`ApiClient`)** – Centralised all API requests:
  - Handles authentication headers.
  - Automatic ticket refresh on 401 responses.

- **Session persistence**
  - `ProxmoxClient::save_session_to_file`
  - `ProxmoxClient::load_session_from_file`
  - Builder support via `with_session`

### Changed
- **`ProxmoxClient` now uses `ApiClient` internally** – Authentication state is managed by the new client.
- **Authentication methods are now `async`**:
  - `is_authenticated`
  - `auth_token`
  - `csrf_token`
  - `is_ticket_expired`
  - `is_csrf_expired`
- **Builder `build()` now returns a `ProxmoxClient` backed by `ApiClient`** – No interface breakage.

### Fixed
- CSRF token validation in tests – now uses valid alphanumeric tokens.

### Removed
- Direct `connection` and `auth` fields from `ProxmoxClient` – They are now encapsulated inside `ApiClient`.

---

## [0.2.0] - 2026-02-19

### Added
- **Validation configuration**: Password strength, DNS resolution, reserved usernames – all optional and off by default.
- **Simplified value objects**: Removed `ValueObject` trait and async locking; values are now plain structs with synchronous getters.
- **Builder improvements**: `ProxmoxClientBuilder` now defaults to secure HTTPS and accepts a custom `ValidationConfig`.
- **New methods**:
  - `ProxmoxClient::is_ticket_expired()`
  - `ProxmoxClient::is_csrf_expired()`
- **Documentation**: Expanded examples and migration guide for 0.1.x users.

### Changed
- **MSRV**: Still nightly; edition updated to 2024.
- **API**: Validation now happens at build time, reducing async noise and improving ergonomics.
- **Error messages**: More descriptive validation errors.

### Removed
- **`ValueObject` trait** – No longer needed.
- **`into_inner` methods** – Removed (can be reintroduced later if required).

### Security
- Password strength checking is opt-in (default off).
- DNS resolution is opt-in to avoid blocking during build.

---

## [0.1.2] - 2026-02-19

### Added
- Dependabot configuration.
- Code coverage reporting via Codecov.
- Robust CI pipeline (formatting, linting, audit, docs, tests).

### Changed
- Updated all dependencies (resolves RUSTSEC advisories).
- Migrated to Rust 2024 edition.
- Integration tests ignored by default (require real Proxmox instance).

---

## [0.1.1] - 2025-01-14

### Added
- Public `value()` and `expires_at()` methods for tickets and CSRF tokens.
- Enhanced token lifetime visibility.

### Fixed
- CSRF token validation to match Proxmox VE format.
- Login response parsing.

---

## [0.1.0] - 2025-01-13

### Added
- Initial release with core authentication, value objects, and async support.
