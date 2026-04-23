# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0-beta.0] - 2026-04-23

### Added
- **TOS Editor System**: Full-featured editor pane with Viewer, Editor, and Diff modes. Includes PrismJS syntax highlighting, auto-open on build error, and LSP integration (§6.2).
- **Vibe Coder**: Multi-step AI planning and execution engine with staging and individual approval of diffs (§4.8).
- **Kanban & Agent Orchestration**: Integrated Kanban Board service, Workflow Manager UI, and persona-driven agent execution (§7, Eco §1.6).
- **Predictive Intelligence**: Command predictor (ghost text), path completion chips, and typo correction via `tos-heuristicd` (§4.4, §31).
- **AI Contextual Awareness**: Rolling context aggregator and context-signal skill activation for passive observation (§4.7).
- **Tactical UI Polish**: Kinetic borders, depth-aware priority indicators, and a tactical mini-map with projection overlays (§21, §22).
- **Offline AI Support**: Request queue with auto-drain logic for intermittent connectivity (§4.9).
- **Advanced Accessibility**: Full keyboard navigation tab-stop chain, focus containment (focus traps) for modals, and LCARS-compliant focus indicators (§5.7).
- **Roadmap Maintenance Standards**: Formalized requirements for documentation archival and changelog integration.

### Fixed
- **Security Hardening**: Enforced `tool_bundle` permissions at runtime and verified module manifest signatures.
- **IPC Performance**: Optimized handler logic to ensure round-trip latency < 16ms.
- **Session Stability**: Implemented silent restore and atomic session persistence to prevent state corruption (§2.6.2).
- **Daemon Resilience**: Added exponential backoff for service registration retries.

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
