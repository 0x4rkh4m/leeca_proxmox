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
[![License][license-shield]][license-url]
[![MSRV][msrv-shield]][msrv-url]
[![Security][security-shield]][security-url]

A modern, safe, and async-first SDK for interacting with Proxmox Virtual Environment servers, following industry best practices and clean architecture principles.

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

- 🔒 **Enterprise-Grade Security**
  - Token-based authentication
  - Comprehensive input validation
  - Secure default configurations
  - Thread-safe operations
  - Built-in SSL/TLS support

- 🚀 **Modern Architecture**
  - Async-first design using Tokio
  - Clean Architecture principles
  - Domain-Driven Design
  - SOLID principles
  - Immutable Value Objects

- 💪 **Robust Error Handling**
  - Type-safe error propagation
  - Detailed error contexts
  - Stack traces for debugging
  - Custom error types
  - Validation error handling

## 🚀 Getting Started

### Prerequisites

- Rust (nightly)  <!-- Right now, we are using nightly for #[backtrace] not being stable yet -->
- Cargo
- Tokio runtime

### Installation

```bash
cargo add leeca_proxmox
```

## 📖 Usage

```rust
use leeca_proxmox::{ProxmoxClient, ProxmoxResult};
use std::time::UNIX_EPOCH;

#[tokio::main]
async fn main() -> ProxmoxResult<()> {
    let mut client = ProxmoxClient::builder()
        .host("192.168.1.182")?
        .port(8006)?
        .credentials("leeca", "Leeca_proxmox1!", "pam")?
        .secure(false)
        .build()
        .await?;

    println!("\n🔑 Authentication Status");
    println!("------------------------");
    println!(
        "Initial state: {}",
        if client.is_authenticated() {
            "✅ Authenticated"
        } else {
            "❌ Not authenticated"
        }
    );

    println!("\n📡 Connecting to Proxmox...");
    client.login().await?;
    println!(
        "Connection state: {}",
        if client.is_authenticated() {
            "✅ Authenticated"
        } else {
            "❌ Failed"
        }
    );

    if let Some(token) = client.auth_token() {
        println!("\n🎟️  Session Token");
        println!("------------------------");
        println!("Value: {}", token.value().await);
        let expires = token
            .expires_at()
            .await
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        println!("Expires at: {} (Unix timestamp)", expires);
    }

    if let Some(csrf) = client.csrf_token() {
        println!("\n🛡️  CSRF Protection");
        println!("------------------------");
        println!("Token: {}", csrf.value().await);
        let expires = csrf
            .expires_at()
            .await
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        println!("Expires at: {} (Unix timestamp)", expires);
    }

    println!("\n✨ Connection established successfully!\n");
    Ok(())
}
```

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
- [Architecture Guide](docs/architecture.md) <!-- TODO: Add architecture guide -->
- [Examples](examples/)

## 🛡️ Security

See our [Security Policy](SECURITY.md) for reporting vulnerabilities.

## 📄 License

Licensed under Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

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

⚠️ **Note**: This project is in active development. APIs may change before 1.0.0 release.

## 🙏 Acknowledgments

- Proxmox VE team for their excellent API documentation
- Rust community for their tools and crates
- All [contributors](https://github.com/0x4rkh4m/leeca_proxmox/graphs/contributors)

---

<div align="center">

<sub>Built with ❤️ by <a href="[repo-url]"><strong>Leeca Team</strong></a> && <a href="[rust-community-url]"><strong>Rust Community</strong></a></sub>

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
[msrv-shield]: https://img.shields.io/badge/MSRV-nightly--2024--01--15-blue?style=for-the-badge
[msrv-url]: https://blog.rust-lang.org/2024/01/15/Rust-nightly-2024-01-15-released.html
[security-shield]: https://img.shields.io/badge/Security-Report-green?style=for-the-badge

<!-- RUST LINKS -->
[rust-community-url]: https://www.rust-lang.org/community

<!-- REPOSITORY LINKS -->
[repo-url]: https://github.com/0x4rkh4m/leeca_proxmox
[issues-url]: https://github.com/0x4rkh4m/leeca_proxmox/issues
[discussions-url]: https://github.com/0x4rkh4m/leeca_proxmox/discussions
[changelog-url]: CHANGELOG.md
[license-url]: LICENSE
[contributing-url]: CONTRIBUTING.md
[security-url]: SECURITY.md
[bug-url]: https://github.com/0x4rkh4m/leeca_proxmox/issues/new?labels=bug&template=bug_report.md
[feature-url]: https://github.com/0x4rkh4m/leeca_proxmox/issues/new?labels=enhancement&template=feature_request.md
