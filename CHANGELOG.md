# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0-beta.0] - 2026-03-27

### Added
- **Dynamic Service Registry**: Implemented discovery gate at `/tmp/brain.sock` for ephemeral daemon registration (§4.1).
- **Service Security**: Ed25519-signed registration handshake for all system services.
- **Protocol Library**: Extracted `tos-protocol` into a standalone workspace crate for Face-Brain contracts.
- **Input Abstraction**: Normalized all raw physical input into `SemanticEvent` enums (§14.1).
- **Search Daemon**: Integrated `tos-searchd` with literal and semantic (vector) search capabilities.
- **Marketplace Daemon**: Service registration and discovery for installable modules.
- **Session Persistence**: Live session state auto-save and restore.
- **Beta-0 Architecture Specs**: Canonical documentation suite in `docs/spec/`.

### Fixed
- Standardized logging across all daemons using `tracing` macros (§2.1).
- Unified `tos_common` registration helper for satellite daemons.
- Removed legacy string-based socket paths; all internal discovery now uses the `ServiceRegistry`.
- Decoupled Android platform layer (`tos-android`) for independent build pipelines.
