# Blueprint: LCARS Spatial Desktop Environment (SDE)
**Version:** 3.0 (Full Integration)
**Concept:** A High-Performance, Zoomable, Command-Centric Linux Desktop.

---

## 1. Vision & Core Philosophy
The LCARS SDE moves away from the "Windowed Folder" metaphor. It treats the OS as a **Navigable Information Space**.
* **Infinite Canvas:** Every application is a surface on a massive 3D plane.
* **Command Hybrid:** A persistent terminal at the base provides instant execution, while touch-optimized LCARS "elbows" provide contextual shortcuts.
* **Direct Retrieval:** File interaction is based on MIME-aware "Action Profiles" rather than generic right-click menus.

---

## 2. Technical Architecture Stack

| Layer | Component | Technical Specification |
| :--- | :--- | :--- |
| **I/O & Kernel** | Linux Kernel | Handles DRM/KMS (Graphics) and libinput (Gestures). |
| **Compositor** | **Rust + Smithay** | Manages the Scene Graph ($x, y, z$ coordinates) and Wayland protocols. |
| **UI Renderer** | **WPE WebKit** | Hardware-accelerated Webview via `WPEBackend-fdo` for the React/WASM shell. |
| **Data Engine** | **Nushell** | Provides structured JSON output for directory and system data. |
| **IPC Bridge** | **Unix Sockets** | High-speed JSON-RPC communication between the UI and the Engine. |

---

## 3. Spatial System Design

### A. Viewport & Transformation Logic
The compositor maintains a global transformation matrix. 
- **Zooming:** Pinching adjusts the $z$-axis scale.
- **Panning:** Two-finger drag translates $x$ and $y$.
- **Focusing:** When a file is opened, the compositor calculates a transition to the app's coordinates:
  $$T(x, y, z) \rightarrow \text{Target App Surface}$$



### B. The Layer Shell (HUD)
The LCARS frame (buttons and terminal) is rendered as a **Wayland Layer Shell**. 
* **Exclusive Zone:** It occupies a fixed area at the bottom that apps cannot overlap.
* **Static Scale:** While the workspace zooms, the HUD stays at 1:1 pixel perfection for legibility.

---

## 4. Data Scheme & File Handling

### A. MIME-Aware Data Flow
When a directory is accessed, the system fetches "Entity" objects.

**Data Packet (Nushell → React):**
```json
{
  "type": "DIRECTORY_LISTING",
  "path": "/root/logs",
  "items": [
    {
      "name": "diagnostic.sh",
      "mime": "application/x-shellscript",
      "lcars_class": "CLASS_EXEC",
      "actions": ["RUN", "DEBUG", "EDIT"]
    },
    {
      "name": "sensor_feed.mp4",
      "mime": "video/mp4",
      "lcars_class": "CLASS_MEDIA",
      "actions": ["PLAY", "STREAM", "ANALYZE"]
    }
  ]
}
