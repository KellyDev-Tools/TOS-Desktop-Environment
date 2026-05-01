# TOS Architecture Specification

**Purpose:** This document is the single source of truth for the architecture and visual design of **TOS** (**Terminal On Steroids**). It defines system structure, process boundaries, IPC contracts, the visual hierarchy, rendering model, input abstraction, security, and all platform behaviour. The terminal and command line are the absolute centre of every design decision. All features exist to augment the terminal, never to bypass it.

**Version:** 0.1.0

---

## Table of Contents

1. [Core Philosophy: Terminal First](#1-core-philosophy-terminal-first)
2. [System Overview](#2-system-overview)
3. [Process Architecture: Brain & Face](#3-process-architecture-brain--face)
4. [Modular Service Architecture](#4-modular-service-architecture)
5. [The Extended Hierarchy](#5-the-extended-hierarchy)
6. [Global Overview – Level 1](#6-global-overview--level-1)
7. [Command Hub – Level 2](#7-command-hub--level-2)
8. [Application Focus – Level 3](#8-application-focus--level-3)
9. [Deep Inspection & Recovery – Level 4](#9-deep-inspection--recovery--level-4)
10. [Sectors and the Tree Model](#10-sectors-and-the-tree-model)
11. [Split Viewports & Pane Management](#11-split-viewports--pane-management)
12. [Remote Sectors](#12-remote-sectors)
13. [Collaboration](#13-collaboration)
14. [Input Abstraction Layer](#14-input-abstraction-layer)
15. [Platform Abstraction & Rendering](#15-platform-abstraction--rendering)
16. [Performance and Compositing](#16-performance-and-compositing)
17. [Security Model](#17-security-model)
18. [Modules](#18-modules)
19. [TOS Log](#19-tos-log)
20. [Reset Operations](#20-reset-operations)
21. [Visual Design, Sensory Interface, and Accessibility](#21-visual-design-sensory-interface-and-accessibility)
22. [Sector Templates and Marketplace](#22-sector-templates-and-marketplace)
23. [Settings Data Model & IPC](#23-settings-data-model--ipc)
24. [Shell API Enhancements](#24-shell-api-enhancements)
25. [Bezel IPC Contracts](#25-bezel-ipc-contracts)
26. [Terminal Output Rendering](#26-terminal-output-rendering)
27. [UI Module Interaction APIs](#27-ui-module-interaction-apis)
28. [Predictive Fillers & Intuitive Interaction](#28-predictive-fillers--intuitive-interaction)
29. [Implementation Roadmap](#29-implementation-roadmap)
30. [Glossary of Terms](#30-glossary-of-terms)

---

## 1. Core Philosophy: Terminal First

TOS was born from the acronym **Terminal On Steroids** — a vision to take the raw power of the command line and amplify it across every platform, from desktop to VR to mobile. The terminal is not just one mode among many; it is the **primary and permanent interface**. Every action a user takes — whether clicking a file, speaking a command, or gesturing in VR — must ultimately be expressible as a command line that appears in the **Persistent Unified Prompt** and is executed by the underlying shell.

This philosophy ensures that:

- **Power users never lose their terminal.** All visual augmentations are simply different ways to view and interact with the same data the terminal already exposes. They generate commands, never bypass them.
- **The prompt is the source of truth.** Whatever is staged in the prompt is what will be executed. Clicking a file appends its path; selecting a process inserts its PID. The user always sees the command before running it.
- **All modes are overlays on the terminal.** The terminal output area remains visible and central; chip regions, the bezel, and other UI elements are helpers, not replacements.
- **The Shell API (OSC integration) is the backbone.** Deep bidirectional communication with the shell ensures the UI stays in sync with the real environment.

The Face is built on standard web technologies (HTML/CSS/JS) acting as a dynamic graphical frontend to the Brain. The design language features LCARS-inspired elements with modern additions: glassmorphism, smooth kinetic transitions, and a high-density grid aesthetic. All augmentations are overlays designed to empower the terminal, never to obscure it permanently.

---

## 2. System Overview

TOS is built around a strictly vertical hierarchy of **levels**, a tree of **sectors**, and a **Persistent Unified Prompt** that drives all interaction. The system is composed of:

- A **platform-agnostic core** (the **Brain**) implementing the hierarchy, command execution, security, and coordination.
- A **Unified Tactical Bezel** — a persistent frame surrounding the entire viewport across all levels, divided into four segments:
  - **Top Bezel Segment:** System controls (logout, shutdown, reboot, settings) and dual expansion handles. Hosts **Configurable Horizontal Slots** for telemetry and quick-access tools.
    - **Left Section:** Expand/collapse handle for the left lateral segment; defaults to Active Viewport Title.
    - **Center Section:** Telemetry cluster — configurable slots for Brain Connection Status and Resource Telemetry (CPU, MEM, NET, time, date).
    - **Right Section:** Expand/collapse handle for the right lateral segment, then system controls menu and System Status Badges.
  - **Bottom Bezel Segment:** The Persistent Unified Prompt. **Strictly static — no configurable slots.** Contains the Universal Mode Selector (left), command input (center), and mic/stop controls (right).
  - **Lateral Segments (Left & Right):** Slender vertical bars containing **Configurable Vertical Slots**. Left defaults to Hierarchy Navigation and Tactical Mini-Map. Right defaults to Priority Indicators and Mini-Log Telemetry.
  - **Configurability:** All Top, Left, and Right slots are user-definable via the Settings panel or direct manipulation. Any system tool can be docked to any slot.
- **Platform backends** (Wayland, OpenXR, Android) providing rendering, input, and system services via three core traits: `Renderer`, `InputSource`, `SystemServices`.
- **Remote connectivity** via the TOS Remote Server protocol, enabling remote sectors, collaboration, and web portals.
- **Module system** for Application Models, Sector Types, AI backends, Terminal Output Modules, Theme Modules, and Shell Modules, all sandboxed and permissioned (see [Ecosystem Specification](./TOS_v0.1.0_Ecosystem.md)).
- A set of **auxiliary services** running as independent processes communicating via IPC.
- A **system-level terminal output** at Level 1 displaying the Brain's own console, powered by the terminal output module system.

### 2.1 Slot Projection Mechanism

Modules docked within the Bezel use a projection mechanism to reveal detailed information without shifting the stable bezel frame:

- **Lateral Projection:** Components in Left or Right slots expand horizontally inward toward the viewport center (e.g., sliding out the Mini-Map).
- **Vertical Projection (Downward):** Components in Top slots expand downward (e.g., dropping a Resource Telemetry glass panel).

---

## 3. Process Architecture: Brain & Face

TOS adopts a clean separation between logic and presentation by running two concurrent threads (or optionally separate processes) communicating via a well-defined IPC protocol.

### 3.1 The Brain (Logic Thread/Process)

- Maintains the core state machine: sectors, command hubs, application surfaces, zoom levels, and user sessions.
- Handles all command execution (shell integration, PTY management) for both sector terminals and its own system console.
- Processes semantic events and updates state accordingly.
- Manages collaboration, remote connections, and module lifecycle.
- Emits state snapshots and deltas to be consumed by the Face and other services.
- Streams its own console output (system logs, background task results, errors) to the Face for display in the system-level terminal output at Level 1.

### 3.2 The Face (UI Thread/Process)

- Runs the platform-specific renderer.
- Captures raw input from devices and forwards it to the Brain (after optional local echo for immediate feedback).
- Receives state updates from the Brain and renders the interface.
- Hosts the Tactical Bezel, mini-map, and all visual overlays.
- Instantiates and manages Terminal Output Modules for each terminal context.
- Instantiates terminal output areas for each active agent in the Workflow Manager pane, ensuring isolated output context per agent.
- Manages viewport focus based on the session's `active_view` field; if empty, defaults to the primary sector terminal.

**Kinetic Zoom Transitions:** Transformations between levels use a **Kinetic Zoom Transition**:
- **Borders as Anchors:** When zooming from Level 1 to 2, the sector tile's borders expand outward to become the Tactical Bezel.
- **Layer Stacking:** Background layers use depth-blur and fade (z-axis displacement) as the focal layer moves forward.
- **Viewport Expansion:** The terminal canvas unfurls from the center of the tile, ensuring a seamless visual link between the tile and the functional hub.
- **Persistence:** Level 4 sub-views are transient; Tactical Reset provides a low-overhead global recovery view.

### 3.3 Communication

- **IPC Protocol:** JSON-RPC or MessagePack over a local socket or channel.
- **Shared Protocol Library (`tos-protocol`):** All state structures and IPC prefix definitions are extracted into a shared Rust crate, ensuring a stable contract between the Brain and any remote Face implementation.
- **Local-First Connectivity:** A Face MUST attempt to connect to a Local Brain via Unix Domain Sockets or Shared Memory first. If unavailable, the Face falls back to acting as a Remote Client over the network via the TOS Remote Server (§12).
- **Messages from Brain to Face:** State deltas, audio/haptic commands, UI control signals, and lines for the system-level terminal output.
- **Messages from Face to Brain:** Semantic events (after mapping), prompt submissions, bezel clicks, and context menu actions.

#### 3.3.5 Face Capability Profile

Every Face MUST send a `face_register` message immediately after connecting. The Brain uses this profile to adapt layout defaults, AI skill activation, and bezel slot behavior to the connecting device.

```json
{
  "type": "face_register",
  "profile": "mobile",
  "capabilities": ["touch", "voice"],
  "viewport": { "w": 390, "h": 844 }
}
```

| Profile | Description | Default Hub Layout | Default AI Skill |
|:---|:---|:---|:---|
| `desktop` | Mouse + keyboard, large viewport | Vertical split (terminal + editor) | Chat Companion |
| `handheld` | Touch-first, Phone/Tablet viewport | Tab layout (terminal / editor tabs) | Voice-first Chat Companion |
| `spatial` | OpenXR, VR/AR/MR spatial input | Single pane with spatial bezel | Passive Observer |

On a `handheld` profile registration, the Brain automatically:
- Sets default hub layout to `tabs` unless the session file specifies otherwise
- Collapses all Left/Right bezel slots to their minimal state
- Enables the Expanded Bezel Command Surface as the primary prompt interaction surface
- Routes AI skill activation to voice-first mode if `voice` is declared in capabilities

#### 3.3.1 Message Format Standard

All IPC messages sent from the Face to the Brain MUST follow this scheme:
- **Format:** `prefix:payload`
- **Prefix:** A unique action identifier ending in a colon.
- **Payload:** Message-specific data. If multiple arguments are required, they MUST be delimited by **semicolons** (`;`).
- **Example:** `set_setting:theme;lcars-dark` or `signal_app:uuid;SIGTERM`.

#### 3.3.2 State Delta (Brain → Face)

The Brain broadcasts state updates as JSON deltas to minimize bandwidth. A delta contains only changed fields or new objects.

```json
{
  "type": "state_delta",
  "timestamp": 1709300000,
  "sectors": {
    "sector_uuid": {
      "status": "active",
      "priority_score": 85,
      "hubs": {
        "hub_uuid": {
          "mode": "CMD",
          "terminal_lines": [
            {"id": 1024, "content": "build successful", "type": "stdout", "priority": 2}
          ],
          "active_app_uuid": "app_uuid_1"
        }
      }
    }
  },
  "global_priority_alerts": [
    {"source": "power", "level": 4, "message": "Low Battery"}
  ]
}
```

#### 3.3.3 Settings IPC (Face ↔ Settings Daemon)

- **Get Setting:** `get_setting:key` → Response: `setting_val:key;value`
- **Set Setting:** `set_setting:key;value` → Response: `setting_status:key;success`
- **Change Notification:** The Daemon broadcasts `setting_changed:key;value` to all subscribers.

#### 3.3.4 TOS Log Query (Face → Log Service)

- **Query:** `log_query:{"surface": "browser", "since": "-10m", "limit": 50}`
- **Response:** `{"query_id": "uuid", "results": [{"ts": 1709299400, "level": "INFO", "source": "browser", "event": "navigation", "data": "https://..."}]}`

### 3.4 Face Disconnected Mode

The Face must operate gracefully when no Brain is reachable. This section defines the connection lifecycle, visual states, degraded capabilities, and reconnection logic for a Face that has lost contact with its Brain — or never had one.

#### 3.4.1 Connection Lifecycle

The Face's relationship with the Brain has four states:

| State | Description |
|:---|:---|
| **Connecting** | Face is actively attempting to reach a Brain (local socket → remote anchor → saved hosts → mDNS scan). |
| **Connected** | Brain link established. Face has received at least one `state_delta` and sent `face_register`. Normal operation. |
| **Disconnected** | Brain link was previously established but has been lost (socket closed, heartbeat timeout, network failure). |
| **No Brain** | Face has exhausted all discovery methods and has never connected in this session. |

**State Transitions:**

- **Launch →** `Connecting` (always).
- **`Connecting` →** `Connected` when Brain responds and `face_register` is accepted.
- **`Connecting` →** `No Brain` when all discovery methods are exhausted.
- **`Connected` →** `Disconnected` on heartbeat timeout or socket closure.
- **`Disconnected` →** `Connecting` when automatic retry triggers.
- **`No Brain` →** `Connecting` when the user initiates manual connect, or on periodic background probe (every 60 seconds).

#### 3.4.2 Connection Health Detection

The Brain already maintains a 1Hz background heartbeat that increments `brain_time` and the state `version` counter on every tick (see §3.3.2). These version-bumped `state_delta` messages serve as an implicit heartbeat

**Face-side detection:** The Face monitors the incoming `state_delta` stream. If **no `state_delta` is received for 5 consecutive seconds** (5 missed 1Hz ticks), the Face transitions to `Disconnected`. This threshold accounts for minor network jitter without triggering false disconnects.

**Brain-side detection:** The Brain's Service Registry (Ecosystem §4.3) already probes registered services at 30-second intervals and marks unresponsive daemons as `offline`. For Face connections specifically, the Brain detects Face loss when the WebSocket or IPC channel reports a closed/errored socket, at which point it unregisters the Face internally.

**Clock drift:** The `brain_time` field in `state_delta` messages is used by the **Brain Connection Status** bezel component (§8.1) to display Brain time and detect clock drift between the Face and Brain.

#### 3.4.3 Visual States

##### Connecting State

The Face displays a **Connection Screen** — a full-viewport LCARS-styled overlay. This is the first screen the user sees when no Brain is immediately available.

```
┌──────────────────────────────────────────────────────────────────┐
│                                                                  │
│                                                                  │
│                          ╔═══════════╗                           │
│                          ║  TOS ◉    ║   ← animated pulse       │
│                          ╚═══════════╝                           │
│                                                                  │
│                    Connecting to Brain...                         │
│                                                                  │
│              ┌──────────────────────────────────┐                │
│              │  ● Local socket       checking... │                │
│              │  ○ Remote anchor      waiting     │                │
│              │  ○ mDNS scan          waiting     │                │
│              │  ○ Saved hosts         waiting     │                │
│              └──────────────────────────────────┘                │
│                                                                  │
│         [ Connect Manually ]     [ Continue Offline ]            │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

- **Discovery progress:** Each method shows a status indicator: `●` active (with spinner), `✓` succeeded, `✗` failed, `○` waiting.
- **[Connect Manually]** — opens the connection dialog (Ecosystem §5.3) so the user can enter a host and port directly.
- **[Continue Offline]** — transitions to the **No Brain** state immediately (skips remaining discovery).

The Face cycles through discovery methods in the order defined by §3.3 and Ecosystem §5.4, with a **3-second timeout** per method.

##### Disconnected State

When transitioning from Connected to Disconnected, the existing UI remains visible but enters a degraded state:

```
┌──────────────────────────────────────────────────────────────────┐
│ ▮ Top Bezel ▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮ │
│┌────────────────────────────────────────────────────────────────┐│
││  ⚠ Brain connection lost — reconnecting...  attempt 2 (in 3s) ││ ← amber banner
│├────────────────────────────────────────────────────────────────┤│
││                                                                ││
││  ┌─────────┐  ┌─────────┐  ┌─────────┐                       ││
││  │ Sector1 │  │ Sector2 │  │ Sector3 │  ← dimmed 50% opacity ││
││  │  (idle) │  │  (idle) │  │  (idle) │                        ││
││  └─────────┘  └─────────┘  └─────────┘                        ││
││                                                                ││
│├────────────────────────────────────────────────────────────────┤│
││  ▮ ⊘ Disconnected ▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮  [ Connect → ]   ││ ← prompt disabled
│└────────────────────────────────────────────────────────────────┘│
└──────────────────────────────────────────────────────────────────┘
```

1. **Immediate:** The **Brain Connection Status** bezel component (§8.1) switches to a pulsing red "OFFLINE" indicator.
2. **After 1 second:** An amber banner slides down from the top bezel showing the retry counter and next-attempt countdown.
3. **All sector content freezes** in its last-known state. Sector tiles remain visible but are dimmed (50% opacity). Terminal output stops scrolling but the last-known content remains visible.
4. **The Persistent Unified Prompt** (§7.1) is disabled — input is blocked with a "Disconnected" placeholder. A **[Connect →]** button replaces the mic/stop controls, opening the connection dialog (Ecosystem §5.3).
5. **Bezel slots** remain visible but non-interactive (except Brain Connection Status and the prompt's Connect button).

##### No Brain State

If no Brain is found (all discovery exhausted), or the user chose **[Continue Offline]**, the Face renders a connection-focused UI:

```
┌──────────────────────────────────────────────────────────────────┐
│ ▮ Brain: No Brain ▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮ │
│                                                                  │
│                     No Brain Connected                           │
│                                                                  │
│   ─── Connect to a Brain ─────────────────────────────────────   │
│                                                                  │
│   [ Scan Network ]    Discover TOS instances on your LAN (mDNS)  │
│   [ Enter Address ]   Connect to a known host and port           │
│                                                                  │
│   ─── Recent Hosts ───────────────────────────────────────────   │
│                                                                  │
│   ┌──────────────────────────────────────────────────────────┐   │
│   │  ★ Workstation        10.0.0.42:7000    2h ago  [ → ]   │   │
│   │    Dev Server         192.168.1.5:7000  3d ago  [ → ]   │   │
│   │    Laptop             192.168.1.12:7000 1w ago  [ → ]   │   │
│   └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│   ─── Last Session ───────────────────────────────────────────   │
│   Lost connection to 10.0.0.42:7000 at 16:05:12                  │
│                                                                  │
│   [ Reconnect Last ]                                             │
│                                                                  │
│   Keyboard shortcut: Ctrl+Shift+R to reconnect at any time       │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

- **[Scan Network]** — triggers mDNS discovery (`_tos-brain._tcp`). Results appear as selectable rows below the button as they are found.
- **[Enter Address]** — opens the connection dialog (Ecosystem §5.3) for manual host:port entry.
- **Recent Hosts** — pulled from `~/.config/tos/remote-hosts.toml`. Each row shows the saved name, address, time since last connection, and a one-tap **[→]** connect button. The most recently used host is marked with ★.
- **Last Session** — if the Face was previously connected and lost the brain this session, displays the last-known endpoint and time of disconnection, with a **[Reconnect Last]** shortcut.
- **Bezel:** Only the **Brain Connection Status** component is active (gray "No Brain" indicator). If the Face has cached settings from a prior session, the Settings panel is accessible in read-only mode.
- **Theme:** The active theme CSS remains applied (themes are loaded by the Face from local files, not streamed from the Brain).
- **No sectors, no prompt, no chips.** The Face cannot create or display sectors without a Brain.

#### 3.4.4 Reconnection Logic

##### Automatic Retry (Disconnected State Only)

When the Face enters the Disconnected state, it initiates automatic reconnection:

| Attempt | Delay | Method |
|:---|:---|:---|
| 1–3 | 1s, 2s, 4s | Reconnect to last-known Brain endpoint (same socket/host:port) |
| 4–6 | 8s, 16s, 30s | Full discovery cycle (local socket → anchor port → mDNS → saved hosts) |
| 7+ | 60s (steady) | Background probe every 60 seconds until successful or user cancels |

The amber banner displays: `"Reconnecting... attempt N (next in Xs)"`.

##### Manual Retry

At any time during Disconnected or No Brain states, the user can:
- Click **[Connect to Brain]** to open the connection dialog.
- Press `Ctrl+Shift+R` (semantic event: `reconnect_brain`) to force an immediate discovery cycle.

##### Successful Reconnection

When reconnection succeeds:

1. Face sends `face_register` (§3.3.5) as if freshly connecting.
2. Brain responds with a **full state snapshot** (not a delta — the Face's cached state may be stale).
3. The amber banner transitions to green: `"Brain reconnected"` for 3 seconds, then dismisses.
4. All sectors, bezel components, and the prompt restore to their Brain-authoritative state.
5. An earcon plays (Visual Design §3): `reconnect_success`.
6. If the Brain's state differs significantly from the cached state (e.g., sectors were created/destroyed while disconnected), the Face performs a full viewport rebuild rather than attempting to diff.

#### 3.4.5 Degraded Capability Matrix

| Capability | Connected | Disconnected | No Brain |
|:---|:---|:---|:---|
| View sectors | ✓ Live | ✓ Frozen (last known) | ✗ None |
| Execute commands | ✓ | ✗ | ✗ |
| View terminal history | ✓ | ✓ Read-only (cached) | ✗ |
| Change settings | ✓ | ✗ | ✗ (read-only if cached) |
| Switch themes | ✓ | ✓ (local CSS only) | ✓ (local CSS only) |
| View bezel telemetry | ✓ | ✗ (stale values shown) | ✗ |
| Use AI | ✓ | ✗ (queued per Features §4.9) | ✗ |
| Connect to remote Brain | ✓ | ✓ | ✓ |
| Navigate hierarchy | ✓ | ✗ | ✗ |
| Copy text from terminals | ✓ | ✓ (from cached buffer) | ✗ |

#### 3.4.6 State Caching

The Face maintains a **local state cache** to support the Disconnected state:

- **Cache contents:** The most recent `state_delta` snapshot, including sector tree, hub states, terminal buffer (last 500 lines per hub), and bezel slot states.
- **Cache lifetime:** Valid for the current Face process lifetime only. Not persisted to disk (session persistence is the Brain's responsibility via `tos-sessiond`).
- **Cache invalidation:** Fully replaced on reconnection when the Brain sends a fresh state snapshot.

#### 3.4.7 IPC Contracts — Connection Lifecycle

| Message | Direction | When |
|:---|:---|:---|
| `state_delta` | Brain → Face | Every 1 second (1Hz heartbeat tick). Absence for 5s triggers Disconnected state. |
| `face_reconnect` | Face → Brain | Face is reconnecting after disconnect; requests full state snapshot |
| `state_snapshot` | Brain → Face | Full state (not delta) sent in response to `face_reconnect` |
| `connection_lost` | Face (internal) | Synthetic event fired when no `state_delta` received for 5 seconds; triggers Disconnected visual state |

#### 3.4.8 Audio & Haptic Cues

| Event | Earcon | Haptic |
|:---|:---|:---|
| Connection lost | `connection_lost` — descending two-tone alert | Medium pulse (200ms) |
| Reconnection attempt | None (silent) | None |
| Reconnection successful | `reconnect_success` — ascending chime | Light pulse (100ms) |
| No Brain (discovery exhausted) | `no_brain` — single low tone | Long pulse (400ms) |

#### 3.4.9 Developer Mode (`make dev-web`)

When the Face is started via `make dev-web` (no Brain), it operates in the **No Brain** state by default. For UI development convenience:

- An environment variable `TOS_DEV_MODE=1` is set by the `dev-web` target.
- When `TOS_DEV_MODE=1`, the Face loads a **mock state fixture** from `svelte_ui/fixtures/mock_state.json` instead of waiting for a Brain connection.
- The fixture provides synthetic sectors, hubs, terminal output, and bezel state so that all UI components can be developed and tested visually.
- The mock state is static — no IPC, no real PTY. But all rendering paths are exercised.
- The **Brain Connection Status** component shows "DEV MODE" in amber instead of "No Brain".

#### 3.4.10 Platform-Specific Notes

| Platform | Behavior |
|:---|:---|
| **Electron (Windows/Linux/macOS)** | Face launches and enters Connecting state. If Brain is expected to be co-located (same machine), the Electron main process can optionally spawn the Brain binary itself before the renderer loads. |
| **Web Face (Browser)** | Connects to Brain via WebSocket. Disconnected state shows the reconnection banner within the browser tab. Browser refresh triggers a full reconnect cycle. |
| **Android Handheld** | Face app shows a dedicated connection screen with LAN scan and saved hosts. Background retry uses Android WorkManager to respect battery limits. |
| **Horizon OS (Quest)** | Remote-only client. Disconnected state overlays the VR environment with a 2D reconnection panel. |

---

## 4. Cortex Orchestration Layer

The Cortex is the Brain’s modular reasoning, context, and behavior framework. It replaces the monolithic AI Engine and the former Skill/Behavior subsystem. Cortex components – **Assistants**, **Curators**, and **Agents** – run as independent services or subprocesses and communicate via MCP (Model Context Protocol), JSON‑RPC over Stdin/Stdout, or direct IPC.

The Cortex manages:
- **Assistant lifecycle:** LLM backend connection, model discovery, and request routing.
- **Curator federation:** Multi‑source context aggregation (GitNexus, Jira, Filesystem) into a Global Knowledge Graph.
- **Agent stacking:** Hierarchical prompt merging for persona, constraints, and formatting.

All Cortex manifests are hot‑loaded from `~/.local/share/tos/cortex/` and configured through the Settings UI.

| Service | Responsibilities | API / Protocol |
|---------|-----------------|----------------|
| **Brain** | Core state machine, command execution, coordination (§3.1). | JSON-RPC (IPC) |
| **Face** | UI rendering, input capture (§3.2). | JSON-RPC (IPC) |
| **Marketplace Service** | Package index, download, installation, dependency resolution, signature verification, update monitoring. | `marketplace_search`, `marketplace_install` |
| **Settings Daemon** | Store/retrieve configuration values, persistence, change notifications. | `get_setting`, `set_setting` |
| **TOS Log Service** | Collect events from all components, manage log storage, retention, redaction, query interface. | `log_query` |
| **Cortex** | Manages Assistants, Curators, Agents; handles context broadcast and tool routing. | MCP (for Curators), JSON‑RPC (for Assistants/Agents) |
| **File Sync Service** | Monitor directories, perform bidirectional sync, conflict resolution via WebDAV extensions. | WebDAV + Inotify/FSEvents |
| **Search Service** | Indexing of file contents, logs, and metadata. Query syntax supports regex and semantic filters. | `search_query` |
| **Notification Center** | Aggregate notifications, manage history, deliver to Face with priority levels (1–5). | `notify_push` |
| **Update Daemon** | Atomic update check, download, and staging. Coordination of the "Yellow Alert" status. | `update_check`, `update_apply` |
| **Heuristic Service** | Predictive fillers, autocomplete-to-chip logic, typo correction, heuristic sector labeling. | `heuristic_query` |
| **Audio & Haptic Engine** | Mix three-layer audio, play earcons, trigger haptic patterns. | `play_earcon`, `trigger_haptic` |
| **Session Service** | Session persistence and workspace memory (live state auto-save, named sessions). | See [Features Specification §2](./TOS_v0.1.0_Features.md) |

All services communicate with the Brain via IPC. The Brain maintains authoritative state and routes messages as needed.

---

## 5. The Extended Hierarchy

| Level | Name | Description |
|-------|------|-------------|
| **1** | **Global Overview** | Bird's-eye view of all sectors, with System Output Area (Brain console) rendered as a terminal output module layer below the tiles. |
| **2** | **Command Hub** | Central control for a sector, with full terminal and prompt. |
| **3** | **Application Focus** | Full-screen application surface wrapped in the Tactical Bezel. |
| **4** | **Deep Inspection & Recovery** | Unified diagnostic level with three sub-views: Detail View (structured metadata), Buffer View (raw hex dump, privileged), and Tactical Reset (God Mode wireframe recovery). |

**Lifecycle:** Level 4 sub-views are transient; Tactical Reset flushes all inspection buffers and provides a global recovery environment.

The Expanded Bezel Command Surface (see [Features Specification §1](./TOS_v0.1.0_Features.md)) is a cross-cutting overlay state — not a level. It is available at all levels except Tactical Reset (God Mode):

| Level | Expanded Bezel Available? |
|-------|--------------------------|
| LVL 1 — Global Overview | ✓ Yes |
| LVL 2 — Command Hub | ✓ Yes (extends the existing prompt) |
| LVL 3 — Application Focus | ✓ Yes — primary use case |
| LVL 4 — Deep Inspection (Detail / Buffer) | ✓ Yes |
| LVL 4 — Tactical Reset (God Mode) | ✗ Disabled |

---

## 6. Global Overview – Level 1

The Global Overview is the top level of the hierarchy, providing a bird's-eye view of all sectors. It features a dedicated **System Output Area** that displays the Brain's console output in a live terminal view, powered by the same Terminal Output Module system used in Command Hubs but in **read-only** mode (no prompt, no command input).

### 6.1 Sector Tiles as Mini Command Hubs

Each sector tile's borders mirror the Command Hub's structure and provide at-a-glance status:

- **Top border** — Represents the Tactical Bezel (collapsed). A coloured strip indicates alert status (green/yellow/red) or active collaboration (presence of guests).
- **Bottom border** — Embodies the Persistent Unified Prompt and reflects the status of the last command executed:
  - **Solid color:** Solid line — command completed. Green (success) or red (failure).
  - **Animated gradient:** Sliding gradient between success and failure colors if a command is currently running. Direction and speed are user-configurable.
  - **No history:** Neutral color (gray) or invisible for fresh sectors.
- **Left/right borders** — Mode indicators (CMD, DIR, ACT, SEARCH) as small coloured chips and priority indicator chips (Visual Design §1).
- **Tile Interior** — Live or cached thumbnail of the sector's primary Command Hub or focused application surface. If idle, shows sector name and type icon.

**Additional information conveyed:**
- Recent activity: subtle "wave" animation along the bottom border after command completion.
- Collaboration presence: tiny avatar dots along the top border for active guests.

Tiles may have semi-transparent backgrounds allowing the System Output Area to be visible behind them (user-configurable opacity).

### 6.2 System Output Area (Brain Console)

The System Output Area is a live terminal view of the Brain's console, rendering the `system` domain from the TOS Log Service (§19). It is rendered as a distinct layer **above** a solid background and **below** the sector tiles, unless toggled to the front via the bezel.

**Layering:**
- **Bottom layer:** Solid color or wallpaper.
- **Middle layer:** System Output Area (rendered by Terminal Output Module). Visible behind tiles; tile opacity is user-adjustable.
- **Top layer:** Sector tiles, Global Overview bezel, and interactive UI elements.

When the user activates **"Bring Terminal to Front"** from the bezel, the System Output Area temporarily moves to the top layer. A subsequent command or `Esc` returns it to the middle layer.

**Content:** System logs, Brain startup messages, service lifecycles, background task output, security events, collaboration events.

**Configuration:** Size/position (full-screen, bottom panel, left panel), opacity, Terminal Output Module overrides (font size, color scheme, animation speed).

### 6.3 Zoom Transition

Selecting a sector tile smoothly expands its borders into the full Command Hub. The System Output Area fades out or slides away, replaced by the sector's own terminal output area. The System Output Area remains accessible via a bezel shortcut after zooming.

### 6.4 Global Overview Bezel

**Collapsed:** Thin top strip with Settings icon, Add Sector button, expand handle, collaboration indicator, and Terminal Toggle (eye icon).

**Expanded:** Reveals a command strip with:
- **Navigation:** Zoom Out (if applicable), Home.
- **Sector Management:** New Sector, Import Sector, Remote Connection.
- **System:** Settings, Updates, Security Dashboard.
- **Terminal Controls:** Bring Terminal to Front, Scroll Terminal Up/Down, Clear Terminal.
- **Collaboration:** Share Overview, Active Sessions, Invite Users.
- **View Controls:** Toggle Mini-Map, Toggle Sector Labels, Arrange Tiles.
- **Power:** Sleep, Restart TOS, Log Out (with confirmation per §17.2).

### 6.5 Sector Tile Context Menu (Secondary Select)

A long press (touch) or secondary click on a sector tile opens a context menu rendered as a floating LCARS panel near the tile.

#### 6.5.1 Menu Actions

| Action | Description | Confirmation |
|--------|-------------|-------------|
| **Close Sector** | Terminates all processes and removes the tile. | If processes are running, a warning modal lists them. Requires tactile confirmation (§17.2). |
| **Freeze Sector** | Suspends all processes (SIGSTOP). Tile is dimmed with a snowflake badge. | No additional confirmation. |
| **Clone Sector** | Creates a new sector with identical configuration. Running processes are not duplicated. | Optional confirmation dialog. |

#### 6.5.2 Visual Feedback

- **Freeze:** Tile opacity reduced, snowflake icon in top-right corner. Bottom border freezes in current state.
- **Close:** Tile animates out (shrink and fade). Optional undo notification (5s).
- **Clone:** New tile animates in near the original with a brief "Cloning..." indicator.

#### 6.5.3 IPC Messages

- `sector_close:<sector_id>`
- `sector_freeze:<sector_id>`
- `sector_unfreeze:<sector_id>`
- `sector_clone:<sector_id>`

#### 6.5.4 Accessibility

- Fully keyboard-navigable (arrow keys, Enter).
- Screen reader labels announced.
- Haptic feedback on menu opening (user-configurable).

---

## 7. Command Hub – Level 2

When the user zooms into a sector, the System Output Area is replaced by the sector's own terminal output area. A **Show System Output** button in the bezel can temporarily overlay the system console without leaving the sector.

### 7.1 Persistent Unified Prompt (Bottom Bezel Segment)

The Bottom Bezel Segment houses the Persistent Unified Prompt. Visible across all levels and modes. **Strictly static — no configurable slots.**

- **Left Section (Origin):** Universal Mode Selector (CMD, SEARCH, AI, ACTIVITY). An integral part of the prompt assembly, not a dockable module.
- **Center Section:** The input field. Always reflects the current staged command across all interaction modes.
- **Right Section:** Mic and Stop buttons for voice-first interaction and command termination.
- **Visual State by Level:**

| Visual State | Applicable Levels | Description |
|---|---|---|
| **Expanded** | Level 2 | Fully visible and interactive. |
| **Collapsed & Expandable** | Level 3 | Tapping or hovering expands the prompt temporarily. |
| **Collapsed & Locked** | Level 4 (Detail / Buffer) | Visible but not interactive; focus is inspection. |
| **Disabled** | Level 4 (Tactical Reset) | Hidden or locked; Tactical Reset takes priority. |

### 7.2 Terminal Output as Primary Canvas

The terminal output area is rendered by the currently active **Terminal Output Module** (see Ecosystem Specification §1.5). The module defines the visual appearance, layout, and any special effects (perspective, animation). The Face provides the module with a stream of lines from the sector's PTY and handles user interactions via the module's API.

Default: **Rectangular Terminal Output Module** — standard full-width rectangle with uniform text and vertical scrolling. Additional modules (e.g., **Cinematic Triangular Module**) can be installed via the Marketplace.

The output area occupies the full width between the left and right chip regions. Users can temporarily bring it to the front using a bezel toggle (§7.8).

### 7.3 Context-Aware Terminal Augmentation

TOS treats the **Terminal Canvas** and the **Dual-Sided Chip Layout** as a unified interface. The system context dictates what appears in the terminal and how chips are populated:

| Context | Terminal Canvas | Chip Layout Integration |
|---------|-----------------|------------------------|
| **Command** | Standard shell `stdout`/`stderr`. | Chips show command history, autocomplete suggestions, tool flags. |
| **Search** | Semantic or exact search results. | Chips populate with search scopes, filters, quick-action buttons. |
| **AI** | The LLM's rationale, thought process, or raw output. | Chips act as command staging buttons for AI-suggested shell operations. |
| **Directory** | Raw directory listing (`ls` / `cd`). | Chips populate with interactive file and folder paths. Chips also provide file or image previews when applicable. |
| **Activity** | Raw process table (`top` / `ps`). | Chips populate with process-handling actions (kill, renice, monitor). Running apps show 10Hz live thumbnails. |

### 7.4 Dual-Sided Chip Layout

The chip columns are a terminal-layer feature — they float above the PTY output but sit beneath the physical bezel frame. They are not docked modules and they are not user-configurable in the same way as Bezel Slots (§8.1). They are Brain-driven, context-reactive overlays that repopulate automatically as the active mode and command state changes (§7.3, §7.7).

**Left Region — Context & Options:** Surfaces static or slowly changing context relevant to the current mode: Favorites, Pinned Paths, Directory Nav trees, File targets, Search Filters, and Application Model hooks. This region changes in response to mode switches (§7.7) but remains relatively stable within a session. Can be toggled off via a bezel control.

**Right Region — Priority Stack & Actions:** Highly dynamic and predictive. Driven by the Priority Indicator engine, this region surfaces Command Completions, AI-suggested commands, Actionable alerts, and Process kill-switches. Content updates continuously as the user types or as system state changes. The Autocomplete Overlay (§7.6) is an expanded presentation of this region.

**Interaction model:** Tapping a Left Region chip stages or populates the command prompt with the chip's value. Tapping a Right Region chip appends its action or argument at the cursor position, or replaces the current token. Chips for flags that accept arguments expand into secondary chips listing possible values.

#### 7.4.1 Viewport Layer Model

The Level 2 viewport is composed of three spatial layers, stacked front-to-back:

| Layer | Contents | Who Controls It |
|---|---|---|
| **Bezel Frame** (front) | Bezel Slot modules (§8.1): Mini-Map, Telemetry, Priority Indicator, Navigation, etc. | User-configured via Settings or direct manipulation |
| **Chip Layer** (middle) | Dual-Sided Chip Columns (§7.4) and Autocomplete Overlay (§7.6) | Brain — reactive to context and mode state (§7.3, §7.7) |
| **Terminal Canvas** (back) | PTY output rendered by the active Terminal Output Module (§7.2) | Module system |

The Chip Layer floats above the Terminal Canvas and below the Bezel Frame. Context switches (§7.7) operate exclusively on the Chip Layer — Bezel Slot contents are unaffected by mode or command changes. The Terminal Foreground Toggle (§7.8) temporarily brings the Terminal Canvas to the front, occluding the Chip Layer without disturbing the Bezel Frame.

### 7.5 Secondary Select on Chips (Context Menus)

Long-press (>500ms) or right-click on a chip summons a glassmorphism context menu:

#### 7.5.1 File & Directory Chips

- **[Inspect Path]:** Transition to Level 4 Detail View for metadata and cryptographic verification.
- **[Open With...]:** Select from compatible Application Models.
- **[Stage Action]:** Copy path to the active command prompt without submitting.
- **[Trust Tier...]:** Manually elevate or restrict the path's security context.
- **[Purge Nodes]:** Destructive deletion (requires confirmation per §17.2).

#### 7.5.2 Process & App Chips

- **[Tactical Signal...]:** Sub-menu to send `SIGINT`, `SIGTERM`, or `SIGKILL` to the PTY/Process.
- **[Renice Priority]:** Adjust process priority (LCARS levels 1–5).
- **[Inspect Buffer]:** Transition to Level 4 Buffer View.
- **[Isolate Process]:** Force the process into a more restrictive sandbox tier.
- **[Clone to Sector]:** Duplicate the process state in a new terminal sector.

### 7.6 Autocomplete Overlay

The Autocomplete Overlay is an expanded presentation of the Right Region chip column (§7.4), not a separate system. When the user is typing in CMD mode, the Right Region can expand into a full-height, glassmorphism-styled scrollable panel that unfurls upward from the right chip column toward the top bezel. It presents a comprehensive, ranked list of completions — flags, paths, known commands — each rendered as a chip with a short description.

Because it is an expansion of the Right Region, it occupies the same Chip Layer (§7.4.1) and is dismissed and rebuilt whenever a context switch occurs (§7.7). It is dismissed manually by tapping outside the overlay, pressing Escape, or executing a command.

### 7.7 Context-Aware Mode Switching

Certain shell commands signal an intent to change the active context. TOS can detect these and switch the Mode Selector (§7.1) accordingly. This behaviour is user-configurable per sector: **Off**, **Suggest**, or **Auto**.

| Command Family | Example Commands | Resulting Context |
|---|---|---|
| Filesystem | `ls`, `cd`, `cp`, `mv`, `find`, etc. | Directory |
| Process Management | `kill`, `ps`, `top`, `htop`, etc. | Activity |

**Off:** No automatic switching. Mode stays as-is.
**Suggest:** A chip appears in the Right Region offering the switch. The user taps to confirm.
**Auto:** The context switch fires immediately, accompanied by a mode-transition earcon (Visual Design §3) and a brief visual indicator on the Mode Selector.

**Effect on the Chip Layer:** A context switch — whether confirmed by the user in Suggest mode or automatic in Auto mode — immediately drives chip column repopulation as defined in §7.3. The Left Region reflects the new context's static options; the Right Region reflects its predictive action set. Any open Autocomplete Overlay (§7.6) is dismissed and rebuilt for the new context.

### 7.8 Terminal Foreground Toggle

A bezel control allows the user to temporarily bring the terminal output area to the foreground, overlaying chip regions and other dynamic elements. Affects only the current Command Hub.

### 7.9 Multitasking with Multiple Terminals

When using split viewports (§11), each viewport contains its own Terminal Output Module instance. The **Cinematic Triangular Module** supports a "pinwheel" arrangement where multiple triangular terminals are arranged radially around a central point. Users rotate the pinwheel to bring a different terminal into focus. This is module-specific behaviour.

---

## 8. Application Focus – Level 3

Applications occupy the full viewport, with the Tactical Bezel. A **System Output** button in the bezel can overlay the system console for quick monitoring.

### 8.1 Tactical Bezel (Logical Slot Architecture)

The Tactical Bezel is the persistent structural frame surrounding the entire viewport at every level. It is divided into four segments: Top, Bottom, Left Sidebar, and Right Sidebar. The Bottom Bezel houses the Persistent Unified Prompt (§7.1) and is strictly non-slottable. The remaining three segments expose five configurable slots where Bezel Component Modules can be docked.

**The Five Configurable Slots:**

| Slot ID | Segment | Orientation | Projection Direction |
|---|---|---|---|
| `Top_Left` | Top bezel, left section | Horizontal | Downward |
| `Top_Center` | Top bezel, center section | Horizontal | Downward |
| `Top_Right` | Top bezel, right section | Horizontal | Downward |
| `Left_Sidebar` | Left lateral bar | Vertical | Inward (rightward) |
| `Right_Sidebar` | Right lateral bar | Vertical | Inward (leftward) |

**What Can Occupy a Slot:**
Only **Bezel Component Modules** (`.tos-bezel` packages, §27.2) may be docked into a slot. This includes the built-in system components listed below and any compatible third-party modules installed via the Marketplace (Ecosystem §1.8). A slot holds one module at a time.

**What Cannot Occupy a Slot:**
- The Dual-Sided Chip Columns (§7.4) — these are Chip Layer overlays, not dockable modules.
- The Autocomplete Overlay (§7.6) — a transient expansion of the Chip Layer.
- The Persistent Unified Prompt — the Bottom Bezel is non-slottable (§7.1).
- Navigation Controls (OVERVIEW / COMMAND HUB / APP FOCUS) — fixed structural elements of the Left Sidebar, not user-repositionable.

**Slot Isolation:** Each docked module is an independent logic unit. It publishes its state to the Brain, which routes updates back to the slot's rendered view. Modules do not share state with each other or with the Chip Layer.

**Slot Compatibility — "Any":** Modules listed as compatible with **Any** slot may be docked into all five slot IDs. Placement is user-defined via Settings or direct manipulation.

**Navigation Controls (Left Sidebar — fixed, not slottable):**
- **OVERVIEW:** Zoom out to Level 1.
- **COMMAND HUB:** Navigate to Level 2.
- **APP FOCUS:** Zoom into Level 3.

**Built-in Bezel Component Modules:**

| Component | Default Slot | Compatible Slots | Description |
|---|---|---|---|
| **Tactical Mini-Map** | `Left_Sidebar` | `Left_Sidebar`, `Right_Sidebar` | Spatial sector overview; anchors trigger level transitions (Visual Design §2). |
| **Priority Indicator** | `Right_Sidebar` | `Left_Sidebar`, `Right_Sidebar` | Dynamically ranked system alerts and notification badges (Visual Design §1). |
| **Resource Telemetry** | `Top_Center` | `Top_Left`, `Top_Center`, `Top_Right` | Real-time CPU, Memory, Network, and PTY latency. |
| **Mini-Log Telemetry** | `Right_Sidebar` | `Left_Sidebar`, `Right_Sidebar` | Persistent readout of system state and last executed command. |
| **Active Viewport Title** | `Top_Left` | Any | Real-time text readout of current Level, Sector, or App context. |
| **Brain Connection Status** | `Top_Center` | Any | Connection state (Connecting/Connected/Disconnected/No Brain) and Brain time. See §3.4 for state definitions. |
| **System Status Badges** | `Top_Right` | Any | Quick-toggles for UI settings, sandboxes, and Terminal output overlay. |
| **Collaboration Hub** | — | Any | Multi-user avatars and follow-mode toggles. |
| **Media Controller** | — | Any | Global audio playback controls. |

### 8.2 Application Models

A module that customizes an application's integration at Level 3. Provides:
- Custom bezel actions.
- Zoom behaviour (e.g., internal app zoom).
- Legacy decoration policy (Suppress, Overlay, Native).
- Thumbnail for Activity Mode.
- Searchable content (for unified search).
- Opt-out from deep inspection.

### 8.3 Deep Inspection Access

An **Inspect** button in the expanded bezel zooms to Level 4 for the current application.

---

## 9. Deep Inspection & Recovery – Level 4

Level 4 provides all deep diagnostic, inspection, and recovery tools in a unified interface. Three sub-views serve distinct purposes: forensic inspection, raw data analysis, and emergency recovery.

### 9.1 Detail View

A modal overlay presenting structured metadata:
- **System Resources:** CPU, memory, uptime, network/disk I/O.
- **Event History:** Scrollable timeline of lifecycle events, commands, inspections (from TOS Log).
- **Configuration:** Environment variables, args, app settings.
- **Metadata:** Surface UUID, PID, parent, session ownership.
- **Security:** Permissions, sandbox status, audit excerpts.
- **Collaboration:** Active guests, recent guest actions.

Interactive elements (e.g., PID) can jump to Activity Mode or log searches. Export as JSON/plain text.

### 9.2 Buffer View

Hex dump viewer of the target surface's process memory (read-only). Features:
- Offset, hex, ASCII columns.
- Seek, search, export, refresh controls.
- Unavailable on Android; apps may opt out via manifest.
- **Disabled by default;** requires explicit privilege elevation (§9.5).

### 9.3 Tactical Reset (God Mode)

The Tactical Reset sub-view is the system's ultimate fallback and diagnostic layer — a low-overhead, wireframe visualization that bypasses standard sectoral rendering logic.

#### 9.3.1 Global Resource Diagnostics
- **Visualization:** Non-textured, high-contrast wireframe map showing all Brain sectors, services, and associated OS processes.
- **Resource Monitoring:** Real-time CPU, memory, and I/O pressure gauges for every active PID.
- **Emergency Management:** Integrated "Force Kill" capabilities that send `SIGKILL` directly via the Brain's root-tier services.
- **Recovery Logic:** Triggering a Tactical Reset flushes all transient diagnostic buffers and resets the Face-Brain IPC sync to a known stable state.

#### 9.3.2 Initiation
- **Manual Trigger:** Bezel "Tactical Reset" button or `Ctrl+Alt+Backspace`.
- **Safety Fallback:** Automatically triggered if the Face detects sustained latency >500ms or if the Brain reports a service-level deadlock.

#### 9.3.3 Security & Privilege Isolation
- **Read-Only by Default:** View is strictly read-only upon entry.
- **Explicit Elevation:** Destructive actions (Force Kill, Renice) require session-tier re-authentication.
- **Metadata Isolation:** Renderer only receives sanitized process metadata (PID, user, %CPU, %MEM). No process memory or application surfaces.
- **No Prompt Access:** The Persistent Unified Prompt is **locked/disabled** during Tactical Reset.
- **Expanded Bezel Disabled:** The Expanded Bezel Command Surface trigger is disabled during Tactical Reset.
- **Remote Constraint:** Guests are strictly prohibited from initiating or interacting with Tactical Reset. It is a **Host-Only** capability.

### 9.4 Sub-View Switching

Users navigate between Level 4 sub-views via bezel controls or keyboard shortcuts. An **Inspect** button enters Detail View; a **Buffer** button (visible only when deep inspection is enabled) enters Buffer View; the Tactical Reset trigger enters God Mode. Exiting any sub-view returns to Level 3.

### 9.5 Privilege Elevation & Platform Restrictions

- Buffer View is **disabled by default**.
- Enabling requires explicit elevation (`sudo tos enable-deep-inspection` or Polkit dialog on Linux; biometric prompt on Android for Detail View extended metadata; Buffer View generally unavailable on Android).
- When enabled, a 🔓 indicator appears in the bezel; clicking it disables deep inspection immediately.
- All enable/disable events and Buffer View accesses are audited.

| Platform | Detail View | Buffer View | Tactical Reset |
|----------|-------------|-------------|----------------|
| Linux Wayland | Full | With sudo/Polkit | Full |
| Android XR | Partial (no raw memory) | Not available | Limited (no Force Kill) |
| Android Phone | Limited metadata | Not available | Limited |

---

## 10. Sectors and the Tree Model

A **sector** is a self-contained workspace with its own identity, settings, and (if remote) connection. Internally it follows a tree:

### 10.1 Project Context & Shared Kanban Boards

Sectors can associate with a **project** to share a kanban board and collaborate with other sectors.

#### 10.1.1 Project Association

A sector optionally specifies a project context:

```json
{
  "id": "sector_laptop",
  "name": "dev-laptop",
  "project_context": {
    "project_path": "/home/user/projects/tos-desktop",
    "project_id": "tos-desktop-v0.5",
    "shared_kanban_board": true
  }
}
```

If `shared_kanban_board: true`:
- Multiple sectors can open the same project
- All sectors see the same kanban board (`.tos/kanban.tos-board`)
- Task state updates propagate across sectors in real-time
- Agents in different sectors can work on different tasks simultaneously

#### 10.1.2 Multi-Sector Synchronization

When multiple sectors reference the same project:

1. **File system watches** (`inotify`, FSEvents) detect changes to `.tos/kanban.tos-board`
2. **Brain broadcasts** task state changes via IPC to all connected sectors
3. **Face updates** kanban board view in real-time (cards move, agents show progress)

For remote sectors:
- Synchronization via TOS Remote Server protocol (§12)
- Conflicts resolved by "last write wins" or user prompt (configurable)

#### 10.1.3 Agent Isolation

Each agent operates in its own terminal context, even when multiple agents are active:

- **Isolated PTY**: Each agent's commands execute in a separate pseudo-terminal
- **Isolated output**: Terminal output is captured separately per agent
- **No contention**: Agents do not block each other (except for explicitly exclusive resources)

See §7.7 (Features) for concurrency details.

#### 10.1.4 Internal Tree Structure

```
SECTOR
├── Command Hub A
│   ├── Application 1
│   └── Application 2
├── Command Hub B (created via split)
│   └── Application 3
└── Command Hub C
    └── Application 4
```

- Each Command Hub has its own state (mode, history, environment).
- Applications are children of the hub that launched them.
- Splits create additional viewports that can contain a new hub or an existing hub.

---

## 11. Split Viewports & Pane Management

Split viewports allow a sector to display multiple viewports simultaneously, each with independent depth and content.

### 11.1 Philosophy

The split system follows three principles:

- **THE DISPLAY DECIDES ORIENTATION.** The first split direction is determined by the display's aspect ratio, not by the user having to choose. On a landscape display the first split is vertical (side by side). On a portrait display it is horizontal (top and bottom). The geometry makes the obvious decision automatically.
- **INFINITE BUT BOUNDED.** Splitting is recursive — any pane can be split again. But at the point where a pane can no longer contain meaningful content, splitting is blocked. The boundary is calculated, not arbitrary.
- **MANAGEMENT THROUGH THE BEZEL.** Pane management actions surface as chips when the user activates the Expanded Bezel Command Surface at Level 3, keeping the split view itself clean.

### 11.2 Supported Pane Content Types

| Content Type | `pane_type` | Description |
|:---|:---|:---|
| **Terminal (Command Hub)** | `terminal` | A full Command Hub instance — prompt, chip columns, terminal output module. Shares the sector's shell context. |
| **Editor** | `editor` | The TOS Editor surface — code/text viewer and editor with live AI context integration. See [Features Specification §6](./TOS_v0.1.0_Features.md). |
| **Workflow Manager** | `workflow` | The Kanban board and agent terminal orchestration interface. See [Features Specification §7](./TOS_v0.1.0_Features.md). |
| **Level 3 Application** | `app` | Any running graphical application in Application Focus. |

These can be combined freely. The default desktop layout is a vertical split with `terminal` on the left and `editor` on the right. On mobile, the default is a `tabs` layout with `terminal` as the first tab and `editor` as the second.

**Example hub layout with editor pane:**
```json
{
  "type": "splits",
  "panes": [
    { "id": "pane_1", "pane_type": "terminal", "weight": 0.55 },
    { "id": "pane_2", "pane_type": "editor", "weight": 0.45, "file": "/8TB/tos/src/brain/main.rs" }
  ]
}
```

### 11.3 Aspect-Ratio-Driven Split Orientation

#### 11.3.1 First Split Direction

| Display Aspect Ratio | First Split Orientation | Result |
|:---|:---|:---|
| Wider than tall (16:9, ultrawide) | **Vertical** — left / right | `[ A ][ B ]` |
| Taller than wide (9:16, portrait) | **Horizontal** — top / bottom | `[ A ] / [ B ]` |
| Square or near-square (±10%) | **Vertical** — left / right (default) | `[ A ][ B ]` |

#### 11.3.2 Subsequent Splits

When a pane is split again, the same aspect-ratio logic applies evaluated against the **pane's own dimensions**, not the full display. A tall narrow pane splits horizontally. A wide short pane splits vertically.

#### 11.3.3 Orientation Override

The user can override the auto-detected orientation for any individual split by holding `Shift` while triggering the split shortcut. This rotates the split 90 degrees from the auto-detected default for that operation only. Future splits continue to use auto-detection.

### 11.4 Creating Splits

| Action | Shortcut |
|:---|:---|
| Split focused pane (auto-orientation) | `Ctrl+\` |
| Split focused pane (orientation override) | `Shift+Ctrl+\` |
| Close focused pane | `Ctrl+W` |
| Move focus between panes | `Ctrl+Arrow` |
| Resize pane (move divider) | Drag divider |
| Equalize all pane weights | Double-click any divider |

### 11.5 Minimum Pane Size & Split Blocking

#### 11.5.1 Minimum Size Calculation

The minimum is the larger of two constraints:

**Constraint 1 — Ratio minimum:** No pane may be smaller than 1/6 of the total split axis. On a 1920px wide display, no pane may be narrower than 320px.

**Constraint 2 — Content-aware minimum:**

| Content Type | Minimum Width | Minimum Height |
|:---|:---|:---|
| Terminal (Command Hub) | 400px | 200px |
| Level 3 Application | 320px | 240px |

If splitting a pane would produce any child pane smaller than this value, the split is blocked.

#### 11.5.2 Blocked Split Feedback

When a split is blocked, `Ctrl+\` produces a brief amber flash on the pane border and an earcon indicating the action is unavailable. No error message.

### 11.6 Divider Behaviour

Dividers are freely draggable within the bounds set by minimum size constraints. No fixed snap points by default.

**Snap assist (optional):** Dividers softly snap to 50% if released within 5% of center. Disable in **Settings → Interface → Split Viewport → Divider Snap**.

**Divider appearance:** Thin LCARS-styled lines. On hover or touch, they thicken to indicate draggability. The focused pane has an amber border; adjacent dividers are slightly brighter.

### 11.7 Pane Focus & Input Routing

Only one pane is focused at a time. The focused pane:
- Receives all keyboard input
- Has its amber border active
- Is the target of all Expanded Bezel prompt commands

Unfocused panes remain fully live — applications keep running, terminals keep streaming output — but do not receive keyboard input.

### 11.8 Pane Management via Expanded Bezel

When the user activates the expanded bezel at Level 3 with a split layout active, pane management chips appear:

```
┌──────────────────────────────────────────────────────────────────┐
│  [⛶ Fullscreen]  [⇄ Swap]  [⊞ Detach →Sector]  [💾 Save Layout] │
└──────────────────────────────────────────────────────────────────┘
```

These chips operate on the **focused pane** at the time the bezel was opened.

#### 11.8.1 Fullscreen — Promote Without Closing

**[⛶ Fullscreen]** expands the focused pane to fill the full viewport temporarily. The split layout is preserved in memory — other panes continue running in the background. A persistent **[⊞ Return to Split]** chip appears in the Top Bezel to return at any time. `active_level` remains unchanged; this is a temporary visual promotion, not a level change.

#### 11.8.2 Swap Pane Positions

**[⇄ Swap]** swaps the focused pane's position with an adjacent pane. If multiple adjacent panes exist, secondary chips appear for each neighbour. Swap is purely visual — it changes rendered positions without affecting shell contexts, processes, or histories.

#### 11.8.3 Detach to Sector

**[⊞ Detach →Sector]** removes the focused pane from the split and promotes it into a new independent sector at Level 1. Two options:

```
[📦 Bring Context]   [✦ Fresh Start]
```

**Bring Context:** The pane's entire terminal output area, PTY, process tree, shell history, cwd, and backgrounded processes are moved to the new sector. The Brain re-parents the PTY and its process group from the source sector's hub to the new sector's hub.

**Fresh Start:** The pane's running process is detached from its terminal and re-attached as a **background process** in the new sector. The process keeps running — not killed — but is no longer connected to an interactive terminal. The new sector opens with a clean shell in the same `cwd`. A background job chip appears in the new sector's right chip column.

#### 11.8.4 Save Layout as Template

**[💾 Save Layout]** serialises the current split configuration — pane count, orientations, weights, content types — as a named sector template (`.tos-template`). The user is prompted for a name. The template saves layout geometry and content types but not content state; it is a structural blueprint, not a snapshot.

### 11.9 Split State Persistence

Split layouts are part of the sector's `hub_layout` object and are persisted to the session file as specced in the Session Persistence Specification (see [Features Specification §2](./TOS_v0.1.0_Features.md)). On restore:
- All panes are recreated with their saved weights and orientations.
- Terminal panes have their scrollback histories loaded before the shell spawns.
- Level 3 application panes attempt to relaunch the application in the same position. If the application is no longer running, the pane renders a **[Relaunch]** chip.

### 11.10 Relationship with Terminal Output Modules

Architecture §7.9 notes the Cinematic Triangular Module supports a "pinwheel" arrangement. That capability is preserved and complementary. The pinwheel is a **module-defined layout** (`layout_type: module_defined`) within a single pane — it is not a split viewport. A split viewport can contain a pane running the Cinematic Triangular Module in pinwheel mode alongside another pane running a Level 3 application. The two systems compose without conflict.

### 11.11 IPC Contracts

| Message | Effect |
|:---|:---|
| `split_create:<orientation>` | Creates a new split in the focused pane (`vertical` / `horizontal` / `auto`) |
| `split_close:<pane_id>` | Closes a pane and rebalances remaining panes |
| `split_focus:<pane_id>` | Moves focus to the specified pane |
| `split_focus_direction:<dir>` | Moves focus directionally (`left` / `right` / `up` / `down`) |
| `split_resize:<pane_id>:<weight>` | Sets a pane's weight (0.0–1.0, sibling weights rebalance) |
| `split_equalize` | Sets all sibling panes to equal weights |
| `split_fullscreen:<pane_id>` | Promotes a pane to fullscreen (layout preserved) |
| `split_fullscreen_exit` | Returns from fullscreen to split layout |
| `split_swap:<pane_id_a>:<pane_id_b>` | Swaps two panes' positions |
| `split_detach:<pane_id>:context` | Detaches pane to new sector, bringing context |
| `split_detach:<pane_id>:fresh` | Detaches pane to new sector as fresh start |
| `split_save_template:<n>` | Saves current layout as a named sector template |

---

## 12. Remote Sectors

Remote sectors are enabled by the **TOS Remote Server** daemon on the target machine.

### 12.1 TOS Remote Server Protocol

- **Handshake & Auth:**
  1. Client connects via TLS.
  2. Server sends `auth_challenge:salt;nonce`.
  3. Client responds with `auth_response:token` (HMAC/SSH-Signature).
  4. Server sends `session_init:capabilities_json`.
- **Control Channel (WebSocket):** Sends/receives IPC messages (prefixed with `remote:`).
- **WebRTC Signalling:** `webrtc_offer`, `webrtc_answer`, `webrtc_ice_candidate` with standard SDP/ICE payloads.
- **Video/Audio Stream:** WebRTC (H.264/H.265) with hardware decoding.
- **File Transfer:** WebDAV/HTTPS or custom protocol.
- **Authentication:** SSH keys, passwords, time-limited tokens (Android Keystore for credential storage).

**Capabilities:**
- Full sector tree synchronisation if remote runs TOS.
- For non-TOS machines: virtual sector with filesystem, processes, terminal.
- Fallback to SSH/HTTP if server not installed.

### 12.2 Web Portal & Live Feed Testing

Any sector or viewport can be exported as a unique URL accessible via any modern browser. Optional password or tactile approval. Supports multiple viewers, recording, and replay. Security:
- **One-Time URLs:** `https://tos.live/sector/<sector_uuid>?token=<short_lived_jwe>`
- **Expiry:** Tokens expire after 30 minutes of inactivity or manual session termination.
- **MFA:** Can require biometrics or tactile confirmation before a portal guest can upgrade to Operator role.

---

## 13. Collaboration

Collaboration is **host-owned**: a sector resides on one host; guests connect via the host's TOS Remote Server.

### 13.1 Host-Owned Sharing Model

- Host invites guests via secure token or contact list.
- Guests see a synchronised view of the host's sector tree.
- By default, each guest controls their own viewports independently.

### 13.2 Roles & Permissions

| Role | Capabilities |
|------|-------------|
| **Viewer** | See content only. |
| **Commenter** | Type in prompt (commands execute in restricted shell or are ignored). |
| **Operator** | Full control (launch apps, execute any command). |
| **Co-owner** | Invite others, change roles. |

### 13.3 Visual Presence & Alerts

- Avatars in Global Overview, hub mode, and on app bezels.
- Coloured borders/cursors for each participant.
- Collaboration alerts (user join/leave, role change, hand raise) trigger visual, auditory, and haptic cues.

### 13.4 Following Mode & Chat

- Guests can follow another user's view (viewport synchronisation).
- Lightweight chat overlay with `/run` for command execution (subject to permissions).

### 13.5 AI in Collaboration

The AI assistant can summarize recent activity, translate commands/chat, suggest collaboration actions, explain guest intent, and mediate role changes. Guests are notified if AI processes their actions.

### 13.6 Privacy & Auditing

- Guest actions are recorded in the host's TOS Log (tagged with guest identity).
- Privacy notice shown upon joining.
- Critical events written to a non-disableable audit log.

### 13.7 Collaboration Data Channel Payloads

| Payload Type | Structure | Description |
|---|---|---|
| **Presence** | `{"user": "uuid", "status": "active|idle", "level": 2}` | Current location in hierarchy. |
| **Cursor Sync** | `{"user": "uuid", "x": 0.5, "y": 0.2, "target": "element_id"}` | Normalized coordinates (0.0 to 1.0). |
| **Following** | `{"follower": "u1", "leader": "u2", "sync": true}` | Viewport synchronization toggle. |
| **Role Change** | `{"target": "u1", "new_role": "operator", "admin": "u3"}` | Role escalation/de-escalation. |

---

## 14. Input Abstraction Layer

All physical input is normalized into **semantic events**, which are then mapped to TOS actions via a user-configurable layer. Logic MUST respond to `SemanticEvent`, never to physical input directly.

### 14.1 Semantic Event Categories

| Category | Events |
|----------|--------|
| Navigation | `zoom_in`, `zoom_out`, `next_element`, `next_viewport`, `home`, `command_hub` |
| Selection | `select`, `secondary_select`, `multi_select_toggle`, `drag_start`, `drop` |
| Mode Control | `cycle_mode`, `set_mode_command`, `set_mode_directory`, `set_mode_activity`, `set_mode_search`, `set_mode_ai`, `toggle_hidden_files` |
| Bezel Control | `toggle_bezel_expanded`, `split_view`, `close_viewport`, `inspect` |
| System | `open_hub`, `open_global_overview`, `tactical_reset_sector`, `tactical_reset_system`, `open_settings`, `toggle_minimap` |
| Text Input | `text_input`, `command_history_prev`, `autocomplete_request` |
| Voice | `voice_command_start`, `voice_transcription` |
| AI | `ai_submit`, `ai_stop`, `ai_suggestion_accept` |
| Collaboration | `show_cursor`, `follow_user`, `raise_hand`, `share_sector` |
| Stop | `stop_operation` |

### 14.2 Default Keyboard Shortcuts

| Key Combination | Semantic Event | Description |
|---|---|---|
| `Ctrl + [` | `zoom_out` | Move one level up in hierarchy. |
| `Ctrl + ]` | `zoom_in` | Move one level down into focus. |
| `Ctrl + Space` | `toggle_bezel` | Expand/Collapse the Top Bezel. |
| `Ctrl + /` | `set_mode_ai` | Focus prompt and switch to AI mode. |
| `Ctrl + T` | `new_sector` | Create a new sector. |
| `Alt + [1-9]` | `switch_sector` | Rapidly switch between first 9 sectors. |
| `Ctrl + M` | `toggle_minimap` | Show/Hide the Tactical Mini-Map. |
| `Ctrl + Alt + Backspace` | `tactical_reset` | Trigger immediate Tactical Reset (Level 4 God Mode). |
| `Ctrl + \` | `split_view` | Split focused pane (auto-orientation). |
| `Shift + Ctrl + \` | `split_view_override` | Split focused pane (orientation override). |

Users can remap all shortcuts via the Settings panel, which provides a visual conflict detection interface.

### 14.3 Voice Command Grammar

Voice input is processed context-sensitively. Commands are structured as `Action + Target + [Modifier]`.

| Command Pattern | Example | Logical Translation |
|---|---|---|
| "Focus [Sector]" | "Focus Development" | `sector_zoom:dev_uuid` |
| "Run [Command]" | "Run build script" | `prompt_submit:./build.sh` |
| "Inspect [Target]" | "Inspect browser" | `zoom_to:level_4;pid_1234` |
| "Alert Status" | "Report alert status" | TTS summary of priority chips. |
| "Stop everything" | "Stop everything" | `tactical_reset_system` |

### 14.4 Device Support & Mapping

Supported devices: keyboard, mouse/trackpad, touch, game controllers, VR/AR controllers, hand tracking, gaze/eye tracking, voice, accessibility switches.

**Android XR (OpenXR) Action Mapping:**
- `pinch_left`: Triggers `zoom_out`.
- `pinch_right`: Triggers `zoom_in`.
- `gaze_dwell`: Fires `select` semantic event.
- `wrist_tap`: Fires `open_hub` semantic event.

Multiple devices can be used simultaneously; the last active device determines cursor appearance.

### 14.5 Accessibility Integration

- Switch scanning (single/multi-switch).
- Sticky keys, slow keys.
- Dwell clicking (gaze/head tracking). Default dwell: 500ms (configurable).
- Voice commands for all actions.
- Haptic feedback as input confirmation.

---

## 15. Platform Abstraction & Rendering

The core TOS logic is platform-independent and interacts with the platform through three core traits.

### 15.1 Core Traits

```rust
pub trait Renderer {
    fn create_surface(&mut self, config: SurfaceConfig) -> SurfaceHandle;
    fn update_surface(&mut self, handle: SurfaceHandle, content: &dyn SurfaceContent);
    fn composite(&mut self);
}

pub trait InputSource {
    fn poll_events(&mut self) -> Vec<RawInputEvent>;
    fn map_to_semantic(&self, raw: RawInputEvent) -> Option<SemanticEvent>;
}

pub trait SystemServices {
    fn spawn_process(&self, cmd: &str, args: &[&str]) -> Result<ProcessHandle>;
    fn read_dir(&self, path: &Path) -> Result<Vec<DirEntry>>;
    fn get_system_metrics(&self) -> SystemMetrics;
    fn open_url(&self, url: &str);
}
```

### 15.2 Linux Wayland Implementation

- **Layer Shell:** The Face renders as a `wlr-layer-shell` on the `TOP` layer, ensuring it remains above all native applications unless explicitly toggled.
- **Surface Embedding:** Native Wayland windows are rendered into Level 3 viewports using `dmabuf` sharing. The Face acts as a sub-compositor, projecting application buffers onto the LCARS-themed surface.
- **Input Forwarding:** The Face intercepts all pointer/touch events. If an event occurs within a native application's bounds, the Face translates coordinates and forwards the raw event to the application's `wl_surface`.
- **Communication:** Uses standard `wl_shm` or `dmabuf` for zero-copy texture transfer.
- **Connection Fallback:** If the Wayland compositor is unavailable (e.g., SSH session), the RendererManager falls back to HeadlessRenderer (§15.7) automatically.

### 15.3 Android XR (OpenXR) Implementation

- **World Space Compositing:** The UI is a set of three-dimensional cylinders and quads positioned in a "Cockpit" configuration around the user — not a 2D overlay.
- **Performance:** Uses `EGLImage` for high-throughput terminal rendering to avoid CPU pipeline stalls.

### 15.4 Android Phone Implementation

- **Platform:** Native Android activity or Compose view host.
- **Input & Accessibility:** Managed by phone-tier Input Hub and platform services.

### 15.5 Native Application Embedding (Wayland/X11)

To embed native apps into Level 3 focus:
1. **Virtual Output:** TOS provides a virtual `wl_output`.
2. **Composition:** Application textures are mapped to logical areas and composited into the Level 3 texture with a glassmorphism border.
3. **Bezel Overlay:** The Tactical Bezel is rendered on top of the native app, providing system-level "Close" and "Inspect" triggers via `xdg_toplevel` signals.
4. **Event Routing:** Input is captured by the Face, translated, and routed via the Brain to the native PID.

### 15.6 Renderer Mode Detection & Fallback

The Brain implements automatic detection of the rendering environment and selects an appropriate Renderer implementation at runtime. This ensures TOS can operate in any context: local Wayland, headless (SSH), or remote streaming.

**Mode Selection (Priority Order):**
1. **Explicit Flag:** `TOS_HEADLESS=1` environment variable or `--headless` CLI flag.
2. **Wayland Detection:** Check `WAYLAND_DISPLAY` and verify compositor connectivity.
3. **Remote Fallback:** Default to streaming buffers to a remote Face via WebRTC.

**Renderer Implementations:**
- `WaylandRenderer`: Local Wayland compositor (Architecture §15.2).
- `HeadlessRenderer`: CPU-based buffers, no GPU (for SSH/headless/testing).
- `RemoteRenderer`: Stream buffers to remote Face (Architecture §12).

**Key Principle:** Brain initialization must never block or panic due to missing hardware.

### 15.7 Native Horizon OS Client (Meta Quest)

A dedicated Android application connecting to a remote TOS instance via the TOS Remote Server protocol:
- **Connection Manager:** WebSocket/TLS control channel.
- **Rendering Engine:** WebRTC video decoded via `MediaCodec`, displayed as OpenXR texture.
- **Input Processor:** Maps Quest inputs (trigger, grip, hand tracking) to TOS semantic events.
- **File Sync Service:** Bidirectional sync with remote host (WebDAV), device-aware.
- **Collaboration Module:** Full guest participation with avatars, following mode, alerts.
- **Local UI Overlay:** Connection status, sync progress, native menus composited on video.

---

## 16. Performance and Compositing

### 16.1 Depth-Based Rendering & Caching

- Only focused level receives full frame rate; background levels are static textures or throttled.
- Texture caching for thumbnails; GPU memory pruning for surfaces more than two levels away.
- Hardware acceleration (OpenGL ES / Vulkan).

### 16.2 Development & QA Architecture

- **Headless Brain Integration Testing:** The Brain supports a "Headless Mode" where a test harness acts as a virtual Face, exercising the `tos-protocol` IPC without a graphical environment.
- **Unified Visual Token System:** All UI aesthetics (colors, blurs, typography) are defined in a central JSON/TOML configuration, consumed by both the Web CSS and Native Vulkan/GLES shaders for pixel-perfect consistency across platforms.

### 16.3 Intelligent View Synchronisation

To prevent flicker during high-frequency updates:
- HTML diffing — skip DOM update if payload identical.
- Animation suppression on core structural elements.
- State preservation (input fields, scroll positions) across refreshes.
- Throttled backgrounds (1–5 Hz) for non-focused viewports.

### 16.4 Tactical Alert (Performance Warning)

If frame rate drops below target (60 FPS desktop, 90 FPS VR) for >2s, a non-intrusive alert appears showing current FPS and suggestions.

---

## 17. Security Model

### 17.1 Authentication & Authorization

- Local: PAM (Linux), Android Keystore + biometric (Android).
- Remote: SSH keys, passwords, time-limited tokens; mutually authenticated TLS.
- RBAC roles (Viewer, Commenter, Operator, Co-owner) enforced host-side.

### 17.2 Dangerous Command Handling — Command Trust & Confirmation System

TOS does not decide what is dangerous. The user does. The trust system ensures that a user who has not yet expressed an opinion about a command class gets a moment to notice what they are about to do — nothing more. Once they have expressed an opinion, the system remembers it and never asks again.

Two principles:
- **INFORM, DON'T BLOCK.** Warning chips are non-blocking. The user can proceed immediately. The chip is information, not a gate.
- **EXPLICIT PROMOTION.** Trust is never earned automatically by use count or time. The user explicitly decides when a command class is trusted. The system never makes that decision on their behalf.

#### 17.2.1 Trust Configuration

**First-Run Trust Setup:** During the onboarding flow (see [Features Specification §3](./TOS_v0.1.0_Features.md)), the user is presented with a trust configuration screen for each command class. No defaults are pre-selected. They can skip to defer all classes to WARN.

**Trust Levels:**

| Level | Behaviour |
|:---|:---|
| **WARN** | A non-blocking warning chip appears in the prompt area when the command is staged. The user can proceed immediately or dismiss. |
| **TRUST** | The command runs without any chip or interruption. No UI. No friction. |

There are no intermediate levels. No countdowns. No sliders. No re-authentication.

#### 17.2.2 Command Classes

**Explicit Classes:**

| Class ID | Covers | Default at First-Run Setup |
|:---|:---|:---|
| `privilege_escalation` | `sudo`, `su`, `doas`, `pkexec` | User choice (no pre-selection) |
| `recursive_bulk` | Any command operating on 10+ files, or using `-r`/`-R`/`--recursive` flags with a destructive verb | User choice |

**Implicit Bulk Detection:** Commands not in an explicit class but detected as operating on a large file set at execution time. When the Brain's PTY analysis estimates a command will affect 10 or more filesystem objects, it is temporarily treated as `recursive_bulk` for that invocation only. This fires per-command, not as a persistent class assignment.

#### 17.2.3 The Warning Chip

When a command in a **WARN** class is staged in the prompt, a warning chip appears in the right chip column immediately above the prompt:

```
┌──────────────────────────────────────────────────┐
│  ⚠  sudo apt remove nginx          [Trust Class] │
│     Privilege escalation — runs as root           │
└──────────────────────────────────────────────────┘
```

- **⚠ Icon** — amber warning glyph.
- **Command echo** — repeats the staged command.
- **Class label** — human-readable description of why this chip appeared.
- **[Trust Class]** — a single tap permanently promotes this class to TRUST. The chip disappears immediately.
- The chip dismisses automatically when the prompt is cleared or a different command is staged.
- The chip does not prevent Enter from running the command. No delay, countdown, or required interaction.

For implicitly detected bulk operations, the chip shows an estimated file count with a `~` prefix.

#### 17.2.4 Promoting and Demoting Trust

**Promoting to TRUST:**
- **Inline:** Tap **[Trust Class]** on a warning chip. Immediate, no confirmation.
- **Settings panel:** **Settings → Security → Trust** — toggle any class to TRUST globally.

**Per-Sector Trust Override:**
The global config applies system-wide by default. Individual sectors can override any class via **Sector Settings → Trust**. Resolution order: Sector Override → Global Config.

**Demoting to WARN:**
Via **Settings → Security → Trust** or sector settings. Takes effect immediately. Trust does not expire; only the user removes it.

#### 17.2.5 Brain Implementation

The Brain's command dispatcher runs a classification pass on every staged command before PTY submission:

- **Stage 1 — Explicit class matching:** Checks the command verb against the class registry.
- **Stage 2 — Implicit bulk detection:** For commands not matched in Stage 1, estimates filesystem objects affected by expanding globs and counting directory entries.

The command is never held. In the WARN path, the chip is emitted and the prompt remains live. The user presses Enter; the command runs.

#### 17.2.6 IPC Contracts — Trust System

| Message | Effect |
|:---|:---|
| `trust_promote:<class_id>` | Promotes a command class to TRUST globally |
| `trust_demote:<class_id>` | Returns a command class to WARN globally |
| `trust_promote_sector:<sector_id>:<class_id>` | Sector-level TRUST override |
| `trust_demote_sector:<sector_id>:<class_id>` | Sector-level WARN override |
| `trust_clear_sector:<sector_id>:<class_id>` | Removes sector override, falls back to global |
| `trust_get_config` | Returns full trust config for all classes |

#### 17.2.7 Voice Confirmation (Preserved from §17.3)

For voice-submitted commands that match a WARN class, the voice confirmation path applies: speaking a unique, time-limited passphrase displayed on screen. This applies to prompt-submitted commands only; the chip system covers manual prompt use.

### 17.3 Process Isolation & Sandboxing

- **Applications:** Run as the user's own processes. Optional sandboxing via Containerization/Flatpak/Firejail/bubblewrap/appimage (Linux) or Android platform sandbox.
- **Standard Modules:** Run in strictly sandboxed processes with declared permissions.
- **Trusted System Modules:** Shell Modules and certain Sector Types run with the user's full shell privileges and have access to the PTY.

**Sandbox Profiles (Linux/Bubblewrap):**
- **Default:** `--unshare-all --dir /tmp --ro-bind /usr /usr --ro-bind /lib /lib --proc /proc`
- **Network Profile:** Adds `--share-net`.
- **FileSystem Profile:** Adds `--bind ~/TOS/Sectors/<id> /mnt/sector`.

**Exhaustive Permission List:**

| Permission | Description | Enforcement |
|---|---|---|
| `network:client` | Initiate outgoing connections. | `unshare-net` (off) |
| `fs:read` | Read sector-specific directories. | `ro-bind` |
| `fs:write` | Write to sector-specific directories. | `bind` (rw) |
| `sys:camera` | Access to video input (XR passthrough). | `/dev/video*` bind |
| `ai:stream` | Send telemetry to AI Backend. | Internal IPC gate |

### 17.4 Deep Inspection Privilege

Buffer View is disabled by default. Requires explicit elevation. Audited. See §9.5.

### 17.5 Auditing

All commands, security events, role changes, and deep inspection accesses are logged. Critical events go to a non-disableable audit log (Linux: `/var/log/tos/audit.log`, Android: app-private).

**Audit Log Schema (JSON Lines):**

```json
{
  "ts": 1709300000,
  "actor": "user|guest_id|ai",
  "action": "privilege_escalation|sector_close|exec",
  "target": "res_id",
  "status": "allowed|denied",
  "details": "optional context"
}
```

---

## 18. Modules

TOS implements a robust, modular plugin architecture using platform-specific dynamic libraries distributed via the Marketplace ecosystem.

**All detailed specifications regarding module manifests (`module.toml`), sandboxing rules, terminal UI injection, and module profiles (Shells, Themes, AI Backends, AI Skills, Bezel Components, Language Modules) are documented in the [Ecosystem Specification](./TOS_v0.1.0_Ecosystem.md).**

### 18.1 Module Type Summary

| Assistant | `.tos-assistant` | LLM backend provider; model discovery. |
| Curator | `.tos-curator` | MCP‑based context/data source. |
| Agent | `.tos-agent` | Prompt‑based persona & strategy stack. |
| Application Model | `.tos-appmodel` | Level 3 application integration |

---

## 19. TOS Log

Every surface maintains its own event history, collectively forming a system-wide timeline.

### 19.1 Recorded Events & Unified Storage

The Global TOS Log Sector provides a unified view transparently merging:
- **User Logs:** `~/.local/share/tos/logs/` (Standard activity).
- **System Audit Logs:** `/var/log/tos/` (Privileged events, filtered by user visibility).
- **Remote Logs:** Captured from the TOS Remote Server and cached locally during the session.

| Event Type | Examples |
|---|---|
| Lifecycle | Creation, focus, move, close |
| Commands | Executed commands with exit status, duration |
| Inspections | Level 4 views accessed |
| Telemetry | Periodic resource snapshots (if enabled) |
| Collaboration | User joins/leaves, role changes, guest actions |
| System Events | Notifications, alerts, security events |
| Priority Changes | Score changes and contributing factors |
| AI Interactions | Queries and responses (if enabled) |

### 19.2 Access Methods

- **Per-Surface (Level 4):** Scrollable timeline in Detail View.
- **Global TOS Log Sector:** A dedicated Sector/Command Hub (Level 2) providing full interactive filtering, searching, and exporting.
- **Prompt Queries:** Commands like `log --surface browser --since 10min`.

### 19.3 OpenSearch Compatibility

- OpenSearch description document for browser address bar queries.
- Optional forwarding to OpenSearch cluster (user consent required).

### 19.4 Privacy & Retention

- Master toggle to enable/disable logging (except critical audit events).
- Per-surface opt-out, retention policies, regex-based redaction.
- Logs stored locally in `~/.local/share/tos/logs/` (JSON Lines or SQLite).

---

## 20. Reset Operations

### 20.1 Sector Reset

- **Trigger:** `Super+Backspace`, `tos sector reset`, bezel button, voice.
- Sends SIGTERM to all processes in current sector, closes splits, returns to fresh Level 2.
- Optional undo button (5s) if enabled.

### 20.2 System Reset

- **Trigger:** `Super+Alt+Backspace`, `tos system reset`, bezel button (Level 1).
- Dialog with three options: Restart Compositor, Log Out, Cancel.
- Requires confirmation + countdown. All attempts are audited.

---

## 21. Visual Design, Sensory Interface, and Accessibility

*The specifications for Priority-Weighted Visual Indicators, the Tactical Mini-Map, Auditory/Haptic interfaces, and Accessibility have been consolidated and moved to the [TOS Visual Design Specification](./TOS_v0.1.0_Visual_Design.md).*

---

## 22. Sector Templates and Marketplace

Schema for `.tos-template` distribution, AI-assisted discoverability, and the Marketplace package daemon are documented in the [Ecosystem Specification §2](./TOS_v0.1.0_Ecosystem.md).

---

## 23. Settings Data Model & IPC

### 23.1 Layered Settings

Settings are resolved via cascade: **per-application > per-sector > global key-value bag > global scalar field defaults**.

### 23.2 Canonical Keys & Defaults

| Key | Type | Default | Description |
|---|---|---|---|
| `theme` | string | `"lcars-light"` | Active theme module ID. |
| `default_shell` | string | `"fish"` | Default shell module ID for new sectors. |
| `terminal_output_module` | string | `"rectangular"` | Active terminal output module ID. |
| `master_volume` | `"0"`–`"100"` | `"80"` | Master audio volume. |
| `logging_enabled` | `"true"`\|`"false"` | `"true"` | Master log toggle. |
| `deep_inspection` | `"true"`\|`"false"` | `"false"` | Enable Buffer View. |
| `terminal_buffer_limit` | integer | `500` | Maximum lines to keep in terminal buffer (FIFO). |

### 23.3 IPC Messages for Settings

- `open_settings`, `close_settings`
- `set_fps:<value>`, `set_master_volume:<value>`
- `set_theme:<module_id>` — Switch theme.
- `set_default_shell:<module_id>` — Set default shell.
- `set_terminal_output_module:<module_id>` — Set terminal output module.
- `toggle_sandboxing`
- `enable-deep-inspection`, `disable-deep-inspection`
- `set_setting:<key>;<value>` (standardized with semicolon)
- `set_sector_setting:<key>;<value>`
- `set_terminal_buffer_limit:<value>` — Adjust terminal history cap.
- `settings_tab:<tab>` (for modal navigation)

### 23.4 Persistence

Settings saved to `~/.config/tos/settings.json` (Linux) or app-private storage (Android) as JSON. Debounced writes (≤1s). Only canonical keys and extensions are persisted; runtime-only state is skipped.

---

## 24. Shell API Enhancements

### 24.1 `command_result` Payload Format

OSC `9002` payload extended to three semicolon-delimited fields: `<command>;<exit_status>;<base64(stdout+stderr)>`. Base64 encoding prevents control characters from breaking OSC parsing. Third field optional for backwards compatibility.

### 24.2 Shell Integration Script Requirements

- Capture full combined stdout/stderr of each command.
- Base64-encode and emit `ESC]9002;<command>;<exit_status>;<base64>BEL`.
- Also emit `ESC]9003;<cwd>BEL` on directory change.
- Do not capture TOS-internal commands (`EXEC`, `CD`, etc.).

### 24.3 Fallback: Raw PTY Output & Filesystem Fallback

- Without integration, PTY reader strips ANSI, splits lines, caps at 500 lines (default; user-adjustable).
- **Local Directory Mode:** Falls back to `std::fs::read_dir` if `hub.shell_listing` is `None`.
- **Remote Directory Mode:** If `hub.shell_listing` is missing, the Brain attempts to fetch listing data via the TOS Remote Server's File Service (§12.1).
  - **Graceful Fallback:** If the remote server is not installed (e.g., raw SSH link), Directory Mode visuals are disabled and the interface stays in standard shell output.
  - **Connection Loss:** If an active remote server connection is lost, the sector displays a "Remote session disconnected" banner and closes after 5 seconds of inactivity.

### 24.4 Line-Level Priority Metadata

The shell can emit a priority sequence before producing a line of output:

```
ESC]9012;<level>BEL
```

**Levels:** `0` Normal / Inherit, `1` Low, `2` Notice, `3` Warning, `4` Critical, `5` Urgent (Tactile/Auditory trigger).

The Brain applies this level to all subsequent lines until a new 9012 sequence is received or the command completes.

### 24.5 Command Auto-Detection (`ls`, `cd`)

- If submitted command starts with `ls` (case-insensitive), resolve target path, set `hub.current_directory`, switch to Directory Mode, clear stale listing.
- If starts with `cd`, resolve target path, set `hub.current_directory` (if exists), do not change mode.
- No false positives (`rls`, `echo cd`).

### 24.6 Directory Mode Pick Behaviour

| Prompt state | Item type | Click action |
|---|---|---|
| Empty | File | Insert absolute path into prompt |
| Empty | Folder | Navigate into folder |
| Command staged | File | Append absolute path as next argument |
| Command staged | Folder | Append absolute path as next argument |

**Staging banner** appears above file grid when command staged, with current command and hint. Multi-select appends all selected paths in order.

IPC messages: `dir_pick_file:<n>`, `dir_pick_dir:<n>`, `dir_navigate:<path>`.

---

## 25. Bezel IPC Contracts

### 25.1 Action-Identifier Rule

All IPC messages from bezel buttons and UI controls must use **action identifiers**, not display labels. Labels are for rendering only and must not be forwarded to the shell.

✅ Correct: `<button onclick="window.ipc.postMessage('zoom_out')">ZOOM OUT</button>`
❌ Incorrect: `<button onclick="window.ipc.postMessage(this.innerText)">ZOOM OUT</button>`

**Prompt Interception Layer:** The `prompt_submit:` message is an exception. The Brain performs a "sniffing" pass on the submitted string to detect `ls` or `cd` and trigger mode switches. This logic lives entirely in the Brain's command dispatcher.

### 25.2 Reserved IPC Prefixes

| Prefix | Purpose | Payload Delimiter |
|---|---|---|
| `prompt_submit:` | Submit prompt value to PTY | N/A |
| `prompt_input:` | Update staged prompt text | N/A |
| `stage_command:` | Pre-populate prompt | N/A |
| `set_mode:` | Switch hub mode | N/A |
| `set_theme:` | Switch active theme | N/A |
| `set_default_shell:` | Set system default shell | N/A |
| `set_terminal_module:` | Switch terminal output module | N/A |
| `dir_navigate:` | Navigate directory | N/A |
| `dir_pick_file:` | Append file path | N/A |
| `dir_pick_dir:` | Append directory path | N/A |
| `dir_toggle_select:` | Toggle file selection | N/A |
| `dir_toggle_hidden` | Toggle hidden files | N/A |
| `dir_context:` | Open context menu (target;x;y) | Semicolon (`;`) |
| `dir_clear_select` | Deselect all files | N/A |
| `dir_action_copy`, `dir_action_paste`, `dir_action_delete` | Batch file operations | N/A |
| `app_toggle_select:` | Toggle app in Activity mode | N/A |
| `app_batch_kill`, `app_batch_signal:` | Batch process management | Semicolon (`;`) |
| `zoom_in`, `zoom_out`, `zoom_to:` | Zoom actions | N/A |
| `sector_close:`, `sector_freeze:`, `sector_unfreeze:`, `sector_clone:` | Sector management | N/A |
| `play_audio:` | Trigger specific earcon | N/A |
| `marketplace_search:` | Query marketplace | N/A |
| `marketplace_install:` | Install module | N/A |
| `set_terminal_buffer_limit:` | Adjust terminal history cap | N/A |
| `bezel_expand` | Open Expanded Bezel Command Surface | N/A |
| `bezel_collapse` | Collapse expanded bezel | N/A |
| `bezel_output_action:<action>` | Post-output action chip (`hub`, `split`, `dismiss`, `keep`) | N/A |
| `split_create:` | Create a new split pane | N/A |
| `split_close:` | Close a pane | N/A |
| `trust_promote:` | Promote a command class to TRUST | N/A |
| `trust_demote:` | Return a command class to WARN | N/A |

Unknown messages are logged and ignored (not forwarded to PTY).

> **Note:** The deprecated `update_confirmation_progress:` IPC message (tactile confirmation slider) has been removed. All dangerous command handling uses the trust chip system described in §17.2.

---

## 26. Terminal Output Rendering

### 26.1 ANSI Stripping

Before storage in `hub.terminal_output`, strip:
- CSI sequences (`ESC[ ... m`)
- OSC sequences (`ESC] ... BEL` or `ESC\`)
- C0/C1 controls except TAB, LF, CR.

Result must be valid printable UTF-8.

### 26.2 Buffer Limits & Rendering Requirements

- Cap at 500 lines (FIFO) by default; user-adjustable via `terminal_buffer_limit` setting.
- Monospace font, `white-space: pre-wrap`.
- Auto-scroll to latest line.
- Distinct styling for command echo (`> command`) vs output.

---

## 27. UI Module Interaction APIs

Terminal and Bezel modules interact with the Face via these specific UI-hooks:

### 27.1 Terminal Output API

- **`render(surface, lines)`:** The Face provides a `RenderSurface` (DOM or GPU buffer). The module handles font-rendering and ANSI color application.
- **`on_click(x, y)`:** Returns the line index and context-action (e.g., `copy`, `inspect_pid`).
- **`on_scroll(delta)`:** Handles the visual transition of lines.

### 27.2 Bezel Component API

- **`update_view(html, data)`:** Components push their rendered state to the Face.
- **`component_click(id, x, y)`:** The Face forwards clicks on specific component IDs to the underlying module.
- **`request_projection(mode)`:** Components can request to "unfurl" a detailed panel (e.g., the Mini-Map expanding into the viewport).

### 27.3 Editor Pane API (Brain → Face)

| Message | Payload | Effect |
|:---|:---|:---|
| `editor_open` | `path;line` | Opens file in Viewer Mode at specified line |
| `editor_open_ai` | `path;line;context_id` | Opens file with AI annotation from context_id |
| `editor_diff` | `path;proposed_content_id` | Opens Diff Mode with proposed AI edit |
| `editor_annotate` | `path;line;severity;message;context_id` | Adds annotation chip to editor margin |
| `editor_clear_annotations` | `path` | Removes all annotations from a file |
| `editor_scroll` | `path;line` | Scrolls editor to line (semantic scroll sync) |
| `editor_edit_proposal` | `proposal_json` | Triggers Diff Mode with proposed changes |

### 27.4 Editor Pane API (Face → Brain)

| Message | Payload | Effect |
|:---|:---|:---|
| `editor_activate` | `pane_id` | Switches pane to Editor Mode |
| `editor_save` | `pane_id` | Saves current buffer to file |
| `editor_save_as` | `pane_id;path` | Saves buffer to new path |
| `editor_context_update` | `pane_id;context_json` | Sends updated editor context to Brain on scroll/cursor move |
| `editor_send_context` | `pane_id;scope` | Explicitly sends file content to AI at given scope |
| `editor_edit_apply` | `proposal_id` | Applies the pending edit proposal |
| `editor_edit_reject` | `proposal_id` | Rejects the pending edit proposal |
| `editor_promote` | `pane_id` | Promotes editor pane to Level 3 |
| `editor_mode_switch` | `pane_id;mode` | Switches between `viewer`, `editor`, `diff` |

### 27.5 AI Context Sync API

These messages support cross-device AI context roaming (see §12 Remote Sectors and Features §6).

| Message | Direction | Effect |
|:---|:---|:---|
| `ai_context_sync:<sector_id>` | Face → Brain | Requests full AI context for a sector on remote Face connect |
| `ai_context_delta:<sector_id>` | Brain → Face | Pushes AI context updates (chat history, annotations, pending proposals) |
| `ai_skill_load:<skill_id>;<sector_id>` | Face → Brain | Loads a skill into the sector's AI engine |
| `ai_skill_unload:<skill_id>;<sector_id>` | Face → Brain | Unloads a skill from the sector |
| `ai_skill_list:<sector_id>` | Face → Brain | Returns active skills for the sector |

---

### 27.8 Workflow Management API

Messages for kanban board, task, and agent orchestration.

#### 27.8.1 Kanban Board Management (Face ↔ Brain)

| Message | Direction | Payload | Effect |
|:---|:---|:---|:---|
| `workflow_board_load:<project_path>` | Face → Brain | project_path | Load kanban board definition + tasks |
| `workflow_board_watch:<project_path>` | Face → Brain | project_path | Subscribe to real-time updates |
| `workflow_task_move:<task_id>:<lane_id>` | Face → Brain | task_id, lane_id | Move task to lane (triggers auto-promotion checks) |
| `workflow_task_update:<task_id>` | Face → Brain | JSON (title, description, agent, etc.) | Update task properties |
| `workflow_task_assign:<task_id>:<agent_id>` | Face → Brain | task_id, agent_id | Assign/reassign agent to task |
| `workflow_board_state:<project_path>` | Brain → Face | Full board state JSON | Broadcast board state update |

#### 27.8.2 Agent Orchestration (Face ↔ Brain)

| Message | Direction | Payload | Effect |
|:---|:---|:---|:---|
| `workflow_agent_start:<task_id>` | Face → Brain | task_id | Agent begins task (reads LLM decomposition or generates new) |
| `workflow_agent_step_next:<task_id>:<agent_id>` | Face → Brain | task_id, agent_id | Agent advances to next step |
| `workflow_agent_step_pause:<task_id>:<agent_id>` | Face → Brain | task_id, agent_id | Agent pauses at current step |
| `workflow_agent_step_retry:<task_id>:<agent_id>` | Face → Brain | task_id, agent_id, retry_reason | Retry current step with different approach |
| `workflow_agent_step_skip:<task_id>:<agent_id>` | Face → Brain | task_id, agent_id | Skip to next step (manual override) |
| `workflow_agent_abort:<task_id>:<agent_id>` | Face → Brain | task_id, agent_id | Abort task, move to BLOCKED |
| `workflow_agent_output:<agent_id>:<pane_id>` | Brain → Face | agent_id, pane_id | Route agent terminal to pane |
| `workflow_agent_progress:<agent_id>` | Brain → Face | step_current, step_total, status | Update agent progress (for kanban card) |
| `workflow_agent_sandbox:<agent_id>:<action>`| Brain → Disk | agent_id, action | Manage transient filesystem overlay (`create`, `destroy`, `stage`) |
| `workflow_task_merge:<task_id>` | Brain → Disk | task_id | Merge sandboxed changes into project tree; triggers Diff Mode if conflicts |

#### 27.8.3 LLM Interaction Archive (Brain ↔ Storage)

| Message | Direction | Payload | Effect |
|:---|:---|:---|:---|
| `workflow_llm_history_save:<task_id>` | Brain → Storage | Full LLM interaction object | Archive all LLM requests/responses for task |
| `workflow_llm_history_load:<task_id>` | Brain ← Storage | Full LLM interaction object | Retrieve LLM history for resuming task |
| `workflow_patterns_update:<agent_id>` | Brain → Storage | Learned patterns JSON | Update agent's learned patterns file |
| `workflow_patterns_load:<agent_id>` | Brain ← Storage | Learned patterns JSON | Load agent's patterns for new decomposition |

#### 27.8.4 Dream Consolidation (Brain → Storage)

| Message | Direction | Payload | Effect |
|:---|:---|:---|:---|
| `workflow_dream_consolidate:<project_path>` | Face → Brain | project_path | Consolidate completed tasks' LLM histories into project memory |
| `workflow_dream_update:<project_path>` | Brain → Storage | Project memory markdown | Write synthesized memory to `.tos/memory/project_memory.md` |
| `workflow_dream_query:<project_path>:<tag>` | Face → Brain | project_path, tag | Search project memory by pattern/tag |
| `workflow_memory_export:<project_path>` | Face → Brain | project_path, export_path | Export project memory as markdown file |

---

## 28. Predictive Fillers & Intuitive Interaction

### 28.1 Predictive Path & Command Chips

As the user interacts with the Unified Prompt, the Dual-Sided Chip Layout populates with "Intuitive Fillers":
- **Path Completion (Left Chips):** Typing `/` or starting a path triggers immediate chips for the most frequent/recent child nodes at that path depth.
- **Parameter Hints (Right Chips):** For known commands (e.g., `git`, `docker`, `npm`), the Priority Indicator engine suggests the most likely next arguments or flags as clickable chips.
- **Command History Echo:** Suggestions based on commands previously executed within the current sector appear with a subtle "History" icon.

### 28.2 Implicit Search & Typo Correction

If a user submits a command that results in a "File not found" or "Command not found" state:
- The Search Service performs a background fuzzy-match.
- A **Typo Correction Chip** appears in the Right column (e.g., "Did you mean `ls -la`?").
- Clicking the chip replaces the prompt and re-submits automatically.

### 28.3 Dynamic Sector Labeling

When creating a new sector, the "New Sector" name is a placeholder. As the user navigates, the system **heuristically renames** the sector based on the `cwd` (e.g., "TOS Core" if in `~/TOS-Desktop-Environment/src`). This can be locked by the user to prevent auto-renaming.

### 28.4 PTY Output Extraction

For long terminal outputs (e.g., a build failure):
- The system highlights the **authoritative error line** with a higher priority (visual weight/color).
- A **"Focus Error" Chip** appears; when clicked, it scrolls the terminal to the specific failure point.

### 28.5 Notification Display Center

Notifications appear in the **Right Lateral Bezel** and unfurl inward:
- **Priority 1–2 (Normal):** Quiet slide-in, disappears after 5s.
- **Priority 3 (Warning):** Amber pulse, remains until dismissed.
- **Priority 4–5 (Critical):** Red border, accompanied by a tactical earcon and haptic pulse. Requires manual interaction or "Clear" voice command.

---

## 29. Implementation Roadmap

1. **Core Terminal Integration** — Establish basic terminal functionality using the Standardized IPC Format (§3.3.1). Implement bidirectional OSC communication including Line-Level Priority (OSC 9012) and Brain console output stream.
2. **Basic Face & IPC Foundation** — Minimal webview implementing the Action-Identifier Scheme (§25). Level 1 sector tiles + System Output Area. Level 2 interactive terminal + Persistent Unified Prompt.
3. **Modular Trust & Sandboxing** — Implement the Dual-Tier Trust Model (Ecosystem Specification §1). Establish secure sandbox runtime for Standard Modules and privileged execution path for Trusted System Modules.
4. **Input Hub & Semantic Events** — Normalize raw input into semantic actions (§14); implement Voice Confirmation fallback (§17.2.7).
5. **Sector Concept & Management** — Multiple sectors. Sector Tile Context Menu (§6.5) with confirmation and lifecycle controls.
6. **Directory Mode** — Local and remote directory listing as terminal overlay. Remote Directory Fallback (§24.3) with SSH fallback logic.
7. **Activity Mode** — Visual process management via `ps` parsing.
8. **SEARCH Mode** — Unified search domain integration.
9. **Terminal Output Module API** — Define interface for high-speed rendering. Support metadata-driven highlighting based on line priority.
10. **Theme Module API** — CSS variable injection and multi-sensory asset loading.
11. **Shell Module API** — Executable spawning and OSC integration scripts.
12. **AI Engine** — Natural language processing and staged command generation.
13. **Marketplace & Module Discovery** — Package management and permission-based installation.
14. **Auxiliary Services** — TOS Log Service (§19), Settings Daemon, Audio/Haptic Engine.
15. **Remote Sectors & Session Management** — TOS Remote Server protocol. Connection Loss Logic (§24.3).
16. **Platform Backends** — Linux Wayland, Android, OpenXR.
17. **Collaboration** — Host-owned sharing, presence synchronization, guest role enforcement.
18. **Optional High-Fidelity Modules** — Cinematic Triangular Module, community themes, advanced shell plugins.

---

## 30. Glossary of Terms

| Term | Definition |
|---|---|
| **Sector** | A self-contained workspace with its own identity, environment variables, and process tree. |
| **Command Hub** | The Level 2 interface within a sector, featuring a terminal, prompt, and chip regions. |
| **Chip** | Contextual UI elements in Level 2 that stage commands or provide quick actions. |
| **Bezel Slot** | Defined areas in the Tactical Bezel (Top, Left, Right) where components can be docked. |
| **Tactical Reset** | A Level 4 sub-view providing emergency wireframe diagnostics and recovery (God Mode). |
| **Brain** | The logical center of TOS; handles state, command execution, and coordination. |
| **Face** | The visual and input frontend of TOS; handles rendering and event capture. |
| **Earcon** | A unique auditory cue associated with a specific system event or state change. |
| **Level** | A specific depth in the vertical hierarchy (1 to 4). |
| **Projection** | The animation of a bezel component expanding inward to reveal more detail. |
| **SemanticEvent** | An abstract input action (e.g., `zoom_in`, `select`) derived from raw physical input. |
| **Terminal Output Module** | An installable plugin (`.tos-terminal`) that defines how terminal output is visually rendered. |
| **Disconnected Mode** | A Face state where a previously established Brain connection has been lost. The Face displays frozen last-known state while attempting reconnection (§3.4). |
| **No Brain** | A Face state where no Brain has been found after exhausting all discovery methods. The Face displays a minimal connection UI (§3.4). |
| **Connection Health Detection** | The mechanism by which the Face detects Brain loss: if no `state_delta` arrives for 5 seconds (5 missed 1Hz ticks), the Face transitions to Disconnected (§3.4.2). |
