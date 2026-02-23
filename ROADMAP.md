# Roadmap

## Completed Versions

### 0.3.0
- ✅ **VM operations** – List, status, start, stop, shutdown, reboot, reset, delete, create, get/update config.
- ✅ **HTTP Client Refactor** – Centralised request handling with automatic authentication and ticket refresh.
- ✅ **Enhanced Security**
  - ✅ Client-side rate limiting
  - ✅ Automatic token refresh
  - ✅ Session persistence
- ✅ **Cluster resource discovery** – Unified view of all resources via `/cluster/resources`.
- ✅ **Node management**
  - List nodes
  - Node status inspection
  - DNS configuration retrieval

---

### 0.2.0
- ✅ **Validation overhaul** – Password strength, DNS resolution, reserved usernames (opt-in).
- ✅ **Simplified API** – Removed `ValueObject` trait and async locking.
- ✅ **Builder improvements** – Secure HTTPS by default.
- ✅ **Documentation & Migration notes**
- ✅ **CI/CD pipeline**

---

### 0.1.2
- ⬆️ Dependency updates and security fixes.
- 🦀 Rust 2024 edition migration.
- 🤖 Robust CI and Dependabot integration.

### 0.1.1
- ✅ Public token management.
- ✅ Enhanced CSRF validation.

### 0.1.0
- ✅ Core client and authentication.

---

## Version 0.4.0

### Resource Management Expansion
- 📦 **Container (LXC) operations**
  - List containers
  - Start/stop/reboot
  - Create/delete
  - Config inspection and updates

- 💾 **Storage management**
  - List storage backends
  - Create/delete storage
  - Inspect storage usage

- 🌐 **Network configuration**
  - Bridge configuration
  - VLAN support
  - Network inspection APIs

### Task Management
- Task queuing and polling
- Progress tracking
- Cancellation support
- Retry mechanisms

### Monitoring
- Resource metrics (CPU, memory, disk, network)
- Performance statistics
- Event logging
- Alert integration

---

## Version 1.0.0

### High-Level Features
- Backup/restore operations
- Cluster management
- Template management
- Migration tools

### Developer Experience
- Optional CLI tool
- Comprehensive integration examples
- Performance optimizations
- Extended documentation

---

## Planned Features
- [ ] WebSocket support for real-time updates
- [ ] Batch operations
- [ ] Resource pooling
- [ ] Custom role management
- [ ] Firewall configuration
- [ ] HA cluster support
- [ ] Storage replication
- [ ] Snapshot management

---

## Under Consideration
- Integration with other virtualization platforms (Proxmox Backup Server, oVirt)
- TUI/GUI tools
- Plugin system
- Configuration management helpers
- Automated testing against a live Proxmox cluster (beta)

---

*Note: This roadmap is subject to change based on community feedback and project evolution.*
