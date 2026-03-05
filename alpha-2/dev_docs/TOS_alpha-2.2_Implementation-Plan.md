# TOS Alpha-2.2 Implementation Plan
### Active Development Tracker — Started 2026-03-04

---

## Current Sprint: Phase 1 — Core Infrastructure

The roadmap defines 5 phases. We execute Phase 1 first because *every subsequent phase depends on it*. Phase 1 tasks are parallelizable — they have no dependencies on each other.

### Phase 1 Task Status

| # | Task | Status | Notes |
|---|------|--------|-------|
| 1.1 | `tos-protocol` Extraction | ✅ DONE | Workspace created, 14 protocol tests passing, bridge re-exports preserve all import paths |
| 1.2 | Settings Daemon Schema Extensions | ⬜ QUEUED | New namespaces: `tos.onboarding`, `tos.trust`, `tos.ai.behaviors`, `tos.interface.bezel`, `tos.interface.splits`, `tos.network` |
| 1.3 | Service Registry & Port Infrastructure | ⬜ QUEUED | Unix socket, ephemeral ports, anchor port, mDNS, `tos ports` CLI |
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

---

## Reference: Full Phase Map

- **Phase 1** — Core Infrastructure (current)
- **Phase 2** — Brain Services (Session, Trust, LLM Bridge, AIService, Split Pane Tree, Heuristic)
- **Phase 3** — Face Features & Intelligence (Onboarding, Session UI, Split Viewport, Expanded Bezel, AI Behaviors, Marketplace, Predictive, Vector Search)
- **Phase 4** — High-Fidelity Visual Layer (Console, Zoom, Thumbnails, Context Menus, Borders, Frame Captures, God Mode)
- **Phase 5** — Native Platform Faces (Wayland, OpenXR, Android)

See `TOS_alpha-2.2_Production-Roadmap.md` for the full dependency graph.
