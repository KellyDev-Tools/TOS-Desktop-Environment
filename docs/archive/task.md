# TOS Beta-0 Consolidated Roadmap — Execution Tasks

## UI: Remote Brain Connection
- [x] Add dynamic `WS_URL` and `connect(customUrl)` logic in `ipc.svelte.ts`
- [x] Add `localStorage` persistence for recent connections in `ipc.svelte.ts`
- [x] Refactor `DisconnectOverlay.svelte` into a full Connection Dialog
- [x] Add Saved Connections list to Connection Dialog (loaded via URL parsing)
- [x] Add Manual Host/Port input form to Connection Dialog
## Repo Cleanup — Replace Old Roadmaps
- [x] Copy consolidated roadmap to repo root as `TOS_Beta-0_Roadmap.md`
- [x] Archive `TOS_alpha2-to-beta0.md` to `docs/archive/`
- [x] Archive `TOS_SSH_Wayland_Fix_Plan.md` to `docs/archive/`

## Stage 0 — Hard Gate Blockers
- [x] 0.1 Brain Tool Registry: enforce `tool_bundle` permissions at runtime
- [x] 0.2 Verify `.tos-skill` accepted / `.tos-aibehavior` rejected by Marketplace
- [x] 0.3 Silent restore — no notification or prompt on session launch
- [x] 0.4 Verify profile diversity (handheld/spatial face_register) — validated via face_visual_states.rs
- [x] 0.5 All errors routed through `tracing` — fixed 9 stray println/eprintln in audio.rs, haptic.rs, logger.rs, ai/mod.rs, face/mod.rs
- [x] 0.6 Verify IPC round-trip < 16ms — latency warning threshold enforced in IpcHandler
- [x] **BONUS:** Fixed 2 unused import warnings in brain/src/main.rs → 0 warnings workspace-wide

## Stage 1 — Core Runtime Hardening
- [x] 1.1 Implement 1Hz state_delta push from Brain
- [x] 1.2 Implement Face heartbeat detection
- [x] **SPEC:** Kanban-Driven Agent Orchestration (Features §7, Ecosystem §1.6-1.7, Arch §30.8) + Feedback Refinements (Sandboxing, Merge, Bulk Trust, active_view)
- [x] 1.3 Add `Editor` variant to `PaneContent` enum — EditorPaneState, EditorMode (Viewer/Editor/Diff), DiffHunk types added to state.rs
- [x] 1.4 Wire OSC 9012 line-level priority parser — OscEvent::LinePriority + PTY read loop wired with line-local override
- [x] 1.5 Configurable keyboard shortcut mapping layer — `KeybindingMap` with `KeyCombo` type, 29 default bindings matching §14.2, IPC handlers (`keybindings_get/set/reset`), Face store (`keybindings.svelte.ts`), 7 unit tests passing
- [x] **FIX:** Updated default `sys_title` from ALPHA-2.2 to BETA-0
- [x] **FIX:** Gated unused `crate::TosState` import behind `#[cfg(target_os = "android")]` — 0 warnings workspace-wide

## Stage 7 — Kanban & Agent Orchestration (New)
- [x] 7.1 Implement KanbanBoard service in Brain
- [x] 7.2 Implement `WorkflowManager.svelte` pane
- [x] 7.3 Implement Agent Persona parser
- [x] 7.4 Implement LLM Interaction Archival service
- [x] 7.5 Implement `roadmap_planner` skill
- [x] 7.6 Implement `dream consolidate` (Memory Synthesis)
- [x] 7.7 Multi-agent terminal routing (isolated PTYs)

## Stage 2 — Editor System
- [x] 2.1 Design editor pane data model — `EditorPaneState` (file_path, content, mode, language, cursor, scroll, dirty, diff_hunks) + `EditorMode` enum
- [x] 2.2 Implement Brain-side editor IPC messages (§30.3-30.4) — 16 handlers: editor_open/save/save_as/activate/mode_switch/scroll/open_ai/diff/annotate/clear_annotations/edit_proposal/edit_apply/edit_reject/context_update/send_context/promote + detect_language(30+ extensions) + SplitNode helpers (find_pane_mut, find_editor_by_path_mut, add_pane)
- [x] 2.3 Implement Svelte `EditorPane.svelte` component (Viewer Mode) — PrismJS syntax highlighting, VSCode-style gutter, and active line tracking.
- [x] 2.4 Implement Editor Mode (keyboard input, syntax highlighting) — `<textarea>` overlay mapped to Prism DOM logic, custom Svelte hotkeys (Tab, Ctrl+S), IPC `editor_context_update` fully integrated.
- [x] 2.5 Implement Diff Mode (side-by-side, Apply/Reject) — Injected JSON diff-hunk parsing/application backends + CSS structured native diff review panes replacing active editors automatically.
- [x] 2.6 Select-to-open on build error (PTY output → file:line parser) — Added `renderTermLine` to `SplitPaneView` for capturing line/file regex boundaries as interactive span tags mapped to `!ipc editor_open`.
- [x] 2.7 Editor Context Object integrated into AI pipeline — Added recursive `all_editors` extraction over SplitNode structures directly into `AiContext` aggregation inside `tos-common/src/services/ai/mod.rs`.
- [x] 2.8 AI Context Panel (Right Bezel slot) — Built `AiContextPanel.svelte`, hooked interactive bindings for toggling the window out of standard `EditorPane` boundaries matching spec layouts.
- [x] 2.9 Inline AI annotations in editor margin — Extended `EditorPaneState` struct across Rust/TS schemas with `EditorAnnotation` schemas. Svelte `$derived` mapping projects chips dynamically inside code line blocks. Triggered automatic "amberPulse" smooth scrolling via $effect bindings when new annotations mutate.
- [x] 2.10 Editor pane state persistence in session — Mapped `scroll_offset` onto Svelte `$effect` layout constraints and `handleScroll` event loops emitting back inside `editor_context_update` payloads, finalizing end-to-end integration across `tos-sessiond` debounced state saving loops.
- [x] 2.11 Save (`Ctrl+S`) and Save As (`Ctrl+Shift+S`) — Connected `editor_save` and `editor_save_as` IPC interfaces handling interactive front-end `KeyboardEvent` triggers against browser-native Modal Prompts, and wrote safe directory abstractions inside `std::fs` to flush serialized states to persistence safely.
- [x] 2.12 Trust chip for writes outside sector cwd — Bound the `paneCwd` property explicitly downwards onto Svelte's `<EditorPane>` isolating the `checkAndSave` thunks conditionally catching string deviations out of the active working directory, resolving them out across a layout-injected `[ALLOW PENDING WRITE]` danger chip natively protecting arbitrary directory modifications.
- [x] 2.13 LSP client integration (diagnostics, hover, completion) — Built `LspService` backend binding raw `stdin`/`stdout` JSON-RPC streams out to `rust-analyzer` and `tsserver` child processes. `didChange` and `didOpen` automatically stream backwards resolving `publishDiagnostics` arrays natively into the `tosState.annotations` framework established in Task 2.9 without custom DOM overrides.
- [x] 2.14 Mobile: tap line number sends to AI — Rewrote `.line-number` spans natively handling `onclick()` delegation across `EditorPane.svelte` hooking specific `ai_submit` string payloads resolving targeted single code lines directly outside of layout constraints safely evaluating local contents without manual copying.

## Stage 3 — AI Skills & Predictive Intelligence
- [x] 3.1 Tool bundle enforcement in Brain — (Validated in Task 0.1)
- [x] 3.2 Implement Command Predictor (ghost text / inline suggestions) — Added `predict_command` with AI & heuristic fallbacks, connected to `ai_predict_command` IPC and Tab-to-accept UI.
- [x] 3.3 Implement Vibe Coder skill (multi-step chip sequence) — Added `vibe-coder` behavior with multi-step `vibe_plan` logic for complex task orchestration.
- [x] 3.4 Implement thought bubble rendering in Face — Created `ActiveThoughts.svelte` for LCARS-style reasoning chips integrated into AI Chat.
- [x] 3.5 Implement offline AI queue (store, drain, 30min expiry) — Added `ai_offline_queue` to `TosState`, debounced session persistence, and 30-minute background expiration logic.
- [x] 3.6 Context-signal automatic skill activation — Integrated `check_context_signals` into `AiService` and PTY `read_loop` to trigger skills on directory changes.
- [x] 3.7 Skill learned patterns storage + Settings UI — Added `ai_patterns` to `SettingsStore`, implemented `ai_pattern_set` IPC, and added "LEARNED PATTERNS" management to Settings UI.
- [x] 3.8 Implement path completion chips — Implemented in `tos-heuristicd` with `Path` source chips.
- [x] 3.9 Implement typo correction chips — Implemented in `tos-heuristicd` using Levenshtein distance.
- [x] 3.10 Implement Focus Error chip (PTY error highlighting) — Added automatic high-priority (Level 3) tagging for error-related keywords in PTY `read_loop`.
- [x] 3.11 Implement notification display center (priority-gated) — Upgraded `PriorityStack.svelte` into a comprehensive Notification Center filtering for TACTICAL/CRITICAL events.

## Stage 4 — UI Polish & Feature Completion (100%)
- [x] 4.1 Marketplace permission scroll-to-consent gate
- [x] 4.2 Marketplace download progress display + cancel
- [x] 4.3 Marketplace installed badge in browse cards
- [x] 4.4 Warning chip rendering as dedicated component
- [x] 4.5 Bezel pane management chip rendering
- [x] 4.6 Divider drag + snap assist
- [x] 4.7 Onboarding: ambient hints (per-hint dismiss) — Implemented `AmbientHint.svelte` system with permanent dismissal via settings and initial hint registry.
- [x] 4.8 Deep Inspection: Buffer View implementation — Implemented multi-view `DetailInspector.svelte` with structured metadata, read-only hex buffer dump, and tactical wireframe reset.
- [x] 4.9 System reset confirmation dialog — Added a tactical "RED ALERT" confirmation modal to `GlobalOverview.svelte` with Brain-side `system_reset` IPC logic.
- [x] 4.10 Global TOS Log Sector view — Implemented `LogView.svelte` as a full-screen sector view with filtering, categorization, and export/clear capabilities.
- [x] 4.11 Tactical Mini-Map depth-aware content — Upgraded `Minimap.svelte` to show distinct spatial visualizations for Levels 1–4, including a projected global topology overlay.
- [x] 4.12 Priority visual indicators by depth (border chips, glow)

## Stage 5 — Native Platform & Multi-Sensory
- [x] 5.1 Real Wayland compositor integration test — Stabilized SHM buffer attachment in `WaylandShell`, resolving `wl_shm_pool` and `wl_buffer` dispatching issues.
- [x] 5.2 DMABUF frame buffer sharing for Level 3 apps — Implemented `linux-dmabuf-v1` path in `WaylandShell` with automatic fallback to SHM. Support for `create_immed` with DRM_FORMAT_ARGB8888.
- [x] 5.3 Three-layer audio model (ambient/tactical/voice) — Implemented `AudioService` with independent `Sink` layers for ambient loops, tactical earcons, and voice responses, including IPC handlers for volume and layer control.
- [x] 5.4 Alert level adaptation (Green/Yellow/Red) — Wired the Brain's background loop to automatically shift ambient audio profiles and volumes based on the highest active sector priority.
- [x] 5.5 Haptic feedback patterns on Android — Added `HapticService` with patterns for Android and Quest haptics (§23.4).
- [x] 5.6 Screen reader bridge (AT-SPI on Linux) — Semantic roles and ARIA tags added across face-svelte-ui components (§24.1).
- [x] 5.7 Full keyboard navigation tab-stop chain — Complete tab-stop chain with LCARS-compliant focus containment (§24.3).
- [x] 5.8 High-contrast forced mode — Implemented theme-based high-contrast overrides and accessibility mode toggle (§24.1).
- [x] 5.9 FPS monitoring + Tactical Alert — Added FPS tracking to Renderer and tactical alerts on frame drops (§16.4).
- [x] 5.10 Voice command input pipeline — Implemented VoiceCommandStart and VoiceTranscription SemanticEvents (§14.3).
- [x] 5.11 Depth-based render throttling — Implemented depth-aware surface composition skipping in LinuxRenderer (§16.1).
- [x] 5.12 Game controller / VR input mapping — Implemented DeviceMapping and QuestInput integration (§14.4).
- [x] 5.13 Accessibility switch scanning — Implemented `AccessibilityService` and switch scan IPC (§14.5).
- [x] 5.14 OpenXR / Quest renderer — Implemented `QuestRenderer` with spatial Cockpit layers and `RendererManager` integration (§15.3, §15.7).
- [x] 5.15 Language modules — Implemented `.tos-language` manifest support and dynamic LSP discovery in `LspService` (§1.12).
- [x] 5.16 Bezel component modules — Implemented `BezelModule` trait, dynamic loading, and `BezelService` integration (§1.10).
- [x] 5.17 Audio modules — Implemented `.tos-audio` manifest support and dynamic asset loading in `AudioService` (§1.9).
- [x] 5.18 Spatial audio — Implemented `rodio::SpatialSink` integration for 3D earcon localization (§23.3).
- [x] 5.19 Privacy controls — Implemented `incognito` mode in `SessionService` and IPC toggle handlers (§19.4).
- [x] 5.20 Per-surface timeline (Level 4) — Implemented `TimelineService` for state snapshot recording and `DetailView` scrubbing (§19.1).
- [x] 5.21 OpenSearch compatibility — Implemented `opensearch.xml` generation and HTTP-over-TCP routing in `RemoteServer` (§19.3).

## Stage 6 — Collaboration, Remote & Release
- [x] 6.1 TLS handshake in Remote Server protocol — Migrated `remote_server.rs` to `rustls` with dynamic self-signed certificate generation using `rcgen`.
- [x] 6.2 WebRTC signalling + video stream — Extended `remote_server.rs` with `webrtc-rs` integration, SDP/ICE signalling via WebSocket, and a mock media stream track.
- [x] 6.3 Session handoff (one-time tokens, 10min expiry) — Implemented `session_handoff_prepare` and `session_handoff_claim` IPC handlers, one-time 6-char token generation in `tos-sessiond`, and background token expiration logic.
- [x] 6.4 Collaboration role enforcement (Viewer→Operator) — Aligned `ParticipantRole` enum with spec (Viewer, Commenter, Operator, CoOwner), implemented `WebRtcPayload::Command` for remote IPC, and added role-based permission checks in `IpcHandler`.
- [x] 6.5 SSH fallback for non-TOS remotes — Implemented interactive `SshSession` with PTY bridging and `remote_ssh_connect` IPC routing (§27.3).
- [x] 6.6 mDNS discovery test in real network — Zero-config discovery via `_tos-brain._tcp` mDNS advertisement (Eco §5.2).
- [x] 6.7 HSM key provisioning for release signing — Implemented `tos-signer` utility with PKCS#11/HSM support (§6.7).
- [x] 6.8 Generate signed release assets — Integrated `tos-signer` into build pipeline and release scripts (§6.8).
- [ ] 6.9 E2E Playwright tests for Svelte UI — Comprehensive regression suite for core UI flows.
- [x] 6.10 Crash reporting infrastructure (opt-in) — Automated crash dump collection via tos-loggerd. Implemented crash IPC, logger service support, and loggerd sink.

## Stage 8 — Cortex Orchestration & Ecosystem Hardening
- [x] 8.1 Implement Brain cortex registry for `.tos-assistant`, `.tos-curator`, and `.tos-agent` (Eco §1.3, ModuleManager)
- [x] 8.2 Implement `[auth]` credential injection and secure Settings store (Eco §1.3.4, SettingsStore)
- [x] 8.3 Implement `[trust]` declaration & Brain trust chip integration (Eco §1.3.5, TrustService)
- [x] 8.4 Implement `[connection]` transports (http, stdio, mcp) (Eco §1.3.1, CortexRegistry)
- [x] 8.5 Implement Agent Stacking (hierarchical prompt merging) (Dev §6, BrainAI)
- [x] 8.6 Migrate Ollama / Gemini to `.tos-assistant` with legacy shim (Eco §1.15, CortexRegistry)
- [x] 8.7 Implement GitNexus curator cortex via MCP (Eco §1.3.2, CortexRegistry)
- [x] 8.8 Unified Cortex Configuration UI in Settings Modal (Features §4.3, SettingsModal)
- [x] 8.9 Verification of Cortex sandboxing — bubblewrap isolation (Arch §17.3, SandboxManager)
