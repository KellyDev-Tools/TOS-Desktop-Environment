# Dashboard & UI Components

This document defines the high-level UI components and dashboard widgets that populate the TOS environment, specifically within the **Global Overview (Level 1)** and **Sector Launcher (Level 2)** contexts.

---

## 1. Dashboard & Launcher Depth

The zoomed-out views (Level 1 and 2) are not just navigation hubs but functional dashboards providing real-time system intelligence.

### System Monitors (Tactical Readouts)
*   **Visual**: Integrated into the LCARS "elbows" and side panels of the Level 1 Overview.
*   **Data**: Real-time visualization of CPU load, Memory usage, Network throughput (Up/Down), and Power/Battery status.
*   **Interaction**: Tapping a monitor widget zooms the user into a dedicated "System Diagnostics" Sector for detailed analysis.

### Recent Activity & History
*   **Recent Files**: A sliding panel or grid section showing the last used documents/media. These act as "Staged" targets for the **Persistent Unified Prompt**.
*   **Terminal History**: A scrollable LCARS list showing recent command executions. Selecting an entry populates the current prompt for re-execution or editing.

### Command & Management Widgets
*   **High-Frequency Buttons**: Dedicated LCARS buttons for common utilities (e.g., `ls`, `cd ~`, `top`, `clear`). These are user-configurable via the settings panel.
*   **Touch-Friendly Clipboard**: A dedicated "Transporter Buffer" (clipboard) UI.
    - **Visual**: A persistent or slide-in panel showing current clipboard contents (text snippets or file thumbnails).
    - **Logic**: Provides large, touch-optimized "Copy", "Cut", and "Paste" targets that interface with the system-wide Wayland clipboard.

---

## 2. Spatial Navigator UI

To prevent disorientation within the Recursive Zoom Hierarchy, the **Spatial Navigator** provides continuous contextual awareness.

### The Tactical Mini-Map
*   **Visual**: A translucent, minimalist wireframe overview situated in an upper corner (typically top-left).
*   **Function**: Shows the current Viewport's position within the global tree. 
    - **Level 1**: Highlighted sectors.
    - **Level 2/3**: A "pip" indicating depth and parent-child relationships.
*   **Interaction**: Clicking a sector in the mini-map triggers an **Automated Vertical Transition** to that location.

### Zoom Level Indicators
*   **Visual**: Subtle numeric or iconographic markers (e.g., "L1", "L2", "L3") integrated into the main LCARS frame.
*   **Feedback**: Changes color or pulsates during transitions to indicate movement through the hierarchy.

### Voice & Audio Feedback
*   **Voice Status**: Audible confirmation of navigation actions (e.g., "Zooming to Browser", "Sector 4 Active"). Powered by a PipeWire-linked speech synthesis daemon.
*   **Auditory Interface**: High-fidelity LCARS "chirps" and "beeps" provide tactile feedback for successful inputs, while low-frequency tones indicate errors or blocked actions.

---

## 3. Help System & Context Overlays

The Help System provides real-time, non-intrusive guidance that learns from user behavior.

### Context-Aware Prompt Suggestions
*   **Logic**: As the user types into the **Persistent Unified Prompt**, an overlay panel appears with suggested sub-commands or flags based on the current `cwd` and command history.
*   **Visual**: A vertical LCARS menu that slides out from the prompt area.

### Previews & Overlays
*   **File Previews**: Hovering or long-pressing a file tile in the Launcher or File Browser spawns an overlay panel with metadata (size, MIME type) and a high-resolution thumbnail/preview.
*   **Directory Peek**: Allows the user to "peek" into a directory context (Level N+1) without fully zooming in, rendered as a translucent overlay.

---

## 4. Interaction Framework

*   **LCARS Standard Library**: All components must use the standardized TOS CSS library to ensure consistent color shifts (e.g., "Alert" state turning panels red) and sliding animations.
*   **Transient States**: Popups, help panels, and terminal history are rendered as **Temporary Overlays** that sit above the zoom hierarchy, ensuring the navigation stack is never disrupted by transient interactions.
