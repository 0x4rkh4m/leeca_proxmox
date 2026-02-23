# Roadmap

## Completed Versions

### 0.3.0 (In Progress)
- ✅ **VM operations** – List, status, start, stop, shutdown, reboot, reset, delete, create, get/update config.
- ✅ **HTTP Client Refactor**: Centralised request handling with automatic authentication and ticket refresh.
- ✅ **Enhanced Security**
  - ✅ Rate limiting (client‑side)
  - ✅ Token refresh mechanism
  - ✅ Session persistence
- ✅ **Cluster resource discovery** – Unified view of all resources (VMs, containers, storage, nodes) via `/cluster/resources`.
- ✅ **Node management** – List nodes, get node status, and retrieve DNS configuration via `/nodes`, `/nodes/{node}/status`, and `/nodes/{node}/dns` endpoints.
- 🔄 **Resource Management** (next)
  - ✅ VM operations (start, stop, reboot, create, delete)
  - Container (LXC) operations
  - Storage management (list, create, delete)
  - Network configuration
  - ✅ Node management

### 0.2.0 (Current Release)
- ✅ **Validation overhaul**: All extra checks (password strength, DNS, reserved usernames) are now opt‑in, off by default.
- ✅ **Simplified API**: Removed `ValueObject` trait and async locking; value objects are plain structs.
- ✅ **Builder improvements**: Defaults to secure HTTPS, accepts custom `ValidationConfig`.
- ✅ **Documentation**: Updated examples, added migration notes.
- ✅ **CI/CD**: Full pipeline with formatting, linting, audit, coverage, and docs.

### 0.1.2
- ⬆️ Dependency updates and security fixes.
- 🦀 Rust 2024 edition migration.
- 🤖 Robust CI and Dependabot integration.

### 0.1.1
- ✅ Public token management.
- ✅ Enhanced CSRF validation.

### 0.1.0
- ✅ Core client and authentication.

## Version 0.4.0
- 📋 **Task Management**
  - Task queuing and polling
  - Progress tracking
  - Cancellation support
  - Retry mechanisms
- 🔍 **Monitoring**
  - Resource metrics (CPU, memory, disk, network)
  - Performance statistics
  - Event logging
  - Alert integration

## Version 1.0.0
- 🌟 **High‑Level Features**
  - Backup/restore operations
  - Cluster management
  - Template management
  - Migration tools
- 🛠 **Developer Experience**
  - CLI tool (optional)
  - Comprehensive integration examples
  - Performance optimizations
  - Extended documentation

## Planned Features
- [ ] WebSocket support for real‑time updates
- [ ] Batch operations
- [ ] Resource pooling
- [ ] Custom role management
- [ ] Firewall configuration
- [ ] HA cluster support
- [ ] Storage replication
- [ ] Snapshot management

## Under Consideration
- Integration with other virtualization platforms (Proxmox Backup Server, oVirt)
- GUI tools (TUI)
- Plugin system
- Configuration management helpers
- Automated testing against a live Proxmox cluster

*Note: This roadmap is subject to change based on community feedback and project needs.*
