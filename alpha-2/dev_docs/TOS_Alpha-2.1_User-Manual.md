# TOS Alpha-2.1 End-to-End User Manual

## 1. System Philosophy: The Augmented Desktop Entity
TOS (The Operating System) is not a static workspace; it is a **dynamic augmented desktop entity**. Inspired by LCARS design principles, it prioritizes hierarchical depth, multi-sensory feedback, and context-aware rendering.

---

## 2. Navigation Architecture: Hierarchical Levels
TOS uses a 6-level depth system to allow rapid transitions between high-level oversight and low-bit buffer inspection.

| Level | Name | Description | Visualization |
| :--- | :--- | :--- | :--- |
| **LVL 1** | **Global Overview** | Tactical map of all active system sectors. | Topological nodes |
| **LVL 2** | **Command Hub** | The primary workspace for shell and data interaction. | Dual-column chip-terminal |
| **LVL 3** | **Application Focus** | Dedicated window surface for a single graphical process. | Chrome-window overlay |
| **LVL 4** | **Detail Inspector** | Real-time metadata, cryptographic hashes, and ownership data. | Property chip layout |
| **LVL 5** | **Buffer View** | Raw telemetry stream and hex data inspection. | Monospace hex viewer |
| **LVL 6** | **Spatial View** | 3D-transformed XR workspace for spatial sector mapping. | Perspective glass panels |

---

## 3. The Command Hub (LVL 2) Modes
Run commands or use AI to modulate the Hub's rendering state.

*   **[CMD] Command Mode**: Standard interactive PTY terminal.
*   **[SEARCH] Search Mode**: Semantic or global FS indexing with instant results.
*   **[AI] AI Augmentation**: Natural language shell queries with explanation and staging.
*   **Directory Context**: Triggered by `ls` or `cd`. Shows real-time file previews.
*   **Activity Context**: Triggered by `top` or `ps`. Shows "Activity Lungs" with live process window snapshots.

---

## 4. Slot Architecture: Bezel Docking & Projection
The UI is partitioned into **Symmetrical Bezel Segments** (§8.1) with modular slots.

### Sidebar Slots (Left/Right)
Components like the **Minimap**, **Priority Alert Section**, and **Mini-Log** can be docked into lateral slots. 
*   **Lateral Shift (Ctrl+S / Ctrl+R)**: Reassigns slots to primary viewport projection.
*   **Bezel Projection**: Clicking a bezel segment expands its associated docked component.

### Top Bezel Segments
Specifically partitioned for high-frequency awareness:
*   **Left (Handles)**: Screen title and hierarchy level mapping.
*   **Center (Telemetry)**: Real-time Brain clock and system performance metrics.
*   **Right (Controls)**: Global toggles, Settings Access, and the **Web Portal** satellite button.

---

## 5. Multi-Sensory Interface
TOS uses immersive feedback loops to minimize cognitive load during sensitive system operations.

*   **Earcons**: Distinct sinusoidal audio cues for mode switches (`nav_switch`), modal actions (`modal_open`), and data commits (`data_commit`).
*   **Tactical Vibration**: Haptic pulses for physical confirmation of virtual actions (e.g., successful link generation).
*   **Visual Filters**: "Sensor-grade" pixelated rendering for remote snapshots to maintain LCARS cohesive aesthetics.

---

## 6. Global Shortcuts & Operations
*   **Ctrl+M**: Toggle Minimap Projection.
*   **Ctrl+P**: Toggle Priority Stream.
*   **Ctrl+Tab**: Cycle through active Sector Indexes.
*   **Esc**: Dismiss all system modals (Settings / Portal).
*   **📡 Status Badge (Top Right)**: Generate a secure **Web Portal** token for remote collaboration.

---

## 7. Configuration: Systems Settings
Access the **System Settings** modal (⚙ icon) to adjust:
1.  **Global Parameters**: Set default wallpapers, network masks, and system keys.
2.  **Sector Rules**: Adjust sandboxing tiers and resource limits per sector.
3.  **Interface Calibration**: Toggle multi-sensory feedback and UI animation speeds.

---
*TOS Alpha-2.1 // Interface Specification Version 3.8 // Authorized Access Only*
