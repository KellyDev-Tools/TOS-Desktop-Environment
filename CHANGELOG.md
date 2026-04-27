# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.2-beta.0] - 2026-04-27

### Added
- **Cortex Orchestration System**: Decoupled hardcoded AI backends into a pluggable registry supporting `.tos-assistant`, `.tos-curator`, and `.tos-agent` modules (Eco §1.3).
- **Secure Auth Injection**: Implemented `[auth]` credential injection via the secure Settings store for all Cortex modules (Eco §1.3.4).
- **Brain Trust Chip**: Integrated `[trust]` declarations and manifest verification for behavioral modules (Eco §1.3.5).
- **Multi-Transport Cortex**: Support for `http`, `stdio`, and `mcp` connection protocols for AI services (Eco §1.3.1).
- **Agent Stacking**: Hierarchical prompt merging and identity resolution for nested agent personas (§6).
- **Cortex Configuration UI**: Unified management interface for AI backends and behaviors in the Settings Modal.
- **GitNexus Curator**: Reference implementation of a curator module using MCP for workspace analysis.

### Changed
- **Assistant Migration**: Migrated legacy Ollama and Gemini shims to the new `.tos-assistant` cortex standard (Eco §1.15).

## [0.2.1-beta.0] - 2026-04-24

### Added
- **Remote Collaboration Stack**: Full WebRTC signalling and video streaming support in `remote_server.rs` (§12.1).
- **Collaboration Roles**: Implemented administrative role enforcement (Viewer, Commenter, Operator, Co-Owner) for remote participants (§13.2).
- **SSH Fallback**: Interactive PTY bridge for controlling non-TOS legacy remotes via SSH (§27.3).
- **Web Portal**: Secure sector sharing via time-limited one-time tokens and expiry logic (§12.2).
- **Native Platform Features**: Three-layer audio model (Ambient, Tactical, Voice) with adaptive alert-level volume escalation (§23.1, §23.2).
- **Haptic & Visual Feedback**: Integrated haptic patterns and FPS-based Tactical Alerts (§23.4, §16.4).
- **Advanced Accessibility**: Linux Screen Reader bridge (AT-SPI) and full keyboard navigation tab-stop chain (§24).
- **Security Provisioning**: PKCS#11-compliant HSM key provisioning and release signing infrastructure via `tos-signer` utility.

### Fixed
- **Audit Logging**: Implemented participant-aware command logging for guest actions (§13.6).
- **Render Throttling**: Depth-based frame rate optimization for background viewports (§16.1).
- **mDNS Discovery**: Zero-config discovery via `_tos-brain._tcp` mDNS advertisement.

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
