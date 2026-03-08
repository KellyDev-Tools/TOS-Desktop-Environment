# TOS Architecture Specification

**Purpose:** This document is the single source of truth for the architecture and visual design of **TOS** (**Terminal On Steroids**). It defines system structure, process boundaries, IPC contracts, the visual hierarchy, rendering model, input abstraction, security, and all platform behaviour. The terminal and command line are the absolute centre of every design decision. All features exist to augment the terminal, never to bypass it.

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
21. [Priority-Weighted Visual Indicators](#21-priority-weighted-visual-indicators)
22. [Tactical Mini-Map](#22-tactical-mini-map)
23. [Auditory and Haptic Interface](#23-auditory-and-haptic-interface)
24. [Accessibility](#24-accessibility)
25. [Sector Templates and Marketplace](#25-sector-templates-and-marketplace)
26. [Settings Data Model & IPC](#26-settings-data-model--ipc)
27. [Shell API Enhancements](#27-shell-api-enhancements)
28. [Bezel IPC Contracts](#28-bezel-ipc-contracts)
29. [Terminal Output Rendering](#29-terminal-output-rendering)
30. [UI Module Interaction APIs](#30-ui-module-interaction-apis)
31. [Predictive Fillers & Intuitive Interaction](#31-predictive-fillers--intuitive-interaction)
32. [Implementation Roadmap](#32-implementation-roadmap)
33. [Glossary of Terms](#33-glossary-of-terms)

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
- **Module system** for Application Models, Sector Types, AI backends, Terminal Output Modules, Theme Modules, and Shell Modules, all sandboxed and permissioned (see [Ecosystem Specification](./TOS_beta-0_Ecosystem.md)).
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

---

## 4. Modular Service Architecture

TOS decomposes functionality into independent services. Each service runs as a separate OS process and communicates with the Brain via a well-defined IPC protocol. This provides fault isolation, resource management, testability, and flexibility.

| Service | Responsibilities | API / Protocol |
|---------|-----------------|----------------|
| **Brain** | Core state machine, command execution, coordination (§3.1). | JSON-RPC (IPC) |
| **Face** | UI rendering, input capture (§3.2). | JSON-RPC (IPC) |
| **Marketplace Service** | Package index, download, installation, dependency resolution, signature verification, update monitoring. | `marketplace_search`, `marketplace_install` |
| **Settings Daemon** | Store/retrieve configuration values, persistence, change notifications. | `get_setting`, `set_setting` |
| **TOS Log Service** | Collect events from all components, manage log storage, retention, redaction, query interface. | `log_query` |
| **AI Engine** | Load AI backends, process natural language queries, handle function calling, stream responses. | `ai_query` |
| **File Sync Service** | Monitor directories, perform bidirectional sync, conflict resolution via WebDAV extensions. | WebDAV + Inotify/FSEvents |
| **Search Service** | Indexing of file contents, logs, and metadata. Query syntax supports regex and semantic filters. | `search_query` |
| **Notification Center** | Aggregate notifications, manage history, deliver to Face with priority levels (1–5). | `notify_push` |
| **Update Daemon** | Atomic update check, download, and staging. Coordination of the "Yellow Alert" status. | `update_check`, `update_apply` |
| **Heuristic Service** | Predictive fillers, autocomplete-to-chip logic, typo correction, heuristic sector labeling. | `heuristic_query` |
| **Audio & Haptic Engine** | Mix three-layer audio, play earcons, trigger haptic patterns. | `play_earcon`, `trigger_haptic` |
| **Session Service** | Session persistence and workspace memory (live state auto-save, named sessions). | See [Features Specification §2](./TOS_beta-0_Features.md) |

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

The Expanded Bezel Command Surface (see [Features Specification §1](./TOS_beta-0_Features.md)) is a cross-cutting overlay state — not a level. It is available at all levels except Tactical Reset (God Mode):

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
- **Left/right borders** — Mode indicators (CMD, DIR, ACT, SEARCH) as small coloured chips and priority indicator chips (§21).
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
**Auto:** The context switch fires immediately, accompanied by a mode-transition earcon (§23) and a brief visual indicator on the Mode Selector.

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
Only **Bezel Component Modules** (`.tos-bezel` packages, §30.2) may be docked into a slot. This includes the built-in system components listed below and any compatible third-party modules installed via the Marketplace (Ecosystem §1.8). A slot holds one module at a time.

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
| **Tactical Mini-Map** | `Left_Sidebar` | `Left_Sidebar`, `Right_Sidebar` | Spatial sector overview; anchors trigger level transitions (§22). |
| **Priority Indicator** | `Right_Sidebar` | `Left_Sidebar`, `Right_Sidebar` | Dynamically ranked system alerts and notification badges (§21). |
| **Resource Telemetry** | `Top_Center` | `Top_Left`, `Top_Center`, `Top_Right` | Real-time CPU, Memory, Network, and PTY latency. |
| **Mini-Log Telemetry** | `Right_Sidebar` | `Left_Sidebar`, `Right_Sidebar` | Persistent readout of system state and last executed command. |
| **Active Viewport Title** | `Top_Left` | Any | Real-time text readout of current Level, Sector, or App context. |
| **Brain Connection Status** | `Top_Center` | Any | Connection state (Online/Offline) and Brain time. |
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

| Content Type | Description |
|:---|:---|
| **Terminal (Command Hub)** | A full Command Hub instance — prompt, chip columns, terminal output module. Shares the sector's shell context. |
| **Level 3 Application** | Any running graphical application in Application Focus. |

These can be combined freely. Terminal + web portal is the same as terminal + Level 3 app.

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

Split layouts are part of the sector's `hub_layout` object and are persisted to the session file as specced in the Session Persistence Specification (see [Features Specification §2](./TOS_beta-0_Features.md)). On restore:
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

### 15.6 Native Horizon OS Client (Meta Quest)

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

**First-Run Trust Setup:** During the onboarding flow (see [Features Specification §3](./TOS_beta-0_Features.md)), the user is presented with a trust configuration screen for each command class. No defaults are pre-selected. They can skip to defer all classes to WARN.

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

**All detailed specifications regarding module manifests (`module.toml`), sandboxing rules, terminal UI injection, and module profiles (Shells, Themes, AI Backends, AI Behaviors, Bezel Components) are documented in the [Ecosystem Specification](./TOS_beta-0_Ecosystem.md).**

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

## 21. Priority-Weighted Visual Indicators

Non-intrusive indicators convey relative importance without altering size or position.

### 21.1 Indicator Types

| Type | Description |
|---|---|
| **Border Chips** | Small coloured notches along tile border; number reflects priority level (1–5). |
| **Chevrons** | LCARS arrows; pulsing indicates pending notification or critical status. |
| **Glow / Luminance** | Subtle inner/outer glow; intensity varies with priority. |
| **Status Dots** | Small coloured circles (blue=normal, yellow=caution, red=critical). |

**Color Shifts by Urgency:** Subtle accents at Level 1; dominant hazard colors (Orange/Red) at Level 4. Critical alerts may gently pulse their border opacity. High-priority visual changes trigger synchronized UI sounds or haptic pulses on Android/XR.

### 21.2 Priority Scoring & Configuration

Weighted factors (user-configurable):
- Recency of focus (40%)
- Frequency of use (20%)
- Activity level (CPU, memory, I/O) (15%)
- Notification priority (10%)
- Collaboration focus (10%)
- AI suggestion (5%)
- User pinning (override)
- Sector-specific rules

**Configuration:** Master toggle, colour per factor, sensitivity, per-factor visibility, hover tooltips.

### 21.3 Behaviour by Depth

- Level 1: Sector tiles show aggregate priority.
- Level 2: Application tiles show individual priority; chip regions use indicators.
- Level 3: Bezel may show priority chevron/glow; split viewport borders.
- Level 4: Inspection panels show inspected surface's priority and sibling mini-map.

---

## 22. Tactical Mini-Map

Ephemeral overlay providing spatial awareness.

### 22.1 Passive & Active States

- **Passive:** Semi-transparent, input passes through.
- **Active:** Activated by hover (dwell), keyboard (`Ctrl+M`), modifier+click, double-tap, voice. Captures input; shows close button.

### 22.2 Content by Depth

- Level 1: All sectors as miniature tiles.
- Level 2: Current sector with hubs, active hub highlighted.
- Level 3: Focused app highlighted, other viewports shown.
- Level 4: Current surface and siblings.

### 22.3 Monitoring Layer (Resource Usage)

Optional overlay (toggle) showing live resource usage:
- Level 1: Aggregated CPU/memory per sector.
- Level 2: All apps with CPU%, memory%, sparkline.
- Level 3: Detailed stats for focused app + compact for others.
- Throttled to 1–2 Hz.

### 22.4 Bezel Integration (Slot Projection)

The Tactical Mini-Map is docked within a slot in the **Left Bezel Segment**.
- **Docked State:** Occupies the 1.5rem width of the left bezel, showing only high-alert status lines.
- **Projected State:** When activated, it projects a wide glassmorphism overlay into the viewport center without expanding the sidebar.
- **Contextual Anchors:** Clicking tiles within the projected overlay triggers immediate level transitions.

---

## 23. Auditory and Haptic Interface

### 23.1 Three-Layer Audio Model

| Layer | Purpose | Characteristics |
|---|---|---|
| **Ambient** | Atmosphere | Continuous, depth-varying background. |
| **Tactical** | Action confirmation | Discrete earcons for zoom, commands, notifications, alerts, collaboration. |
| **Voice** | Speech synthesis | TTS for announcements, screen reader, AI responses. |

Each layer has independent volume and enable/disable.

### 23.2 Context Adaptation (Green/Yellow/Red Alerts)

- **Green:** Normal.
- **Yellow:** Ambient shifts urgent, tactical adds periodic pulse, voice more verbose.
- **Red:** Ambient replaced by repeating tone; tactical suppresses non-critical earcons; voice prioritises critical messages.

### 23.3 Spatial Audio (VR/AR)

Sounds positioned in 3D space. Notifications from the left sector sound left.

### 23.4 Haptic Feedback Taxonomy

| Event | Pattern |
|---|---|
| `zoom_in` | Ascending pulses |
| `select` | Quick click |
| `dangerous_command` | Sharp, insistent buzz |
| `red_alert` | Pulsing, escalating |

Scrolling the Cinematic Triangular terminal triggers subtle haptic detents. Spatial haptics in VR/AR (directional).

### 23.5 Theming & Configuration

- Audio themes (`.tos-audio`) installable via Marketplace (see Ecosystem Specification).
- Applications can contribute custom sounds.
- Configuration: master volume, per-category toggles, test patterns, hearing-impaired mode (route tactical to haptics).

---

## 24. Accessibility

### 24.1 Visual

- High-contrast themes, font scaling, colourblind filters.
- Screen reader support (AT-SPI/Orca on Linux, TalkBack on Android).
- Braille display support.
- Focus indicators (thick border, optional haptic/auditory).
- **High-Visibility Mode:** Forced thick borders, monochromatic glassmorphism for better contrast, increased font sizes.
- **Screen Reader Bridge:** Every UI element publishes a semantic role (button, line, chip) to the platform's accessibility bridge (AT-SPI / TalkBack).

### 24.2 Auditory

- Screen reader via Voice layer.
- Earcons for navigation and feedback.
- Voice notifications (TTS) with adjustable verbosity.

### 24.3 Motor

- Switch device support (single/multi-switch scanning, linear/row-column).
- Dwell clicking (gaze/head tracking).
- Sticky keys, slow keys.
- **Voice Confirmation:** Users can confirm commands via speech using a randomized challenge-response system if a tactile prompt is physically impossible.
- Haptic confirmation for actions.
- Customisable input mapping.

### 24.4 Cognitive

- Simplified mode (reduced clutter, larger elements, limited features).
- Built-in tutorials (eval-help mapping, interactive guides).
- Consistent spatial model (four levels, context-aware modes).

### 24.5 Profiles & Platform Integration

- Central Accessibility panel with profiles (save/load/export).
- Per-sector overrides.
- Integration with platform accessibility services (AT-SPI, TalkBack, Switch Access).

---

## 25. Sector Templates and Marketplace

Schema for `.tos-template` distribution, AI-assisted discoverability, and the Marketplace package daemon are documented in the [Ecosystem Specification §2](./TOS_beta-0_Ecosystem.md).

---

## 26. Settings Data Model & IPC

### 26.1 Layered Settings

Settings are resolved via cascade: **per-application > per-sector > global key-value bag > global scalar field defaults**.

### 26.2 Canonical Keys & Defaults

| Key | Type | Default | Description |
|---|---|---|---|
| `theme` | string | `"lcars-light"` | Active theme module ID. |
| `default_shell` | string | `"fish"` | Default shell module ID for new sectors. |
| `terminal_output_module` | string | `"rectangular"` | Active terminal output module ID. |
| `master_volume` | `"0"`–`"100"` | `"80"` | Master audio volume. |
| `logging_enabled` | `"true"`\|`"false"` | `"true"` | Master log toggle. |
| `deep_inspection` | `"true"`\|`"false"` | `"false"` | Enable Buffer View. |
| `terminal_buffer_limit` | integer | `500` | Maximum lines to keep in terminal buffer (FIFO). |

### 26.3 IPC Messages for Settings

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

### 26.4 Persistence

Settings saved to `~/.config/tos/settings.json` (Linux) or app-private storage (Android) as JSON. Debounced writes (≤1s). Only canonical keys and extensions are persisted; runtime-only state is skipped.

---

## 27. Shell API Enhancements

### 27.1 `command_result` Payload Format

OSC `9002` payload extended to three semicolon-delimited fields: `<command>;<exit_status>;<base64(stdout+stderr)>`. Base64 encoding prevents control characters from breaking OSC parsing. Third field optional for backwards compatibility.

### 27.2 Shell Integration Script Requirements

- Capture full combined stdout/stderr of each command.
- Base64-encode and emit `ESC]9002;<command>;<exit_status>;<base64>BEL`.
- Also emit `ESC]9003;<cwd>BEL` on directory change.
- Do not capture TOS-internal commands (`EXEC`, `CD`, etc.).

### 27.3 Fallback: Raw PTY Output & Filesystem Fallback

- Without integration, PTY reader strips ANSI, splits lines, caps at 500 lines (default; user-adjustable).
- **Local Directory Mode:** Falls back to `std::fs::read_dir` if `hub.shell_listing` is `None`.
- **Remote Directory Mode:** If `hub.shell_listing` is missing, the Brain attempts to fetch listing data via the TOS Remote Server's File Service (§12.1).
  - **Graceful Fallback:** If the remote server is not installed (e.g., raw SSH link), Directory Mode visuals are disabled and the interface stays in standard shell output.
  - **Connection Loss:** If an active remote server connection is lost, the sector displays a "Remote session disconnected" banner and closes after 5 seconds of inactivity.

### 27.4 Line-Level Priority Metadata

The shell can emit a priority sequence before producing a line of output:

```
ESC]9012;<level>BEL
```

**Levels:** `0` Normal / Inherit, `1` Low, `2` Notice, `3` Warning, `4` Critical, `5` Urgent (Tactile/Auditory trigger).

The Brain applies this level to all subsequent lines until a new 9012 sequence is received or the command completes.

### 27.5 Command Auto-Detection (`ls`, `cd`)

- If submitted command starts with `ls` (case-insensitive), resolve target path, set `hub.current_directory`, switch to Directory Mode, clear stale listing.
- If starts with `cd`, resolve target path, set `hub.current_directory` (if exists), do not change mode.
- No false positives (`rls`, `echo cd`).

### 27.6 Directory Mode Pick Behaviour

| Prompt state | Item type | Click action |
|---|---|---|
| Empty | File | Insert absolute path into prompt |
| Empty | Folder | Navigate into folder |
| Command staged | File | Append absolute path as next argument |
| Command staged | Folder | Append absolute path as next argument |

**Staging banner** appears above file grid when command staged, with current command and hint. Multi-select appends all selected paths in order.

IPC messages: `dir_pick_file:<n>`, `dir_pick_dir:<n>`, `dir_navigate:<path>`.

---

## 28. Bezel IPC Contracts

### 28.1 Action-Identifier Rule

All IPC messages from bezel buttons and UI controls must use **action identifiers**, not display labels. Labels are for rendering only and must not be forwarded to the shell.

✅ Correct: `<button onclick="window.ipc.postMessage('zoom_out')">ZOOM OUT</button>`
❌ Incorrect: `<button onclick="window.ipc.postMessage(this.innerText)">ZOOM OUT</button>`

**Prompt Interception Layer:** The `prompt_submit:` message is an exception. The Brain performs a "sniffing" pass on the submitted string to detect `ls` or `cd` and trigger mode switches. This logic lives entirely in the Brain's command dispatcher.

### 28.2 Reserved IPC Prefixes

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

## 29. Terminal Output Rendering

### 29.1 ANSI Stripping

Before storage in `hub.terminal_output`, strip:
- CSI sequences (`ESC[ ... m`)
- OSC sequences (`ESC] ... BEL` or `ESC\`)
- C0/C1 controls except TAB, LF, CR.

Result must be valid printable UTF-8.

### 29.2 Buffer Limits & Rendering Requirements

- Cap at 500 lines (FIFO) by default; user-adjustable via `terminal_buffer_limit` setting.
- Monospace font, `white-space: pre-wrap`.
- Auto-scroll to latest line.
- Distinct styling for command echo (`> command`) vs output.

---

## 30. UI Module Interaction APIs

Terminal and Bezel modules interact with the Face via these specific UI-hooks:

### 30.1 Terminal Output API

- **`render(surface, lines)`:** The Face provides a `RenderSurface` (DOM or GPU buffer). The module handles font-rendering and ANSI color application.
- **`on_click(x, y)`:** Returns the line index and context-action (e.g., `copy`, `inspect_pid`).
- **`on_scroll(delta)`:** Handles the visual transition of lines.

### 30.2 Bezel Component API

- **`update_view(html, data)`:** Components push their rendered state to the Face.
- **`component_click(id, x, y)`:** The Face forwards clicks on specific component IDs to the underlying module.
- **`request_projection(mode)`:** Components can request to "unfurl" a detailed panel (e.g., the Mini-Map expanding into the viewport).

---

## 31. Predictive Fillers & Intuitive Interaction

### 31.1 Predictive Path & Command Chips

As the user interacts with the Unified Prompt, the Dual-Sided Chip Layout populates with "Intuitive Fillers":
- **Path Completion (Left Chips):** Typing `/` or starting a path triggers immediate chips for the most frequent/recent child nodes at that path depth.
- **Parameter Hints (Right Chips):** For known commands (e.g., `git`, `docker`, `npm`), the Priority Indicator engine suggests the most likely next arguments or flags as clickable chips.
- **Command History Echo:** Suggestions based on commands previously executed within the current sector appear with a subtle "History" icon.

### 31.2 Implicit Search & Typo Correction

If a user submits a command that results in a "File not found" or "Command not found" state:
- The Search Service performs a background fuzzy-match.
- A **Typo Correction Chip** appears in the Right column (e.g., "Did you mean `ls -la`?").
- Clicking the chip replaces the prompt and re-submits automatically.

### 31.3 Dynamic Sector Labeling

When creating a new sector, the "New Sector" name is a placeholder. As the user navigates, the system **heuristically renames** the sector based on the `cwd` (e.g., "TOS Core" if in `~/TOS-Desktop-Environment/src`). This can be locked by the user to prevent auto-renaming.

### 31.4 PTY Output Extraction

For long terminal outputs (e.g., a build failure):
- The system highlights the **authoritative error line** with a higher priority (visual weight/color).
- A **"Focus Error" Chip** appears; when clicked, it scrolls the terminal to the specific failure point.

### 31.5 Notification Display Center

Notifications appear in the **Right Lateral Bezel** and unfurl inward:
- **Priority 1–2 (Normal):** Quiet slide-in, disappears after 5s.
- **Priority 3 (Warning):** Amber pulse, remains until dismissed.
- **Priority 4–5 (Critical):** Red border, accompanied by a tactical earcon and haptic pulse. Requires manual interaction or "Clear" voice command.

---

## 32. Implementation Roadmap

1. **Core Terminal Integration** — Establish basic terminal functionality using the Standardized IPC Format (§3.3.1). Implement bidirectional OSC communication including Line-Level Priority (OSC 9012) and Brain console output stream.
2. **Basic Face & IPC Foundation** — Minimal webview implementing the Action-Identifier Scheme (§28). Level 1 sector tiles + System Output Area. Level 2 interactive terminal + Persistent Unified Prompt.
3. **Modular Trust & Sandboxing** — Implement the Dual-Tier Trust Model (Ecosystem Specification §1). Establish secure sandbox runtime for Standard Modules and privileged execution path for Trusted System Modules.
4. **Input Hub & Semantic Events** — Normalize raw input into semantic actions (§14); implement Voice Confirmation fallback (§17.2.7).
5. **Sector Concept & Management** — Multiple sectors. Sector Tile Context Menu (§6.5) with confirmation and lifecycle controls.
6. **Directory Mode** — Local and remote directory listing as terminal overlay. Remote Directory Fallback (§27.3) with SSH fallback logic.
7. **Activity Mode** — Visual process management via `ps` parsing.
8. **SEARCH Mode** — Unified search domain integration.
9. **Terminal Output Module API** — Define interface for high-speed rendering. Support metadata-driven highlighting based on line priority.
10. **Theme Module API** — CSS variable injection and multi-sensory asset loading.
11. **Shell Module API** — Executable spawning and OSC integration scripts.
12. **AI Engine** — Natural language processing and staged command generation.
13. **Marketplace & Module Discovery** — Package management and permission-based installation.
14. **Auxiliary Services** — TOS Log Service (§19), Settings Daemon, Audio/Haptic Engine.
15. **Remote Sectors & Session Management** — TOS Remote Server protocol. Connection Loss Logic (§27.3).
16. **Platform Backends** — Linux Wayland, Android, OpenXR.
17. **Collaboration** — Host-owned sharing, presence synchronization, guest role enforcement.
18. **Optional High-Fidelity Modules** — Cinematic Triangular Module, community themes, advanced shell plugins.

---

## 33. Glossary of Terms

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
