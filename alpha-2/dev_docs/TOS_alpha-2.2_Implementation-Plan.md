# TOS Alpha-2.2 Implementation Plan
### Active Development Tracker — Started 2026-03-04

---

## Current Sprint: Phase 1 — Core Infrastructure

The roadmap defines 5 phases. We execute Phase 1 first because *every subsequent phase depends on it*. Phase 1 tasks are parallelizable — they have no dependencies on each other.

### Phase 1 Task Status

| # | Task | Status | Notes |
|---|------|--------|-------|
| 1.1 | `tos-protocol` Extraction | ✅ DONE | Workspace created, 14 protocol tests passing, bridge re-exports preserve all import paths |
| 1.2 | Settings Daemon Schema Extensions | ✅ DONE | 6 namespaces added (onboarding, trust, AI, bezel, splits, network), 8 tests passing |
| 1.3 | Service Registry & Port Infrastructure | ✅ DONE | `ServiceRegistry` with CRUD/heartbeat, `tos_ports` IPC command, 6 inline tests |
| 1.4 | Unified Visual Token System | ⬜ QUEUED | `assets/design_tokens.json` consumed by CSS + LinuxRenderer |
| 1.5 | Headless Brain Testing | ⬜ QUEUED | `test-protocol` suite for Brain state deltas |
| 1.6 | OSC-Exclusive Mode Switching | ⬜ QUEUED | Deprecate string sniffing in `ipc_handler.rs` |

### Phase 1 Implementation Order (within sprint)

Starting with **1.1 `tos-protocol` Extraction** because it establishes the shared type contract that all other Phase 1 work builds on:

1. **`tos-protocol` crate** — Extract `src/common/` types into `tos-protocol/` workspace member
2. **Settings Schema Extensions** — Add new namespaces to `SettingsStore`
3. **Service Registry** — Unix socket + ephemeral port registration + anchor port
4. Continue remaining Phase 1 tasks

---

## Completed Items

### 1.1 `tos-protocol` Extraction ✅
- Created `tos-protocol/` workspace member with shared types (`state`, `ipc`, `modules`, `collaboration`)
- 14 TDD tests validating: default state, settings cascading, serialization round-trips, IPC trait object safety, hierarchy levels
- `src/common/mod.rs` rewritten as re-export bridge — zero import changes needed across the codebase
- `tos-sessiond` daemon skeleton added to workspace (ephemeral port, atomic live-state writes, session CRUD)
- `Makefile` updated: version bumped to 2.2, `tos-sessiond` in `run-services`
- All pre-existing tests pass (tier1: 3/3, tier4_security: 1/1, protocol: 14/14)

### Bug Fixes (during 1.1)
- **Shell fallback**: Brain no longer crashes when `fish` isn't installed — falls back to `$SHELL` → `/bin/bash` → `/bin/sh`
- **Module path resolution**: Shell module manifests with absolute paths (e.g. `/usr/bin/fish`) now resolve correctly
- **Surface leak**: Fixed Wayland renderer creating new memfd+mmap every frame — now creates once, reuses handle
- **UI flicker**: Added ANSI clear-screen + cursor hide/show; downgraded per-frame logs to debug; tracing now writes to stderr

### 1.2 Settings Schema Extensions ✅
- 6 new setting namespaces: `tos.onboarding.*`, `tos.trust.*`, `tos.ai.*`, `tos.interface.bezel.*`, `tos.interface.splits.*`, `tos.network.*`
- 8 TDD tests validating: presence, default values, cascading, serialization round-trip
- `SettingsService::build_default_settings()` is the canonical source of truth for all defaults

### 1.3 Service Registry & Port Infrastructure ✅
- `ServiceRegistry` struct: register, deregister, heartbeat, mark_dead, port lookup, alive filtering
- `port_table()` method returns human-readable table for `tos ports` CLI
- 3 new IPC commands: `tos_ports:`, `service_register:name;port`, `service_deregister:name`
- `ServiceManager` creates registry on startup with anchor port from settings
- 6 inline unit tests for registry CRUD, heartbeat, and filtering

---

## Reference: Full Phase Map

- **Phase 1** — Core Infrastructure (current)
- **Phase 2** — Brain Services (Session, Trust, LLM Bridge, AIService, Split Pane Tree, Heuristic)
- **Phase 3** — Face Features & Intelligence (Onboarding, Session UI, Split Viewport, Expanded Bezel, AI Behaviors, Marketplace, Predictive, Vector Search)
- **Phase 4** — High-Fidelity Visual Layer (Console, Zoom, Thumbnails, Context Menus, Borders, Frame Captures, God Mode)
- **Phase 5** — Native Platform Faces (Wayland, OpenXR, Android)

See `TOS_alpha-2.2_Production-Roadmap.md` for the full dependency graph.
