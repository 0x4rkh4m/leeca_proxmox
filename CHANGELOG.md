# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1] - 2025-01-14
### Changed
- Removed base64 validation requirement for CSRF tokens to match Proxmox VE API format
- Updated LoginResponse structure to properly handle Proxmox API response format
- Improved error handling in login service
- Enhanced documentation with security best practices and examples
- Updated builder pattern documentation with detailed security considerations

### Fixed
- Fixed CSRF token validation to accept Proxmox VE format
- Fixed login response parsing to handle nested data structure
- Fixed API path validation to support sub-paths in ProxmoxUrl

### Security
- Improved certificate validation controls
- Enhanced security documentation for TLS and certificate handling
- Added detailed warnings for development-only security options

## [0.1.0] - 2025-01-13
### Added
- Project structure
- Basic client implementation
- Core domain models
- Authentication flow
- Initial documentation
- Initial SDK implementation
- Authentication support
- Value Objects for domain entities
- Async operations support
- Comprehensive test suite
