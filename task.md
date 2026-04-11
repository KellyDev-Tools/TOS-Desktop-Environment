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
- [ ] 7.1 Implement KanbanBoard service in Brain
- [ ] 7.2 Implement `WorkflowManager.svelte` pane
- [ ] 7.3 Implement Agent Persona parser
- [ ] 7.4 Implement LLM Interaction Archival service
- [ ] 7.5 Implement `roadmap_planner` skill
- [ ] 7.6 Implement `dream consolidate` (Memory Synthesis)
- [ ] 7.7 Multi-agent terminal routing (isolated PTYs)

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

## Stage 3–6 — Deferred to subsequent sessions
