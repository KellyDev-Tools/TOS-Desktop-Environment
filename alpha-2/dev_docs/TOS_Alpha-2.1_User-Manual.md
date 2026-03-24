# TOS Alpha-2.1 End-to-End User Manual

## 1. System Philosophy: The Augmented Desktop Entity
TOS (**Terminal On Steroids**) is not a static workspace; it is a **dynamic augmented desktop entity**. Inspired by LCARS design principles, it prioritizes hierarchical depth, multi-sensory feedback, and context-aware rendering.

---

## 2. Navigation Architecture: Hierarchical Levels
TOS uses a 4-level depth system to allow rapid transitions between high-level oversight and low-bit buffer inspection.

| Level | Name | Description | Visualization |
| :--- | :--- | :--- | :--- |
| **LVL 1** | **Global Overview** | Tactical map of all active system sectors. | Topological nodes |
| **LVL 2** | **Command Hub** | The primary workspace for shell and data interaction. | Dual-column chip-terminal |
| **LVL 3** | **Application Focus** | Dedicated window surface for a single graphical process. | Chrome-window overlay |
| **LVL 4** | **Deep Inspection & Recovery** | Detail View (metadata), Buffer View (hex dump, privileged), and Tactical Reset (God Mode wireframe recovery). | Property chips / hex viewer / wireframe map |

---

## 3. The Command Hub (LVL 2) Modes
Run commands or use AI to modulate the Hub's rendering state.

*   **[CMD] Command Mode**: Standard interactive PTY terminal.
*   **[SEARCH] Search Mode**: Semantic or global FS indexing with instant results.
*   **[AI] AI Augmentation**: Natural language shell queries with explanation and staging.
*   **Directory Context**: Triggered by `ls` or `cd`. Shows real-time file previews.
*   **Activity Context**: Triggered by `top` or `ps`. Shows "Activity Lungs" with hierarchical process visualization:
    - **Live View**: 10Hz snapshots for active, graphical applications.
    - **Resource View**: App icon and name for inactive or non-graphical applications.
    - **System View**: Symbolic placeholders for background and system processes.

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
*   **Long-Press / Right-Click**: Trigger the **Secondary Select** context menu on any chip (Files, Processes, Apps) to access deep operations like `Signal`, `Renice`, and `Inspect`.
*   **Ctrl+Alt+Backspace**: Trigger **Tactical Reset (Level 4 God Mode)** - The system-wide recovery fallback.
*   **📡 Status Badge (Top Right)**: Generate a secure **Web Portal** token for remote collaboration.

---

## 7. Configuration: Systems Settings
Access the **System Settings** modal (⚙ icon) to adjust:
1.  **Global Parameters**: Set default wallpapers, network masks, and system keys.
2.  **Sector Rules**: Adjust sandboxing tiers and resource limits per sector.
3.  **Interface Calibration**: Toggle multi-sensory feedback and UI animation speeds.

---

## 8. Desktop Application (Electron)

TOS is available as a native desktop application for **Windows**, **macOS**, and **Linux** via Electron.

### Installation

| Platform | Format | How to Install |
|----------|--------|----------------|
| **Windows** | `.exe` installer | Run the NSIS installer; creates Start Menu and Desktop shortcuts |
| **Windows** | Portable `.exe` | No installation required — run directly |
| **macOS** | `.dmg` | Open DMG, drag TOS to Applications |
| **Linux** | `.AppImage` | `chmod +x TOS-*.AppImage && ./TOS-*.AppImage` |
| **Linux** | `.deb` | `sudo dpkg -i TOS-*.deb` |
| **Linux** | `.rpm` | `sudo rpm -i TOS-*.rpm` |

### Launch

Once installed, TOS will:
1. Start the Face window with the full TOS interface
2. Create a **system tray icon** for quick access
3. Connect to the Brain via WebSocket (`ws://127.0.0.1:7001`)

### Platform Features

*   **System Tray**: Right-click the tray icon to access Brain status, Settings, and Quit.
*   **Deep Links**: Click `tos://` links to navigate directly (e.g., `tos://sector/3`, `tos://settings`).
*   **Auto-Update**: TOS checks for updates on launch and every 4 hours. When a new version is downloaded, you'll be prompted to restart.
*   **Window State**: Your window position, size, and maximized state are remembered across sessions.
*   **Native Menus**: macOS uses the full AppKit menu bar; Windows/Linux use platform-appropriate menus.
*   **Native Dialogs**: File open/save and print dialogs use your platform's native UI.

### Keyboard Shortcuts (Desktop App)

| Action | macOS | Windows/Linux |
|--------|-------|---------------|
| New Sector | `Cmd+N` | `Ctrl+N` |
| Settings | `Cmd+,` | `Ctrl+,` |
| Global Overview | `Cmd+1` | `Ctrl+1` |
| Command Hub | `Cmd+2` | `Ctrl+2` |

### Environment Variable

Set `TOS_BRAIN_WS` to connect to a remote Brain: `TOS_BRAIN_WS=ws://192.168.1.100:7001`

---
*TOS Alpha-2.2.1 // Interface Specification Version 4.0 // Terminal On Steroids // Authorized Access Only*
