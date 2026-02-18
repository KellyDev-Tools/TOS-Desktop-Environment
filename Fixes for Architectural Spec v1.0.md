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

**Status: ⚠️ PARTIALLY IMPLEMENTED**

### 2.1 Path Bar Not Breadcrumb-Style
- **Spec (§3.2):** "Path bar (breadcrumb style)"
- **Current:** Flat uppercase string (e.g., `/HOME/TIM/DOCUMENTS`)
- **Fix needed:** Each path segment should be a clickable `<span>` that navigates to that level via `dir_navigate` with the absolute path

**File:** `src/ui/render/hub.rs` lines 95-97

### 2.2 No Selection Controls (Multi-Select)
- **Spec (§3.2):** "Selection controls for multi-select (checkbox, lasso, Ctrl+click)"
- **Current:** Single-click only. No checkbox, no multi-select state, no Ctrl+click
- **Fix needed:**
  - Add `selected_files: Vec<String>` to `CommandHub`
  - Render checkboxes on each file item
  - Handle `dir_select:<name>` and `dir_deselect:<name>` IPC messages
  - Support `dir_select_all` and `dir_deselect_all`

### 2.3 No Action Toolbar
- **Spec (§3.2):** "Action toolbar (New Folder, Copy, Paste, etc.) — buttons construct the corresponding CLI command"
- **Current:** No toolbar rendered
- **Fix needed:** Add an action bar below the path bar with buttons like:
  - `NEW FOLDER` → stages `mkdir <name>` in prompt
  - `COPY` → stages `cp <selected files>` in prompt
  - `PASTE` → stages `cp <clipboard> .` in prompt
  - `DELETE` → stages `rm <selected files>` in prompt (with §11.4 dangerous command handling)
  - `RENAME` → stages `mv <file> <new_name>` in prompt

### 2.4 Prompt Integration Incomplete
- **Spec (§3.2):** "Selecting a file appends its path; multi-select appends all paths"
- **Current:** Clicking a file does `stage_command:view <filename>` which **overwrites** the prompt with `view <filename>`. It stages a relative filename, not the full path.
- **Fix needed:**
  - File click should **append** the full path to the existing prompt, not overwrite
  - Multi-select should append all selected paths separated by spaces
  - Clicking should use the absolute path: `<current_directory>/<filename>`

### 2.5 No Context Menu
- **Spec (§3.2):** "Context menu (right-click/long press) for file-specific actions"
- **Current:** No right-click handling, no context menu
- **Fix needed:**
  - Intercept `contextmenu` event in JavaScript
  - Send `dir_context:<filename>` IPC message
  - Render a floating context menu with actions: Open, Copy, Cut, Paste, Rename, Delete, Properties
  - Each action constructs the CLI command and stages it in the prompt

### 2.6 Shell CWD Not Synced
- **Spec (§13.1):** Shell OSC `cwd` should inform compositor of current working directory  
- **Spec (§13.2):** Compositor should send `CD <path>` to shell via PTY
- **Current:** `dir_navigate` only mutates `hub.current_directory` in Rust state. The shell PTY never receives a `CD` command. If a user `cd`s in Command Mode, Directory Mode stays at the old path.
- **Fix needed:**
  - When `dir_navigate` changes `current_directory`, also send `CD <new_path>` to the PTY via `PtyHandle`
  - When the shell sends a `cwd` OSC sequence back, update `hub.current_directory` to match

---

## 3. Activity Mode Hardcoded Data (§3.3)

**Status: ❌ HARDCODED**

### 3.1 CPU/Memory Stats Are Fake
- **Spec (§3.3):** "Status indicators (PID, CPU/memory on hover)"
- **Current:** `hub.rs` lines 209-210 render hardcoded stats:
  ```html
  <div class="stat">CPU: 2.1%</div>
  <div class="stat">MEM: 82MB</div>
  ```
- **Fix needed:** Query actual process stats via `/proc/<pid>/stat` and `/proc/<pid>/status` on Linux, or use `sysinfo` crate

**File:** `src/ui/render/hub.rs` lines 208-211

### 3.2 No PID on Application Struct
- **Spec (§3.3):** "PID, CPU/memory on hover"
- **Current:** `Application` struct (`lib.rs` L158-163) has only `id`, `title`, `app_class`, `is_minimized`. No `pid` field.
- **Fix needed:** Add `pub pid: Option<u32>` to `Application` struct and populate it when spawning processes

### 3.3 Sector Templates Hardcoded
- **Spec (§15.1):** Templates should be loadable `.tos-template` packages  
- **Current:** `hub.rs` lines 233-246 render hardcoded template entries:
  ```html
  DEV-GRID (3 HUBS // 5 APPS)
  SCIENCE-LAB (1 HUB // 2 APPS)
  ```
- **Fix needed:** Read templates from `~/.local/share/tos/templates/` directory and render dynamically

**File:** `src/ui/render/hub.rs` lines 232-247

### 3.4 No Multi-Select for Batch Actions
- **Spec (§3.3):** "Multi-select for batch actions (close, kill, move)"
- **Current:** No multi-select mechanism on app tiles
- **Fix needed:** Add checkboxes to app tiles, track selection state, render batch-action toolbar

### 3.5 No Integration with Prompt
- **Spec (§3.3):** "Selecting a tile populates the prompt with PID/window ID; contextual chips suggest relevant commands"
- **Current:** Clicking an app tile stages `focus <title>` in the prompt. No PID population, no contextual chips.
- **Fix needed:** Should stage the PID, and show chips like `kill <PID>`, `nice -n 10 <PID>`, `strace -p <PID>`

---

## 4. Global Overview Hardcoded Data (§2, §10)

**Status: ❌ HARDCODED**

### 4.1 System Time Is Static
- **Spec:** Telemetry bar should show live system data
- **Current:** `global.rs` line 36 renders:
  ```html
  <span class="value">10:39</span>
  ```
- **Fix needed:** Use JavaScript `setInterval` or IPC to push the current system time

**File:** `src/ui/render/global.rs` line 36

### 4.2 Stardate Is Static
- **Current:** `global.rs` line 47 renders a hardcoded stardate: `02-33 // 02-1478`
- **Fix needed:** Either calculate a stardate from current time via an algorithm, or show real date/time

**File:** `src/ui/render/global.rs` line 47

### 4.3 Sector Descriptions Are Name-Matched Strings
- **Current:** `global.rs` lines 61-66 match sector names to hardcoded descriptions:
  ```rust
  "Alpha Sector" => ("Primary coordination and terminal access.", "⌨️"),
  "Science Labs" => ("Data analysis and sensor array telemetry.", "🔬"),
  "Engineering" => ("Core systems and resource management.", "⚙️"),
  _ => ("Remote node established via TOS protocol.", "📡"),
  ```
- **Fix needed:** Descriptions and icons should be fields on the `Sector` struct, or derived from its `SectorType` (§12.2), not matched by name string

### 4.4 "MOCK" Button on Remote Card
- **Current:** `global.rs` line 135 has a `MOCK` button:
  ```html
  <button class="action-btn" onclick="...">MOCK</button>
  ```
- This exposes internal testing functionality to the user. Should be removed or gated behind a developer mode.

**File:** `src/ui/render/global.rs` line 135

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
