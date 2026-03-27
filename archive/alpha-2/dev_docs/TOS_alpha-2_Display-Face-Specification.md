# TOS Alpha-2 Display & Face Specification

**Purpose:** This document consolidates all architectural and design specifications related to the **Face** of **TOS** (**Terminal On Steroids**). It defines how the system looks, the visual mechanisms of user interaction, the layout of the UI elements, and the modular rendering systems that power the terminal and bezels. For comprehensive details on system logic, process architecture, and IPC boundaries that drive these visual elements, refer to the [TOS Core Architecture Specification](./TOS_alpha-2_Architecture-Specification.md).

---

## 1. Core Visual Philosophy: Terminal First
TOS is a terminal-centric environment. The command line and terminal output are the underlying graphical anchors. All visual augmentations (chips, modes, bezels) are overlays designed to empower the terminal, never to bypass or obscure it permanently. 

The Face is built on standard web technologies (HTML/CSS/JS) acting as a dynamic graphical **frontend** to the **brain_node** (see the [Architecture Specification](./TOS_alpha-2_Architecture-Specification.md) for backend process separation, native shell, and Wayland/XR/Android composition). The design language heavily features LCARS-inspired elements with modern additions like glassmorphism and smooth kinetic transitions.

---

## 2. The Visual Hierarchy (Levels 1–4)
The visual experience in TOS is structured as a vertical zoom hierarchy. The user moves "closer" to the data by zooming in.

| **LVL 1** | Global Overview | Overview of all system sectors | Sector tiles + System Output Area |
|---|---|---|---|
| **LVL 2** | Command Hub | Central sector control | Terminal + prompt + chips |
| **LVL 3** | Application Focus | Full-screen application surface | Application + visible bezel |
| **LVL 4** | Deep Inspection & Recovery | Detail View, Buffer View (privileged), and Tactical Reset (God Mode) | Metadata overlay / hex viewer / wireframe diagnostics |

### 2.1 Kinetic Zoom Transitions
Transformations between levels utilize a specialized **Kinetic Zoom Transition**:
- **Borders as Anchors:** When zooming from Level 1 to 2, the sector tile's borders expand outward to become the Tactical Bezel.
- **Layer Stacking:** Background layers (like the System Output Area) use a **depth-blur and fade** (z-axis displacement) as the focal layer moves forward.
- **Viewport Expansion:** The terminal canvas "unfurls" from the center of the tile, ensuring a seamless visual link between the representative tile and the functional hub.
- **Transitions:** Levels transition via a **kinetic zoom** — the current view recedes spatially as the new view assembles from its components.
- **Persistence:** Level 4 sub‑views are transient; Tactical Reset (within Level 4) provides a low‑overhead global recovery view.
- **Prompt:** The Persistent Unified Prompt is visible at every level, with its visual state (expanded, collapsed, locked) varying by level.

---

## 3. The Unified Tactical Bezel
The **Tactical Bezel** is a persistent frame that surrounds the viewport across all levels. It guarantees stable screen real estate for consistent system control and monitoring. 

### 3.1 Bezel Geometry & Segments
The Bezel is divided into four distinct segments. Three of these sections utilize a **Configurable Slot Architecture**, allowing users to dock and arrange modules as they see fit.

*   **Top Bezel Segment (Divided into three sections):**
    *   **Left Section:** The expand/collapse handle for the left lateral segment. By default, it docks the **Active Viewport Title** for high-level context.
    *   **Center Section:** The telemetry cluster. Component slots for high-level monitoring, defaulting to the **Brain Connection Status** and **Resource Telemetry** (CPU, MEM, NET).
    *   **Right Section:** The expand/collapse handle for the right lateral segment, followed by the system controls menu (Settings, Logout, Shutdown, Terminal Toggle) and **System Status Badges**.
*   **Bottom Bezel Segment (Unified Prompt):**
    *   **Strictly Static:** This is a locked assembly and **does not contain configurable slots**.
    *   **Left (Origin):** Universal Mode Selector (CMD, SEARCH, AI, ACTIVITY).
    *   **Center:** The active command input field.
    *   **Right:** Microphone/Voice controls and Stop/Kill switch.
    | **Visual State** | **Applicable Levels** | **Description** |
    |---|---|---|
    | **Expanded** | Level 2 | The prompt is fully visible and interactive, ready for command input. |
    | **Collapsed & Expandable** | Level 3 | Bottom bezel visible; tapping or hovering it expands the prompt temporarily. |
    | **Collapsed & Locked** | Level 4 (Detail / Buffer) | Prompt is visible but not interactive; the focus is inspection. |
    | **Disabled** | Level 4 (Tactical Reset) | Prompt hidden or locked; Tactical Reset takes priority. |
*   **Lateral Segments (Left & Right):** Slender vertical bars containing **Configurable Vertical Slots**. 
    *   **Left:** Defaults to Hierarchy Navigation buttons and the **Tactical Mini-Map**.
    *   **Right:** Defaults to **Priority Indicators** and **Mini-Log Telemetry**.

### 3.2 Slot Projection Mechanism
Modules docked within the Bezel use a projection mechanism to reveal detailed information without shifting the stable bezel frame:
*   **Lateral Projection:** Components docked in the Left or Right slots expand horizontally inward towards the center of the viewport (e.g., sliding out the Mini-Map).
*   **Vertical Projection (Downward):** Components in the Top slots expand *downward* (e.g., dropping down the Resource Telemetry glass panel).

---

## 4. Bezel Component Modules
The items that populate the Top, Left, and Right bezel slots are modular and user-assignable. The system ships with several built-in core UI components:

1.  **Tactical Mini-Map:** Provides high-level spatial overview, showing the topology of sectors and allowing rapid teleportation (see [Architecture Spec §22](./TOS_alpha-2_Architecture-Specification.md)). (Default: Left Segment).
2.  **Priority Indicator (§21):** Features dynamically ranked system alerts and notification badges. (Default: Right Segment).
3.  **Resource Telemetry:** Real-time metrics for CPU, Memory, Network, and PTY latency. (Default: Top Center).
4.  **Mini-Log Telemetry:** Persistent readout of the authoritative system state and the last executed command. (Default: Right Segment).
5.  **Active Viewport Title:** Real-time text readout of the current Level, Sector name, or App context. (Default: Top Left).
6.  **Brain Connection Status:** Connection state (Online/Offline) and Brain Time. (Default: Top Center).
7.  **System Status Badges:** Quick-toggles for UI settings, sandboxes, and the Terminal output overlay. (Default: Top Right).
8.  **Collaboration Hub:** Multi-user avatars and follow-mode toggles. 
9.  **Media Controller:** Global audio playback controls.

---

## 5. Command Hub & Terminal Canvas (Level 2)

### 5.1 Terminal Output Modules
The aesthetic of the scrolling output is entirely decoupled from the shell logic (which is defined in the [Architecture Specification](./TOS_alpha-2_Architecture-Specification.md)). It is controlled by **Terminal Output Modules** (`.tos-terminal`):
*   **Rectangular Module:** The standard, flat, full-width scrolling block typical of modern terminals.
*   **Cinematic Triangular Module:** Adds 3D depth. Lines recede toward a vanishing point at the user's focus, scaling down as they get older. Includes a "pinwheel" layout for multi-viewport scenarios.

### 5.2 Context-Aware Terminal Augmentation
Rather than utilizing separate graphical pop-ups, grids, or overlays for different tasks, TOS treats the **Terminal Canvas** and the **Dual-Sided Chip Layout** as a unified interface. The system context dictates what appears in the terminal and how the chips are populated, ensuring a consistent function-over-form interaction model:

1.  **Command Context:** The terminal displays standard `stdout`/`stderr`. Chips populate with command history, autocomplete suggestions, and tool flags.
2.  **Search Context:** The terminal streams semantic or exact search results. Chips populate with search scopes, filters, and quick-action buttons for selected results.
3.  **AI Context:** The terminal displays the LLM's rationale, thought process, or raw output. The chips act as command staging buttons for the AI's suggested shell operations.
4.  **Directory Context:** When executing `ls` or `cd`, the terminal shows the raw directory listing. The chips dynamically populate with interactive file and folder paths for rapid prompt building. When applicable, chips also provide file or image previews.
5.  **Activity Context:** When executing `top` or `ps`, the terminal shows the raw process table. The chips populate with immediate process-handling actions (kill, renice, monitor). 
    - **Running Apps:** For user applications with active displays, these chips feature a **live 10Hz low-resolution thumbnail** of the application's surface.
    - **Inactive Apps:** Applications that are not currently running but are pinned or historically relevant appear as standard chip buttons featuring the **App Icon (Image) and Name** without the live viewport.
    - **Generic/System Processes:** For background processes or applications without a defined icon or active frame buffer, the chip displays a generic **System Node Icon (Symbolic Placeholder)** and the **Process Name**.

### 5.3 Dual-Sided Chip Layout
In Level 2, the viewport features dynamic vertical chip columns floating over the terminal output (but inside the bounds of the Lateral Bezels). These chips physically manifest the Contextual Augmentations described above:
*   **Left Chips (Context & Options):** Static or slowly changing context (Favorites, Pinned Paths, Directory Nav trees, File targets, Search Filters).
*   **Right Chips (Priority Stack & Actions):** Highly dynamic, predictive context (Command Completions, AI-suggested commands, Actionable alerts, Process kill-switches). Driven by the Priority Indicator engine.

### 5.4 Secondary Select (Context Menus)
Interaction with chips supports a **Secondary Select** state (triggered by long-press >500ms or right-click). This summons a glassmorphism context menu with specialized operations:

#### 5.4.1 File & Directory Chips
- **[Inspect Path]:** Transition to **Level 4 (Detail View)** for metadata and cryptographic verification.
- **[Open With...]:** Select from a list of compatible Application Models.
- **[Stage Action]:** Copy path to the active command prompt without submitting.
- **[Trust Tier...]:** Manually elevate or restrict the path's security context.
- **[Purge Nodes]:** Destructive deletion (requires **Confirmation Slider**).

#### 5.4.2 Process & App Chips
- **[Tactical Signal...]:** Sub-menu to send `SIGINT`, `SIGTERM`, or `SIGKILL` directly to the PTY/Process.
- **[Renice Priority]:** Adjust the process priority (LCARS levels 1-5).
- **[Inspect Buffer]:** Transition to **Level 4 (Buffer View)** for raw memory/IO monitoring.
- **[Isolate Process]:** Force the process into a more restrictive sandbox tier.
- **[Clone to Sector]:** Duplicate the process state in a new terminal sector.

---

## 6. Aesthetics, Themes, and Multi-Sensory Design

### 6.1 Theme Modules
The Face supports full re-theming via **Theme Modules** (`module.toml`, as defined in [Ecosystem Spec §1.6](./TOS_alpha-2_Ecosystem-Specification.md)). 
*   Themes define CSS variables injected into the HTML root (colors, border radii, glassmorphism opacities).
*   Can distribute custom fonts (`.ttf`/`.woff`) and icons.
*   Includes accessibility metadata (High Contrast, Reduced Motion flags).
*   TOS defaults to a dynamic LCARS dark theme with vibrant accent colors and neon/glow elements.

### 6.2 Priority-Weighted Visual Indicators
Important elements (Priority Chips, Bezel Alerts) use visual cues corresponding to their urgency (1 to 5):
*   **Color Shifts:** Subtle accents at Level 1; dominant hazard colors (Orange/Red) at Level 4.
*   **Pulsing Animations:** Critical alerts may gently pulse their border opacity.
*   **Haptic / Audio Hooks:** High-priority visual changes trigger synchronized UI sounds or haptic pulses (if running on Android/XR).

### 6.3 Haptic & Audio Integration
The visual state is tightly coupled to non-visual feedback:
*   Mode switching, command execution, and Level zooming emit distinct "Earcons".
*   Scrolling the Cinematic Triangular terminal triggers subtle haptic detents.
*   Voice Input state is visually represented by a glowing microphone icon that syncs to user amplitude.

---

## 7. Platform Implementation & Rendering

This section defines how the Face composites the UI across different hardware environments.

### 7.1 Linux Wayland Implementation
- **Layer Shell:** The Face renders as a `wlr-layer-shell` on the `TOP` layer, ensuring it remains above all native applications unless explicitly toggled.
- **Surface Embedding:** Native Wayland windows are rendered into Level 3 viewports using `dmabuf` sharing. The Face acts as a sub-compositor, projecting the application's buffer onto the LCARS-themed surface.
- **Input Forwarding:** The Face intercepts all pointer/touch events. If an event occurs within a native application's bounds, the Face translates the coordinates and forwards the raw event to the application's `wl_surface`.

### 7.2 Android XR (OpenXR) Implementation
- **World Space Compositing:** The UI is not a 2D overlay but a set of three-dimensional cylinders and quads positioned in a "Cockpit" configuration around the user.
- **Action Mapping:**
  - `pinch_left`: Triggers `zoom_out`.
  - `pinch_right`: Triggers `zoom_in`.
  - `gaze_dwell`: Fires `select` semantic event.
  - `wrist_tap`: Fires `open_hub` semantic event.
- **Performance:** Uses `EGLImage` for high-throughput terminal rendering to avoid CPU pipeline stalls.

### 7.3 Native Application Embedding (Wayland/X11)
To embed native apps into the Level 3 focus:
1. **Virtual Output:** TOS provides a virtual `wl_output`.
2. **Composition:** Application textures are mapped to specific logical areas which are then composited into the Level 3 texture with a glassmorphism border.
3. **Bezel Overlay:** The Tactical Bezel is rendered on top of the native app, providing system-level "Close" and "Inspect" triggers via `xdg_toplevel` signals.

---

## 8. UI Module Interaction APIs

Terminal and Bezel modules interact with the Face via these specific UI-hooks:

### 8.1 Terminal Output API
- **`render(surface, lines)`:** The Face provides a `RenderSurface` (DOM or GPU buffer). The module is responsible for font-rendering and ANSI color application.
- **`on_click(x, y)`:** Returns the line index and context-action (e.g., `copy`, `inspect_pid`).
- **`on_scroll(delta)`:** Handles the visual transition of lines.

### 8.2 Bezel Component API
- **`update_view(html, data)`:** Components push their rendered state to the Face.
- **`component_click(id, x, y)`:** The Face forwards clicks on specific component IDs to the underlying module.
- **`request_projection(mode)`:** Components can request to "unfurl" a detailed panel (e.g., the Mini-Map expanding into the viewport).

---

## 9. User Interaction & Accessibility

### 9.1 Default Keyboard Shortcuts
The interaction model respects the vertical hierarchy.

| Key Combination | Semantic Event | Description |
|---|---|---|
| `Ctrl + [` | `zoom_out` | Move one level up in hierarchy. |
| `Ctrl + ]` | `zoom_in` | Move one level down into focus. |
| `Ctrl + Space` | `toggle_bezel` | Expand/Collapse the Top Bezel. |
| `Ctrl + /` | `set_mode_ai` | Focus prompt and switch to AI mode. |
| `Ctrl + T` | `new_sector` | Create a new sector. |
| `Alt + [1-9]` | `switch_sector` | Rapidly switch between first 9 sectors. |
| `Ctrl + M` | `toggle_minimap` | Show/Hide the Tactical Mini-Map. |
| `Ctrl + Alt + Backspace`| `tactical_reset`| Trigger immediate Tactical Reset (Level 4 God Mode). |

Users can remap all shortcuts via the Settings panel, which provides a visual conflict detection interface.

### 9.2 Voice Command Grammar
Voice input is processed context-sensitively. Commands are structured as `Action + Target + [Modifier]`.

| Command Pattern | Example | Logical Translation |
|-----------------|---------|---------------------|
| "Focus [Sector]" | "Focus Development" | `sector_zoom:dev_uuid` |
| "Run [Command]" | "Run build script" | `prompt_submit:./build.sh` |
| "Inspect [Target]" | "Inspect browser" | `zoom_to:level_4;pid_1234` |
| "Alert Status" | "Report alert status" | TTS summary of priority chips. |
| "Stop everything" | "Stop everything" | `tactical_reset_system` |

### 9.3 Accessibility Profiles
TOS supports several distinct interaction profiles for diverse user needs:
- **Switch Scanning:** Automatically cycles focus through bezel components and chips. Supports 1-switch (timed) or 2-switch (move/select) modes.
- **Dwell Clicking:** Used in Gaze and Head tracking scenarios. Staring at an element for 500ms (configurable) triggers a `select` event.
- **High-Visibility Mode:** Forced thick borders, monochromatic glassmorphism for better contrast, and increased font sizes.
- **Screen Reader Bridge:** Every UI element publishes a semantic role (button, line, chip) to the platform's accessibility bridge (AT-SPI / TalkBack).

## 10. Intuitive Interaction & Predictive Fillers
To fulfill the "Augmented Desktop Entity" philosophy, the Face implements **Predictive Fillers** that minimize manual typing and cognitive friction.

### 10.1 Predictive Path & Command Chips
As the user interacts with the **Bottom Bezel Segment (Unified Prompt)**, the **Dual-Sided Chip Layout** (§5.3) populates with "Intuitive Fillers":
- **Path Completion (Left Chips):** Typing a directory separator `/` or starting a path triggers immediate chips for the most frequent/recent child nodes at that path depth.
- **Parameter Hints (Right Chips):** For known commands (e.g., `git`, `docker`, `npm`), the Priority Indicator engine suggests the most likely next arguments or flags as clickable chips.
- **Command History Echo:** Suggestions based on commands previously executed *within the current sector* appear with a subtle "History" icon.

### 10.2 Implicit Search & Typo Correction
If a user submits a command that results in a "File not found" or "Command not found" state:
- The **Search Service** (§7.2) performs a background fuzzy-match.
- A **Typo Correction Chip** appears in the Right column (e.g., "Did you mean `ls -la`?").
- Clicking the chip replaces the prompt and re-submits automatically.

### 10.3 Dynamic Sector Labeling
When creating a new sector, the "New Sector" name is a placeholder. As the user navigates:
- The system **heuristically renames** the sector based on the `Cwd` (e.g., "TOS Core" if in `~/TOS-Desktop-Environment/src`).
- This can be locked by the user to prevent auto-renaming.

### 10.4 PTY Output Extraction
For long terminal outputs (e.g., a build failure):
- The system highlights the **autoritative error line** with a higher priority (visual weight/color).
- A **"Focus Error" Chip** appears, which when clicked, scrolls the terminal to the specific failure point.

### 10.5 Notification Display Center
Notifications appear in the **Right Lateral Bezel** and unfurl inward.
- **Priority 1-2 (Normal):** Quiet slide-in, disappears after 5s.
- **Priority 3 (Warning):** Amber pulse, remains until dismissed.
- **Priority 4-5 (Critical):** Red border, accompanied by a tactical earcon and haptic pulse. Requires manual interaction or "Clear" voice command.
