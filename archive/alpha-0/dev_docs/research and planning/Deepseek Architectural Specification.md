# Architectural Specification: LCARS Spatial Desktop Environment (SDE)
**Model:** Hybrid Pluggable Micro-Shell (v3.1)

---

## 1. System Overview
This architecture combines a high-performance, unified **Rust Core** with a flexible **Plugin UI Layer**. The system separates the "How" (spatial management and window compositing) from the "What" (the LCARS aesthetic and interface design).

## 2. The "Micro-Shell" Core (Unified Services)
The Core is a set of persistent services that manage the heavy lifting of the OS.

### A. Spatial Compositor (`sde-core`)
* **Technology:** Rust + Smithay.
* **Responsibility:** Manages the Global Scene Graph using a transformation matrix for $x, y, z$ coordinates.
* **Spatial Logic:** Handles the "Prezi-style" zooming where windows are textures sampled at higher resolutions as the $z$-axis increases.
* **Input Routing:** Captures hardware gestures via `libinput` (pinch-to-zoom, two-finger pan) and converts them into coordinate transformations.

### B. Data & Command Engine (`sde-executor`)
* **Technology:** Nushell.
* **Responsibility:** Acts as the primary shell providing structured JSON output for system queries.
* **MIME Logic:** Analyzes files to provide "Action Profiles" (e.g., CLASS_MEDIA -> PLAY).

---

## 3. The Plugin Interface (The "Face")
Themes and UI layouts are treated as external plugins that communicate with the Core via standardized protocols.

### A. The UI Host (WPE WebKit)
* The Core embeds **WPE WebKit** using the `fdo` backend to render UI frames.
* This host acts as the "socket" into which the LCARS plugin is plugged.

### B. The LCARS Theme Plugin (`lcars-official`)
* **Frontend:** React + WASM.
* **Interface:** Implements the LCARS "elbows," the persistent terminal footer, and curved HUD elements.
* **Independence:** The plugin contains its own CSS, assets, and layout logic, allowing it to be updated without recompiling the Rust compositor.

---

## 4. Communication Bridge (JSON-RPC)
The Core and the Plugin interact through a Unix Domain Socket using a structured message scheme.

| Message Type | Direction | Data Payload Example |
| :--- | :--- | :--- |
| **Viewport State** | Core → Plugin | Current $(x, y, z)$ and active window ID. |
| **System Data** | Core → Plugin | JSON list of files with associated actions. |
| **User Command** | Plugin → Core | Command string to be injected into the Nushell PTY. |
| **Transition Req** | Plugin → Core | Request to zoom to a specific coordinate/app. |

---

## 5. Directory Structure
```text
/sde-workspace
├── /core (The Micro-Shell)
│   ├── /compositor        # Rust: Smithay, DRM/KMS, Spatial Matrix
│   ├── /bridge            # Rust: JSON-RPC server & Unix Socket
│   └── /pty_host          # Rust: Manages hidden Nushell instances
├── /data                  # Nushell: Scripts for file/system logic
└── /plugins               # Swappable UI Modules
    └── /lcars-theme       # React: UI Shell, CSS, and Animations
        ├── /components    # Elbows, Grids, Terminal UI
        └── /assets        # Star Trek inspired icons and audio
        