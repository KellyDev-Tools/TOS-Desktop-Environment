# Fixes for Architectural Spec v1.0

### Compliance Audit — tos-dream Codebase
**Date:** 2026-02-17  
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
- **Fix:** Confirmed `get_process_stats` uses `/proc` FS to query actual PID stats.

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

**Status: ❌ HARDCODED**

### 5.1 Permissions Value Is Static
- **Current:** `inspector.rs` line 19: `<span>0755</span>`
- **Fix needed:** Query actual file/process permissions

### 5.2 Uptime Is Static
- **Current:** `inspector.rs` line 20: `<span>00:14:32</span>`
- **Fix needed:** Calculate from process start time or application launch time

### 5.3 Buffer Hex Dump Is Hardcoded
- **Current:** `inspector.rs` lines 37-39 show a static hex dump of `LCARS DREAM COMPLETE VERSION 1.0`
- **Fix needed:** Should display actual buffer data from the inspected application or process memory

**File:** `src/ui/render/inspector.rs` lines 19-20, 37-39

---

## 6. SVG Engine Hardcoded Data (Rendering)

**Status: ❌ HARDCODED**

### 6.1 Stardate in SVG Is Static
- **Current:** `svg_engine.rs` line 56: `02-33 // 02-1478`
- Same issue as §4.2

### 6.2 Telemetry Bar Comment Says "Mock"
- **Current:** `svg_engine.rs` line 53: `// Telemetry Bar (Mock)`
- The SVG renderer has the same static data issues as the HTML renderer

### 6.3 SVG Icon Always `⌨️`
- **Current:** `svg_engine.rs` line 73 always renders the keyboard emoji regardless of sector type
- **Fix needed:** Use sector type to determine icon

**File:** `src/ui/render/svg_engine.rs` lines 53-56, 73

---

## 7. Application Model Missing Fields (§3.3, §12)

**Status: ❌ MISSING**

### 7.1 Application Struct Lacks Required Fields
- **Spec (§3.3):** Activity Mode shows "icon, title, optional live thumbnail, and status indicators (PID, CPU/memory)"
- **Spec (§12.1):** App Models provide "Custom bezel actions, zoom behavior, legacy decoration policy, thumbnail"
- **Current `Application` struct:**
  ```rust
  pub struct Application {
      pub id: uuid::Uuid,
      pub title: String,
      pub app_class: String,
      pub is_minimized: bool,
  }
  ```
- **Missing fields:**
  - `pid: Option<u32>` — process ID for kill/nice/strace
  - `icon: Option<String>` — icon path or emoji
  - `thumbnail: Option<Vec<u8>>` — live thumbnail data
  - `decoration_policy: DecorationPolicy` — Suppress/Overlay/Native (§4.3)
  - `bezel_actions: Vec<BezelAction>` — per-app bezel buttons (§4.2)

**File:** `src/lib.rs` lines 157-163

---

## 8. Shell API Not Wired (§13)

**Status: ❌ STUB**

### 8.1 Shell API Module Exists But Is Not Connected
- The `system/shell_api.rs` module (1054 lines) defines a comprehensive `OscParser`, `ShellCommand` enum, and integration script generators (Fish, Bash), but:
  - `OscParser` is never instantiated in the IPC handler
  - No PTY output is fed through the parser
  - `ShellCommand::Cd` is never sent when `dir_navigate` changes directory  
  - `ShellCommand::Ls` is never sent when entering Directory Mode
  - The `cwd` OSC sequence is parsed but never updates `hub.current_directory`
- **Spec (§13.2):** `CD`, `LS`, `COMPLETE`, `EXEC`, `SETENV` should all flow from compositor to shell
- **Fix needed:** Wire `OscParser` into the PTY read loop and IPC dispatcher

### 8.2 Comment Confirms Stub Status
- `shell_api.rs` line 723: `"// In real implementation, would update UI"`
- `shell_api.rs` line 789: `"// In a real implementation, we would send back a ContextInfo packet"`

**Files:** `src/system/shell_api.rs`, `src/system/ipc.rs`

---

## 9. Tactical Reset Stubs (§14)

**Status: ⚠️ PARTIAL**

### 9.1 SIGTERM Not Actually Sent
- **Spec (§14.1):** "Sends SIGTERM to all processes in the current sector"
- **Current:** `reset.rs` lines 201-205:
  ```rust
  // Send SIGTERM to all applications (in real implementation)
  // For now, just clear the applications
  hub.applications.clear();
  // In a real implementation, this would send SIGTERM to PIDs
  ```
- **Fix needed:** Use `nix::sys::signal::kill(Pid, Signal::SIGTERM)` on each app's PID. Requires PID tracking on `Application` struct (see §7.1).

### 9.2 Compositor Restart Is a Print Statement
- **Current:** `reset.rs` line 386:  
  ```rust
  println!("TACTICAL RESET: Restarting TOS compositor...");
  ```
- **Fix needed:** Execute actual `systemctl restart tos-compositor` or equivalent

**File:** `src/system/reset.rs` lines 201-205, 386

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

**Status: ❌ STUB**

### 11.1 No Actual Audio Playback
- **Spec (§18.1):** Navigation earcons, command feedback, system status sounds
- **Current:** `audio/earcons.rs` line 337:  
  `"// In a real implementation, this would trigger actual audio playback"`
- `audio.rs` line 68:  
  `"// Real implementation would trigger rodio sinks here"`
- **Fix needed:** Integrate `rodio` crate for actual `.wav`/`.ogg` playback

**File:** `src/system/audio.rs`, `src/system/audio/earcons.rs`

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

**Status: ❌ PLACEHOLDER**

### 13.1 Application Content Is a Text String
- **Spec (§4):** Level 3 should show the actual application surface wrapped in the Tactical Bezel
- **Current:** `app.rs` lines 108-112 render:
  ```html
  <div class="application-surface">
      <div class="app-mock-content">
          APPLICATION DATA FEED: {title}
      </div>
  </div>
  ```
- This is a placeholder div with text. No actual Wayland surface, X11 window, or web content is embedded.
- **Fix needed:** Embed actual application content via Wayland subsurface or XWayland redirect

**File:** `src/ui/render/app.rs` lines 108-112

### 13.2 Bezel Sliders Have No Effect
- **Current:** `app.rs` lines 96-103 render `PRIORITY` and `POWER` range sliders with static default values (`5` and `80`). No IPC messages are sent on change.
- **Fix needed:** Wire `oninput` events to IPC handlers

**File:** `src/ui/render/app.rs` lines 96-103

---

## 14. Collaboration Cue Gaps (§8)

**Status: ⚠️ PARTIAL

### 14.1 Participants Are Rendered But Not Real
- Participants are added via mock `invite_participant` IPC handler which creates fake participant data with random colors
- No actual network handshake, WebSocket connection, or token exchange occurs
- **Spec (§8.1):** "Host invites guests via secure token or contact list"

### 14.2 No Following Mode
- **Spec (§8.2):** "Optional following mode allows a guest to synchronise their view"
- **Current:** Not implemented at all

### 14.3 No Role Enforcement  
- **Spec (§8.3):** Viewer/Commenter/Operator/Co-owner with different permissions
- **Current:** Roles are stored as strings but never enforced. Any participant can execute any command.

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
| 1 | Shell API not wired (`CD`/`LS`/`cwd` sync) | §13 | `shell_api.rs`, `ipc.rs` |
| 2 | App surface is placeholder text | §4 | `app.rs` |
| 3 | Activity Mode CPU/MEM are fake numbers | §3.3 | `hub.rs` |
| 4 | Application struct missing PID | §3.3, §14 | `lib.rs` |

### P1 — High (Major spec deviation)
| # | Issue | Spec Section | File(s) |
|---|-------|-------------|---------|
| 5 | Directory Mode lacks multi-select | §3.2 | `hub.rs`, `lib.rs` |
| 6 | Directory Mode lacks action toolbar | §3.2 | `hub.rs` |
| 7 | Directory Mode lacks context menu | §3.2 | `hub.rs` |
| 8 | Path bar not breadcrumb-style | §3.2 | `hub.rs` |
| 9 | SIGTERM not sent on reset | §14 | `reset.rs` |
| 10 | Sector templates hardcoded | §15 | `hub.rs` |

### P2 — Medium (Missing integration)
| # | Issue | Spec Section | File(s) |
|---|-------|-------------|---------|
| 11 | System time / stardate hardcoded | §10 | `global.rs`, `svg_engine.rs` |
| 12 | Sector descriptions name-matched | §12 | `global.rs` |
| 13 | Inspector permissions/uptime static | §4 | `inspector.rs` |
| 14 | Audio playback is stub | §18 | `audio.rs`, `earcons.rs` |
| 15 | Remote sectors have no network I/O | §7 | `remote.rs` |
| 16 | Bezel sliders have no effect | §4 | `app.rs` |
| 17 | "MOCK" button exposed to user | — | `global.rs` |

### P3 — Low (Future roadmap items)
| # | Issue | Spec Section | File(s) |
|---|-------|-------------|---------|
| 18 | Voice/STT not implemented | §9 | `voice.rs` |
| 19 | Collaboration role enforcement | §8 | `collaboration.rs` |
| 20 | Following mode not implemented | §8 | — |
| 21 | Script engine is dead code | §12 | `script.rs` |
| 22 | Remote desktop shows mock windows | §7 | `remote.rs` |
| 23 | Minimap uses placeholder geometry | §17 | `minimap.rs` |
| 24 | Buffer inspector hex is static | §4 | `inspector.rs` |

---

*This document should be updated as fixes are applied. Each fix should move items from their current status to ✅ FIXED with a brief description of the change.*
