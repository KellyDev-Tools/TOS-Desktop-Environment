# TOS Beta-0 Roadmap Archive

This document contains completed sections and stages from the [TOS_v0.1_Roadmap.md](../../TOS_v0.1_Roadmap.md), moved here to maintain focus on remaining work.

---

## Part 1 — Completed Audit Sections

### 1.1 Core Architecture (Architecture §1–§4)

| Feature | Spec Ref | Status | Evidence |
|---|---|---|---|
| 4-Level hierarchy (Global→Hub→App→Detail) | §2 | ✅ | `HierarchyLevel` enum, `HierarchyManager` zoom_in/zoom_out/set_level |
| Buffer View (Level 5) | §9 | ✅ | `HierarchyLevel::BufferView` enum + `DetailInspector.svelte` BUFFER_VIEW mode with `getBuffer()` integration |
| Marketplace as Level 6 | §2 (extension) | ✅ | `HierarchyLevel::Marketplace` + Svelte `Marketplace.svelte` |
| Brain / Face process separation | §3.1 | ✅ | Separate `brain/` crate, `face-svelte-ui/`, `face-electron-any/` |
| IPC prefix:payload protocol | §3.3.1 | ✅ | `IpcHandler::handle_request()` — 1998 lines, 80+ message types |
| Face registration with capability profile | §3.3.5 | ✅ | `face_register` IPC, `FaceProfile` enum (Desktop/Handheld/Spatial) |
| Disconnected Mode (heartbeat, frozen state) | §3.4 | ✅ | `DisconnectOverlay.svelte` + 5s heartbeat timeout in `ipc.svelte.ts` |
| No Brain state (connection UI) | §3.4 | ✅ | `DisconnectOverlay.svelte` + dynamic WebSocket connection |
| State delta sync (1Hz tick) | §3.4.2 | ✅ | `remote_server.rs` 1Hz push loop + `get_state_delta` handler in IpcHandler |
| Bezel slot mechanism (Top/Left/Right) | §5 | ✅ | `ExpandedBezel.svelte`, slot components: BrainStatus, MiniLog, Minimap, PriorityStack, Telemetry |
| Expanded Bezel Command Surface | §5.4 | ✅ | `bezel_expand`/`bezel_collapse` IPC + overlay rendering |
| Bezel as overlay (not a level) | Features §1.9 | ✅ | `bezel_expanded: bool` flag in TosState |

### 1.2 Sector & Command Hub (Architecture §6–§8, §10)

| Feature | Spec Ref | Status | Evidence |
|---|---|---|---|
| Sector CRUD (create, clone, freeze, close) | §6 | ✅ | `SectorManager` + IPC handlers + `SectorContextMenu.svelte` |
| Sector from template | §6 | ✅ | `sector_create_from_template` IPC + built-in templates |
| Dynamic sector labeling from cwd | §31.3 | ✅ | PTY read loop auto-relabels sectors with default names on OscEvent::Cwd |
| Sector tree model | §10 | ✅ | `Sector` → `Vec<CommandHub>` hierarchy |
| Command Hub modes (CMD/DIR/ACT/SEARCH/AI) | §7 | ✅ | `CommandHubMode` enum + IPC set_mode + auto-detection |
| Persistent Unified Prompt | §7.2 | ✅ | Prompt visible at all levels in `CommandHub.svelte` |
| Auto Directory Mode on `ls`/`cd` | §27.5 | ✅ | Brain command dispatcher sniffs `ls`/`cd` prefix |
| Auto Activity Mode on `top`/`ps` | §7.3 | ✅ | First-token sniffing in `handle_prompt_submit` for top/htop/btop/ps/atop/glances |
| Directory pick behavior | §27.6 | ✅ | `dir_pick_file`/`dir_pick_dir` handlers implemented; staging banner added |
| Shell OSC integration (9002/9003/9004) | §27.1 | ✅ | Shell scripts in `scripts/`, OSC parsing in ShellApi |
| Line-level priority (OSC 9012) | §27.4 | ✅ | `OscEvent::LinePriority` variant + `OscParser.process()` wired in PTY read loop |
| Command auto-detection (no false positives) | §27.5 | ✅ | Tested — `rls`, `echo cd` don't trigger |
| Terminal buffer limit (500 default, adjustable) | §29.2 | ✅ | `buffer_limit: 500` in CommandHub + `set_terminal_buffer_limit` IPC |
| ANSI stripping before storage | §29.1 | ✅ | Implemented in shell reader |

### 1.3 Cortex Orchestration Layer (Features §4, §7 & Ecosystem §1.3)

| Feature | Spec Ref | Status | Evidence |
|---|---|---|---|
| AiService with behavior registry | §4.1 | ✅ | `AiService` with register/enable/disable/configure |
| Rolling context aggregator | §4.7 | ✅ | `AiContext` with field-level filtering |
| Per-behavior backend override cascade | §4.3 | ✅ | `resolve_backend()` with cascade |
| Passive Observer (correction chips) | §4.5 | ✅ | `passive_observe()` with exit code analysis |
| Chat Companion | §4.6 | ✅ | `query()` with OpenAI fallback + offline heuristics |
| Command Predictor (ghost text) | §4.4 | ✅ | `predict_command` with AI/Heuristic fallbacks + Tab-to-accept UI |
| Vibe Coder (multi-step planning) | §4.8 | ✅ | `vibe_plan` orchestration + staged AI thought sequence |
| Thought bubble / expand | §4.6 | ✅ | `ActiveThoughts.svelte` component + `ai_thought_stage` IPC |
| AI safety contracts (no auto-submit) | §4.12 | ✅ | Enforced via `ai_chip_stage` staging only |
| Offline AI queue | §4.9 | ✅ | `ai_offline_queue` in `TosState` + storage/drain logic in `AiService` |
| Context-signal skill activation | §4.7 | ✅ | `check_context_signals` wired in PTY read loop |
| Editor Context Object | §6.5.1 | ✅ | `AiContext` aggregates all active editor states recursively |
| Kanban Board Model (JSON, lanes, tasks) | Features §7.2 | ✅ | `KanbanBoard` struct in `state.rs` + session persistence |
| Agent Manifest Format (`.tos-agent`) | Ecosystem §1.3.3 | ✅ | `parse_agent_toml` in `ai/mod.rs` + stacking logic |
| Roadmap Agent (Task generation) | Ecosystem §1.3.3 | ✅ | `roadmap_plan` agent implemented in `AiService` |
| Workflow Manager Pane (`workflow`) | Arch §11.2 | ✅ | `WorkflowManager.svelte` view implemented |
| Agent Sandboxing & Merge Logic | Features §7.7 | ✅ | `exec_sandboxed` with `overlayfs`; automated merge logic via `similar` diffing |
| LLM Interaction Archival service | Features §2.9.1 | ✅ | `dream_consolidate` skill + logger integration |
| Multi-agent terminal routing (isolated PTYs) | Arch §10.1.3 | ✅ | `PtyShell::exec_isolated` for independent agent execution |

### 1.4 Split Viewports (Architecture §11)

| Feature | Spec Ref | Status | Evidence |
|---|---|---|---|
| Recursive split tree data model | §11 | ✅ | `SplitNode`, `SplitPane`, `SplitOrientation` types |
| Aspect-ratio-driven orientation | §11.3 | ✅ | `SplitNode::ideal_orientation()` |
| Minimum pane size / split blocking | §11.5 | ✅ | `SplitNode::can_split()` with ratio + content-aware minimums |
| Split IPC (create, close, focus, resize, swap, etc.) | §11.11 | ✅ | All 14 split IPC messages handled |
| Pane content types (terminal, editor, app) | §11.2 | ✅ | `PaneContent::Terminal`, `PaneContent::Application`, `PaneContent::Editor(EditorPaneState)` |
| Bezel pane management chips (Fullscreen, Swap, Detach) | §11.8 | ✅ | `ExpandedBezel.svelte` renders Fullscreen/Swap/Detach chips wired to `splitFullscreen()`, `splitSwap()`, `splitDetachContext()` |
| Divider drag / snap assist | §11.6 | ✅ | `SplitLayout.svelte` full mouse drag interaction + snap assist at 25/50/75% thresholds |
| Split state persistence | §11.9 | ✅ | `split_layout` field persisted in session via `CommandHub` Serde + `split_save_template` IPC for named layouts |

### 1.5 Remote Sectors & Collaboration (Architecture §12–§13)

| Feature | Spec Ref | Status | Evidence |
|---|---|---|---|
| Remote Server protocol (TLS, WebSocket) | §12.1 | ✅ | `remote_server.rs` — `rustls` integration + self-signed cert gen |
| WebRTC signalling | §12.1 | ✅ | `remote_server.rs` — `webrtc-rs` stack + SDP/ICE handlers |
| SSH fallback | §27.3 | ✅ | `ssh_fallback.rs` — PTY bridge for legacy host control |
| Remote disconnect (5s auto-close) | §27.3 | ✅ | `handle_remote_disconnect` with tokio timer |
| Collaboration roles (Viewer/Commenter/Operator/Co-owner) | §13.2 | ✅ | `collaboration.rs` roles + `IpcHandler::check_permission` enforcement |
| Following mode & cursor sync | §13.4 | ✅ | `WebRtcPayload::Following` + `CursorSync` handled in `IpcHandler` |
| Web Portal (sector sharing URL) | §12.2 | ✅ | `PortalService` — secure token generation + expiry logic |
| Audit logging for guest actions | §13.6 | ✅ | `IpcHandler` logs remote commands with participant ID and role context |

### 1.10 Service Daemons (Ecosystem §3–§4)

| Feature | Spec Ref | Status | Evidence |
|---|---|---|---|
| Dynamic port registration via brain.sock | §4.1 | ✅ | All 7 daemons register dynamically |
| ServiceRegistry (port map, discovery) | §4.2 | ✅ | `registry.rs` with register/deregister/port_of |
| tos-sessiond (live + named sessions) | §3.2 | ✅ | Full local + daemon dual-path persistence |
| tos-settingsd (cascading settings) | §3.2 | ✅ | `SettingsStore` with 3-level cascade |
| tos-loggerd (event logging) | §3.2 | ✅ | Running daemon with structured log output |
| tos-searchd (filesystem search) | §3.2 | ✅ | Daemon with basic index + semantic bridge |
| tos-marketplaced (module registry) | §3.2 | ✅ | Daemon + MarketplaceService facade |
| tos-heuristicd (sector labeling) | §3.2 | ✅ | Running daemon |
| tos-priorityd (priority scoring) | §3.2 | ✅ | Running daemon |
| mDNS advertisement | §5.2 | ✅ | `mdns-sd` dependency; `_tos-brain._tcp` advertised |
| Exponential backoff on registration retry | §3.3 | ✅ | 10 retries with 100ms→10s doubling in `register_with_brain()` |

### 1.11 Marketplace UI (Features §5)

| Feature | Spec Ref | Status | Evidence |
|---|---|---|---|
| Marketplace home view (featured + categories) | §5.3 | ✅ | `Marketplace.svelte` (31KB) + `marketplace_home` IPC |
| Category browse view | §5.4 | ✅ | `marketplace_category` IPC + grid rendering |
| Module detail page | §5.5 | ✅ | `marketplace_detail` IPC |
| Permission review step | §5.6.1 | ✅ | Detail page shows permissions + scroll-to-consent gate in `Marketplace.svelte` |
| Install flow (progress, cancellation) | §5.6 | ✅ | `marketplace_install` + `marketplace_install_cancel` IPC + progress display |
| AI-assisted search | §5.7 | ✅ | `marketplace_search_ai` IPC |
| Installed state badge | §5.8 | ✅ | `[Installed ✓]` badge rendered in browse cards |

### 1.12 TOS Editor (Features §6)

| Feature | Spec Ref | Status | Evidence |
|---|---|---|---|
| Editor pane type | §6.3.1 | ✅ | `PaneContent::Editor(EditorPaneState)` with `EditorMode` (Viewer/Editor/Diff) + `DiffHunk` |
| Viewer / Editor / Diff modes | §6.2 | ✅ | Svelte `EditorPane.svelte` + PrismJS + textarea overlay |
| Auto-open on build error | §6.3.2 | ✅ | `renderTermLine` interactive span tags + `!ipc editor_open` |
| AI Context Panel | §6.5.2 | ✅ | `AiContextPanel.svelte` in Right Bezel slot |
| Inline AI annotations | §6.5.4 | ✅ | `EditorAnnotation` schema + amberPulse scroll $effects |
| AI Edit Flow / Diff Mode | §6.6 | ✅ | Side-by-side Diff Mode + IPC proposal pipeline |
| Multi-file edit chip sequence | §6.6.3 | ✅ | Vibe Coder `vibe_plan` logic with multi-stage thoughts |
| LSP diagnostics integration | §6.9 | ✅ | `LspService` backend (rust-analyzer/tsserver) + diagnostic streams |
| Editor IPC messages (§30.3–§30.4) | §30 | ✅ | 16 dedicated IPC handlers implemented in IpcHandler |

### 1.13 Session Persistence (Features §2)

| Feature | Spec Ref | Status | Evidence |
|---|---|---|---|
| Live auto-save (debounced) | §2.1 | ✅ | `debounced_save_live()` with 1s debounce |
| Named session save/load/delete | §2.3 | ✅ | Full CRUD via daemon or local disk |
| Session export / import | §2.5 | ✅ | `session_export` / `session_import` IPC |
| Cross-device handoff (one-time tokens) | §2.6 | ✅ | `session_handoff_prepare/claim` IPC + `tos-sessiond` token registry |
| Crash recovery (atomic rename) | §2.4 | ✅ | `_live.tos-session.tmp` → rename on success |
| Silent restore (no notification) | §2.6.2 | ✅ | Notification suppressed on restore in `brain/mod.rs` |
| Editor pane state persistence | §2.9 | ✅ | `EditorPaneState` serialized via `PaneContent::Editor` → `SplitNode` → `CommandHub.split_layout` in session snapshots |

### 1.14 Onboarding (Features §3)

| Feature | Spec Ref | Status | Evidence |
|---|---|---|---|
| Cinematic intro (skippable) | §3.2 | ✅ | `OnboardingOverlay.svelte` full cinematic flow with skippable logic |
| Guided demo in live system | §3.3 | ✅ | Onboarding sequence with integrated tactical demo |
| Trust configuration during wizard | §3.4 | ✅ | Initial trust tier selection and class promotion in wizard |
| Cold-start ≤ 5s gate | §3.1 | ✅ | Brain init ~1s; verified in telemetry |
| Ambient hints (per-hint dismiss) | §3.6 | ✅ | `AmbientHint.svelte` (121 lines) with per-hint dismiss, settings persistence + `onboarding_hint_dismiss/suppress/reset` IPC handlers |

### 1.17 RESET OPERATIONS (ARCHITECTURE §20)

| Feature | Spec Ref | Status | Evidence |
|---|---|---|---|
| Sector reset (SIGTERM, clean) | §20.1 | ✅ | SIGTERM to shell PGID + sandbox cleanup + state reset implemented |
| System reset dialog | §20.2 | ✅ | Full confirmation modal in `GlobalOverview.svelte` with "RED ALERT" keyword gate + EXECUTE_RESET button |

### 1.17 Priority & Visual Indicators (Architecture §21)

| Feature | Spec Ref | Status | Evidence |
|---|---|---|---|
| Priority scoring (weighted factors) | §21.2 | ✅ | `PriorityStack.svelte` + `tos-priorityd` + `GlobalOverview.svelte` depth-aware priority classes (3/4/5) |
| Border chips / chevrons / glow | §21.1 | ✅ | Kinetic borders (`border-running` gradient animation), priority glow (`priority-3/4/5` CSS), `priority-chip` component, `redAlertPulse` animation in `GlobalOverview.svelte` |
| Tactical Mini-Map | §22 | ✅ | `Minimap.svelte` (359 lines) with depth-aware content: L1 sector tiles, L2 hub hierarchy, L4 inspection target + expanded projection overlay |

### 1.18 Settings (Architecture §26)

| Feature | Spec Ref | Status | Evidence |
|---|---|---|---|
| Layered settings cascade | §26.1 | ✅ | `SettingsStore::resolve()` — app → sector → global |
| Settings IPC (get/set/tab) | §26.3 | ✅ | All settings IPC messages handled |
| Settings UI (modal) | §26 | ✅ | `SettingsModal.svelte` (28KB) — comprehensive tabs |
| Persistence to disk | §26.4 | ✅ | JSON file via tos-settingsd |

---

## Part 2 — Completed Roadmap Stages

### Stage 5 — Native Platform & Multi-Sensory

| # | Task | Priority | Spec Ref | Deps | Status |
|---|---|---|---|---|---|
| 5.1 | Real Wayland compositor integration test | HIGH | Arch §15.2 | WaylandRenderer | ✅ |
| 5.2 | DMABUF frame buffer sharing for Level 3 apps | HIGH | Arch §15.2 | WaylandRenderer | ✅ |
| 5.3 | Three-layer audio model (ambient/tactical/voice) | MEDIUM | Arch §23.1 | AudioService | ✅ |
| 5.4 | Alert level adaptation (Green/Yellow/Red) | MEDIUM | Arch §23.2 | AudioService, SettingsStore | ✅ |
| 5.5 | Haptic feedback patterns on Android | LOW | Arch §23.4 | HapticService | ✅ |
| 5.6 | Screen reader bridge (AT-SPI on Linux) | HIGH | Arch §24.1 | Face components | ✅ |
| 5.7 | Full keyboard navigation tab-stop chain | HIGH | Arch §24.3 | All Svelte components | ✅ |
| 5.8 | High-contrast forced mode | MEDIUM | Arch §24.1 | Theme system | ✅ |
| 5.9 | FPS monitoring + Tactical Alert | LOW | Arch §16.4 | Renderer, alerting | ✅ |
| 5.10 | Voice command input pipeline | LOW | Arch §14.3 | Input hub | ✅ |
| 5.11 | Depth-based render throttling | LOW | Arch §16.1 | Renderer | ✅ |

### Stage 6 — Collaboration, Remote & Release

| # | Task | Priority | Spec Ref | Deps | Status |
|---|---|---|---|---|---|
| 6.1 | TLS handshake in Remote Server protocol | HIGH | Arch §12.1 | remote_server.rs | ✅ |
| 6.2 | WebRTC signalling + video stream | HIGH | Arch §12.1 | remote_server.rs | ✅ |
| 6.3 | Session handoff (one-time tokens, 10min expiry) | HIGH | Features §2.6 | SessionService | ✅ |
| 6.4 | Collaboration role enforcement (Viewer→Operator) | MEDIUM | Arch §13.2 | collaboration.rs | ✅ |
| 6.5 | SSH fallback for non-TOS remotes | MEDIUM | Arch §27.3 | ssh_fallback.rs | ✅ |
| 6.6 | mDNS discovery test in real network | MEDIUM | Eco §5.2 | mdns-sd | ✅ (verify) |
| 6.7 | HSM key provisioning for release signing | HIGH | — | CI infrastructure | ✅ |
| 6.8 | Generate signed release assets | HIGH | — | 6.7 | ✅ |
| 6.9 | E2E Playwright tests for Svelte UI | HIGH | Dev §4.2 | face-svelte-ui | ✅ |
| 6.10 | Crash reporting infrastructure (opt-in) | LOW | — | tos-loggerd | ✅ |
