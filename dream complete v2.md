# TOS (Tactical Operating System) – Unified Architectural Specification

## Version 2.1 (Consolidated with Extensions – Reorganized)

---

## 1. Core Philosophy

TOS (Tactical Operating System) is a reimagining of the Linux desktop, inspired by the LCARS interface from Star Trek. It replaces traditional window management with a **recursive zoom hierarchy** centered on a **command‑first** philosophy. The environment is **input‑agnostic**, supporting touch, mouse, keyboard, voice, game controllers, VR/AR controllers, hand tracking, eye tracking, and accessibility switches. It scales from embedded IoT devices to collaborative distributed workspaces through a spatial command platform.

All interactions are organized into a strictly vertical, multi‑level structural hierarchy, with a tree‑like organization of **sectors**, **command hubs**, and **applications**. Navigation is achieved by zooming through these layers, with a persistent focus on terminal‑driven intent and graphical augmentation. The design is extended with **priority visual indicators**, **unified search**, **AI assistance**, **detailed logging**, and **multi‑sensory feedback** to create a truly tactical environment.

---

## 2. The Three‑Level Hierarchy

The core hierarchy consists of three primary levels, extended with two deeper inspection levels for system introspection.

| Level | Name                 | Description |
|-------|----------------------|-------------|
| **1** | **Global Overview**  | Bird’s‑eye view of all sectors (local and remote). Sectors appear as zoomable tiles with priority indicators. This level is reserved for sector setup, configuration, remote management, and global settings. |
| **2** | **Command Hub**      | Central control point for a sector. Contains the **Persistent Unified Prompt** and four toggleable modes (Command, Directory, Activity, Search). All modes can affect the entire system. Search results are displayed here as an actionable grid. |
| **3** | **Application Focus**| Full‑screen (or tiled) application surface, wrapped in the **Tactical Bezel**. |
| **4** | **Detail**           | Structured metadata view for any surface: CPU, memory, uptime, event history, and configuration. |
| **5** | **Buffer**           | Raw memory / hex dump of the application’s process space. **Privileged access** – requires explicit elevation (see §11.6). |

Navigation is consistent across input methods: pinch/scroll/trigger to zoom, tap/click/select to choose. The hierarchy is strictly enforced, but the system may learn frequent paths and offer predictive zoom with a “ghost” animation that visually traverses intermediate levels.

Levels 4 and 5 are available for any surface (application, terminal, dashboard) and provide deep inspection capabilities. Level 5 is disabled by default and must be activated per session via a privileged action.

---

## 3. Command Hub: Four Modes

The Command Hub (Level 2) is the exclusive home of the **Persistent Unified Prompt**. It provides four modes, switchable via a four‑way toggle (button, keyboard shortcut, gesture, voice). Each mode is designed to help the user interact with the system, and all can affect the entire TOS environment.

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
- Each tile shows icon, title, optional live thumbnail, and status indicators (PID, CPU/memory on hover). Priority indicators (see §5.1) are also shown.
- **Multi‑select** for batch actions (close, kill, move).
- **Integration with prompt**: selecting a tile populates the prompt with PID/window ID; contextual chips suggest relevant commands (kill, focus, etc.).
- Clicking a tile zooms to Level 3 (Application Focus).

### 3.4 Search Mode

Search Mode transforms the Command Hub into a **unified search interface**, allowing users to find anything across all domains. Results are displayed as an actionable grid at Level 2; selecting a result triggers an **Automated Vertical Transition** to the target’s exact location.

#### 3.4.1 Scope

Unified Search searches across the following domains:

| Domain | Examples |
|--------|----------|
| **Surfaces** | Window titles, application IDs, process names, surface UUIDs, content (if exposed via Application Model). |
| **Files and Directories** | File names, directory names, extensions, paths, metadata (size, date, tags). |
| **TOS Log Events** | Commands executed, lifecycle events, inspections, notifications, alerts, collaboration events. |
| **Commands and History** | Shell command history, favourite commands, aliases. |
| **Settings and Preferences** | Setting names, preference values, direct links to config panels. |
| **Sectors and Viewports** | Sector names, sector types, viewport positions, split configurations. |
| **Help and Documentation** | Built‑in help topics, man pages, tutorial content. |
| **Contacts and Collaboration** | User names, active collaborators, shared sectors. |
| **Marketplace Packages** | Installed and available modules (AI backends, sector types, etc.). |
| **Notifications** | Active and recent notifications. |

#### 3.4.2 Search Behaviour

- **Federated by default**: All domains are searched simultaneously unless the user specifies a filter (e.g., `files:budget` or `log:failed`).
- **Ranked results**: Relevance scoring considers recency, frequency of access, and explicit user signals (e.g., pinned items). Priority indicators (see §5.1) are shown on result tiles.
- **Grid presentation**: Results appear as a grid at Level 2, replacing the normal Command Hub layout temporarily. The grid can be filtered, sorted, and navigated.
- **Direct navigation**: Clicking a result triggers an **Automated Vertical Transition** to the target’s exact location (e.g., selecting a file opens it in Directory Mode; selecting a surface zooms to its Level 3 view).

#### 3.4.3 External Search Providers

The results grid includes **tiles representing external search engines** alongside local matches. Clicking such a tile sends the current query to the corresponding search engine, opening the results in the **user's default web browser**.

- **Default providers**: Google, Bing, DuckDuckGo, Wikipedia, GitHub, Docker Hub, crates.io, PyPI, Stack Overflow, etc. (user‑configurable).
- **URL templates**: Each provider has a base URL with a `{searchTerms}` placeholder.
- **Privacy**: External searches are clearly marked (e.g., a globe icon). Users may enable per‑provider confirmation.
- **Configuration**: Users can add, remove, or edit providers in Settings → Search → External Providers.

#### 3.4.4 Integration

- **Prompt**: In SEARCH mode (see §3.5), typing refines the results grid in real time. Pressing Enter selects the top result.
- **AI Assistant**: The AI can assist with search queries, suggest providers, or explain results.
- **TOS Log**: Search queries themselves can be logged (if enabled) for audit and recall.

### 3.5 Multi‑Mode Prompt with AI Assistant

The Persistent Unified Prompt is extended with a three‑way mode selector and a stop button, enabling seamless switching between command execution, unified search, and AI‑powered assistance. The prompt is always visible at the bottom of the Command Hub.

#### 3.5.1 Mode Selector

A compact LCARS strip displays `CMD`, `SEARCH`, and `AI`. The active mode is always the rightmost label. Clicking a label cycles the order, making the clicked mode active. The input text is preserved across mode changes.

- **CMD Mode**: Standard shell and TOS command input (as described in §3.1).
- **SEARCH Mode**: Triggers Unified Search (see §3.4). Typing updates the live results grid; pressing Enter selects the top result.
- **AI Mode**: Accepts natural language queries for an AI assistant. The assistant can answer questions, control the system, or perform complex tasks. Responses appear in a dedicated output area (e.g., a bubble above the prompt) and can include suggested commands.

#### 3.5.2 Stop Button

A dedicated button (⏹️) on the right side of the input field immediately interrupts the current operation:

- **CMD Mode**: Sends `SIGINT` to the foreground process (like Ctrl+C).
- **SEARCH Mode**: Cancels an ongoing search (especially if it's slow or indexing).
- **AI Mode**: Stops the AI from generating a response.

A keyboard shortcut (e.g., `Ctrl+Shift+C` or `Esc` twice) also triggers the stop action. The button is always visible but may be disabled when no operation is in progress.

#### 3.5.3 AI Backend Framework

The AI assistant uses a pluggable backend architecture.

- **Default Backend**: TOS ships with an **Ollama** integration module, providing local, private AI capabilities.
- **Additional Backends**: Other AI providers (OpenAI, Anthropic, Google, etc.) are available as modules via the Marketplace (see §16.4). Each module declares its capabilities (chat, function calling, vision) and required permissions.
- **Configuration**: Users select the active backend in Settings → AI Assistant and configure per‑backend options (model, API keys, permissions).
- **Privacy**: Cloud backends require explicit user confirmation and declare network permissions. All AI interactions may be logged to the TOS Log subject to user privacy settings.
- **Function calling**: If a backend supports system control, the user must explicitly grant permission. Dangerous commands still require tactile confirmation as per §11.4.

---

## 4. Tactical Bezel

The Tactical Bezel is an **immutable system overlay** rendered by the compositor at Level 3. It serves as both the guaranteed navigation escape and the unified window decoration for all applications.

### 4.1 Default (Collapsed) State
- Thin, semi‑transparent strip along the top edge (user‑configurable position).
- Contains:
  - **Zoom Out** button (returns to Level 2).
  - Application icon and title.
  - **Expand handle** (down chevron).
  - When AI mode is active, the stop button may also appear here (if not in the prompt area).

### 4.2 Expanded State
Activated by dragging down the handle, clicking/tapping it, or using a keyboard shortcut (`Ctrl+Space`). Reveals a command strip with:

- **Navigation**: Zoom Out, Split View, Teleport, Close Application.
- **Window Controls**: Minimize, Full‑screen Toggle, Always on Top (if applicable).
- **Application‑Specific Actions** (provided by the Application Model).
- **System‑Wide Shortcuts**: Open Command Hub, Toggle Mini‑Map, Settings, Toggle AI Mode.
- **Collaboration Indicators**: Avatars, share button, and priority indicators for the current surface.

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
- Splits create additional viewports that can contain either a new (fresh) Command Hub or an existing hub.

### 5.1 Priority‑Weighted Layouts (Visual Indicators)

To convey the relative importance of surfaces without altering their size or position, TOS uses **non‑intrusive visual indicators**—border chips, chevrons, status dots, and glows—applied to tiles at all levels. These indicators are inspired by authentic LCARS design language and preserve spatial stability.

#### 5.1.1 Indicator Types

| Indicator | Description | Usage |
|-----------|-------------|-------|
| **Border Chips** | Small, pill‑shaped coloured accents along the border of a surface tile. | Number of chips or colour intensity reflects priority score. More chips = higher priority. |
| **Chevrons** | LCARS‑style arrow shapes (▶) at corners or edges. | A pulsing chevron indicates a surface with a pending notification or critical status. |
| **Glow / Luminance** | Subtle inner or outer glow around the tile. | Intensity varies with priority; integrates with alert colours (yellow/red). |
| **Status Dots** | Small circles in a corner of the tile. | Colour‑coded: blue for normal, yellow for caution, red for critical. Multiple dots can indicate multiple factors (e.g., active guest + high CPU). |

All indicators are **configurable**: users can enable/disable specific types, adjust colours, and set what each indicator represents.

#### 5.1.2 Priority Scoring

A surface’s priority score is calculated from weighted factors (user‑configurable):

| Factor | Description | Default Weight |
|--------|-------------|----------------|
| **Recency of focus** | Time since last interaction. | 40% |
| **Frequency of use** | Session‑wide or historical usage. | 20% |
| **Activity level** | CPU, memory, or I/O activity. | 15% |
| **Notification priority** | Active notifications associated with the surface. | 10% |
| **User pinning** | Manually pinned surfaces maintain a minimum indicator level. | (override) |
| **Collaboration focus** | Surfaces being viewed or edited by guests. | 10% |
| **Sector‑specific rules** | Defined by Sector Type. | (sector‑defined) |
| **AI suggestion** | AI may temporarily boost a surface based on predicted need. | 5% |

Scores map to indicator configurations:

| Priority Level | Indicator Appearance |
|----------------|----------------------|
| Low (0–20)     | No border chips; minimal or no glow. |
| Normal (20–50) | One border chip (default colour). |
| Elevated (50–70) | Two border chips, or chips with a subtle pulse. |
| High (70–90)   | Three border chips, plus a chevron at one corner. |
| Critical (90+)  | Four border chips, pulsing chevron, and optional red alert colour integration. |

#### 5.1.3 Behaviour by Depth

- **Level 1 (Global)**: Sector tiles display border chips indicating overall sector activity (e.g., number of active guests, pending alerts, recent use). A sector in Red Alert may have a prominent chevron.
- **Level 2 (Command Hub)**: Application tiles show chips/chevrons based on individual surface priority.
- **Level 3 (Application Focus)**: In split viewports, each pane has indicators along shared borders.
- **Level 4/5 (Inspection)**: Inspection panels can show priority indicators for the surface being inspected, as well as a mini‑map of sibling surface priorities.

#### 5.1.4 Configuration

- **Master toggle**: Enable/disable priority indicators globally.
- **Customisation**:
  - Choose which indicator types appear.
  - Set colours per priority level or per factor (e.g., blue for recency, green for activity, purple for collaboration).
  - Adjust sensitivity (how quickly indicators respond to changes).
  - Define exceptions: pinned surfaces can have fixed indicator states.
- **Hover details**: Hovering over an indicator shows a tooltip explaining why that surface is prioritised (e.g., “Focused 2 minutes ago” or “CPU usage 85%”).

#### 5.1.5 Integration

- **Search Mode** (§3.4): Search result tiles use priority indicators to show relevance.
- **System Alert Mode** (§1.4): During Yellow/Red Alert, indicators become more prominent or change colour.
- **AI Assistant** (§3.5): The AI can explain indicator changes and suggest priority boosts.
- **Collaboration** (§8): Guest activity adds a collaboration chip (e.g., a small avatar icon). Multiple guests show multiple chips.
- **TOS Log** (§14): Priority changes are logged, allowing users to query why indicators fluctuated.
- **Accessibility** (§17): Users can increase indicator size, enable high‑contrast colours, or have indicators announced by screen readers.

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
- If the TOS Remote Server cannot be installed, fallback to SSH (terminal only) or HTTP (full desktop) with reduced functionality (no file system sync or local process awareness) just remote desktop functionality.

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
- Priority indicators (see §5.1) show guest activity on surfaces (collaboration chips).

### 8.5 TOS Log Integration
All guest actions within a shared sector are recorded in the **host's TOS Log** (see §14). Entries include guest identity, action type, timestamp, and outcome. Guest actions are **never** written to the guest's local TOS Log. Hosts may filter and review guest activity via the TOS Log sector or prompt queries (e.g., `log --guest alex`). A privacy notice is shown to guests upon joining: *“Your actions in this sector will be logged by the host.”*

### 8.6 AI Assistant for Collaboration
The AI Assistant (see §3.5) enhances collaboration by:
- **Summarizing recent activity**: “What did Alex do while I was away?”
- **Translating commands or chat** between languages (with user consent).
- **Suggesting collaboration actions**: “You and Jamie are editing the same file. Start a split view?”
- **Explaining guest intent**: “Alex ran `rsync` – they appear to be backing up the project.”
- **Mediating role changes**: Draft polite notifications when roles change.
- Optionally, an **AI‑driven guest** can be invited for testing or assistance.

AI processing of guest actions uses the host's configured AI backend; guests are notified if their actions may be processed by AI.

### 8.7 Collaboration Alerts
Key collaboration events trigger a **Yellow Alert** (see §1.4):
- A new user joins a sector.
- A guest's role changes.
- A guest requests attention (e.g., via a “raise hand” button in their bezel).
- A guest shares a cursor or enters following mode.
- A guest leaves a sector (optional).

Visual, auditory, and haptic feedback for these events is configurable. All such events are also recorded in the TOS Log.

---

## 9. Input Abstraction Layer

All physical input devices are normalized into **semantic events**, which are then mapped to TOS actions via a user‑configurable mapping layer.

### 9.1 Semantic Event Categories
- **Navigation**: `zoom_in`, `zoom_out`, `next_element`, `next_viewport`, etc.
- **Selection**: `select`, `secondary_select`, `multi_select_toggle`.
- **Mode Control**: `cycle_mode`, `set_mode_command`, `toggle_hidden_files`, `cycle_prompt_mode`.
- **Bezel Control**: `toggle_bezel_expanded`, `split_view`, `close_viewport`.
- **System Commands**: `open_hub`, `open_global_overview`, `tactical_reset`, `stop_operation`.
- **Text Input**: `text_input`, `command_history_prev`.
- **Voice**: `voice_command_start`, voice transcription.
- **Collaboration**: `show_cursor`, `follow_user`, `raise_hand`.
- **AI**: `ai_submit`, `ai_stop`, `ai_mode_toggle`.

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
- **Modules** (Sector Types, Application Models, AI Backends) are sandboxed via the TOS module API and can optionally be run in lightweight containers (bubblewrap, Firejail, Docker) based on user settings or permission requests.

### 11.4 Dangerous Command Handling
- A configurable list of dangerous commands (e.g., `rm -rf /`) requires **tactile confirmation** (hold, slider, voice confirmation, multi‑button press).
- Confirmation methods are multi‑modal.

### 11.5 Auditing
- All commands executed at the Command Hub are logged with timestamp, user, sector, and exit status.
- Security events (authentication, role changes, invite usage) are written to the system journal and visible in the Security Dashboard.

### 11.6 Deep Inspection Privilege
Access to Level 5 (raw memory / hex dump) is considered a highly privileged operation. It is **disabled by default** and can only be activated per user session via explicit privilege elevation (e.g., `sudo tos enable-deep-inspection` or a Polkit‑authenticated dialog). Once enabled, a visual indicator (🔓) appears in the Tactical Bezel; clicking this indicator disables deep inspection immediately without further authentication. All enable/disable events and any use of Level 5 are audited. Individual applications may opt out of deep inspection via their Application Model manifest.

---

## 12. Application Models and Sector Types

### 12.1 Application Model
A module that customizes an application’s integration at Level 3.

- **Provides**: Custom bezel actions, zoom behavior, legacy decoration policy, thumbnail for Activity Mode, priority factor definitions (optional), opt‑out from deep inspection, and the ability to expose content for Unified Search.
- **API**: Rust trait or script (JS/Lua) with methods like `bezel_actions()`, `handle_command()`, `decoration_policy()`, `priority_weight()`, `searchable_content()`.
- **Installation**: Local directory (`~/.local/share/tos/app-models/`), hot‑loadable.

### 12.2 Sector Type
A module that defines a sector’s default behavior.

- **Provides**: Command favourites, interesting directory detection, environment variables, available hub modes, default guest role, associated Application Models, and custom priority rules.
- **API**: Similar to Application Model, with methods like `command_favourites()`, `is_interesting_directory()`, `priority_factors()`.
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
- **Fish** is the reference shell, implemented as a modular **Shell Provider**.
- Shell Providers supply the shell executable, integration scripts, and spawning logic.
- Integration scripts are injected at launch (e.g., via `--init-command` or `source`) to enable bi‑directional communication.
- Bash/Zsh providers are supported through the same Shell Provider interface using `PROMPT_COMMAND` and `DEBUG` traps.

---

## 14. TOS Log

Every surface in TOS maintains its own **event history** – a chronological record of significant interactions and state changes. Collectively, these per‑surface logs form a **system‑wide timeline** accessible via a dedicated Sector, the Level 4 Detail view, and queryable via the Persistent Unified Prompt.

### 14.1 What the TOS Log Records

| Event Type | Description | Example |
|------------|-------------|---------|
| **Lifecycle** | Creation, focus, blur, move, split, clone, close | `[10:32:15] Surface created (pid=1234)` |
| **Commands** | Terminal commands executed (exit status, duration) | `[10:33:01] $ ls -la (exit 0, 0.2s)` |
| **Inspections** | Level 4/5 views accessed | `[10:39:00] Level 4 inspection (CPU=2%, mem=45MB)` |
| **Telemetry** | Periodic resource snapshots (if enabled) | `[10:40:00] Telemetry: CPU 8%, mem 320MB` |
| **Collaboration** | User joins/leaves, role changes, cursor sharing, guest actions | `[10:41:00] User 'Alex' joined sector` |
| **System Events** | Notifications, alerts, security events | `[10:42:00] Red Alert triggered (CPU overload)` |
| **Priority Changes** | Changes in priority score and indicators | `[10:43:00] Terminal priority increased to High (activity + focus)` |
| **AI Interactions** | Queries and responses (if enabled) | `[10:44:00] AI query: "summarize logs"` |


### 14.2 Access Methods

- **Per‑Surface (Level 4 Detail)**: When viewing any surface at Level 4, the TOS Log appears as a scrollable timeline, showing events specific to that surface.
- **Global TOS Log Sector**: A dedicated sector (visible at Level 1) that aggregates logs from **all surfaces** into a unified, filterable timeline. This sector behaves like a specialised application with its own Command Hub.
- **Prompt Queries**: The Persistent Unified Prompt accepts commands like:
  - `log --surface browser --since 10min`
  - `log --user alex --event focus`
  - `log --alert red`
  - `log --priority-changes --surface terminal`

### 14.3 Privacy & User Control

- **Global Toggle**: Settings → Privacy → TOS Log: master switch to enable/disable logging entirely. When disabled:
  - No new events are recorded.
  - Existing logs are retained until explicitly cleared.
  - The TOS Log sector shows: *"Logging is disabled. Enable in Settings to record activity."*
  - Prompt queries return no results.
- **Granular Controls** (optional):
  - Per‑surface opt‑out (certain applications can request suppression).
  - Retention policy (auto‑delete after a set period).
  - Exclude sensitive command patterns.
- **Data Storage**: Logs are stored locally in `~/.local/share/tos/logs/` in a structured, machine‑readable format (e.g., JSON Lines). Each surface has its own log file; the global view aggregates them.
- **Auditing**: Critical security events may still be recorded in a separate, non‑disableable audit log (see §11.5).

### 14.4 OpenSearch Compatibility

The TOS Log is designed to be compatible with both the **OpenSearch Protocol** (for browser integration) and **OpenSearch (AWS/Elastic)** clusters for enterprise analytics.

- **OpenSearch Protocol (Browser Integration)**:
  - TOS provides an OpenSearch description document, allowing users to add TOS Logs as a search engine in their browser.
  - Queries like `tos log failed command` in the browser address bar open a TOS search results page (or directly execute a prompt query).
  - URL templates support parameters: `q={searchTerms}`, `since={time}`, `level={logLevel}`.

- **OpenSearch (Elastic) Compatibility**:
  - Logs are stored in JSON Lines format, suitable for ingestion by tools like Filebeat or Logstash.
  - Users may optionally configure TOS to forward logs to an OpenSearch cluster for centralised analysis, visualization, and long‑term retention.
  - Forwarding requires explicit user consent and configuration; data never leaves the machine without permission.

- **Search Mode Integration**: The TOS Log is one of the domains searched by Search Mode (§3.4). Results from logs appear alongside files and surfaces, with priority indicators showing relevance.

---


## 15. Tactical Reset

A two‑level emergency recovery system.

### 15.1 Level 1 – Sector Reset
- **Trigger**: `Super+Backspace` (configurable) or `tos sector reset`.
- **Action**: Sends SIGTERM to all processes in the current sector, closes all viewports, returns to a fresh Level 2 Command Hub.
- **Confirmation**: None by default; optional undo button (5s) can be enabled.

### 15.2 Level 2 – System Reset
- **Trigger**: `Super+Alt+Backspace` or `tos system reset`.
- **Dialog**: Presents three options:
  - **Restart Compositor**: Terminates all sectors, restarts TOS compositor, returns to Global Overview (user stays logged in).
  - **Log Out**: Ends TOS session, returns to login manager.
  - **Cancel**.
- **Confirmation**: Tactile confirmation required (hold, slider, voice, multi‑button). Countdown with cancel option.

---


## 16. Sector Templates and Marketplace

### 16.1 Package Types
- **Sector Template** (`.tos-template`): Configuration only (no code). Exported from any sector.
- **Sector Type** (`.tos-sector`): Module package.
- **Application Model** (`.tos-appmodel`): Module package.
- **AI Backend Module** (`.tos-ai`): Module package providing an AI backend (see §16.4).

### 16.2 Marketplace
- User‑configurable repository indices (JSON over HTTPS).
- Packages are downloaded manually; installation shows warnings for code‑containing packages and permission requests.
- Dependencies (required Sector Types/Application Models/AI Backends) are checked and can be auto‑installed from the same repository.

### 16.3 Security
- Code packages are sandboxed via module API and optional containerization.
- Signatures (GPG/minisign) can be verified if the user imports trusted keys.

### 16.4 AI Backend Modules

AI Backend Modules (`.tos-ai`) provide pluggable backends for the AI Assistant (see §3.5). They are installed via the Marketplace and follow the same security model as other code packages.

#### 16.4.1 Manifest Example (`module.toml`)

```toml
name = "OpenAI GPT-4"
version = "1.0.0"
type = "ai-backend"
description = "Connect to OpenAI's GPT-4 model for AI assistance."
icon = "openai.svg"

[capabilities]
chat = true
function_calling = true
vision = false
streaming = true

[connection]
protocol = "https"
default_endpoint = "https://api.openai.com/v1/chat/completions"
auth_type = "api-key"  # or "oauth2", "none"

[permissions]
network = ["api.openai.com"]
filesystem = false

[configuration]
model = { type = "string", default = "gpt-4", options = ["gpt-4", "gpt-3.5-turbo"] }
temperature = { type = "float", default = 0.7, min = 0, max = 2 }
```

#### 16.4.2 Installation Flow
- User browses Marketplace, finds an AI module, clicks Install.
- TOS displays the module's requested permissions (e.g., network access to specific domains).
- User confirms (or rejects) installation.
- Module is placed in `~/.local/share/tos/ai-backends/` and appears in the AI Assistant settings panel.

#### 16.4.3 Module Isolation
- Modules run in a sandbox with limited access (using the same module API as Application Models).
- Network permissions are enforced via the host's firewall or a dedicated proxy.
- If a module requires dangerous capabilities (e.g., local file access for context), it must declare them and obtain explicit user consent.

### 16.5 Marketplace Discovery Enhancements

The Marketplace is fully integrated with Search Mode (§3.4) and the AI Assistant:
- **Search Mode**: Typing a query like "openai" or "local llama" in search mode shows relevant modules as tiles, alongside local files and surfaces. Selecting a module opens its details page.
- **AI‑Assisted Discovery**: The AI Assistant can help users find modules based on natural language queries (e.g., "Find me a local AI backend that runs on my GPU"). The AI may query the Marketplace index (with permission) and present recommendations directly.
- **Update Alerts**: When an installed module has an update, a Yellow Alert (§1.4) may notify the user.

---

## 17. Accessibility

### 17.1 Visual
- High‑contrast themes, font scaling, colorblind filters.
- Screen reader support via AT-SPI (Orca compatible).
- Braille display support.
- Focus indicators (thick border, optional haptic/auditory).
- Priority indicators can be enlarged or replaced with high‑contrast variants.

### 17.2 Auditory
- Screen reader integration.
- Three‑layer audio (ambient, tactical, voice) with separate controls (see §19).
- Voice notifications (TTS) with adjustable verbosity.

### 17.3 Motor
- Switch device support (single/multi‑switch scanning).
- Sticky keys, slow keys, dwell clicking.
- Gesture alternatives for all touch/controller actions.
- Adjustable haptic feedback.

### 17.4 Cognitive
- Simplified mode (reduced clutter, larger elements).
- Built‑in tutorials via eval‑help mapping.
- Consistent spatial model (three levels, four modes).
- Priority indicators can be simplified to reduce cognitive load.

### 17.5 Configuration
- Central Accessibility settings panel.
- User‑savable profiles (e.g., “Low Vision”, “Switch User”).

---

## 18. Tactical Mini‑Map

An ephemeral overlay that provides spatial awareness without blocking interaction.

### 18.1 Visuals
- Small, semi‑transparent panel (default bottom‑right).
- Shows current sector, other sectors (dimmed), viewports within the current sector, and current depth.

### 18.2 Activation (Input Pass‑Through by Default)
- In **passive state**, input passes through to underlying UI.
- **Activation methods** (configurable):
  - Hover (dwell time).
  - Keyboard shortcut (`Ctrl+M`).
  - Modifier + click (`Alt+click`).
  - Double‑tap (touch).
  - Game controller button.
  - Voice (“activate mini‑map”).
- Once active, the mini‑map captures input: click to jump to a sector/viewport, drag to reposition.

### 18.3 Content by Depth
- **Level 1**: All sectors as miniature tiles.
- **Level 2**: Current sector with mode indicator; other sectors dimmed.
- **Level 3**: Current sector with focused app highlighted; other viewports shown.

### 18.4 Configuration
- Position, size, opacity (passive/active), activation behavior, visible content.

### 18.5 Monitoring Layer

The mini‑map can optionally display a **monitoring layer** showing live resource usage of processes relevant to the current depth. This layer is toggled via an icon (e.g., a waveform or CPU symbol) on the mini‑map.

- **Level 1**: Aggregated CPU/memory per sector (simple bar or percentage).
- **Level 2**: All applications in the current sector, each with CPU%, memory%, and a mini sparkline. Clicking an app jumps to its Level 3 view.
- **Level 3**: Detailed stats for the focused application, plus compact usage of other viewports.
- Data updates are throttled (e.g., 1–2 Hz) for performance.
- The monitoring layer provides ambient awareness without leaving the current depth, complementing Activity Mode (which remains the full‑featured process manager).

---

## 19. Auditory and Haptic Interface

TOS provides a rich multi‑sensory experience through three independent audio layers and a parallel haptic channel. All layers are configurable and integrated with accessibility features.

### 19.1 Three‑Layer Audio Model

| Layer | Purpose | Characteristics |
|-------|---------|-----------------|
| **Ambient** | Atmosphere, sense of place | Continuous, depth‑varying background (bridge hum at Level 1 → quiet whir at Level 3). Can reflect system load or time of day. |
| **Tactical** | Action confirmation, navigation cues | Discrete earcons for zoom, commands, notifications, splits, alerts, collaboration events. |
| **Voice** | Speech synthesis, accessibility | TTS for announcements, screen reader, verbose feedback, AI responses. |

Each layer has independent volume control and can be enabled/disabled separately.

### 19.2 Context Adaptation

Audio behaviour changes with zoom level, alert state, and user role:

- **Zoom Level**: Ambient sounds become quieter at deeper levels. Tactical sounds may have different pitches for zoom in/out.
- **Alert Mode**:
  - **Green (Normal)**: All layers as configured.
  - **Yellow Alert**: Ambient layer shifts (lower pitch, added tension); tactical layer may add a periodic pulse; voice layer becomes slightly more verbose.
  - **Red Alert**: Ambient layer replaced by a repeating alert tone; tactical layer suppresses non‑critical earcons; voice layer prioritises critical messages.
- **Collaboration**: User join/leave events trigger a soft, friendly chime (configurable).

### 19.3 Spatial Audio

In VR/AR environments, sounds are positioned in 3D space:
- Notifications from a sector to the user’s left appear from that direction.
- Zoom sounds feel like moving through layers.
- Collaboration: other users’ cursors emit subtle positional sounds.

### 19.4 Theming and Extensibility

- Audio themes (`.tos-audio`) can be installed via the Marketplace, providing alternative soundscapes.
- Each theme defines ambient tracks or synthesis parameters, earcon samples, and voice presets.
- Applications may contribute custom tactical sounds through their Application Model.

### 19.5 Accessibility Integration

- Voice layer serves as the foundation for screen reader support.
- Tactile feedback (haptics) can replace or augment audio cues for hearing‑impaired users.
- Verbosity levels adjust the amount of spoken feedback.

### 19.6 Haptic Feedback

Haptic feedback is treated as a parallel channel to the tactical audio layer, providing touch‑based sensations across supported devices.

#### 19.6.1 Device Support
- **Game controllers**: Dual rumble motors, variable intensity.
- **VR controllers**: Linear actuators for nuanced textures and directional pulses.
- **Haptic touchpads**: Click‑like feedback.
- **Mobile devices / tablets**: Built‑in vibration motors.
- **Accessibility switches**: Can be paired with external buzzers or haptic wearables.

TOS queries device capabilities and adapts haptic patterns accordingly.

#### 19.6.2 Haptic Event Taxonomy

The same semantic events used for tactical audio are mapped to haptic patterns:

| Event Category | Example Events | Haptic Pattern Suggestions |
|----------------|----------------|----------------------------|
| **Navigation** | `zoom_in`, `zoom_out`, `level_change`, `split_view` | Ascending/descending pulses, short directional bursts (VR) |
| **Selection** | `select`, `secondary_select`, `multi_select_toggle` | Quick click, double‑tap |
| **Mode Control** | `cycle_mode`, `toggle_hidden_files` | Mode‑specific pulse sequences |
| **Bezel Control** | `toggle_bezel_expanded`, `close_viewport` | Light buzz when expanding, soft thud when closing |
| **System Commands** | `open_hub`, `tactical_reset` | Distinctive long vibration for reset |
| **Text Input** | `text_input` | Subtle feedback per keystroke |
| **Voice** | `voice_command_start` | Short “listening” pulse |
| **Collaboration** | `user_joined`, `cursor_shared` | Gentle ping‑like vibration |
| **Dangerous Actions** | `dangerous_command` | Sharp, insistent buzz |
| **Alerts** | `red_alert`, `yellow_alert` | Pulsing vibration that escalates with alert level |

#### 19.6.3 Spatial Haptics (VR/AR)
- A notification from a sector to the left triggers vibration in the left controller.
- Zooming deeper produces a sensation of “pulling” the hands toward the screen.
- Moving a surface to another sector causes a brief drag‑and‑release vibration.

#### 19.6.4 Configuration
- **Global toggle**: Enable/disable haptics.
- **Master intensity** slider.
- **Per‑category toggles** (e.g., disable haptics for navigation but keep for alerts).
- **Test patterns** in Settings.
- Option to link haptics to audio (default) or map independently.

#### 19.6.5 Accessibility
- Hearing‑impaired mode: route all tactical audio events to haptics (with intensity scaled to urgency).
- Motor‑impaired mode: haptics confirm that a switch input has been registered.

---

## 20. Implementation Roadmap

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

10. **Deep Inspection (L4/L5)**
    - Level 4 structured metadata view.
    - Level 5 raw memory view with privilege elevation (sudo toggle).

11. **TOS Log**
    - Per‑surface logging, global log sector.
    - Privacy controls and OpenSearch compatibility.

12. **Search Mode**
    - Unified search across domains with external providers.
    - Integration with prompt and results grid.

13. **AI Assistant**
    - Multi‑mode prompt with CMD/SEARCH/AI toggle.
    - Pluggable backend framework (Ollama default, Marketplace modules).
    - Stop button.

14. **Priority‑Weighted Layouts (Visual Indicators)**
    - Border chips, chevrons, and glows.
    - Configurable factors and appearance.

15. **Auditory and Haptic Feedback**
    - Three‑layer audio with context adaptation.
    - Haptic feedback for supported devices.

16. **Accessibility and Polish**
    - Screen reader integration, high contrast, etc.
    - Mini‑map monitoring layer, final tuning.
