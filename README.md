# Leeca Proxmox VE SDK for Rust

A modern, safe, and async-first SDK for interacting with Proxmox Virtual Environment servers, following industry best practices and clean architecture principles.

## Features

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

- 🧪 **Quality Assurance**
  - Comprehensive test suite
  - Integration tests
  - Property-based testing
  - Mocked HTTP responses
  - CI/CD pipeline

- 📚 **Rich Documentation**
  - Full API documentation
  - Code examples
  - Architecture guides
  - Best practices
  - Migration guides

## Getting Started

### Installation

```
cargo add leeca_proxmox
```

### Quick Example

```rust
use leeca_proxmox::{ProxmoxClient, ProxmoxResult};

#[tokio::main]
async fn main() -> ProxmoxResult<()> {
    let mut client = ProxmoxClient::builder()
        .host("192.168.1.182")?
        .port(8006)?
        .credentials("leeca", "Leeca_proxmox1!", "pam")?
        .secure(false)
        .build()
        .await?;

    client.login().await?;
    println!("Authenticated: {}", client.is_authenticated());

    if let Some(token) = client.auth_token() {
        println!("Session Token: {}", token.value().await);
        println!("Session Token expires at: {:?}", token.expires_at().await);
    }

    if let Some(csrf) = client.csrf_token() {
        println!("CSRF Token: {}", csrf.value().await);
        println!("CSRF Token expires at: {:?}", csrf.expires_at().await);
    }

    Ok(())
}

```

## Documentation

- [SDK Documentation](https://docs.rs/leeca_proxmox)
- [Architecture Guide](docs/architecture.md) <!-- TODO: Add architecture guide -->
- [Examples](examples/)
- [Contributing Guide](CONTRIBUTING.md)
- [Security Policy](SECURITY.md)

## Project Guidelines

- [Code of Conduct](CODE_OF_CONDUCT.md)
- [Contributing Guidelines](CONTRIBUTING.md)
- [Issue Templates](.github/ISSUE_TEMPLATE/)
- [Pull Request Template](.github/PULL_REQUEST_TEMPLATE.md)
- [Security Policy](SECURITY.md)

## Development

### Requirements

- Rust (nightly) <!-- Right now, we are using nightly for #[backtrace] not being stable yet -->
- Cargo
- Tokio runtime

### Quality Checks

```bash
# Run linter
cargo clippy

# Run formatter
cargo fmt

# Run tests with coverage
cargo tarpaulin
```

## Project Status

See our [CHANGELOG](CHANGELOG.md) for version history and [ROADMAP](ROADMAP.md) for future plans.

## Legal

- [License](LICENSE) - Apache License 2.0
- [Notice](NOTICE) - Third-party licenses
- [Security](SECURITY.md) - Security policy and reporting

## Community

- [Issue Tracker](https://github.com/0x4rkh4m/leeca_proxmox/issues)
- [Discussions](https://github.com/0x4rkh4m/leeca_proxmox/discussions)
- Email: [4rkham@proton.me](mailto:4rkham@proton.me)

## Versioning

This project follows [Semantic Versioning](https://semver.org/). See our [CHANGELOG](CHANGELOG.md) for version history.

⚠️ **Note**: This project is in active development. APIs may change before 1.0.0 release.

## Acknowledgments

- Proxmox VE team for their excellent API documentation
- Rust community for their tools and crates
- All [contributors](https://github.com/0x4rkh4m/leeca_proxmox/graphs/contributors)
