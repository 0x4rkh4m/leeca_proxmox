# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Internal HTTP client (`ApiClient`)** – centralises all API requests, handles authentication headers and automatic ticket refresh on 401 responses.
- **Rate limiting foundation** – prepared for client‑side rate limiting (to be completed in 0.3.x).
- **Session persistence groundwork** – `ApiClient` stores authentication state in an `Arc<RwLock>`, ready for save/load.

### Changed
- **`ProxmoxClient` now uses `ApiClient` internally** – authentication state is managed by the new client.
- **Authentication methods (`is_authenticated`, `auth_token`, `csrf_token`, `is_ticket_expired`, `is_csrf_expired`) are now `async`** – they need to read the internal lock.
- **Builder `build()` now returns a `ProxmoxClient` with an `ApiClient`** – no changes to the builder interface.

### Fixed
- CSRF token validation in tests – now uses valid alphanumeric tokens.

### Removed
- Direct `connection` and `auth` fields from `ProxmoxClient` – they are now inside `ApiClient`.

## [0.2.0] - 2026-02-19

### Added
- **Validation configuration**: Password strength, DNS resolution, reserved usernames – all optional and off by default.
- **Simplified value objects**: Removed `ValueObject` trait and async locking; values are now plain structs with synchronous getters.
- **Builder improvements**: `ProxmoxClientBuilder` now defaults to secure HTTPS, and accepts a custom `ValidationConfig`.
- **New methods**: `ProxmoxClient::is_ticket_expired()`, `is_csrf_expired()` for easy expiry checks.
- **Documentation**: Expanded examples and migration guide for 0.1.x users.

### Changed
- **MSRV**: Now requires nightly (still) but the edition is updated to 2024.
- **API**: All validation now happens at build time, reducing async noise and improving ergonomics.
- **Error messages**: More descriptive validation errors.

### Removed
- **`ValueObject` trait**: No longer needed; each value object has its own `new_unchecked` constructor.
- **`into_inner` methods**: Not used; can be added later if required.

### Security
- Password strength checking is now opt‑in (default off) to respect server‑side policies.
- DNS resolution is opt‑in; avoids blocking on build.

## [0.1.2] - 2026-02-19
### Added
- Dependabot configuration for automated dependency updates.
- Code coverage reporting via Codecov.
- Robust CI pipeline (formatting, linting, audit, docs, tests).

### Changed
- Updated all dependencies to latest versions (resolves RUSTSEC advisories).
- Migrated to Rust 2024 edition.
- Integration tests now ignored by default; require real Proxmox instance.

## [0.1.1] - 2025-01-14
### Added
- Public `value()` and `expires_at()` methods for tickets and CSRF tokens.
- Enhanced token lifetime visibility.

### Fixed
- CSRF token validation to match Proxmox VE format.
- Login response parsing.

## [0.1.0] - 2025-01-13
### Added
- Initial release with core authentication, value objects, and async support.
