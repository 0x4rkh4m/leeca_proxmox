<div align="center">

# Leeca Proxmox VE SDK
### Rust SDK for interacting with Proxmox Virtual Environment servers

<img src="assets/leeca_logo.png" alt="Leeca SDK Logo" width="150"/>

[![CI Status][ci-shield]][ci-url]
[![Coverage][coverage-shield]][coverage-url]
[![Crates.io][crates-shield]][crates-url]
[![Downloads][downloads-shield]][downloads-url]
[![Docs][docs-shield]][docs-url]
[![Deps][deps-shield]][deps-url]
[![License][license-shield]]
[![MSRV][msrv-shield]][msrv-url]
[![Security][security-shield]][security-url]

A modern, safe, and async‑first SDK for interacting with Proxmox Virtual Environment servers.

<div align="center">
<p align="center">
  <a href="https://docs.rs/leeca_proxmox"><strong>📚 Documentation »</strong></a>&nbsp;&nbsp;·&nbsp;&nbsp;
  <a href="#-quick-start"><strong>🚀 Quick Start »</strong></a>&nbsp;&nbsp;·&nbsp;&nbsp;
  <a href="examples/"><strong>📋 Examples »</strong></a>
</p>
<p align="center">
  <a href="CONTRIBUTING.md">Contributing</a>&nbsp;&nbsp;·&nbsp;&nbsp;
  <a href="SECURITY.md">Security</a>&nbsp;&nbsp;·&nbsp;&nbsp;
  <a href="https://github.com/0x4rkh4m/leeca_proxmox/issues/new?labels=bug&template=bug_report.md">Report Bug</a>&nbsp;&nbsp;·&nbsp;&nbsp;
  <a href="https://github.com/0x4rkh4m/leeca_proxmox/issues/new?labels=enhancement&template=feature_request.md">Request Feature</a>
</p>
</div>

</div>

## 📋 Table of Contents

- [✨ Features](#-features)
- [🚀 Getting Started](#-getting-started)
- [📖 Usage](#-usage)
- [🛠️ Development](#️-development)
- [📊 Project Status](#project-status)
- [📚 Documentation](#-documentation)
- [🛡️ Security](#️-security)
- [📄 License](#-license)
- [🤝 Contributing](#-contributing)
- [⚖️ Code of Conduct](#️-code-of-conduct)
- [👥 Community](#-community)
- [📈 Versioning](#versioning)
- [🙏 Acknowledgments](#acknowledgments)

## ✨ Features

- 🔒 **Secure by default**  
  TLS 1.3, optional certificate validation, token‑based authentication.

- ⚙️ **Configurable validation**  
  Password strength, DNS resolution, reserved usernames – all opt‑in, off by default.

- 🧱 **Clean architecture**  
  Domain‑driven design with value objects, clear separation of concerns.

- ⚡ **Async/await**  
  Built on Tokio for high concurrency.

- 🧾 **Error handling**  
  Detailed, type‑safe errors with backtraces.

## 🚀 Getting Started

### Prerequisites

- Rust
- Cargo
- Tokio runtime

### Installation

Add the dependency to your `Cargo.toml`:

```bash
cargo add leeca_proxmox
```

Or edit `Cargo.toml` manually:

```toml
[dependencies]
leeca_proxmox = "0.2"
tokio = { version = "1", features = ["full"] }
```

## 📖 Usage

Basic authentication example:

```rust
use leeca_proxmox::{ProxmoxClient, ProxmoxResult};

#[tokio::main]
async fn main() -> ProxmoxResult<()> {
    let mut client = ProxmoxClient::builder()
        .host("192.168.1.100")
        .port(8006)
        .credentials("leeca", "password", "pam")
        .secure(true)                     // HTTPS (default)
        .accept_invalid_certs(false)      // reject invalid certificates (default)
        .build()
        .await?;

    client.login().await?;
    println!("Authenticated! Ticket: {}", client.auth_token().unwrap().as_str());

    Ok(())
}
```

See the [authentication example](examples/auth/login.rs) for a complete demonstration.

### Enabling extra validation

By default, only basic format checks are performed. To enable additional checks:

```rust
let client = ProxmoxClient::builder()
    .host("...")
    .credentials("user", "pass", "pam")
    .enable_password_strength(3)          // require zxcvbn score ≥ 3
    .enable_dns_resolution()               // verify hostname resolves
    .block_reserved_usernames()            // reject root, admin, etc.
    .build()
    .await?;
```

### Session Persistence

You can save the current authentication session to a file and reload it later, avoiding the need to log in again as long as the tokens are still valid.

```rust
let mut client = ProxmoxClient::builder()
    .host("192.168.1.100")
    .port(8006)
    .credentials("leeca", "password", "pam")
    .secure(true)
    .accept_invalid_certs(false)
    .build()
    .await?;

client.login().await?;

// Save session to a file
client.save_session_to_file("proxmox-session.json").await?;

// Later, create a new client and load the session
let mut new_client = ProxmoxClient::builder()
    .host("192.168.1.100")
    .port(8006)
    .credentials("dummy", "dummy", "pam") // credentials are still required but won't be used
    .secure(true)
    .accept_invalid_certs(false)
    .with_session(std::fs::File::open("proxmox-session.json")?)
    .await?
    .build()
    .await?;

// The new client is already authenticated
assert!(new_client.is_authenticated().await);
```

The session data contains the ticket and CSRF token with their creation timestamps. It is serialized as JSON. You should store it securely (e.g., encrypted at rest) because it grants access to the Proxmox API.

See the [session_persistence example](examples/auth/session_persistence.rs) for a complete demonstration.

### Discovering Cluster Resources

Once authenticated, you can retrieve a unified list of all resources in the cluster – including VMs, containers, storage, and nodes – using the `cluster_resources()` method. This is particularly useful for discovering which nodes contain specific VMs before performing node‑level operations.

```rust
let resources = client.cluster_resources().await?;
for resource in resources {
    match resource {
        ClusterResource::Qemu(vm) => {
            println!(
                "VM {} (ID: {}) on node {} is {}",
                vm.common.name.as_deref().unwrap_or("(unnamed)"),
                vm.vmid,
                vm.common.node,
                vm.common.status
            );
        }
        ClusterResource::Lxc(ct) => {
            println!(
                "Container {} (ID: {}) on node {} is {}",
                ct.common.name.as_deref().unwrap_or("(unnamed)"),
                ct.vmid,
                ct.common.node,
                ct.common.status
            );
        }
        ClusterResource::Storage(st) => {
            println!(
                "Storage '{}' on node {} ({} type) is {}",
                st.storage, st.common.node, st.storage_type, st.common.status
            );
        }
        ClusterResource::Node(node) => {
            println!(
                "Node {} is {} (load: {:?})",
                node.common.node, node.common.status, node.loadavg
            );
        }
    }
}
```

The method returns a Vec<ClusterResource> where each variant contains both common fields (like node, id, name, status) and type‑specific fields (e.g., vmid for VMs, storage for storage). This allows you to programmatically inspect your Proxmox infrastructure without hard‑coding node names.

See the [cluster_resources example](examples/resources/cluster_resources.rs) for a complete demonstration.

### Node Management

Once authenticated, you can inspect the nodes in your cluster:

```rust
// List all nodes
let nodes = client.nodes().await?;
for node in nodes {
    println!("Node: {} (status: {})", node.node, node.status);
}

// Get detailed status of a specific node
let status = client.node_status("pve1").await?;
println!("CPU: {:.2}%, IO Delay: {:.2}%", 
    status.cpu * 100.0, 
    status.wait.unwrap_or(0.0) * 100.0
);

// Get DNS configuration
let dns = client.node_dns("pve1").await?;
println!("DNS servers: {:?}", dns.servers);
```

See the [node_management example](examples/resources/node_management.rs) for a complete demonstration.

### VM Management

After authentication, you can manage QEMU virtual machines on any node:

```rust
// List all VMs on a node
let vms = client.vms("pve1").await?;
for vm in vms {
    println!("{} ({}): {}", vm.name, vm.vmid, vm.status);
}

// Get detailed status
let status = client.vm_status("pve1", 100).await?;
println!("CPU: {:.2}%", status.cpu.unwrap_or(0.0) * 100.0);

// Start a VM
let task = client.start_vm("pve1", 100).await?;
println!("Task ID: {}", task);

// Create a new VM
let params = CreateVmParams {
    vmid: 200,
    name: "my-vm".to_string(),
    memory: Some(4096),
    cores: Some(2),
    ..Default::default()
};
let task = client.create_vm("pve1", &params).await?;
```

See the [vm_operations example](examples/resources/vm_operations.rs) for a complete demonstration.

See the [examples](examples/) directory for more.

## 🛠️ Development

```bash
# Install development dependencies
cargo install cargo-llvm-cov cargo-audit

# Run tests
cargo test --all-features

# Check code coverage
cargo llvm-cov --all-features --lcov --output-path lcov.info

# Run security audit
cargo audit

# Run linters
cargo clippy --all-targets --all-features
cargo fmt --all -- --check
```

## 📊 Project Status

See our [CHANGELOG](CHANGELOG.md) for version history and [ROADMAP](ROADMAP.md) for future plans.

## 📚 Documentation

- [Crate Documentation](https://docs.rs/leeca_proxmox)
- [Architecture Guide](docs/architecture.md) (coming soon)
- [Examples](examples/)

## 🛡️ Security

See our [Security Policy](SECURITY.md) for reporting vulnerabilities.

## 📄 License

Licensed under Apache License 2.0 – see the [LICENSE](LICENSE) file for details.

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## ⚖️ Code of Conduct

Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md).

## 👥 Community

- [Issue Tracker](https://github.com/0x4rkh4m/leeca_proxmox/issues)
- [Discussions](https://github.com/0x4rkh4m/leeca_proxmox/discussions)
- Email: [4rkham@proton.me](mailto:4rkham@proton.me)

## 📈 Versioning

This project follows [Semantic Versioning](https://semver.org/). See our [CHANGELOG](CHANGELOG.md) for version history.

⚠️ **Note**: APIs may change before 1.0.0.

## 🙏 Acknowledgments

- Proxmox VE team for their excellent API documentation.
- Rust community for the tools and crates.
- All [contributors](https://github.com/0x4rkh4m/leeca_proxmox/graphs/contributors).

---

<div align="center">

<sub>Built with ❤️ by <a href="https://github.com/0x4rkh4m">4rkh4m</a> and the Rust community.</sub>

<br/>

[⭐ Star][repo-url] ·
[🐛 Report Bug][bug-url] ·
[✨ Request Feature][feature-url] ·
[🛡️ Security Report][security-url]

</div>

<!-- MARKDOWN LINKS & BADGES -->
[ci-shield]: https://img.shields.io/github/actions/workflow/status/0x4rkh4m/leeca_proxmox/ci.yml?branch=main&style=for-the-badge
[ci-url]: https://github.com/0x4rkh4m/leeca_proxmox/actions/workflows/ci.yml
[coverage-shield]: https://img.shields.io/codecov/c/github/0x4rkh4m/leeca_proxmox?style=for-the-badge
[coverage-url]: https://codecov.io/gh/0x4rkh4m/leeca_proxmox
[crates-shield]: https://img.shields.io/crates/v/leeca_proxmox?style=for-the-badge
[crates-url]: https://crates.io/crates/leeca_proxmox
[downloads-shield]: https://img.shields.io/crates/d/leeca_proxmox?style=for-the-badge
[downloads-url]: https://crates.io/crates/leeca_proxmox
[docs-shield]: https://img.shields.io/docsrs/leeca_proxmox?style=for-the-badge
[docs-url]: https://docs.rs/leeca_proxmox
[deps-shield]: https://img.shields.io/librariesio/release/cargo/leeca_proxmox?style=for-the-badge
[deps-url]: https://deps.rs/repo/github/0x4rkh4m/leeca_proxmox
[license-shield]: https://img.shields.io/crates/l/leeca_proxmox?style=for-the-badge
[msrv-shield]: https://img.shields.io/badge/MSRV-stable--1.93.1-blue?style=for-the-badge
[msrv-url]: https://blog.rust-lang.org/2026/02/12/Rust-1.93.1
[security-shield]: https://img.shields.io/badge/Security-Report-green?style=for-the-badge

<!-- REPOSITORY LINKS -->
[repo-url]: https://github.com/0x4rkh4m/leeca_proxmox
[bug-url]: https://github.com/0x4rkh4m/leeca_proxmox/issues/new?labels=bug&template=bug_report.md
[feature-url]: https://github.com/0x4rkh4m/leeca_proxmox/issues/new?labels=enhancement&template=feature_request.md
[security-url]: SECURITY.md
