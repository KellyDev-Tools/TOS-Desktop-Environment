# TOS (Tactical Operating System) – Unified Architectural Specification

### Version 1.0 (Consolidated)

---

# Table of Contents

1. [Core Philosophy](#1-core-philosophy)
2. [The Three-Level Hierarchy](#2-the-three-level-hierarchy)
3. [Command Hub: Three Modes](#3-command-hub-three-modes)
4. [Tactical Bezel](#4-tactical-bezel)
5. [Sectors and the Tree Model](#5-sectors-and-the-tree-model)
6. [Split Viewports](#6-split-viewports)
7. [Remote Sectors](#7-remote-sectors)
8. [Collaboration](#8-collaboration)
9. [Input Abstraction Layer](#9-input-abstraction-layer)
10. [Performance and Compositing](#10-performance-and-compositing)
11. [Security Model](#11-security-model)
12. [Application Models and Sector Types](#12-application-models-and-sector-types)
13. [Shell API](#13-shell-api)
14. [Tactical Reset](#14-tactical-reset)
15. [Sector Templates and Marketplace](#15-sector-templates-and-marketplace)
16. [Accessibility](#16-accessibility)
17. [Tactical Mini‑Map](#17-tactical-mini‑map)
18. [Auditory Interface](#18-auditory-interface)
19. [Implementation Roadmap](#19-implementation-roadmap)

---

## 1. Core Philosophy

TOS (Tactical Operating System) is a reimagining of the Linux desktop, inspired by the LCARS interface from Star Trek. It replaces traditional window management with a **recursive zoom hierarchy** centered on a **command‑first** philosophy. The environment is **input‑agnostic**, supporting touch, mouse, keyboard, voice, game controllers, VR/AR controllers, hand tracking, eye tracking, and accessibility switches. It scales from embedded IoT devices to collaborative distributed workspaces through a spatial command platform.

All interactions are organized into a strictly vertical, three‑level structural hierarchy, with a tree‑like organization of **sectors**, **command hubs**, and **applications**. Navigation is achieved by zooming through these layers, with a persistent focus on terminal‑driven intent and graphical augmentation.

---

## 2. The Three‑Level Hierarchy

| Level | Name                 | Description |
|-------|----------------------|-------------|
| **1** | **Global Overview**  | Bird’s‑eye view of all sectors (local and remote). Sectors appear as zoomable tiles. |
| **2** | **Command Hub**      | Central control point for a sector. Contains the **Persistent Unified Prompt** and three toggleable modes (Command, Directory, Activity). |
| **3** | **Application Focus**| Full‑screen (or tiled) application surface, wrapped in the **Tactical Bezel**. |

Navigation is consistent across input methods: pinch/scroll/trigger to zoom, tap/click/select to choose. The hierarchy is strictly enforced, but the system may learn frequent paths and offer predictive zoom with a “ghost” animation that visually traverses intermediate levels.

---

## 3. Command Hub: Three Modes

The Command Hub (Level 2) is the exclusive home of the **Persistent Unified Prompt**. It provides three modes, switchable via a three‑way toggle (button, keyboard shortcut, gesture, voice):

### 3.1 Command Mode (CLI‑Centric)
- **Prompt** at the bottom (always visible).
- **Suggestion area**: eval‑help chips, command history, favourites.
- **Auto‑complete** dropdown as the user types.
- Selections from other modes populate the prompt.

### 3.2 Directory Mode (File Manager)
- **Path bar** (breadcrumb style).
- **Grid/list view** of files and folders (respects hidden‑file conventions, with “Show Hidden” toggle).
- **Selection controls** for multi‑select (checkbox, lasso, Ctrl+click).
- **Action toolbar** (New Folder, Copy, Paste, etc.) – buttons construct the corresponding CLI command.
- **Integration with prompt**: selecting a file appends its path; multi‑select appends all paths.
- **Context menu** (right‑click/long press) for file‑specific actions.

### 3.3 Activity Mode (Process/App Manager)
- **Tactical grid** of all running applications within the current sector, organized hierarchically by command hub.
- Each tile shows icon, title, optional live thumbnail, and status indicators (PID, CPU/memory on hover).
- **Multi‑select** for batch actions (close, kill, move).
- **Integration with prompt**: selecting a tile populates the prompt with PID/window ID; contextual chips suggest relevant commands (kill, focus, etc.).
- Clicking a tile zooms to Level 3 (Application Focus).

---
## 4. Tactical Bezel

The Tactical Bezel is an **immutable system overlay** rendered by the compositor at Level 3. It serves as both the guaranteed navigation escape and the unified window decoration for all applications.

### 4.1 Default (Collapsed) State
- Thin, semi‑transparent strip along the top edge (user‑configurable position).
- Contains:
  - **Zoom Out** button (returns to Level 2).
  - Application icon and title.
  - **Expand handle** (down chevron).

### 4.2 Expanded State
Activated by dragging down the handle, clicking/tapping it, or using a keyboard shortcut (`Ctrl+Space`). Reveals a command strip with:

- **Navigation**: Zoom Out, Split View, Teleport, Close Application.
- **Window Controls**: Minimize, Full‑screen Toggle, Always on Top (if applicable).
- **Application‑Specific Actions** (provided by the Application Model).
- **System‑Wide Shortcuts**: Open Command Hub, Toggle Mini‑Map, Settings.
- **Collaboration Indicators**: Avatars, share button.

### 4.3 Legacy Application Handling
- For X11/non‑native apps, TOS suppresses the app’s own decorations where possible, or overlays the bezel on top.
- The Application Model defines the decoration policy (Suppress, Overlay, or Native).

---

## 5. Sectors and the Tree Model

A **sector** is a self‑contained workspace with its own identity, settings, and (if remote) connection. Internally, a sector follows a **tree structure**:

```

SECTOR
├── Command Hub A (Level 2)
│   ├── Application 1 (Level 3)
│   └── Application 2 (Level 3)
├── Command Hub B (Level 2)   ← created via split
│   └── Application 3 (Level 3)
└── Command Hub C (Level 2)
└── Application 4 (Level 3)

```

- Each Command Hub has its own state (mode, history, environment) and can launch multiple applications.
- Applications are children of the hub that launched them.
- Splits create additional viewports that can contain either a new (fresh) Command Hub or an existing hub (parent of the current app, or any hub in the sector).

---

## 6. Split Viewports

Splitting allows a sector to display multiple viewports simultaneously, each with independent depth and content.

### 6.1 Split Initiation
- **From Level 3 (Application Focus)**: In the expanded bezel, a “Split” button. After choosing orientation (horizontal/vertical), the user selects:
  - **New Command Hub**: Creates a fresh Level 2 hub.
  - **Parent Command Hub**: Shows the hub that launched the current app.
  - (Future) **Choose Hub…**: Lists all hubs in the sector.
- **From Level 2 (Command Hub) – Activity Mode**: Multi‑select app tiles, then “Open in Split View”. Creates tiled Level 3 viewports for the selected apps and zooms to Level 3.

### 6.2 Split Behavior
- Each viewport is independent: can be at Level 2 or Level 3, with its own mode, content, and zoom state.
- Viewports can be resized by dragging dividers.
- Closing a viewport removes it; remaining viewports expand.
- A “Close Split” action (in hub or bezel) returns the sector to a single Level 2 Command Hub.

---

## 7. Remote Sectors

Remote sectors are enabled by the **TOS Remote Server**, a daemon installed on the target machine.

### 7.1 Establishing a Remote Sector
- From Global Overview, “Add Remote Sector” (`tos remote add`).
- If the TOS Remote Server is not present, the user is guided to install it (via SSH).
- Once connected, the remote machine appears as a sector tile, indistinguishable from local sectors.

### 7.2 Capabilities
- Full sector tree synchronisation (hubs and apps) – if the remote runs TOS.
- For non‑TOS machines, a virtual sector provides access to filesystem, processes, and terminal.
- Individual application streaming (via Wayland forwarding or custom protocol) with native input forwarding.
- File system integration via Directory Mode.
- Process awareness via Activity Mode.
- Multiple simultaneous connections (for collaboration).

### 7.3 Fallback Modes
- If the TOS Remote Server cannot be installed, fallback to SSH (terminal only) or HTTP (full desktop) with reduced(no file system sync or local process awareness) just remote desktop functionality.

### 7.4 TOS Web Portal (Display Export)
- **Concept**: Any sector or application viewport can be "exported" as a unique URL.
- **Access**: Any modern web browser can connect to this URL to view and interact with the TOS interface as a "virtual display."
- **Capabilities**: Full interaction (mouse/touch/keyboard) via WebSockets/WebRTC. No installation required on the client side.
- **Security**: Portals can be password protected or require "Tactile Approval" on the host machine before a connection is established.

#### 7.4.1 Live Feed Testing
- **Purpose**: Real-time streaming of TOS state and test execution for observation, debugging, and demonstration.
- **WebSocket Streaming**: State snapshots, performance metrics, and user interactions broadcast at 30 FPS.
- **Test Recording**: Automatic capture of test sessions with state history, events, and performance data.
- **Live Observation**: Watch tests execute in real-time via web browser without affecting the test environment.
- **Replay Capability**: Recorded sessions can be replayed for analysis or documentation.
- **Multi-Viewer Support**: Multiple clients can observe the same live feed simultaneously.
- **Authentication**: Optional token-based access control for secure test environments.
- **Integration**: Works with all accessibility features to show screen reader output, auditory cues, and motor input events.

---

## 8. Collaboration

Collaboration is **host‑owned**: a sector resides on one host; guests connect to the host’s TOS Remote Server.

### 8.1 Sharing Model
- Host invites guests via secure token or contact list.
- Guests see a synchronised view of the host’s sector tree (same hubs and apps).

### 8.2 Viewport Independence (Default)
- Each guest controls their own viewports (splits, zooms) independently.
- Optional **following** mode allows a guest to synchronise their view with another participant.

### 8.3 Roles and Permissions
| Role       | Capabilities |
|------------|--------------|
| **Viewer** | See content, cannot issue commands. |
| **Commenter** | Type in prompt (commands execute with restricted shell). |
| **Operator** | Full control over the sector (launch apps, execute any command). |
| **Co‑owner** | Invite others, change roles. |

### 8.4 Visual Collaboration Cues
- Avatars in Global Overview, hub mode, and on app bezels.
- Colored borders/cursors for each participant.
- Optional auditory cues (user join/leave, focus sharing).

---

## 9. Input Abstraction Layer

All physical input devices are normalized into **semantic events**, which are then mapped to TOS actions via a user‑configurable mapping layer.

### 9.1 Semantic Event Categories
- **Navigation**: `zoom_in`, `zoom_out`, `next_element`, `next_viewport`, etc.
- **Selection**: `select`, `secondary_select`, `multi_select_toggle`.
- **Mode Control**: `cycle_mode`, `set_mode_command`, `toggle_hidden_files`.
- **Bezel Control**: `toggle_bezel_expanded`, `split_view`, `close_viewport`.
- **System Commands**: `open_hub`, `open_global_overview`, `tactical_reset`.
- **Text Input**: `text_input`, `command_history_prev`.
- **Voice**: `voice_command_start`, voice transcription.
- **Collaboration**: `show_cursor`, `follow_user`.

### 9.2 Device Support
- **Keyboard**: Fully customizable shortcuts.
- **Mouse/Trackpad**: Click, right‑click, scroll, drag, hover.
- **Touch**: Tap, long press, pinch, swipe, multi‑finger gestures.
- **Game Controller**: Analog sticks, D‑pad, triggers, bumpers, face buttons.
- **VR/AR Controllers**: Trigger, grip, thumbstick, controller pose.
- **Hand Tracking**: Pinch, grab, two‑hand spread, point.
- **Voice**: Wake word, natural language processing, confidence indicators.
- **Eye Tracking**: Gaze, dwell, blink patterns.
- **Accessibility Switches**: Single/multi‑switch scanning, configurable actions.

### 9.3 Concurrent Input
Multiple devices can be used simultaneously; the system intelligently merges input streams. The last active device determines cursor appearance. Conflict resolution is user‑configurable.

---

## 10. Performance and Compositing

TOS is a full Wayland compositor with spatial layer management.

### 10.1 Rendering Optimizations
- **Depth‑based rendering**: Only the focused level receives full frame rate; background levels are rendered as static textures or throttled.
- **Texture caching**: Thumbnails and background textures are cached with configurable resolution.
- **GPU memory pruning**: Surfaces more than two levels away may have pixel data swapped out.
- **Hardware acceleration**: OpenGL ES / Vulkan for 2D/3D; direct scanout for full‑screen apps when possible.

### 10.2 Viewport Management
- Each split viewport is a separate render target, composited with scaling and blending.
- Resizing uses texture scaling until the app provides a new buffer.

### 10.3 Remote Streaming
- Hardware‑accelerated decoding.
- Latency mitigation: local echo for commands, adaptive quality, thumbnail caching.

### 10.4 Tactical Alert (Performance Warning)
If frame rate drops below target (e.g., 60 FPS desktop, 90 FPS VR) for a sustained period (configurable, default 2s), a non‑intrusive alert appears (visual, optional auditory/haptic). It shows current FPS and suggests corrective actions.

---

## 11. Security Model

### 11.1 Authentication
- Local login via PAM (optional biometric).
- Remote connections: SSH keys/passwords + mutually authenticated TLS for TOS Remote Server.
- Invite tokens (cryptographically secure, time‑limited) for shared sectors.

### 11.2 Authorization (RBAC)
Roles: Viewer, Commenter, Operator, Co‑owner (as defined in §8.3). All guest actions are enforced on the host side.

### 11.3 Process Isolation
- Applications run as the user’s own processes (standard Linux security).
- Optional sandboxing via Flatpak, Firejail, or bubblewrap, configurable per Application Model.
- **Modules** (Sector Types, Application Models) are sandboxed via the TOS module API and can optionally be run in lightweight containers (bubblewrap, Firejail, Docker) based on user settings or permission requests.

### 11.4 Dangerous Command Handling
- A configurable list of dangerous commands (e.g., `rm -rf /`) requires **tactile confirmation** (hold, slider, voice confirmation, multi‑button press).
- Confirmation methods are multi‑modal.

### 11.5 Auditing
- All commands executed at the Command Hub are logged with timestamp, user, sector, and exit status.
- Security events (authentication, role changes, invite usage) are written to the system journal and visible in the Security Dashboard.

---

## 12. Application Models and Sector Types

### 12.1 Application Model
A module that customizes an application’s integration at Level 3.

- **Provides**: Custom bezel actions, zoom behavior, legacy decoration policy, thumbnail for Activity Mode.
- **API**: Rust trait or script (JS/Lua) with methods like `bezel_actions()`, `handle_command()`, `decoration_policy()`.
- **Installation**: Local directory (`~/.local/share/tos/app-models/`), hot‑loadable.

### 12.2 Sector Type
A module that defines a sector’s default behavior.

- **Provides**: Command favourites, interesting directory detection, environment variables, available hub modes, default guest role, associated Application Models.
- **API**: Similar to Application Model, with methods like `command_favourites()`, `is_interesting_directory()`.
- **Installation**: `~/.local/share/tos/sector-types/`.

### 12.3 Security
- Modules declare required permissions (network, file access, etc.) in their manifest.
- User grants/denies permissions on installation.
- Optional containerization for extra isolation.

---

## 13. Shell API

The Shell API enables bi‑directional communication between the Command Hub and the underlying shell via OSC escape sequences and special commands.

### 13.1 Shell‑to‑Compositor (OSC)
- `suggestions`: Provides command suggestions based on context.
- `directory`: Sends directory listing.
- `command_result`: Reports exit status and output preview.
- `cwd`: Informs of current working directory.
- `env`: Environment variable updates.
- `dangerous_command`: Flags a dangerous command for confirmation.

### 13.2 Compositor‑to‑Shell (via PTY)
- `EXEC <command>`: Execute a command.
- `CD <path>`: Change directory.
- `COMPLETE <partial>`: Request completions.
- `LS <path>`: Request directory listing.
- `SETENV <var=value>`: Set environment variable.

### 13.3 Reference Implementation
- **Fish** is the reference shell, with a built‑in plugin.
- Bash/Zsh plugins can be implemented using `PROMPT_COMMAND` and `preexec` hooks.

---

## 14. Tactical Reset

A two‑level emergency recovery system.

### 14.1 Level 1 – Sector Reset
- **Trigger**: `Super+Backspace` (configurable) or `tos sector reset`.
- **Action**: Sends SIGTERM to all processes in the current sector, closes all viewports, returns to a fresh Level 2 Command Hub.
- **Confirmation**: None by default; optional undo button (5s) can be enabled.

### 14.2 Level 2 – System Reset
- **Trigger**: `Super+Alt+Backspace` or `tos system reset`.
- **Dialog**: Presents three options:
  - **Restart Compositor**: Terminates all sectors, restarts TOS compositor, returns to Global Overview (user stays logged in).
  - **Log Out**: Ends TOS session, returns to login manager.
  - **Cancel**.
- **Confirmation**: Tactile confirmation required (hold, slider, voice, multi‑button). Countdown with cancel option.

---

## 15. Sector Templates and Marketplace

### 15.1 Package Types
- **Sector Template** (`.tos-template`): Configuration only (no code). Exported from any sector.
- **Sector Type** (`.tos-sector`): Module package.
- **Application Model** (`.tos-appmodel`): Module package.

### 15.2 Marketplace
- User‑configurable repository indices (JSON over HTTPS).
- Packages are downloaded manually; installation shows warnings for code‑containing packages and permission requests.
- Dependencies (required Sector Types/Application Models) are checked and can be auto‑installed from the same repository.

### 15.3 Security
- Code packages are sandboxed via module API and optional containerization.
- Signatures (GPG/minisign) can be verified if the user imports trusted keys.

---

## 16. Accessibility

### 16.1 Visual
- High‑contrast themes, font scaling, colorblind filters.
- Screen reader support via AT-SPI (Orca compatible).
- Braille display support.
- Focus indicators (thick border, optional haptic/auditory).

### 16.2 Auditory
- Screen reader integration.
- Earcons for navigation and feedback (configurable).
- Voice notifications (TTS) with adjustable verbosity.

### 16.3 Motor
- Switch device support (single/multi‑switch scanning).
- Sticky keys, slow keys, dwell clicking.
- Gesture alternatives for all touch/controller actions.
- Adjustable haptic feedback.

### 16.4 Cognitive
- Simplified mode (reduced clutter, larger elements).
- Built‑in tutorials via eval‑help mapping.
- Consistent spatial model (three levels, three modes).

### 16.5 Configuration
- Central Accessibility settings panel.
- User‑savable profiles (e.g., “Low Vision”, “Switch User”).

---

## 17. Tactical Mini‑Map

An ephemeral overlay that provides spatial awareness without blocking interaction.

### 17.1 Visuals
- Small, semi‑transparent panel (default bottom‑right).
- Shows current sector, other sectors (dimmed), viewports within the current sector, and current depth.

### 17.2 Activation (Input Pass‑Through by Default)
- In **passive state**, input passes through to underlying UI.
- **Activation methods** (configurable):
  - Hover (dwell time).
  - Keyboard shortcut (`Ctrl+M`).
  - Modifier + click (`Alt+click`).
  - Double‑tap (touch).
  - Game controller button.
  - Voice (“activate mini‑map”).
- Once active, the mini‑map captures input: click to jump to a sector/viewport, drag to reposition.

### 17.3 Content by Depth
- **Level 1**: All sectors as miniature tiles.
- **Level 2**: Current sector with mode indicator; other sectors dimmed.
- **Level 3**: Current sector with focused app highlighted; other viewports shown.

### 17.4 Configuration
- Position, size, opacity (passive/active), activation behavior, visible content.

---

## 18. Auditory Interface

### 18.1 Sound Categories
- **Navigation earcons**: Zoom in/out, level change, split, focus change.
- **Command feedback**: Accepted, error, dangerous command warning, completion.
- **System status**: Notifications, tactical alerts, battery.
- **Collaboration**: User join/leave, cursor sharing.
- **Bezel/UI**: Expand/collapse, button hover, mode switch.

### 18.2 Speech Output
- TTS for notifications, command confirmation, navigation assistance.
- Configurable voice, rate, pitch, verbosity.

### 18.3 Spatial Audio
- In VR/AR, sounds positioned in 3D space (e.g., notifications from a sector to the left sound left).

### 18.4 Configuration
- Master volume; per‑category enable/volume.
- Sound themes (default, minimal, etc.) installable via Marketplace.

---

## 19. Implementation Roadmap

1. **Minimal Viable Product (MVP)**
   - Single sector, basic file navigation, one or two applications.
   - Core three‑level hierarchy, Command Hub with Command Mode only.
   - Keyboard/mouse input.

2. **Hub Modes Expansion**
   - Directory Mode and Activity Mode.
   - Basic file management and process overview.

3. **Tactical Bezel and Splits**
   - Bezel implementation (collapsed/expanded).
   - Split viewports (from Level 3 and Level 2 Activity Mode).

4. **Remote Sectors**
   - TOS Remote Server prototype (SSH + basic streaming).
   - Remote sector creation and navigation.

5. **Collaboration**
   - Host‑owned sharing with independent viewports.
   - Roles and permissions.

6. **Input Abstraction**
   - Game controller support.
   - Voice commands.
   - Accessibility switches.

7. **Performance Optimizations**
   - Depth‑based rendering, caching, pruning.
   - Tactical alert for frame drops.

8. **Module System**
   - Application Models and Sector Types API.
   - Hot‑loading.

9. **Marketplace and Templates**
   - Package export/import.
   - Repository index support.

10. **Accessibility and Polish**
    - Screen reader integration, high contrast, etc.
    - Mini‑map, auditory interface, final tuning.

---

*This document represents the complete and consistent architectural vision for TOS, incorporating all decisions from the design process.*
