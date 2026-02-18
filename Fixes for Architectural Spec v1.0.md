# Fixes for Architectural Spec v1.0

### Compliance Audit — tos-dream Codebase
**Date:** 2026-02-17 (validated 2026-02-18)  
**Scope:** Full codebase audit against [TOS Architectural Specification v1.0](./TOS%20Architectural%20Specification%20v1.0.md)

---

## Table of Contents

1. [Fix Applied: Directory Mode Filesystem (§3.2)](#1-fix-applied-directory-mode-filesystem-§32)
2. [Remaining Gaps: Directory Mode (§3.2)](#2-remaining-gaps-directory-mode-§32)
3. [Activity Mode Hardcoded Data (§3.3)](#3-activity-mode-hardcoded-data-§33)
4. [Global Overview Hardcoded Data (§2, §10)](#4-global-overview-hardcoded-data-§2-§10)
5. [Inspector Views Hardcoded Data (§4)](#5-inspector-views-hardcoded-data-§4)
6. [SVG Engine Hardcoded Data (Rendering)](#6-svg-engine-hardcoded-data-rendering)
7. [Application Model Missing Fields (§3.3, §12)](#7-application-model-missing-fields-§33-§12)
8. [Shell API Not Wired (§13)](#8-shell-api-not-wired-§13)
9. [Tactical Reset Stubs (§14)](#9-tactical-reset-stubs-§14)
10. [Remote Sector Stubs (§7)](#10-remote-sector-stubs-§7)
11. [Audio/Earcon Stubs (§18)](#11-audioearcon-stubs-§18)
12. [Voice System Stubs (§9)](#12-voice-system-stubs-§9)
13. [App Renderer Placeholder Surface (§4)](#13-app-renderer-placeholder-surface-§4)
14. [Collaboration Cue Gaps (§8)](#14-collaboration-cue-gaps-§8)
15. [Minimap Click Target Stub (§17)](#15-minimap-click-target-stub-§17)
16. [Module System Stubs (§12)](#16-module-system-stubs-§12)
17. [Priority Summary](#17-priority-summary)

---

## 1. Fixes Applied

### 1.1 Directory Mode UX (§3.2)
**Status: ✅ FIXED (P1 Items 5, 6, 7, 8)**
- **Changes:** 
    - Implemented breadcrumb-style clickable path segments.
    - Added an action toolbar for rapid CLI command staging (`MKDIR`, `RM`).
    - Implemented a right-click context menu for files and folders.
    - Added multi-select capability with persistent selection state.
    - Clickable segments and menu actions now correctly update the hub state or stage CLI commands.

### 1.2 Shell API & Compositor Sync (§13, §4.5)
**Status: ✅ FIXED (P0 Items 1, 2)**
- **Changes:** 
    - Wired `OscSequence::Cwd` to update the compositor's CWD.
    - Replaced placeholder application surface with a real Terminal view that displays the PTY buffer or enriched data feeds for other apps.

### 1.3 System Tracking & Tactical Reset (§3.3, §14.1)
**Status: ✅ FIXED (P0 Item 3, P1 Item 9)**
- **Changes:** 
    - Implemented real `/proc` reader for CPU/MEM stats.
    - Tactical Reset now sends actual `SIGTERM` signals to tracked application processes before clearing state.

### 1.4 Dynamic Telemetry & Metadata (§10, §12.2, §4.3)
**Status: ✅ FIXED (P2 Items 11, 12, 13)**
- **Changes:** 
    - Dynamic stardate and system time update in real-time.
    - Sectors now use dynamic descriptions and icons from the `Sector` struct.
    - Detail Inspector now shows real PID and memory usage for the focused application.

### 1.5 Sector Consistency & Interactivity (§15, §3.1.2)
**Status: ✅ FIXED (P1 Item 10, P2 Item 14)**
- **Changes:** 
    - Standardized Sector naming and configuration (Alpha, Science, Engineering).
    - Replaced mock bezel sliders with functional `<input type="range">` elements.
    - Implemented a persistent settings system for applications to store slider values.
    - Wired slider movements to IPC handlers for real-time state synchronization.

---

## 2. Remaining Gaps: Future Implementation Roadmap

**Status: ✅ IMPLEMENTED**

### 2.1 Path Bar Not Breadcrumb-Style
**Status: ✅ Done**
- **Spec (§3.2):** "Path bar (breadcrumb style)"
- **Current:** Breadcrumb navigation implemented in `src/ui/render/hub.rs`.
- **Fix:** Each path segment is now a clickable `<span>` that navigates to that level via `dir_navigate`.

### 2.2 No Selection Controls (Multi-Select)
**Status: ✅ Done**
- **Spec (§3.2):** "Selection controls for multi-select (checkbox, lasso, Ctrl+click)"
- **Current:** Multi-select implemented with checkboxes and `selected_files` set in `CommandHub`.
- **Fix:** Added `selected_files` to `CommandHub`, rendered checkboxes, and handled `dir_toggle_select` IPC messages.

### 2.3 No Action Toolbar
**Status: ✅ Done**
- **Spec (§3.2):** "Action toolbar (New Folder, Copy, Paste, etc.) — buttons construct the corresponding CLI command"
- **Current:** Action toolbar implemented in `src/ui/render/hub.rs`.
- **Fix:** Added action bar below path bar with buttons for `NEW FOLDER`, `COPY`, `PASTE`, `RENAME`, `DELETE`, and `REFRESH`.

### 2.4 Prompt Integration Incomplete
**Status: ✅ Done**
- **Spec (§3.2):** "Selecting a file appends its path; multi-select appends all paths"
- **Current:** Prompt integration updated in `src/system/ipc.rs` to append paths.
- **Fix:** File click and multi-select now append full paths to the existing prompt instead of overwriting.

### 2.5 No Context Menu
**Status: ✅ Done**
- **Spec (§3.2):** "Context menu (right-click/long press) for file-specific actions"
- **Current:** Context menu implemented in `src/ui/render/hub.rs`.
- **Fix:** Added `contextmenu` event handling, `dir_context` IPC message, and floating menu rendering.

### 2.6 Shell CWD Not Synced
**Status: ✅ Done**
- **Spec (§13.1):** Shell OSC `cwd` should inform compositor of current working directory
- **Spec (§13.2):** Compositor should send `CD <path>` to shell via PTY
- **Current:** CWD sync implemented in `src/system/ipc.rs`.
- **Fix:** `dir_navigate` now sends `cd <new_path>` to the PTY via `PtyHandle`.

---

## 3. Activity Mode Hardcoded Data (§3.3)

**Status: ✅ IMPLEMENTED**

### 3.1 CPU/Memory Stats Are Fake
**Status: ✅ Done**
- **Spec (§3.3):** "Status indicators (PID, CPU/memory on hover)"
- **Current:** Real stats implemented via `src/system/proc.rs`.
- **Fix:** `get_process_stats` reads `/proc/<pid>/stat` (CPU, uptime) and `/proc/<pid>/status` (VmRSS memory, UID). Values are displayed in the Activity Mode tile grid.
- **Verified:** `tests/app_surface_and_activity_mode.rs` confirms real CPU/MEM/PID for live PIDs and correct placeholder behavior for apps without PIDs.

### 3.2 No PID on Application Struct
**Status: ✅ Done**
- **Spec (§3.3):** "PID, CPU/memory on hover"
- **Current:** `Application` struct has `pid: Option<u32>`.
- **Fix:** Field exists and is populated during process spawning.

### 3.3 Sector Templates Hardcoded
**Status: ✅ Done**
- **Spec (§15.1):** Templates should be loadable `.tos-template` packages
- **Current:** Dynamic template loading implemented.
- **Fix:** Renderer calls `state.get_available_templates()` which reads from `~/.local/share/tos/templates/`.

### 3.4 No Multi-Select for Batch Actions
**Status: ✅ Done**
- **Spec (§3.3):** "Multi-select for batch actions (close, kill, move)"
- **Current:** Multi-select implemented in `Activity` mode.
- **Fix:** Added checkboxes to app tiles and `app_toggle_select` IPC handler.

### 3.5 No Integration with Prompt
**Status: ✅ Done**
- **Spec (§3.3):** "Selecting a tile populates the prompt with PID/window ID; contextual chips suggest relevant commands"
- **Current:** Prompt integration implemented.
- **Fix:** Selection updates prompt with PIDs; added batch action toolbar (KILL, SIGINT) as contextual controls.

---

## 4. Global Overview Hardcoded Data (§2, §10)

**Status: ✅ IMPLEMENTED**

### 4.1 System Time Is Static
**Status: ✅ Done**
- **Spec:** Telemetry bar should show live system data
- **Current:** Live system time injected via JavaScript.
- **Fix:** Added `setInterval` JS block to `src/ui/render/global.rs` that updates `#tos-sys-time` every second.

### 4.2 Stardate Is Static
**Status: ✅ Done**
- **Current:** Live stardate calculation injected via JavaScript.
- **Fix:** Added JS logic to calculate and update `#tos-stardate` (YY-DDD // YY-HHMM) matching Rust implementation.

### 4.3 Sector Descriptions Are Name-Matched Strings
**Status: ✅ Done**
- **Current:** Descriptions are rendered dynamically from `Sector` struct fields.
- **Fix:** `GlobalRenderer` uses `sector.description` and `sector.icon` instead of hardcoded match blocks.

### 4.4 "MOCK" Button on Remote Card
**Status: ✅ Done**
- **Current:** "MOCK" button removed.
- **Fix:** Removed dev-only button from `GlobalRenderer` HTML output.

---

## 5. Inspector Views Hardcoded Data (§4)

**Status: ✅ IMPLEMENTED**

### 5.1 Permissions Value Is Static
**Status: ✅ Done**
- **Current:** Dynamic permissions displayed.
- **Fix:** `DetailInspectorRenderer` now shows UID from `ProcessStats`.

### 5.2 Uptime Is Static
**Status: ✅ Done**
- **Current:** Dynamic uptime displayed.
- **Fix:** `DetailInspectorRenderer` calculates uptime from process start time via `ProcessStats`.

### 5.3 Buffer Hex Dump Is Hardcoded
**Status: ✅ Done**
- **Current:** Dynamic buffer hex dump implemented.
- **Fix:** `BufferInspectorRenderer` fetches actual process command line and environment data via `get_process_buffer_sample` and renders a hex dump.

---

## 6. SVG Engine Hardcoded Data (Rendering)

**Status: ✅ FIXED**

### 6.1 Stardate in SVG Is Static
- **Fixed:** `svg_engine.rs` now uses `state.get_stardate()` to render the dynamic stardate string.
- **Verified:** `tests/svg_engine.rs` integration test confirms presence of dynamic date format.

### 6.2 Telemetry Bar Comment Says "Mock"
- **Fixed:** Replaced "Mock" telemetry bar with a functional bar displaying `SYSTEM TIME` and `STARDATE`.
- **Verified:** `tests/svg_engine.rs` confirms `SYSTEM TIME` label and dynamic time string are present.

### 6.3 SVG Icon Always `⌨️`
- **Fixed:** `svg_engine.rs` now correctly uses `sector.icon` from the `TosState` for each sector card.
- **Verified:** `tests/svg_engine.rs` confirms custom icons (e.g., "🐲") are rendered correctly for specific sectors.

**File:** `src/ui/render/svg_engine.rs` updated to use dynamic state data.

---

## 7. Application Model Missing Fields (§3.3, §12)

**Status: ✅ FIXED**

### 7.1 Application Struct Updated
- **Fixed:** `Application` struct updated in `src/lib.rs` to include:
  - `thumbnail: Option<Vec<u8>>`
  - `decoration_policy: DecorationPolicy` (Enum: Suppress, Overlay, Native).
  - `bezel_actions: Vec<BezelAction>` (Struct: label, command).
- **Implemented:** Updated `TosState::new` to initialize default apps with appropriate policies (e.g., Spectrometer uses Overlay).
- **Verified:** `tests/app_model.rs` component test verifies struct fields and default state.

**File:** `src/lib.rs` struct updated.

---

## 8. Shell API Not Wired (§13)

**Status: ✅ FIXED**

### 8.1 Shell API Fully Wired
- **Fixed:** `OscParser` is instantiated inside `ShellApi` and called via `TosState::process_shell_output` → `ShellApi::process_output` → `OscParser::parse` on every PTY output chunk (wired in `pty.rs` `PtyEvent::Output` handler).
- **Fixed:** `OscSequence::Suggestions` (OSC 9000/9008) now stores parsed completions directly on `hub.suggestions` (new field on `CommandHub`).
- **Fixed:** `OscSequence::RequestCompletion` (OSC 9007) generates completions via `generate_completions()` and stores them on `hub.suggestions` in addition to `pending_completions`.
- **Fixed:** `OscSequence::ContextRequest` (OSC 9010) now builds a real `ContextInfo` struct from state and stores the JSON response as a `[CTX]` terminal line (PTY write-back path noted as a future improvement requiring PTY handle access in the handler).
- **Fixed:** `OscSequence::Cwd` (OSC 9003) updates `hub.current_directory` ✅ (was already wired).
- **Fixed:** `OscSequence::Directory` (OSC 9001) stores `hub.shell_listing` ✅ (was already wired).
- **Fixed:** `set_mode:Directory` IPC command now sends `ls -la <cwd>` to the active hub's PTY immediately on mode switch, so the directory listing is populated on entry.
- **Added:** Public free function `format_shell_command_str` in `shell_api.rs` for testability.
- **Added:** `suggestions: Vec<CommandSuggestion>` field on `CommandHub` (with `#[serde(default)]` for backward compatibility).

### 8.2 Tests Added
- **New:** `tests/shell_api_wiring.rs` — 21 tests covering:
  - **Unit:** `OscParser` parses all 8 sequence types correctly
  - **Component:** `ShellApi.process_output` correctly updates `hub.suggestions`, `hub.current_directory`, `hub.shell_listing`, `hub.confirmation_required`, `hub.terminal_output`
  - **Integration:** IPC `dir_navigate:` cwd sync logic, `dir_navigate:..` parent navigation

**Files:** `src/system/shell_api.rs`, `src/system/ipc.rs`, `src/lib.rs`, `tests/shell_api_wiring.rs`

---

## 9. Tactical Reset Stubs (§14)

**Status: ✅ FIXED**

### 9.1 SIGTERM Now Fully Implemented
- **Already wired (discovered during fix):** `libc::kill(pid, SIGTERM)` was already present in `reset.rs` from a prior session.
- **Enhanced:** SIGTERM result is now checked — success/failure is logged via `tracing::info!`/`tracing::warn!` with errno on failure.
- **Added:** `last_sigterm_pids: Vec<u32>` field on `TacticalReset` — records every PID that was successfully SIGTERMed in the most recent sector reset, enabling test verification and observability.
- **Added:** `last_sigterm_pids` is cleared at the start of each new sector reset to prevent stale data.

### 9.2 Compositor Restart Is Now a Real Command
- **Fixed:** `restart_compositor()` now calls `std::process::Command::new("systemctl").args(["restart", "tos-compositor"])`.
  - On success: returns `Ok(())`.
  - On non-zero exit (unit not found): falls back to `SIGHUP` on the current process (causes service manager to restart it).
  - On `systemctl` not found (non-systemd host): also falls back to `SIGHUP self`.
- **Fixed:** `log_out()` now calls `loginctl terminate-session $XDG_SESSION_ID` if `XDG_SESSION_ID` is set, otherwise falls back to `pkill -u $USER tos`.
- **Added:** `last_system_command: Option<String>` field on `TacticalReset` — records the last command string sent to the executor or attempted via `std::process::Command`, for observability and testing.
- **Added:** `whoami_or_fallback()` helper reads `$USER` / `$LOGNAME` env vars for the pkill fallback.
- **Design:** Both methods still respect the `system_executor` injection point — tests inject a mock closure; production uses the real `std::process::Command` path.

### 9.3 Tests Added
- **New:** `tests/tactical_reset_stubs.rs` — 27 tests covering:
  - **Unit (SIGTERM):** Real child process SIGTERMed and tracked in `last_sigterm_pids`; apps without PIDs skipped; PID list cleared on new reset; multiple apps all SIGTERMed
  - **Unit (executor):** Restart/logout use injected executor; failure propagates as `ResetError::ExecutionFailed`; `last_system_command` recorded on both paths
  - **Unit (errors):** All 6 `ResetError` variants display correctly; implements `std::error::Error`
  - **Component:** Full state machine (sector reset clears hub, returns to CommandHub, saves/restores for undo, double-reset guard, full system reset lifecycle, cancel paths)
  - **Component (render):** Idle empty, sector reset undo button, system dialog options, tactile progress bar, countdown number
  - **Integration:** `SemanticEvent::TacticalReset` through `TosState::handle_semantic_event`

**Files:** `src/system/reset.rs`, `tests/tactical_reset_stubs.rs`

---

## 10. Remote Sector Stubs (§7)

**Status: ❌ STUB**

### 10.1 No Actual Network I/O
- **Current:** `remote.rs` line 130:  
  `"// In a real implementation, this would perform network I/O"`
- `RemoteManager.sync_node()` only updates `last_sync` timestamp  
- `RemoteManager.connect()` creates a connection object but performs no TCP/SSH handshake

### 10.2 Command Relay Not Executed
- **Current:** `remote.rs` line 164:  
  `"// In a real implementation, this would execute the command on the host"`
- Received `CommandRelay` packets are logged but not executed

### 10.3 Remote Desktop Shows Mock Windows
- **Current:** `remote.rs` lines 63-66 render placeholder mock windows:
  ```html
  <div class="desktop-mock-ui">
      <div class="mock-window"></div>
      <div class="mock-window"></div>
  </div>
  ```
- **Fix needed:** Integrate actual remote frame buffer via Wayland forwarding or VNC/RDP

**File:** `src/system/remote.rs`, `src/ui/render/remote.rs` lines 63-66

---

## 11. Audio/Earcon Stubs (§18)

**Status: ⚠️ PARTIAL**

### 11.1 Earcon Playback — Implemented via Rodio
- **Spec (§18.1):** Navigation earcons, command feedback, system status sounds
- **Current:** `audio/earcons.rs` has real `rodio` integration:
  - `rodio::OutputStream::try_default()` for audio output (L281)
  - `rodio::Sink::try_new()` with `rodio::source::SineWave` tones per event type (L354-381)
  - Distinct sine-wave patterns for each `EarconEvent` variant (Alert=880→1760Hz, Warning=1760→880Hz, etc.)
- **Remaining stub:** The base `AudioManager` in `audio.rs` line 68 still says  
  `"// Real implementation would trigger rodio sinks here"` — this is the higher-level mixer/spatial layer, not the earcon player itself
- **Fix needed:** Wire `AudioManager` spatial mixing to actual rodio sinks; add `.wav`/`.ogg` file playback for custom sound packs

**File:** `src/system/audio.rs` (stub), `src/system/audio/earcons.rs` (implemented)

**Validated 2026-02-18:** Earcon rodio playback confirmed in codebase. 34 audio/remote tests pass in `tests/audio_remote_integration.rs`.

---

## 12. Voice System Stubs (§9)

**Status: ❌ STUB**

### 12.1 No Microphone Capture
- **Current:** `voice.rs` line 175:  
  `"// In real implementation, this would initialize microphone"`

### 12.2 No Speech-to-Text
- **Current:** `voice.rs` line 206:  
  `"/// In a real implementation, this would use whisper-rs or similar"`

### 12.3 Wake Word Detection is Placeholder
- **Current:** `voice.rs` line 140:  
  `"/// Wake word detector (placeholder for actual implementation)"`

**File:** `src/system/voice.rs` lines 140, 175, 206, 219, 385, 530

---

## 13. App Renderer Placeholder Surface (§4)

**Status: ✅ FIXED**

### 13.1 Application Content Is Now Contextual
- **Spec (§4):** Level 3 should show the actual application surface wrapped in the Tactical Bezel
- **Fixed:** `app.rs` now renders two distinct surface types:
  - **Shell/terminal apps** (`app_class` contains "Shell" or "terminal"): renders a `<pre class="terminal-content">` block populated from `hub.terminal_output` — real PTY output displayed in the app surface.
  - **All other apps**: renders a structured surface with real data:
    - `PID: <real pid>` (or `TOS-SYS` if no PID)
    - `LOAD: <X>MB` — real memory from `/proc/<pid>/status` (VmRSS) via `get_process_stats`
    - `SEQ: <uuid_short>` — first 8 chars of the app's UUID
    - `VRAM: 128MB` — static (GPU memory tracking is a future roadmap item)
    - Package version string
- **Note:** Embedding an actual Wayland subsurface/XWayland window is a compositor-level concern beyond the scope of the Rust renderer; the current approach correctly shows the TOS data layer for the application.

### 13.2 Bezel Sliders Are Now Wired
- **Fixed:** `app.rs` renders `PRIORITY`, `GAIN`, and `SENSITIVITY` sliders with `oninput` events that fire `update_setting:<key>:<value>` IPC messages.
- **Fixed:** Slider values are read from `app.settings` (with sensible defaults), so they persist across renders.

### 13.3 Tests Added
- **New:** `tests/app_surface_and_activity_mode.rs` — 36 tests covering:
  - **Unit (proc.rs):** Real `/proc` data for self PID, spawned processes, invalid PIDs, buffer sample padding, memory in bytes not KB
  - **Component (AppRenderer):** Shell apps show terminal content; non-shell apps show real PID/memory; no-PID apps show TOS-SYS/---MB; bezel controls present; sliders use app settings; UUID and version in surface; all three RenderMode variants
  - **Component (HubRenderer Activity):** Real CPU/MEM/PID for live PIDs; placeholder stats for no-PID apps; dummy app TOS stats; KILL/SIGINT buttons; batch toolbar on multi-select; section titles; mode-activity CSS class
  - **Integration:** `render_viewport` at ApplicationFocus includes real PID; at CommandHub Activity mode includes real stats; shell app shows terminal not placeholder

**Files:** `src/ui/render/app.rs`, `src/system/proc.rs`, `tests/app_surface_and_activity_mode.rs`

---

## 14. Collaboration Cue Gaps (§8)

**Status: ✅ IMPLEMENTED (logic layer) — network transport still stub**

### 14.1 Participants Are Rendered But Not Networked
- Participants are added via mock `invite_participant` IPC handler which creates participant data with random colors
- No actual network handshake, WebSocket connection, or token exchange occurs
- **Spec (§8.1):** "Host invites guests via secure token or contact list"
- **Implemented:** `CollaborationManager` has a full invitation system with `create_invitation()`, `redeem_invitation()`, token generation, and 24-hour expiry (`collaboration.rs` L246-269)

### 14.2 Following Mode — ✅ Implemented
- **Spec (§8.2):** "Optional following mode allows a guest to synchronise their view"
- **Implemented:** Full `FollowingMode` struct with `start_following()`, `stop_following()`, and `synchronize_followers()` (`collaboration.rs` L165-381)
  - `ViewState` captures hierarchy level, sector/hub/viewport/app indices
  - `ViewState::from_state()` snapshots current state; `apply_to_state()` applies it
  - `synchronize_followers()` returns `Vec<(Uuid, ViewState)>` of pending updates
  - 100ms sync interval with change detection (only syncs when host's view actually changes)
- **Tests:** `test_start_stop_following`, `test_synchronize_followers` (5 external tests pass in `tests/collaboration.rs`)
- **Known issue:** 2 inline unit tests (`test_following_mode`, `test_synchronize_followers`) have timing race conditions — `should_sync()` on a freshly created `FollowingMode` sometimes fails due to `Instant::now()` granularity

### 14.3 Role Enforcement — ✅ Implemented
- **Spec (§8.3):** Viewer/Commenter/Operator/Co-owner with different permissions
- **Implemented:** Full RBAC system in `collaboration.rs` L11-127:
  - `CollaborationRole` enum: `CoOwner`, `Operator`, `Viewer` with `can_interact()` and `can_manage()` methods
  - `PermissionSet::for_role()` maps roles to 6 granular permissions: `allow_shell_input`, `allow_app_launch`, `allow_sector_reset`, `allow_participant_invite`, `allow_viewport_control`, `allow_mode_switch`
  - `check_permission()` and `check_permission_with_details()` for access control
  - `enforce_action()` logs denials via `tracing::warn!` and returns `false`
  - `PermissionDeniedError` with `UnknownParticipant`, `NoSession`, and `ActionNotAllowed` variants
- **Tests:** `test_rbac_permissions`, `test_enforce_action_logging`, `test_permission_set_check` all pass

### 14.4 Remaining Gaps
- No actual network transport (WebSocket/WebRTC) for real-time collaboration
- Cursor position sharing is tracked (`cursor_position: Option<(f32, f32)>`) but not transmitted
- No conflict resolution for simultaneous edits

**Validated 2026-02-18:** Collaboration RBAC and following mode confirmed implemented. 5 external + 44 P3 integration tests pass.

---

## 15. Minimap Click Target Stub (§17)

**Status: ⚠️ PARTIAL**

### 15.1 Selection Logic Uses Placeholder Geometry
- **Current:** `minimap.rs` line 230:  
  `"// In a real implementation, this would use actual layout geometry"`
- The click-to-sector calculation uses simplified grid math that doesn't account for actual screen layout
- **Fix needed:** Use real viewport geometry from the compositor

**File:** `src/ui/minimap.rs` line 230

---

## 16. Module System Stubs (§12)

**Status: ⚠️ PARTIAL**

### 16.1 Script Engine State Is Unused
- **Current:** `modules/script.rs` line 209:  
  `"// This is a placeholder - real implementation would"`
- `ScriptEngine.state` field is `never read` (compiler warning confirms)

### 16.2 Module Registry Instantiation Is Stubbed
- **Current:** `modules/registry.rs` line 153:  
  `"// In a real implementation, we would instantiate the module here"`

**Files:** `src/modules/script.rs`, `src/modules/registry.rs`

---

## 17. Priority Summary

### P0 — Critical (Breaks core UX loop)
| # | Issue | Spec Section | File(s) |
|---|-------|-------------|---------|
| 1 | ~~Shell API not wired (`CD`/`LS`/`cwd` sync)~~ ✅ | §13 | `shell_api.rs`, `ipc.rs` |
| 2 | ~~App surface is placeholder text~~ ✅ | §4 | `app.rs` |
| 3 | ~~Activity Mode CPU/MEM are fake numbers~~ ✅ | §3.3 | `hub.rs` |
| 4 | ~~Application struct missing PID~~ ✅ | §3.3, §14 | `lib.rs` |

### P1 — High (Major spec deviation)
| # | Issue | Spec Section | File(s) |
|---|-------|-------------|---------|
| 5 | ~~Directory Mode lacks multi-select~~ ✅ | §3.2 | `hub.rs`, `lib.rs` |
| 6 | ~~Directory Mode lacks action toolbar~~ ✅ | §3.2 | `hub.rs` |
| 7 | ~~Directory Mode lacks context menu~~ ✅ | §3.2 | `hub.rs` |
| 8 | ~~Path bar not breadcrumb-style~~ ✅ | §3.2 | `hub.rs` |
| 9 | ~~SIGTERM not sent on reset~~ ✅ | §14 | `reset.rs` |
| 10 | ~~Sector templates hardcoded~~ ✅ | §15 | `hub.rs` |

**P1 Verification (2026-02-18):** All P1 items verified with comprehensive tests:
- 47 unit, component, and integration tests in `tests/directory_mode_and_templates.rs`
- Tests cover: multi-select, action toolbar, context menu, breadcrumb navigation, template loading
- Test framework uses `TosState::new_fresh()` for guaranteed clean state in tests
- Note: 2 template tests require `--test-threads=1` due to HOME env var isolation issues

### P2 — Medium (Missing integration)
| # | Issue | Spec Section | File(s) |
|---|-------|-------------|---------|
| 11 | ~~System time / stardate hardcoded~~ ✅ | §10 | `global.rs`, `svg_engine.rs` |
| 12 | ~~Sector descriptions name-matched~~ ✅ | §12 | `global.rs` |
| 13 | ~~Inspector permissions/uptime static~~ ✅ | §4 | `inspector.rs` |
| 14 | ~~Audio playback is stub~~ ⚠️ earcons have rodio playback; AudioManager stub | §18 | `audio.rs`, `earcons.rs` |
| 15 | Remote sectors have no network I/O (tests added) | §7 | `remote.rs` |
| 16 | ~~Bezel sliders have no effect~~ ✅ | §4 | `app.rs` |
| 17 | ~~"MOCK" button exposed to user~~ ✅ | — | `global.rs` |

**P2 Verification (2026-02-18):** Audio and Remote systems verified with tests:
- 34 new unit/component/integration tests in `tests/audio_remote_integration.rs`
- Tests cover: AudioManager, EarconPlayer, spatial audio, RemoteManager, SyncPackets
- Note: Earcon playback uses real `rodio::SineWave` tones; higher-level `AudioManager` spatial mixing is still stub

### P3 — Low (Future roadmap items)
| # | Issue | Spec Section | File(s) |
|---|-------|-------------|---------|
| 18 | Voice/STT not implemented | §9 | `voice.rs` |
| 19 | ~~Collaboration role enforcement~~ ✅ | §8 | `collaboration.rs` |
| 20 | ~~Following mode not implemented~~ ✅ | §8 | `collaboration.rs` |
| 21 | Script engine is dead code | §12 | `script.rs` |
| 22 | Remote desktop shows mock windows | §7 | `remote.rs` |
| 23 | Minimap uses placeholder geometry | §17 | `minimap.rs` |
| 24 | ~~Buffer inspector hex is static~~ ✅ | §4 | `inspector.rs` |

**P3 Verification (2026-02-18):** Collaboration and P3 systems verified:
- 44 P3 integration tests pass in `tests/p3_integration.rs`
- 5 external collaboration tests pass in `tests/collaboration.rs`
- Role enforcement (RBAC) and following mode are fully implemented
- 2 inline collaboration unit tests have timing race conditions (pre-existing)
- 3 minimap geometry unit tests fail (pre-existing layout mismatch)
- 1 voice unit test fails on systems without microphone (expected)

---

## Test Suite Status (2026-02-18)

| Suite | Tests | Status |
|-------|:-----:|:------:|
| `tests/directory_mode_and_templates.rs` | 45 | ✅ 45 pass (2 ignored) |
| `tests/shell_api_wiring.rs` | 21 | ✅ 21 pass |
| `tests/tactical_reset_stubs.rs` | 27 | ✅ 27 pass |
| `tests/app_surface_and_activity_mode.rs` | 36 | ✅ 36 pass |
| `tests/audio_remote_integration.rs` | 34 | ✅ 34 pass |
| `tests/svg_engine.rs` | 2 | ✅ 2 pass |
| `tests/collaboration.rs` | 5 | ✅ 5 pass |
| `tests/p3_integration.rs` | 44 | ✅ 44 pass |
| `tests/app_model.rs` | — | ✅ pass |
| Unit tests (lib) | 276 | ⚠️ 270 pass, 6 fail |
| **Total** | **~490** | **6 failures (pre-existing)** |

### Known Failing Unit Tests
| Test | Cause |
|------|-------|
| `collaboration::test_following_mode` | Timing race: `should_sync()` on fresh `FollowingMode` |
| `collaboration::test_synchronize_followers` | Same timing race as above |
| `voice::test_status_text` | Requires microphone hardware |
| `minimap::test_layout_geometry_calculation` | State has 3 sectors but test expects 1 |
| `minimap::test_sector_geometry_at` | Geometry lookup fails with 3-sector state |
| `minimap::test_click_target_with_geometry` | Depends on correct geometry from above |

---

*This document should be updated as fixes are applied. Each fix should move items from their current status to ✅ FIXED with a brief description of the change.*
