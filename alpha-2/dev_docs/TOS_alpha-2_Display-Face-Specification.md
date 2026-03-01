# TOS Alpha-2 Display & Face Specification

**Purpose:** This document consolidates all architectural and design specifications related to the **Face** of the Tactical Operating System (TOS). It defines how the system looks, the visual mechanisms of user interaction, the layout of the UI elements, and the modular rendering systems that power the terminal and bezels. For comprehensive details on system logic, process architecture, and IPC boundaries that drive these visual elements, refer to the [TOS Core Architecture Specification](./TOS_alpha-2_Architecture-Specification.md).

---

## 1. Core Visual Philosophy: Terminal First
TOS is a terminal-centric environment. The command line and terminal output are the underlying graphical anchors. All visual augmentations (chips, modes, bezels) are overlays designed to empower the terminal, never to bypass or obscure it permanently. 

The Face is built on standard web technologies (HTML/CSS/JS) acting as a dynamic graphical **frontend** to the **brain_node** (see the [Architecture Specification](./TOS_alpha-2_Architecture-Specification.md) for backend process separation, native shell, and Wayland/XR/Android composition). The design language heavily features LCARS-inspired elements with modern additions like glassmorphism and smooth kinetic transitions.

---

## 2. The Visual Hierarchy (Levels 1–5)
The visual experience in TOS is structured as a vertical zoom hierarchy. The user moves "closer" to the data by zooming in.

- **Level 1 (Global Overview):** A bird's-eye view of all running Sectors represented as interactive live tiles. Behind the tiles sits the **System Output Area** (a read-only cinematic terminal logging global Brain telemetry).
- **Level 2 (Command Hub):** The heart of TOS. The viewport is dominated by the Sector's terminal output, flanked by dynamic, context-aware command chips (Dual-sided Chip Layout) and surrounded by the Tactical Bezel.
- **Level 3 (Application Focus):** The terminal recedes or shares space with an Application Surface (e.g., a Wayland window, browser, or remote desktop). The Tactical Bezel remains to provide system control.
- **Level 4 (Detail View) & Level 5 (Buffer View):** Deep inspection overlays for debugging, surface property manipulation, and hex editing, rendering over the active application.

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
    *   *Visual States:* Collapsed/Unexpandable (Level 1, 4, 5); Expanded (Level 2); Collapsed/Expandable (Level 3).
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
5.  **Activity Context:** When executing `top` or `ps`, the terminal shows the raw process table. The chips populate with immediate process-handling actions (kill, renice, monitor). For user applications with active displays, these chips also feature a live (updating every 100ms) low-resolution thumbnail of the application's surface.

### 5.3 Dual-Sided Chip Layout
In Level 2, the viewport features dynamic vertical chip columns floating over the terminal output (but inside the bounds of the Lateral Bezels). These chips physically manifest the Contextual Augmentations described above:
*   **Left Chips (Context & Options):** Static or slowly changing context (Favorites, Pinned Paths, Directory Nav trees, File targets, Search Filters).
*   **Right Chips (Priority Stack & Actions):** Highly dynamic, predictive context (Command Completions, AI-suggested commands, Actionable alerts, Process kill-switches). Driven by the Priority Indicator engine.

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
*   **Color Shifts:** Subtle accents at Level 1; dominant hazard colors (Orange/Red) at Level 5.
*   **Pulsing Animations:** Critical alerts may gently pulse their border opacity.
*   **Haptic / Audio Hooks:** High-priority visual changes trigger synchronized UI sounds or haptic pulses (if running on Android/XR).

### 6.3 Haptic & Audio Integration
The visual state is tightly coupled to non-visual feedback:
*   Mode switching, command execution, and Level zooming emit distinct "Earcons".
*   Scrolling the Cinematic Triangular terminal triggers subtle haptic detents.
*   Voice Input state is visually represented by a glowing microphone icon that syncs to user amplitude.
