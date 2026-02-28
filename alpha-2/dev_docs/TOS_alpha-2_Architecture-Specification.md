# TOS Architectural Specification – Consolidated

**Purpose:** This document provides a complete, unified architectural vision for the Tactical Operating System (TOS), placing the **terminal and command line** at the absolute centre of the experience. Every feature, from visual modes to AI assistance, exists to augment and empower the terminal, never to bypass it. This revision restores the original ethos of TOS as "Terminated On Steroids" – a power‑user environment that brings the full capability of the command line to any platform, with rich visual feedback and multi‑sensory augmentation. It introduces a **modular terminal output system**, allowing users to install and switch between different visual representations of terminal output, as well as **Theme Modules** for customising appearance and **Shell Modules** for selecting different shell implementations. The Global Overview features a dedicated System Output Area powered by the same terminal output module system, and sector tiles provide a context menu for quick management.

---

## Table of Contents

1. [Core Philosophy: Terminal First](#1-core-philosophy-terminal-first) 
2. [System Overview](#2-system-overview) 
3. [Process Architecture: Brain & Face](#3-process-architecture-brain--face) 
4. [Modular Service Architecture](#4-modular-service-architecture) 
5. [The Extended Hierarchy](#5-the-extended-hierarchy) 
6. [Global Overview – Level 1 (with System Output Area)](#6-global-overview--level-1-with-system-output-area) 
   6.1. Sector Tiles as Mini Command Hubs 
   6.2. System Output Area (Brain Console) – Terminal Output Module Layer 
   6.3. Zoom Transition 
   6.4. Global Overview Bezel 
   6.5. Sector Tile Context Menu (Long Press / Secondary Select) 
7. [Command Hub – Level 2 (The Heart of TOS)](#7-command-hub--level-2-the-heart-of-tos) 
   7.1. Persistent Unified Prompt 
   7.2. Terminal Output as Primary Canvas (Powered by Terminal Output Module) 
   7.3. Context-Aware Terminal Augmentation 
   7.4. Dual‑Sided Chip Layout 
   7.5. Output Area Configurations (Provided by Modules) 
   7.6. Autocomplete Overlay 
   7.7. Context‑Aware Mode Switching 
   7.8. Terminal Foreground Toggle 
   7.9. Multitasking with Multiple Terminals 
8. [Application Focus – Level 3](#8-application-focus--level-3) 
   8.1. Tactical Bezel 
   8.2. Application Models 
   8.3. Deep Inspection Access 
9. [Deep Inspection – Levels 4 & 5](#9-deep-inspection--levels-4--5) 
10. [Sectors and the Tree Model](#10-sectors-and-the-tree-model) 
11. [Split Viewports](#11-split-viewports) 
12. [Remote Sectors](#12-remote-sectors) 
13. [Collaboration](#13-collaboration) 
14. [Input Abstraction Layer](#14-input-abstraction-layer) 
15. [Platform Abstraction](#15-platform-abstraction) 
16. [Performance and Compositing](#16-performance-and-compositing) 
17. [Security Model](#17-security-model) 
18. [Modules: Application Models, Sector Types, AI Backends, Terminal Output, Themes, and Shells](#18-modules) 
    18.1. Application Model 
    18.2. Sector Type 
    18.3. AI Backend Modules 
    18.4. Module Isolation & Permissions 
    18.5. Terminal Output Modules 
    18.6. Theme Modules 
    18.7. Shell Modules 
    18.8. Relationship with Other Modules 
19. [TOS Log](#19-tos-log) 
20. [Tactical Reset](#20-tactical-reset) 
21. [Priority‑Weighted Visual Indicators](#21-priority-weighted-visual-indicators) 
22. [Tactical Mini‑Map](#22-tactical-mini-map) 
23. [Auditory and Haptic Interface](#23-auditory-and-haptic-interface) 
24. [Accessibility](#24-accessibility) 
25. [Sector Templates and Marketplace](#25-sector-templates-and-marketplace) 
26. [Settings Data Model & IPC](#26-settings-data-model--ipc) 
27. [Shell API Enhancements](#27-shell-api-enhancements) 
28. [Bezel IPC Contracts](#28-bezel-ipc-contracts) 
29. [Terminal Output Rendering](#29-terminal-output-rendering) 
30. [Implementation Roadmap](#30-implementation-roadmap) 
31. [Conclusion](#31-conclusion) 

---

## 1. Core Philosophy: Terminal First

TOS was born from the acronym **Terminated On Steroids** – a vision to take the raw power of the command line and amplify it across every platform, from desktop to VR to mobile. The terminal is not just one mode among many; it is the **primary and permanent interface**. Every action a user takes – whether clicking a file, speaking a command, or gesturing in VR – must ultimately be expressible as a command line that appears in the **Persistent Unified Prompt** and is executed by the underlying shell.

This philosophy ensures that:

- **Power users never lose their terminal.** All visual augmentations are simply different ways to view and interact with the same data that the terminal already exposes. They generate commands, never bypass them.
- **The prompt is the source of truth.** Whatever is staged in the prompt is what will be executed. Clicking a file appends its path; selecting a process inserts its PID. The user always sees the command before running it.
- **All modes are overlays on the terminal.** The terminal output area remains visible and central; chip regions, the bezel, and other UI elements are helpers, not replacements.
- **The Shell API (OSC integration) is the backbone.** Deep bidirectional communication with the shell ensures that the UI stays in sync with the real environment.

Additionally, TOS introduces a **system‑level terminal output** at the Global Overview (Level 1), providing a window into the Brain's operations when surveying all sectors. This output is powered by the same modular terminal output system used in Command Hubs, ensuring consistency and reusability.

---

## 2. System Overview

TOS is built around a strictly vertical hierarchy of **levels**, a tree of **sectors**, and a **Persistent Unified Prompt** that drives all interaction. The system is composed of:

- A **platform‑agnostic core** (the **Brain**) implementing the hierarchy, command execution, security, and coordination.
- A **Unified Tactical Bezel** – a persistent frame that surrounds the entire viewport across all hierarchical levels. It is composed of segments:
  - **Top Bezel Segment:** System controls(logout, shutdown, reboot, settings, etc), and dual expansion handles. Now hosts **Configurable Horizontal Slots** for high-level telemetry and quick-access tools.
    - The Top Bezel Segment is divided into three sections: Left, Center, and Right. Each section has a different set of requirements, allowing for a flexible and customizable top-level interface.
      - Left Section: the expand/collapse handles for the left lateral segment.
      - Center Section: component slots for high-level telemetry and quick-access tools (time, date, cpu usage, memory usage, network status, etc).
      - Right Section: the expand/collapse handles for the right lateral segment then in the right most section the system controls menu(logout, shutdown, reboot, settings, etc).
  - **Bottom Bezel Segment:** The Persistent Unified Prompt (Locked assembly, no configurable slots).
  - **Lateral Segments (Left & Right):** Slender vertical bars containing **Configurable Vertical Slots**. 
  - **Configurability:** All component slots (Top, Left, Right) are user-definable. Any system tool (Minimap, AI Stage, Clock, etc.) can be docked to any slot via the Settings panel or direct manipulation.
- **Platform backends** (Wayland, OpenXR, Android) providing rendering, input, and system services via three core traits: `Renderer`, `InputSource`, `SystemServices`.
- **Remote connectivity** via the TOS Remote Server protocol, enabling remote sectors, collaboration, and web portals.
- **Module system** for Application Models, Sector Types, AI backends, Terminal Output Modules, Theme Modules, and Shell Modules, all sandboxed and permissioned.
- A set of **auxiliary services** each running as independent processes and communicating via IPC.
- A **system‑level terminal output** at Level 1 that displays the Brain's own console, powered by the terminal output module system.

---

## 3. Process Architecture: Brain & Face

Inspired by the early `alpha-0` heritage implementation, TOS adopts a clean separation between logic and presentation by running two concurrent threads (or optionally separate processes) that communicate via a well‑defined IPC protocol.

### 3.1 The Brain (Logic Thread/Process)
- Maintains the core state machine: sectors, command hubs, application surfaces, zoom levels, and user sessions.
- Handles all command execution (shell integration, PTY management) for both sector terminals and its own system console.
- Processes semantic events and updates state accordingly.
- Manages collaboration, remote connections, and module lifecycle.
- Emits state snapshots and deltas to be consumed by the Face and other services.
- Its own console output (system logs, background task results, error messages) is streamed to the Face for display in the **system‑level terminal output** at Level 1.

### 3.2 The Face (UI Thread/Process)
- Runs the platform‑specific renderer.
- Captures raw input from devices and forwards it to the Brain (after optional local echo for immediate feedback).
- Receives state updates from the Brain and renders the interface.
- Hosts the Tactical Bezel, mini‑map, and all visual overlays.
- Instantiates and manages Terminal Output Modules for each terminal context (sector terminals and the system output).

### 3.3 Communication
- **IPC Protocol:** JSON‑RPC or MessagePack over a local socket or channel.
- **Messages from Brain to Face:** State deltas, audio/haptic commands, UI control signals, and lines for the system‑level terminal output.
- **Messages from Face to Brain:** Semantic events (after mapping), prompt submissions, bezel clicks, and context menu actions.

#### 3.3.1 Message Format Standard
To ensure consistent parsing across all services, all IPC messages sent from the Face (UI) to the Brain MUST follow this scheme:
- **Format:** `prefix:payload`
- **Prefix:** A unique action identifier ending in a colon.
- **Payload:** Message-specific data. If multiple arguments are required, they MUST be delimited by **semicolons** (`;`).
- **Example:** `set_setting:theme;lcars-dark` or `signal_app:uuid;SIGTERM`.

---

## 4. Modular Service Architecture

Beyond the Brain and Face, TOS decomposes functionality into a set of independent services. Each service runs as a separate OS process (or lightweight thread with strong isolation) and communicates with the Brain (and sometimes directly with the Face) via a well‑defined IPC protocol. This design provides fault isolation, resource management, testability, and flexibility.

| Service | Responsibilities |
|---------|------------------|
| **Brain** | Core state machine, command execution, coordination (see §3.1). |
| **Face** | UI rendering, input capture (see §3.2). |
| **Marketplace Service** | Package index, download, installation, dependency resolution, signature verification, update monitoring. |
| **Settings Daemon** | Store/retrieve configuration values, persistence, change notifications. |
| **TOS Log Service** | Collect events from all components, manage log storage, retention, redaction, query interface. |
| **AI Engine** | Load AI backends, process natural language queries, handle function calling, stream responses. |
| **Collaboration Server** | Manage WebSocket connections to remote guests, WebRTC signalling, broadcast presence and cursors, enforce permissions. |
| **File Sync Service** | Monitor directories, perform bidirectional sync, conflict resolution, progress reporting. |
| **Input Hub** | Collect raw input from devices, normalize to semantic events, apply user mappings, forward to Brain. |
| **Audio & Haptic Engine** | Mix three‑layer audio, play earcons, spatialise sound, trigger haptic patterns. |
| **Update Daemon** | Check for updates, download in background, stage updates, coordinate restart. |
| **Notification Center** | Aggregate notifications from all services, manage history, deliver to Face. |
| **Power Monitor** | Track battery, thermal, power events; trigger alerts; advise power‑saving modes. |
| **Accessibility Bridge** | Interface with platform accessibility services, translate events, provide screen reader output. |
| **Module Runtimes** | Execute third‑party modules (Application Models, Sector Types, AI backends, Terminal Output Modules, Theme Modules, Shell Modules) in sandboxed processes. |

All services communicate with the Brain via IPC. The Brain maintains authoritative state and routes messages as needed. Some services may communicate directly with the Face for performance (e.g., Audio Engine, Input Hub), but semantic decisions are made or approved by the Brain.

---

## 5. The Extended Hierarchy

| Level | Name                 | Description |
|-------|----------------------|-------------|
| **1** | **Global Overview**  | Bird’s‑eye view of all sectors, with System Output Area (Brain console) rendered as a terminal output module layer below the tiles. |
| **2** | **Command Hub**      | Central control for a sector, with full terminal and prompt. |
| **3** | **Application Focus**| Full‑screen application surface wrapped in the Tactical Bezel. |
| **4** | **Detail View**      | Structured metadata for any surface. |
| **5** | **Buffer View**      | Raw memory hex dump (privileged, may be unavailable on some platforms). |

**Lifecycle:** Levels 4 and 5 are transient; a Tactical Reset immediately flushes all inspection buffers and reverts to Level 1 or 2.

---

## 6. Global Overview – Level 1 (with System Output Area)

The Global Overview is the top level of the hierarchy, providing a bird’s‑eye view of all sectors. It features a dedicated **System Output Area** that displays the Brain's console output – system logs, background task results, and diagnostic information – in a live terminal view. This area is powered by the same **Terminal Output Module** system used in Command Hubs (§18.5), but in a **read‑only** mode (no prompt, no command input). It is rendered as a distinct layer **above** a solid background (or wallpaper) and **below** the sector tiles and UI elements, unless the user toggles it to the front via the bezel.

### 6.1 Sector Tiles as Mini Command Hubs

Each sector tile’s borders mirror the Command Hub’s structure and provide at‑a‑glance status of the sector's activity.

- **Top border** – Represents the Tactical Bezel (collapsed). A coloured strip may indicate alert status (green/yellow/red) or active collaboration (presence of guests). The colour and any animation (e.g., pulsing) convey urgency.
- **Bottom border** – Embodies the Persistent Unified Prompt and reflects the **status of the last command executed** in that sector's Command Hub.
  - **Solid color:** A solid line indicates the last command completed. Green (or user‑configured success color) for successful exit (status 0), red (or failure color) for non‑zero exit.
  - **Animated gradient:** If a command is currently running in that sector, the border displays a sliding gradient that transitions between the success and failure colors (e.g., green → red → green) with a smooth, continuous animation. The direction of the slide (left‑to‑right or right‑to‑left) is user‑configurable, and the animation speed can be adjusted.
  - **No command history:** If no command has been run yet (fresh sector), the border may be a neutral color (e.g., gray) or remain invisible.
- **Left/right borders** – House mode indicators (CMD, DIR, ACT, SEARCH) as small coloured chips, and priority indicator chips (see §21). The active mode of the sector's Command Hub is highlighted.

**Additional information conveyed:**
- Recent activity: A subtle "wave" animation along the bottom border can hint at recent output or notifications (e.g., a gentle ripple after a command completes).
- Collaboration presence: Tiny avatar dots along the top border show active guests in that sector.

Tiles may have semi‑transparent backgrounds to allow the System Output Area to be visible behind them. The opacity is user‑configurable.

### 6.2 System Output Area (Brain Console) – Terminal Output Module Layer

The System Output Area is a live terminal view that displays the console output of the Brain. It is a real‑time visualization of the `system` domain from the **TOS Log Service** (§19). It is powered by the same **Terminal Output Module** system used in Command Hubs (§18.5), but in a **read‑only** mode (no prompt, no command input). It is rendered as a distinct layer **above** a solid background (or wallpaper) and **below** the sector tiles and UI elements, unless the user toggles it to the front via the bezel.

**Layering:**
- **Bottom layer:** A solid color or wallpaper (user‑configurable) that serves as the deepest background.
- **Middle layer:** The System Output Area, rendered by the Terminal Output Module. This layer is visible behind the sector tiles, but the tiles are drawn above it. The terminal output may be partially obscured by the tiles, but the user can adjust tile opacity.
- **Top layer:** Sector tiles, the Global Overview bezel, and any other interactive UI elements.

When the user activates the **"Bring Terminal to Front"** bezel command (see §6.4), the System Output Area is temporarily moved to the top layer, overlaying the tiles. This allows full viewing and scrolling of the terminal without obstruction. A subsequent command (or pressing Esc) returns it to the middle layer.

**Appearance:**
- The System Output Area uses the visual style defined by the active Terminal Output Module (e.g., rectangular, cinematic triangular). If the module supports it, the vanishing point or layout can be configured separately for this instance.
- It is scrollable: users can scroll the terminal using gestures (e.g., two‑finger swipe on empty space) or via the bezel's scroll buttons. Scrolling is handled by the Terminal Output Module's API.
- The area is read‑only: clicking on lines does not trigger context actions (unlike in a Command Hub). However, users can copy text via keyboard or context menu.

**Content:**
- System logs: Brain startup messages, service lifecycles, errors, warnings.
- Background task output: results from file sync, updates, AI processing.
- Security events: authentication attempts, privilege escalations.
- Collaboration events: user joins/leaves (if configured).

**Configuration:**
- Users can adjust the size/position of the System Output Area (e.g., full‑screen, bottom panel, left panel) via Settings.
- Opacity of the area and of sector tiles can be tuned independently.
- The same Terminal Output Module configuration (font size, color scheme, animation speed) applies globally, but may have separate overrides for the System Output Area.

### 6.3 Zoom Transition

Selecting a sector tile smoothly expands its borders into the full Command Hub. During the transition, the System Output Area fades out or slides away, replaced by the sector's own terminal output area (also powered by the same Terminal Output Module, but with interactive capabilities). The System Output Area can still be accessed via a bezel shortcut after zooming.

### 6.4 Global Overview Bezel (Top Segment)

The **Top Bezel Segment** at Level 1 provides system‑level controls and includes commands that are sent directly to the **Brain's terminal** (i.e., they affect the system console, not any sector). Bezel buttons use action identifiers as per §28.

**Collapsed:** Thin top strip with:
- Settings icon (gear)
- Add Sector button (+)
- Expand handle (down chevron)
- Collaboration indicator (avatars of active shared sectors)
- **Terminal Toggle** (eye icon) – brings the System Output Area to the front (or sends it back) when clicked.

**Expanded:** Activated by dragging the handle, clicking, or `Ctrl+Space`. Reveals a command strip with:

- **Navigation:** Zoom Out (if applicable), Home (reset overview layout).
- **Sector Management:** New Sector, Import Sector, Remote Connection.
- **System:** Settings, Updates, Security Dashboard.
- **Terminal Controls:**
  - **Bring Terminal to Front** – moves System Output Area to top layer.
  - **Scroll Terminal Up/Down** – for users without gesture input.
  - **Clear Terminal** – sends a clear signal to the Brain's console (does not affect sector terminals).
- **Collaboration:** Share Overview, Active Sessions, Invite Users.
- **View Controls:** Toggle Mini‑Map, Toggle Sector Labels, Arrange Tiles.
- **Power:** Sleep, Restart TOS, Log Out (with tactile confirmation).

All bezel commands at Level 1 are processed by the Brain and may produce output that appears in the System Output Area (e.g., `Clear Terminal` may print a confirmation message).

### 6.5 Sector Tile Context Menu (Long Press / Secondary Select)

A long press (touch) or secondary click (right‑click, or semantic event `secondary_select`) on a sector tile opens a context menu with actions for managing the sector. This menu provides quick access to common sector operations without requiring the user to first zoom into the sector. The menu is rendered as a floating LCARS panel near the tile, respecting the active theme and accessibility settings.


#### 6.5.1 Menu Actions

| Action | Description | Confirmation / Protection |
|--------|-------------|---------------------------|
| **Close Sector** | Terminates all processes in the sector and removes its tile from the overview. Equivalent to `tos sector close <name>`. | If any processes are running, a warning modal lists them (e.g., "3 running tasks: build.sh, server.py, tail -f log") and requires **tactile confirmation** (see §17.3). The user must perform a tactile confirmation slider, biometric prompt, or multi‑button press to proceed. |
| **Freeze Sector** | Suspends all processes in the sector (SIGSTOP). The tile becomes dimmed (e.g., 50% opacity) and displays a frozen badge (e.g., snowflake icon) in the top border. Priority indicators pause updating. A subsequent "Resume" option replaces "Freeze" in the menu. | No additional confirmation required. |
| **Clone Sector** | Creates a new sector with identical configuration (name, type, environment, favourites, and settings). Running processes are **not** duplicated – the clone starts with a fresh shell (Note: Cloning does not duplicate running processes). The new sector tile appears adjacent to the original. | A simple confirmation dialog may be shown (user‑configurable). If the sector contains sensitive data, the user may be prompted to confirm. |

Additional actions (e.g., Rename, Edit Settings, Share) may be added in future versions.

#### 6.5.2 Visual Feedback

- **Freeze:** Tile opacity reduced, a snowflake icon appears in the top‑right corner (or integrated into the top border). The tile's bottom border (command status indicator) may freeze in its current state.
- **Close:** After confirmation, the tile animates out (e.g., shrink and fade) and is removed. An undo notification may appear briefly (if enabled in settings) allowing the user to cancel the close.
- **Clone:** A new tile animates in near the original, with a brief "Cloning..." indicator.

#### 6.5.3 IPC Messages

The following IPC messages are sent from the Face to the Brain when actions are selected:

- `sector_close:<sector_id>`
- `sector_freeze:<sector_id>`
- `sector_unfreeze:<sector_id>` (sent when user selects "Resume")
- `sector_clone:<sector_id>`

The Brain processes these messages, updates the sector state, and returns updated state deltas to the Face.

#### 6.5.4 Integration with Input Abstraction

The context menu is triggered by the `secondary_select` semantic event (see §14.1) when the target is a sector tile. On devices without a secondary button (e.g., touch), a long press (duration configurable) generates the same event. Voice commands (e.g., "show sector options") can also trigger the menu.

#### 6.5.5 Accessibility

- The menu is fully navigable via keyboard (arrow keys, Enter) and screen reader (labels announced).
- High‑contrast theme applies to menu appearance.
- Haptic feedback may accompany menu opening (optional, user‑configurable).

---


## 7. Command Hub – Level 2 (The Heart of TOS)

When the user zooms into a sector, the System Output Area is replaced by the sector's own terminal output area. The sector's terminal is now the primary focus. The bezel provides a **Show System Output** button that temporarily overlays the system console (e.g., as a split or pop‑up) without leaving the sector.

### 7.1 Persistent Unified Prompt (Bottom Bezel Segment)

The **Bottom Bezel Segment** houses the Persistent Unified Prompt, the primary interface for all system interaction. It is visible across all levels and modes. By architectural rule, this segment is **strictly static** and contains **no slots**. It does not support configurable component docking.

- **Left Section (Origin):** **Universal Mode Selector** (CMD, SEARCH, AI, ACTIVITY). This is an integral part of the prompt assembly, not a dockable module.
- **Center Section:** The input field. It always reflects the current command-to-be across different interaction modes.
- **Right Section:** Mic and Stop buttons for voice-first interaction and command termination.
- **Visual State:** The segment is **collapsed and unexpandable** at Level 1, **fully expanded** at Level 2, **collapsed yet expandable** (via hover/click) at Level 3, and **collapsed and unexpandable** at Levels 4 and 5.

### 7.2 Terminal Output as Primary Canvas (Powered by Terminal Output Module)

The terminal output area is rendered by the currently active **Terminal Output Module** (see §18.5). This module is responsible for displaying a scrollable, interactive view of the shell's combined stdout/stderr, with ANSI codes stripped or rendered appropriately. The module defines the visual appearance, layout, and any special effects (e.g., perspective, animation). The Face provides the module with a stream of lines from the sector's PTY and handles user interactions (click, hover, scroll) via the module's API.

By default, TOS ships with a **Rectangular Terminal Output Module** that provides a standard, full‑width rectangle with uniform text and vertical scrolling. Additional modules, such as the **Cinematic Triangular Terminal Output Module**, can be installed via the Marketplace.

The output area occupies the full width between the left and right chip regions. It is rendered as a separate layer, behind the dynamic chip regions and bezel, but the user can temporarily bring it to the front using a bezel toggle (see §7.8).

### 7.3 Context-Aware Terminal Augmentation

Rather than utilizing separate graphical pop-ups, grids, or overlays for different tasks, TOS treats the **Terminal Canvas** and the **Dual-Sided Chip Layout** as a unified interface. The system mode dictates what appears in the terminal and how the chips are populated, ensuring a consistent function-over-form interaction model:

| Context | Terminal Canvas | Chip Layout Integration |
|---------|-----------------|-------------------------|
| **Command** | Standard shell `stdout`/`stderr`. | Chips show command history, autocomplete suggestions, and tool flags. |
| **Search** | Semantic or exact search results. | Chips populate with search scopes, filters, and quick-action buttons for results. |
| **AI** | The LLM's rationale, thought process, or raw output. | Chips act as command staging buttons for the AI's suggested shell operations. |
| **Directory** | Raw directory listing (`ls` / `cd`). | Chips dynamically populate with interactive file and folder paths for rapid prompt building. When applicable, chips also provide file or image previews. |
| **Activity** | Raw process table (`top` / `ps`). | Chips populate with immediate process-handling actions (kill, renice, monitor). For applications with displays, chips show a 10Hz live thumbnail. |


### 7.4 Dual‑Sided Chip Layout

In Level 2, the viewport features dynamic vertical chip columns floating over the terminal output (but inside the bounds of the Lateral Bezels). These chips physically manifest the Contextual Augmentations described above:

**Left Region (Context & Options):** Static or slowly changing context (Favorites, Pinned Paths, Directory Nav trees, File targets, Search Filters, Application Model hooks). Can be toggled off.

**Right Region (Priority Stack & Actions):** Highly dynamic, predictive context (Command Completions, AI-suggested commands, Actionable alerts, Process kill-switches). Driven by the Priority Indicator engine.

**Interaction:** Tapping a left‑region chip populates/stages the command or context; tapping a right‑region chip appends its action/argument at the cursor (or replaces the token). Flags that accept arguments may show secondary chips for possible values.

### 7.5 Output Area Configurations (Provided by Modules)

The appearance and behaviour of the terminal output area are fully determined by the installed Terminal Output Module. Each module can offer multiple configuration options (e.g., font size, color scheme, animation speed). The user can switch between modules via the Settings panel or a bezel shortcut.

**Built‑in Rectangular Module:**
- Full‑width rectangle, uniform text, vertical scrolling.
- Supports ANSI color rendering (configurable).
- Lines are displayed in reverse chronological order (newest at bottom).
- Clicking on a line may offer context actions (e.g., if it contains a file path, offer to open it).

**Cinematic Triangular Module (optional, installable):**
- Output lines recede toward a central vanishing point, creating a sense of depth.
- The base of the triangle (most recent line) retains the full width of the prompt.
- Previous lines progressively narrow as they scroll upward, eventually becoming too narrow to read.
- Hovering over a narrow line expands it into a tooltip with full content.
- The apex (top) of the triangle can be repositioned by the user (e.g., centered, left‑aligned, or following gaze in VR).
- An optional "pinwheel" layout for multitasking (see §7.9) arranges multiple triangular terminals radially.

### 7.6 Autocomplete Overlay

When typing in CMD mode, a temporary overlay unfurls from the right side of the top bezel, showing a comprehensive scrollable list of completions (flags, paths, commands) as chips with descriptions. Dismissed by tapping outside, Escape, or executing command.

### 7.7 Context‑Aware Mode Switching

Certain commands can trigger automatic mode switching (user‑configurable: Off / Suggest / Auto):

- Filesystem commands (`ls`, `cd`, `cp`, etc.) → Directory Mode.
- Process commands (`kill`, `ps`, `top`, etc.) → Activity Mode.

A chip may appear offering to switch, or the switch happens immediately. Visual/auditory feedback accompanies the transition.

### 7.8 Terminal Foreground Toggle

A bezel control (or keyboard shortcut) allows the user to temporarily bring the terminal output area to the foreground, overlaying the chip regions and other dynamic elements. This is useful for reviewing output without distractions. The toggle affects only the current Command Hub and does not change the underlying module.

### 7.9 Multitasking with Multiple Terminals

When using split viewports (§11), each viewport contains its own instance of the Terminal Output Module. Users can configure the layout of these instances. In addition to standard tiling, the **Cinematic Triangular Module** supports a "pinwheel" arrangement where multiple triangular terminals are arranged radially around a central point, each showing a different sector or command history. The user can rotate the pinwheel to bring a different terminal into focus. This feature is module‑specific and requires the module to implement the necessary geometry and interaction.

---

## 8. Application Focus – Level 3

Applications occupy the full viewport, with the Tactical Bezel. A **System Output** button in the bezel can overlay the system console (similar to Level 2) for quick monitoring.

### 8.1 Tactical Bezel (Configurable Slot Architecture)

By definition, the **Tactical Bezel** surrounds the entire viewport. It acts as the "bridge" between the user and the digital environment, providing a consistent set of controls and indicators regardless of the content.

- **Unified Geometry:** The Bezel is a continuous, logically unified frame. While visual segments (Top, Bottom, Left, Right) may collapse or expand independently as contextual overlays, they maintain a stable screen percentage to prevent layout jitter.
- **Configurable Slot System:**
  - **Omni-Directional Slots:** Functional slots are available in the Top (Left, Center, Right sections), Left, and Right segments. 
  - **User Assignment:** Components are not hard-coded to segments. A user may dock the Minimap to the Top Bezel or the Brain Status to the Right Bezel based on preference.
  - **Default Layout:**
    - **Left:** Hierarchy Navigation (Level 1-3), Tactical Mini-Map (§22).
    - **Right:** Priority Indicators (§21), Mini-Log Telemetry, AI Suggestion Stage.
    - **Top (Left):** Active Viewport Title.
    - **Top (Center):** Brain Connection Status, Resource Telemetry.
    - **Top (Right):** System Status Badges.
- **Slot Projection Mechanism:**
  - **Lateral Projection:** Components in Left/Right slots project horizontally into the main viewport.
  - **Vertical Projection:** Components in Top slots project **downward** into the main viewport upon activation (e.g., clicking a status badge).
  - **Constraint:** The underlying Bezel Segment remains fixed; only the projection (Glassmorphism Overlay) is dynamic.
- **Navigation Controls (Left Segment):**
  - **OVERVIEW:** Reverts the interface to Level 1.
  - **COMMAND HUB:** Navigates to Level 2.
  - **APP FOCUS:** Zooms into Level 3.
- **Top Bezel Segment:** Handles system-wide expansions, global status, and configurable tool slots (divided into Left, Center, and Right zones as per §2).
- **Bottom Bezel Segment:** Anchors the Persistent Unified Prompt (collapsed and unexpandable in Level 1, expanded in Level 2, collapsed and expandable in Level 3, and collapsed and unexpandable in Levels 4 and 5). Note: This segment contains no configurable slots.


### 8.2 Application Models

A module that customizes an application’s integration at Level 3. Provides:

- Custom bezel actions.
- Zoom behaviour (e.g., internal app zoom).
- Legacy decoration policy (Suppress, Overlay, Native).
- Thumbnail for Activity Mode.
- Searchable content (for unified search).
- Opt‑out from deep inspection.

### 8.3 Deep Inspection Access

An **Inspect** button in the expanded bezel zooms to Level 4 (Detail View) for the current application. A further zoom (Level 5) provides raw memory inspection but requires explicit privilege elevation.

---

## 9. Deep Inspection – Levels 4 & 5

### 9.1 Level 4 – Detail View

A modal overlay (slides up from bottom or expands from bezel) presenting structured metadata:

- **System Resources:** CPU, memory, uptime, network/disk I/O.
- **Event History:** Scrollable timeline of lifecycle events, commands, inspections (from TOS Log).
- **Configuration:** Environment variables, args, app settings.
- **Metadata:** Surface UUID, PID, parent, session ownership.
- **Security:** Permissions, sandbox status, audit excerpts.
- **Collaboration:** Active guests, recent guest actions.

Interactive elements (e.g., PID) can jump to Activity Mode or log searches. Export as JSON/plain text.

### 9.2 Level 5 – Buffer View

Hex dump viewer of the target surface’s process memory (read‑only). Features:

- Offset, hex, ASCII columns.
- Seek, search, export, refresh controls.
- Unavailable on Android; apps may opt out via manifest.

### 9.3 Privilege Elevation & Platform Restrictions

- Level 5 is **disabled by default**.
- Enabling requires explicit elevation (`sudo tos enable-deep-inspection` or Polkit dialog on Linux; biometric prompt on Android for Level 4 extended metadata, Level 5 generally unavailable).
- When enabled, a 🔓 indicator appears in the bezel; clicking it disables deep inspection immediately.
- All enable/disable events and Level 5 accesses are audited.

| Platform | Level 4 | Level 5 |
|----------|--------|---------|
| Linux Wayland | Full | With sudo/Polkit |
| Android XR | Partial (no raw memory) | Not available |
| Android Phone | Limited metadata | Not available |

---


## 10. Sectors and the Tree Model

A **sector** is a self‑contained workspace with its own identity, settings, and (if remote) connection. Internally, it follows a tree:

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


## 11. Split Viewports

Splitting allows a sector to display multiple viewports simultaneously, each with independent depth and content.

**Initiation:**
- From Level 3 expanded bezel: “Split” button → choose orientation → select target (New Hub, Parent Hub, or Choose Hub).
- From Level 2 Activity Mode: multi‑select app tiles → “Open in Split View” → creates tiled Level 3 viewports.

**Behaviour:**
- Each viewport independent (Level 2 or 3, any mode).
- Resizable dividers.
- Closing a viewport expands remaining ones.
- “Close Split” returns to single Level 2.

---

## 12. Remote Sectors

Remote sectors are enabled by the **TOS Remote Server** daemon on the target machine.

### 12.1 TOS Remote Server Protocol

- **Control Channel:** WebSocket/TLS, JSON‑RPC or MessagePack.
- **Video/Audio Stream:** WebRTC (H.264/H.265) with hardware decoding.
- **File Transfer:** WebDAV/HTTPS or custom protocol.
- **Authentication:** SSH keys, passwords, time‑limited tokens (Android Keystore for credential storage).

**Capabilities:**
- Full sector tree synchronisation if remote runs TOS.
- For non‑TOS machines: virtual sector with filesystem, processes, terminal.
- Fallback to SSH/HTTP if server not installed.

### 12.2 Web Portal & Live Feed Testing

Any sector or viewport can be exported as a unique URL accessible via any modern browser (WebSockets/WebRTC). Optional password or tactile approval.

**Live Feed Testing:** Real‑time streaming of TOS state and test execution (30 FPS) for observation, debugging, and demonstration. Supports multiple viewers, recording, and replay.

---


## 13. Collaboration

Collaboration is **host‑owned**: a sector resides on one host; guests connect via the host’s TOS Remote Server.

### 13.1 Host‑Owned Sharing Model

- Host invites guests via secure token or contact list.
- Guests see a synchronised view of the host’s sector tree.
- By default, each guest controls their own viewports independently.

### 13.2 Roles & Permissions

| Role       | Capabilities |
|------------|--------------|
| **Viewer** | See content only. |
| **Commenter** | Type in prompt (commands execute in restricted shell or are ignored). |
| **Operator** | Full control (launch apps, execute any command). |
| **Co‑owner** | Invite others, change roles. |

### 13.3 Visual Presence & Alerts

- Avatars in Global Overview, hub mode, and on app bezels.
- Coloured borders/cursors for each participant.
- Collaboration alerts (user join/leave, role change, hand raise) trigger visual, auditory, and haptic cues.

### 13.4 Following Mode & Chat

- Guests can follow another user’s view (viewport synchronisation).
- Lightweight chat overlay (slides in from right) with /run for command execution (subject to permissions).

### 13.5 AI in Collaboration

The AI assistant can:
- Summarize recent activity.
- Translate commands/chat between languages.
- Suggest collaboration actions.
- Explain guest intent.
- Mediate role changes.

Guests are notified if AI processes their actions.

### 13.6 Privacy & Auditing

- Guest actions are recorded in the host’s TOS Log (tagged with guest identity). Guests do not see the host’s log unless granted.
- Privacy notice shown upon joining.
- Critical events (role changes, invite usage) are written to a non‑disableable audit log.

---

## 14. Input Abstraction Layer

All physical input is normalized into **semantic events**, which are then mapped to TOS actions via a user‑configurable layer.

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

### 14.2 Device Support & Mapping

Supported devices: keyboard, mouse/trackpad, touch, game controllers, VR/AR controllers, hand tracking, gaze/eye tracking, voice, accessibility switches.

Default mappings are provided (e.g., pinch → zoom, trigger → select). Users can remap any physical action to any semantic event via a graphical configuration panel.

### 14.3 Concurrent Input & Configuration

Multiple devices can be used simultaneously; the last active device determines cursor appearance. Conflict resolution is user‑configurable.

**Configuration:** Per‑device mapping, gesture recording, voice command training, sensitivity/dead zones, profiles.

### 14.4 Accessibility Integration

- Switch scanning (single/multi‑switch).
- Sticky keys, slow keys.
- Dwell clicking (for gaze/head tracking).
- Voice commands for all actions.
- Haptic feedback as input confirmation.

---


## 15. Platform Abstraction

The core TOS logic is platform‑independent and interacts with the platform through three core traits.

### 15.1 Core Traits

```rust
pub trait Renderer {
    fn create_surface(&mut self, config: SurfaceConfig) -> SurfaceHandle;
    fn update_surface(&mut self, handle: SurfaceHandle, content: &dyn SurfaceContent);
    fn composite(&mut self);
    // bezel overlay, viewport management
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
    // notifications, clipboard, etc.
}
``` 



### 15.2 Linux Wayland Implementation

- Custom Wayland compositor with layer shell for bezel.
- Input via evdev/libinput, SDL2 for controllers, OpenXR for VR.
- Shell integration via OSC escapes (Fish reference, Bash/Zsh providers).
- Filesystem access via POSIX.
- Sandboxing via bubblewrap (optional).

### 15.3 Android XR (OpenXR) Implementation

- OpenXR 1.1 with extensions for foveation, spatial anchors, passthrough.
- Input via OpenXR actions (gaze, pinch, hand tracking) + Android touch for phone mode.
- Gemini AI for voice (on‑device transcription).
- Hardware video decoding via `MediaCodec`.
- File sync via Storage Access Framework; device‑aware (full sync on headsets, on‑demand on glasses).

### 15.4 Android Phone Implementation

- egui with Android backend or Jetpack Compose overlay.
- Touch events from Android view system.
- Voice via Google Speech Recognition.
- File access via SAF.
- Sandboxing via Android platform.

### 15.5 Native Horizon OS Client (Meta Quest)

A dedicated Android application (since Horizon OS is Android‑based) connecting to a remote TOS instance via the TOS Remote Server protocol. Architecture:

- **Connection Manager:** WebSocket/TLS control channel.
- **Rendering Engine:** WebRTC video decoded via `MediaCodec`, displayed as OpenXR texture.
- **Input Processor:** Maps Quest inputs (trigger, grip, hand tracking) to TOS semantic events.
- **File Sync Service:** Bidirectional sync with remote host (WebDAV), device‑aware.
- **Collaboration Module:** Full guest participation with avatars, following mode, alerts.
- **Local UI Overlay:** Connection status, sync progress, native menus composited on video.

---

## 16. Performance and Compositing

### 16.1 Depth‑Based Rendering & Caching

- Only focused level receives full frame rate; background levels are static textures or throttled.
- Texture caching for thumbnails; GPU memory pruning for surfaces more than two levels away.
- Hardware acceleration (OpenGL ES / Vulkan).

### 16.2 Intelligent View Synchronization

To prevent flicker during high‑frequency updates (e.g., telemetry):

- HTML diffing – skip DOM update if payload identical.
- Animation suppression on core structural elements.
- State preservation (input fields, scroll positions) across refreshes.
- Throttled backgrounds (1–5 Hz) for non‑focused viewports.

### 16.3 Tactical Alert (Performance Warning)

If frame rate drops below target (e.g., 60 FPS desktop, 90 FPS VR) for >2s, a non‑intrusive alert appears (visual, optional auditory/haptic) showing current FPS and suggestions.

---

## 17. Security Model

### 17.1 Authentication & Authorization

- Local: PAM (Linux), Android Keystore + biometric (Android).
- Remote: SSH keys, passwords, time‑limited tokens; mutually authenticated TLS.
- RBAC roles (Viewer, Commenter, Operator, Co‑owner) enforced host‑side.

### 17.2 Process Isolation & Sandboxing

- **Applications:** Run as the user’s own processes. Optional sandboxing via Containerization (Docker, Podman)/Flatpak/Firejail/bubblewrap/appimage (Linux) or Android platform sandbox.
- **Standard Modules:** Application Models, AI backends, Terminal Output Modules, and Theme Modules run in strictly sandboxed processes with declared permissions.
- **Trusted System Modules:** Shell Modules and certain Sector Types (if using native code) are considered "Trusted." They run with the user's full shell privileges and have access to the PTY, enabling them to execute system commands directly without the sandbox overhead.

### 17.3 Dangerous Command Handling (Tactile & Voice Confirmation)

Commands marked as high‑risk trigger a modal overlay requiring a **Tactile Confirmation Slider** (drag 100% to right). For devices without touch/drag capabilities or users with motor impairments, TOS provides **Voice Confirmation** (speaking a unique, time‑limited passphrase displayed on screen) or **Secure Biometric Prompt** (Android). Shortcuts like `Ctrl+Enter` can bypass only if explicitly permitted in the security policy. This mechanism is also used for confirming sector closure when tasks are running (see §6.5).

### 17.4 Deep Inspection Privilege

Level 5 access is disabled by default; requires explicit elevation (sudo/Polkit on Linux; not available on Android). Audited.

### 17.5 Auditing

- All commands, security events, role changes, and deep inspection accesses are logged.
- Critical events go to a non‑disableable audit log (Linux: `/var/log/tos/`, Android: app‑private).

---

## 18. Modules: Application Models, Sector Types, AI Backends, Terminal Output, Themes, Shells, and Bezel Components

Modules are platform‑specific plugins (`.so` on Linux, `.apk` or dynamic modules on Android) that extend TOS functionality. 

TOS employs a dual‑tier trust model for modules:
1. **Standard Tier (Sandboxed):** Most modules run in an isolated environment and must declare required permissions in a manifest (`module.toml`). 
2. **System Tier (Trusted):** Shell Modules and native Sector Types are trusted by the user and run without TOS‑enforced sandboxing to ensure full local system access.

### 18.1 Application Model

Customizes an application’s integration at Level 3. Manifest includes: name, version, type = "app-model", icon, permissions, capabilities (bezel actions, searchable content, etc.).

### 18.2 Sector Type

Defines a sector’s default behaviour: command favourites, interesting directories, environment, available hub modes, default guest role, associated Application Models.

### 18.3 AI Backend Modules

Package type `.tos-ai`. Manifest includes: capabilities (chat, function_calling, vision, streaming), connection details (protocol, endpoint, auth_type), permissions, configuration options (model, temperature, etc.).

Example:
```toml
name = "OpenAI GPT-4"
version = "1.0.0"
type = "ai-backend"
capabilities.chat = true
connection.default_endpoint = "https://api.openai.com/v1/chat/completions"
auth_type = "api-key"
permissions.network = ["api.openai.com"]
``` 


### 18.4 Module Isolation & Permissions

- Modules run in sandbox with limited access (network filtered, filesystem restricted).
- Permissions are displayed to user during installation; user grants/denies.
- Dangerous capabilities (e.g., local file access) require explicit consent.

### 18.5 Terminal Output Modules

Terminal Output Modules are a new class of installable components that define how terminal output is visually presented within Command Hubs and the System Output Area at Level 1. They are packaged as `.tos-terminal` files.

#### 18.5.1 Module Interface

A Terminal Output Module must implement a well‑defined interface (Rust trait or FFI) that allows the Face to:

- Initialize a new instance for a given **context** (sector terminal or system output). The context determines whether the instance is interactive (accepts input and emits interaction events) or read‑only (for system background).
- Receive a stream of lines, each with metadata:
  - Text content (UTF‑8, receives raw output including ANSI codes; the module is responsible for rendering or stripping them as needed).
  - Timestamp.
  - Exit status of the command that produced the line (if applicable).
  - Whether the line is part of a command echo or output.
  - Priority/importance level (for highlighting).
- **Render the output:** Modules render to a platform-appropriate surface provided by the Face.
  - **Web Profile:** Render into a sandboxed DOM element.
  - **Native Profile:** Render into a shared-memory buffer (e.g., DMABUF on Linux or EGLImage on Android) which the Face then composites into the GPU pipeline. Direct raw GPU access is prohibited for sandboxed modules.
- Handle user interactions (only if context is interactive):
  - Click/tap on a line (with coordinates) → return line index and optional context actions.
  - Scroll requests (delta, to top/bottom).
  - Hover events (for tooltips).
- Provide configuration options (exposed via the Settings Daemon).

The Face is responsible for compositing the rendered output with chip regions, bezel, and other overlays.

#### 18.5.2 Built‑in Module: Rectangular Terminal

TOS includes a default Rectangular Terminal Module, implemented natively for performance. It provides a traditional terminal experience and serves as the fallback if no other module is installed.

#### 18.5.3 Optional Module: Cinematic Triangular Terminal

An example community‑developed module that implements the cinematic triangular perspective. It may offer additional features like:
- Adjustable vanishing point.
- Animation speed and easing.
- Pinwheel multitasking layout.
- Integration with priority indicators (e.g., highlighting important lines with glow).

#### 18.5.4 Installation and Switching

- Users browse the Marketplace for Terminal Output Modules.
- After installation, the module appears in the Settings panel under "Appearance → Terminal Output".
- Users can select the active module globally, or per‑sector (if the module supports it).
- Switching modules takes effect immediately in all open Command Hubs (existing terminal history is re‑rendered by the new module).

### 18.6 Theme Modules

Theme Modules define the visual appearance of TOS across all levels. They are packaged as `.tos-theme` files and control:

- **Color palette:** Background, text, borders, chips, priority indicators, alert colors, and all LCARS‑style elements.
- **Typography:** Font family, sizes, weights, line spacing for terminal output, chip labels, bezel text.
- **Iconography:** Custom icon sets for mode indicators, bezel controls, status dots, and other UI elements.
- **Optional audio integration:** A theme may reference an accompanying audio theme (`.tos-audio`) to provide a cohesive sensory experience, but audio themes are separate modules for modularity.

**Manifest example (`module.toml`):**
```toml
name = "Star Trek: TNG"
version = "1.0.0"
type = "theme"
description = "Classic LCARS color scheme from The Next Generation"
author = "TOS Community"
icon = "tng.png"

[assets]
css = "theme.css"               # Main stylesheet
fonts = ["lcars.ttf"]            # Optional custom fonts
icons = "icons/"                 # Directory with SVG icons

[capabilities]
supports_high_contrast = true    # Theme can adapt to high‑contrast mode
supports_reduced_motion = true   # Respects reduced‑motion setting
```

Interface: The Face applies the theme by loading the manifest and assets. The CSS file defines CSS custom properties (variables) that are injected into the UI's root. Icons are referenced by name and loaded from the module's asset directory. The theme may also provide JavaScript hooks for dynamic theming (e.g., animated transitions).

Permissions: Typically none, as themes are static assets. If a theme includes custom fonts or icons, they are bundled and do not require additional permissions. However, if a theme wishes to access external resources (e.g., web fonts), it must declare network permissions.

Installation and switching:

· User browses the Marketplace for Theme Modules.
· After installation, the theme appears in Settings → Appearance → Theme.
· Users can select the active theme globally; per‑sector theme overrides are possible if the theme supports it (via the sector's settings).
· Switching themes takes effect immediately (UI reloads with new styles).

Built‑in themes: TOS ships with at least two default themes: a light and a dark variant of the LCARS design, plus a high‑contrast accessibility theme. 
### 18.7 Shell Modules

Shell Modules provide different shell implementations that can be used within Command Hubs. They are packaged as `.tos-shell` files and include:

- The shell executable (or a wrapper script) that TOS will spawn for each sector's PTY.
- Integration scripts to enable OSC communication (as defined in §27).
- Default configuration files (e.g., `.bashrc`, `.zshrc`, `config.fish`) that set up aliases, prompt, and environment variables.
- Metadata describing the shell's capabilities (e.g., supports directory notifications, command result capture, etc.).

**Manifest example (`module.toml`):**
```toml
name = "Zsh"
version = "5.9"
type = "shell"
description = "Z shell with powerline support"
icon = "zsh.png"

[executable]
path = "bin/zsh" # Relative path within module
args = ["--login"] # Default arguments

[integration]
osc_directory = true # Supports OSC 1337;CurrentDir
osc_command_result = true # Supports OSC 9002 (with base64)
osc_suggestions = false # (future) Supports command suggestions

[configuration]
default_env = { LANG = "en_US.UTF-8" }
rc_file = "etc/zshrc" # Default rc file to source
```

Interface: The Brain, when creating a new sector's Command Hub, reads the selected shell module, spawns the executable with the given arguments, and attaches the PTY. The shell's output is fed to the Terminal Output Module, and input from the prompt is written to the PTY. The Brain also listens for OSC sequences emitted by the shell and updates state accordingly (e.g., directory changes, command results).

Permissions: Shell modules run as user processes with the same privileges as any shell. They are not sandboxed by TOS (the user's shell is trusted). However, if a shell module includes additional binaries or scripts, they inherit the user's permissions. The module may declare permissions for documentation purposes only.

Installation and switching:

· User installs shell modules from the Marketplace.
· The default shell can be set in Settings → System → Default Shell.
· Per‑sector shell selection is possible via Sector Overrides (if the sector type allows or user overrides).
· Switching shells for an existing sector requires a sector reset (or creating a new hub).

Built‑in shell: TOS includes a reference shell module (Fish) with full OSC integration. Additional modules (Bash, Zsh) are available via the Marketplace, with community‑maintained integration scripts.



### 18.8 Bezel Component Modules

Bezel Components are modular UI elements that can be installed via the marketplace and docked into any available **Tactical Bezel Slot** (Top, Left, or Right). Note that the Bottom Bezel (Prompt Segment) is a static assembly and does not support component docking. They utilize the **Slot Projection** mechanism (§8.1) to expand their presence into the viewport when triggered.

The following components are currently defined in the core system:

1. **Tactical Mini-Map:** Provides high-level spatial overview, sector topology monitoring, and rapid teleportation (§22). Default: Left Segment Slot.
2. **Priority Indicator:** Features dynamically ranked system alerts and notification badges (§21). Default: Right Segment Slot.
3. **Resource Telemetry:** Monitoring tool for system performance metrics. Default: Top Segment Slot.
4. **Collaboration Hub:** Manages multi-user presence and session-sharing (§12).
5. **System Clock & Calendar:** Synchronized temporal anchor with integrated task overlays.
6. **Mini-Log Telemetry:** Persistent readout of authoritative system state and command acknowledges. Default: Right Segment Slot.
7. **Media Controller:** Global playback controls for system audio and remote stream synchronization.
8. **Active Viewport Title:** Modular readout of current Hierarchical Level, Sector Name, or Application Focus. Default: Top Bezel (Left).
9. **Brain Connection Status:** Real-time indicator of the local/remote Brain link state, including heartbeats and latency. Default: Top Bezel (Right).
10. **System Status Badges:** Configurable array of toggles and indicators (e.g., Terminal visibility, Collaboration dots, Sandboxing toggles).

### 18.9 Relationship with Other Modules

- **Sector Types** may specify a preferred shell (e.g., a development sector might default to Zsh).
- **Application Models** are shell‑agnostic; they interact with the Brain, not directly with the shell.
- **Terminal Output Modules** render the shell's output, regardless of which shell is used.
- **Theme Modules** affect the appearance of all UI elements, including terminal output.
- **AI Backend Modules** can be invoked from the command line (via the AI mode) and their responses appear in the terminal output.

All modules coexist within the modular service architecture, communicating with the Brain via IPC. The Brain coordinates the instantiation and lifecycle of each module type, ensuring that permissions are enforced and that modules are properly sandboxed.

---

## 19. TOS Log

Every surface maintains its own event history, collectively forming a system‑wide timeline. The TOS Log service is responsible for aggregating logs from local user activity, background services, and remote sector events.

### 19.1 Recorded Events & Unified Storage
The **Global TOS Log Sector** (§19.2) provides a unified view of all events, regardless of origin. It transparently merges:
- **User Logs:** `~/.local/share/tos/logs/` (Standard activity).
- **System Audit Logs:** `/var/log/tos/` (Privileged events, filtered by current user's visibility).
- **Remote Logs:** Captured from the TOS Remote Server and cached locally during the session.

| Event Type | Examples |
|------------|----------|
| Lifecycle | Creation, focus, move, close |
| Commands | Executed commands with exit status, duration |
| Inspections | Level 4/5 views accessed |
| Telemetry | Periodic resource snapshots (if enabled) |
| Collaboration | User joins/leaves, role changes, guest actions |
| System Events | Notifications, alerts, security events |
| Priority Changes | Score changes and contributing factors |
| AI Interactions | Queries and responses (if enabled) |

### 19.2 Access Methods

- **Per‑Surface (Level 4):** Scrollable timeline in Detail View.
- **Global TOS Log Sector:** A dedicated Sector/Command Hub (Level 2) providing full interactive filtering, searching, and exporting of the system-wide timeline. While the Level 1 System Output Area (§6.2) provides a passive, live view of system logs, the Log Sector allows for deep retrospection and forensic analysis.
- **Prompt Queries:** Commands like `log --surface browser --since 10min`.

### 19.3 OpenSearch Compatibility

- OpenSearch description document for browser address bar queries.
- Optional forwarding to OpenSearch cluster (user consent required).

### 19.4 Privacy & Retention

- Master toggle to enable/disable logging (except critical audit events).
- Per‑surface opt‑out, retention policies, regex‑based redaction.
- Logs stored locally in `~/.local/share/tos/logs/` (JSON Lines or SQLite).

---

## 20. Tactical Reset

Two‑level emergency recovery.

### 20.1 Sector Reset

- **Trigger:** `Super+Backspace`, `tos sector reset`, bezel button, voice.
- Sends SIGTERM to all processes in current sector, closes splits, returns to fresh Level 2.
- Optional undo button (5s) if enabled.

### 20.2 System Reset

- **Trigger:** `Super+Alt+Backspace`, `tos system reset`, bezel button (Level 1).
- Dialog with three options: Restart Compositor, Log Out, Cancel.
- Requires tactile confirmation (hold, slider, voice) + countdown.
- All attempts are audited.

---

## 21. Priority‑Weighted Visual Indicators

Non‑intrusive indicators convey relative importance without altering size or position.

### 21.1 Indicator Types

| Type | Description |
|------|-------------|
| **Border Chips** | Small coloured notches along tile border; number reflects priority level (1–5). |
| **Chevrons** | LCARS arrows; pulsing indicates pending notification or critical status. |
| **Glow / Luminance** | Subtle inner/outer glow; intensity varies with priority. |
| **Status Dots** | Small coloured circles (blue=normal, yellow=caution, red=critical). Multiple dots can appear. |

### 21.2 Priority Scoring & Configuration

Weighted factors (user‑configurable):
- Recency of focus (40%)
- Frequency of use (20%)
- Activity level (CPU, memory, I/O) (15%)
- Notification priority (10%)
- User pinning (override)
- Collaboration focus (10%)
- Sector‑specific rules
- AI suggestion (5%)

Score maps to indicator configuration (e.g., low = no chips, critical = 4 chips + pulsing chevron + red glow).

**Configuration:** Master toggle, colour per factor, sensitivity, per‑factor visibility, hover tooltips.

### 21.3 Behaviour by Depth

- Level 1: Sector tiles show aggregate priority.
- Level 2: Application tiles show individual priority; chip regions use indicators.
- Level 3: Bezel may show priority chevron/glow; split viewport borders.
- Level 4/5: Inspection panels show inspected surface’s priority and sibling mini‑map.

---

## 22. Tactical Mini‑Map

Ephemeral overlay providing spatial awareness.

### 22.1 Passive & Active States

- **Passive:** Semi‑transparent, input passes through.
- **Active:** Activated by hover (dwell), keyboard (`Ctrl+M`), modifier+click, double‑tap, voice. Captures input; shows close button.

### 22.2 Content by Depth

- Level 1: All sectors as miniature tiles.
- Level 2: Current sector with hubs, active hub highlighted.
- Level 3: Focused app highlighted, other viewports shown.
- Level 4/5: Current surface and siblings.

### 22.3 Monitoring Layer (Resource Usage)

Optional overlay (toggle) showing live resource usage:

- Level 1: Aggregated CPU/memory per sector.
- Level 2: All apps with CPU%, memory%, sparkline.
- Level 3: Detailed stats for focused app + compact for others.
- Throttled to 1–2 Hz.

### 22.4 Bezel Integration (Slot Projection)

The Tactical Mini-Map is docked within a slot in the **Left Bezel Segment**.
- **Docked State:** Occupies the 1.5rem width of the left bezel, showing only high-alert status lines.
- **Projected State:** When activated (e.g., `Ctrl+M`), it projects a wide glassmorphism overlay into the center of the screen without expanding the sidebar.
- **Contextual Anchors:** Clicking tiles within the projected overlay triggers immediate level transitions.

---

## 23. Auditory and Haptic Interface

### 23.1 Three‑Layer Audio Model

| Layer | Purpose | Characteristics |
|-------|---------|-----------------|
| **Ambient** | Atmosphere | Continuous, depth‑varying background. |
| **Tactical** | Action confirmation | Discrete earcons for zoom, commands, notifications, alerts, collaboration. |
| **Voice** | Speech synthesis | TTS for announcements, screen reader, AI responses. |

Each layer has independent volume and enable/disable.

### 23.2 Context Adaptation (Green/Yellow/Red Alerts)

- **Green:** Normal.
- **Yellow:** Ambient shifts urgent, tactical adds periodic pulse, voice more verbose.
- **Red:** Ambient replaced by repeating tone; tactical suppresses non‑critical earcons; voice prioritises critical messages.

### 23.3 Spatial Audio (VR/AR)

Sounds positioned in 3D space (e.g., notifications from left sector sound left).

### 23.4 Haptic Feedback Taxonomy

Mapped from semantic events; examples:

| Event | Pattern |
|-------|---------|
| `zoom_in` | Ascending pulses |
| `select` | Quick click |
| `dangerous_command` | Sharp, insistent buzz |
| `red_alert` | Pulsing, escalating |

Spatial haptics in VR/AR (directional).

### 23.5 Theming & Configuration

- Audio themes (`.tos-audio`) installable via Marketplace (see §18.6 for relationship with Theme Modules).
- Applications can contribute custom sounds.
- Audio themes are managed as separate modules to ensure a modular sensory experience.
- Configuration: master volume, per‑category toggles, test patterns, hearing‑impaired mode (route tactical to haptics).

---

## 24. Accessibility

### 24.1 Visual

- High‑contrast themes, font scaling, colourblind filters.
- Screen reader support (AT‑SPI/Orca on Linux, TalkBack on Android).
- Braille display support.
- Focus indicators (thick border, optional haptic/auditory).

### 24.2 Auditory

- Screen reader via Voice layer.
- Earcons for navigation and feedback.
- Voice notifications (TTS) with adjustable verbosity.

### 24.3 Motor

- Switch device support (single/multi‑switch scanning, linear/row‑column).
- Dwell clicking (gaze/head tracking).
- Sticky keys, slow keys.
- **Voice Confirmation:** Users can confirm "Dangerous Commands" via speech using a randomized challenge‑response system if a tactical slider is physically impossible.
- Haptic confirmation for actions.
- Customisable input mapping.

### 24.4 Cognitive

- Simplified mode (reduced clutter, larger elements, limited features).
- Built‑in tutorials (eval‑help mapping, interactive guides).
- Consistent spatial model (three levels, three modes).

### 24.5 Profiles & Platform Integration

- Central Accessibility panel with profiles (save/load/export).
- Per‑sector overrides.
- Integration with platform accessibility services (AT‑SPI, TalkBack, Switch Access).

---

## 25. Sector Templates and Marketplace

### 25.1 Package Types & Manifests

- **Sector Template** (`.tos-template`): Configuration only.
- **Sector Type** (`.tos-sector`): Module with code.
- **Application Model** (`.tos-appmodel`): Module.
- **AI Backend** (`.tos-ai`): Module.
- **Terminal Output Module** (`.tos-terminal`): Module.
- **Theme Module** (`.tos-theme`): Assets and CSS.
- **Shell Module** (`.tos-shell`): Executable and scripts.
- **Audio Theme** (`.tos-audio`): Sounds.

Manifest (`module.toml`) includes name, version, type, icon, permissions, dependencies, configuration schema.

### 25.2 Installation Flow & Permissions

1. Discovery (Search, Marketplace, direct file open).
2. Details panel with description, permissions, dependencies.
3. Permission review (user grants/denies; optional session‑only grant).
4. Dependency resolution.
5. Installation (files copied to `~/.local/share/tos/` or equivalent).
6. Post‑install notification; immediate availability.

### 25.3 Discovery (Search, AI, Updates)

- Search Mode includes packages as a domain.
- AI‑assisted discovery (“I need a Git integration”).
- Update alerts (Yellow Alert) for installed modules; update details show permission changes.

### 25.4 Creating & Sharing Packages

- Export sector as template.
- Developer tools for packaging modules.
- Submission to repositories (optional signature verification).

---

## 26. Settings Data Model & IPC

### 26.1 Layered Settings

Settings are resolved via cascade: **per‑application > per‑sector > global key‑value bag > global scalar field defaults**.

Global scalar fields (e.g., `fps`) are native; global key‑value bag (`state.settings: HashMap<String, String>`) holds canonical keys and extensions.

### 26.2 Canonical Keys & Defaults (extended)

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `theme` | string | `"lcars-light"` | Active theme module ID. |
| `default_shell` | string | `"fish"` | Default shell module ID for new sectors. |
| `terminal_output_module` | string | `"rectangular"` | Active terminal output module ID. |
| `master_volume` | `"0"–"100"` | `"80"` | Master audio volume. |
| `logging_enabled` | `"true"`\|`"false"` | `"true"` | Master log toggle. |
| `deep_inspection` | `"true"`\|`"false"` | `"false"` | Enable Level 5. |
| `terminal_buffer_limit` | integer | `500` | Maximum lines to keep in terminal buffer (FIFO). |
| ... (full list as per earlier design) | | | |

### 26.3 IPC Messages for Settings (extended)

- `open_settings`, `close_settings`
- `set_fps:<value>`, `set_master_volume:<value>`
- `set_theme:<module_id>` – Switch theme.
- `set_default_shell:<module_id>` – Set default shell.
- `set_terminal_output_module:<module_id>` – Set terminal output module.
- `toggle_sandboxing`
- `enable-deep-inspection`, `disable-deep-inspection`
- `set_setting:<key>;<value>` (Standardized with semicolon)
- `set_sector_setting:<key>;<value>`
- `set_terminal_buffer_limit:<value>` – Adjust terminal history cap.
- `settings_tab:<tab>` (for modal navigation)

### 26.4 Persistence

Settings saved to `~/.config/tos/settings.json` (Linux) or app‑private storage (Android) as JSON. Debounced writes (≤1s). Only canonical keys and extensions are persisted; runtime‑only state (e.g., `settings_open`) is skipped.

---

## 27. Shell API Enhancements

### 27.1 `command_result` Payload Format

OSC `9002` payload extended to three semicolon‑delimited fields:

```

<command>;<exit_status>;<base64(stdout+stderr)>

```

Base64 encoding prevents control characters from breaking OSC parsing. Third field optional for backwards compatibility.

### 27.2 Shell Integration Script Requirements

- Capture full combined stdout/stderr of each command.
- Base64‑encode and emit `ESC]9002;<command>;<exit_status>;<base64>BEL`.
- Also emit `ESC]9003;<cwd>BEL` on directory change.
- Do not capture TOS‑internal commands (`EXEC`, `CD`, etc.).

### 27.3 Fallback: Raw PTY Output & Filesystem Fallback

- Without integration, PTY reader strips ANSI, splits lines, caps at 500 lines (default; user‑adjustable).
- **Local Directory Mode:** Falls back to `std::fs::read_dir` if `hub.shell_listing` is `None`.
- **Remote Directory Mode:** If `hub.shell_listing` is missing, the Brain attempts to fetch listing data via the **TOS Remote Server's File Service** (§12.1). 
  - **Graceful Fallback:** If the remote server is not installed (e.g., raw SSH link), Directory Mode visuals are disabled and the interface stays in standard Shell output.
  - **Connection Loss:** If an active remote server connection is lost or the session logs out unexpectedly, the sector will display a "Remote session disconnected" banner and close after 5 seconds of inactivity.
- `DirectoryChanged` PTY event (OSC `1337;CurrentDir=`) updates current directory.

### 27.5 Line‑Level Priority (Importance) Metadata

To support the Terminal Output Module’s highlighting capabilities (§18.5.1), the shell can emit a priority sequence before producing a line of output:

```
ESC]9012;<level>BEL
```

**Levels:**
- `0`: Normal / Inherit (Default)
- `1`: Low (Verbose/Diagnostic)
- `2`: Notice (Success/Milestone)
- `3`: Warning (Recoverable error)
- `4`: Critical (Panic/Failure)
- `5`: Urgent (Tactile/Auditory trigger)

Once received, the Brain applies this level to all subsequent lines until a new 9012 sequence is received or the command completes.

### 27.6 Command Auto‑Detection (`ls`, `cd`)

- If submitted command starts with `ls` (case‑insensitive), resolve target path, set `hub.current_directory`, switch to Directory Mode, clear stale listing.
- If starts with `cd`, resolve target path, set `hub.current_directory` (if exists), do not change mode.
- No false positives (`rls`, `echo cd`).

### 27.5 Directory Mode Pick Behaviour

| Prompt state | Item type | Click action |
|--------------|-----------|--------------|
| Empty | File | Insert absolute path into prompt |
| Empty | Folder | Navigate into folder |
| Command staged | File | Append absolute path as next argument |
| Command staged | Folder | Append absolute path as next argument |

Breadcrumb and `..` always navigate.

**Staging banner** appears above file grid when command staged, with current command and hint. Items get pick‑mode visual treatment (amber border). Multi‑select appends all selected paths in order.

IPC messages: `dir_pick_file:<name>`, `dir_pick_dir:<name>`, `dir_navigate:<path>`.

---

## 28. Bezel IPC Contracts

### 28.1 Action‑Identifier Rule

All IPC messages from bezel buttons and UI controls must use **action identifiers**, not display labels. Labels are for rendering only and must not be forwarded to the shell.

✅ Correct: `<button onclick="window.ipc.postMessage('zoom_out')">ZOOM OUT</button>` 
❌ Incorrect: `<button onclick="window.ipc.postMessage(this.innerText)">ZOOM OUT</button>`

**Prompt Interception Layer:** 
While the Action-Identifier rule is strict for UI controls, the `prompt_submit:` message is an exception. The Brain performs a "sniffing" pass on the submitted string (e.g., detecting `ls` or `cd` to trigger mode switches). This logic lives entirely in the Brain's command dispatcher and does not violate the Face-side identifier rule.

### 28.2 Reserved IPC Prefixes

| Prefix | Purpose | Payload Delimiter |
|--------|---------|-------------------|
| `prompt_submit:` | Submit prompt value to PTY | N/A |
| `prompt_input:` | Update staged prompt text | N/A |
| `stage_command:` | Pre‑populate prompt | N/A |
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
| `update_confirmation_progress:` | Tactile slider progress (id;val) | Semicolon (`;`) |
| `set_terminal_buffer_limit:` | Adjust terminal history cap | N/A |

Unknown messages are logged and ignored (not forwarded to PTY).

---

## 29. Terminal Output Rendering

### 29.1 ANSI Stripping

Before storage in `hub.terminal_output`, strip:
- CSI sequences (`ESC[ ... m`)
- OSC sequences (`ESC] ... BEL` or `ESC\`)
- C0/C1 controls except TAB, LF, CR.

Result must be valid printable UTF‑8.

### 29.2 Buffer Limits & Rendering Requirements

- Cap at 500 lines (FIFO) by default; user‑adjustable via `terminal_buffer_limit` setting.
- Monospace font, `white-space: pre-wrap`.
- Auto‑scroll to latest line.
- Distinct styling for command echo (`> command`) vs output.

---

## 30. Implementation Roadmap (Terminal‑First Prioritisation)

1. **Core Terminal Integration (Brain + PTY + Shell API)** – Establish basic terminal functionality using the **Standardized IPC Format** (§3.3.1). Implement bidirectional OSC communication, including **Line‑Level Priority** (OSC 9012) and the Brain's own console output stream.
2. **Basic Face (UI) & IPC Foundation** – Minimal webview implementing the **Action‑Identifier Scheme** (§28).
   - At Level 1: sector tiles + System Output Area (Brain console) via the built‑in Rectangular Terminal Module.
   - At Level 2: interactive terminal output + Persistent Unified Prompt.
3. **Modular Trust & Sandboxing Architecture** – Implement the **Dual‑Tier Trust Model** (§18). Establish the secure sandbox runtime for Standard Modules and the privileged execution path for Trusted System Modules (Shells).
4. **Input Hub & Semantic Events** – Normalise raw input into semantic actions (§14); implement **Voice Confirmation** fallback for dangerous actions (§17.3).
5. **Sector Concept & Management** – Introduce multiple sectors. Implement the **Sector Tile Context Menu** (§6.5) with tactile confirmation and sector lifecycle controls (freeze, close, clone).
6. **Directory Mode (Local & Remote)** – Integrated as a terminal overlay. Implement **Remote Directory Fallback** (§27.3) with TOS Remote Server integration and SSH fallback logic.
7. **Activity Mode** – Visual process management via `ps` parsing.
8. **SEARCH Mode** – Unified search domain integration.
9. **Terminal Output Module API** – Define interface for high‑speed rendering (Web Profile DOM / Native Profile DMABUF). Support metadata‑driven highlighting based on line priority.
10. **Theme Module API** – CSS variable injection and multi‑sensory asset loading.
11. **Shell Module API** – Executable spawning and OSC integration scripts (Fish/Bash/Zsh).
12. **AI Engine** – Natural language processing and staged command generation.
13. **Marketplace & Module Discovery** – Package management and permission-based installation for all `.tos` types.
14. **Auxiliary Services** – **Unified TOS Log Service** (§19), Settings Daemon (with cascading persistence), and Audio/Haptic Engine.
15. **Remote Sectors & Session Management** – Implement TOS Remote Server protocol. Add **Connection Loss Logic** (§27.3) with disconnection banners and auto‑close timers.
16. **Platform Backends** – Native implementations for Linux Wayland, Android, and OpenXR (Meta Quest).
17. **Collaboration** – Host‑owned sharing, presence synchronization, and guest role enforcement.
18. **Optional High‑Fidelity Modules** – Cinematic Triangular Module, community themes, and advanced shell plugins.

---

## 31. Conclusion

By unifying the terminal output experience across the Global Overview and Command Hubs through the same modular system, TOS reinforces its terminal‑first identity while enabling unprecedented customisation. The System Output Area at Level 1 serves as a dynamic, readable window into the Brain's operations, placed as a distinct layer that can be toggled to the front for detailed inspection. The addition of a sector tile context menu empowers users to manage their workspaces quickly and safely, with tactile confirmation for destructive actions. Combined with theme and shell modules, TOS offers a deeply personalisable yet coherent environment – a true "Terminated On Steroids" for the modern era. All future development should reference this document as the single source of truth. 




