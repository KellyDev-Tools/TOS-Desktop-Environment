# Architecture & Implementation Plan: LCARS Spatial Desktop (SDE)

## 1. System Philosophy
To move beyond the "stacking windows" metaphor into a **Spatial-Command Hybrid**. The OS treats the workspace as an infinite 3D plane where navigation is performed via geometric transformations (zoom/pan) rather than minimizing/maximizing.

---

## 2. The Tech Stack (The "Four Pillars")

| Layer | Technology | Role |
| :--- | :--- | :--- |
| **Kernel & Input** | Linux + libinput | Hardware abstraction and multi-touch gesture parsing. |
| **Compositor** | **Rust + Smithay** | The engine. Manages the $x, y, z$ coordinate space and window buffers. |
| **UI Renderer** | **WPE WebKit (FDO)** | Renders the React-based LCARS shell as a hardware-accelerated Wayland surface. |
| **System Logic** | **Nushell** | Provides structured data (JSON) to the UI and executes terminal commands. |

---

## 3. Detailed Architecture

### A. The "Infinite Canvas" Coordinate System
The compositor does not use absolute pixel coordinates for windows. Instead, it uses a **Global Scene Graph**:
* **World Space:** An infinite grid where windows are placed at $(x, y, z)$.
* **Viewport:** The user's screen acts as a camera moving through this space.
* **Zooming:** Handled by scaling the Viewport's $z$-axis. As $z$ increases, textures are sampled at higher resolutions or simplified (Semantic Zooming).



### B. The WPE-Rust Bridge (Data Scheme)
Communication between the **React LCARS Overlay** and the **Rust Compositor** occurs via a Unix Domain Socket using JSON-RPC.

**Sample Command (Zoom to App):**
```json
{
  "jsonrpc": "2.0",
  "method": "viewport_transition",
  "params": {
    "target_surface_id": "window_uuid_01",
    "duration_ms": 400,
    "easing": "cubic-bezier"
  }
}
