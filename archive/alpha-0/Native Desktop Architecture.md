# Native Desktop Architecture: TOS as a Primary Environment

In a traditional implementation, TOS functions as a native Linux Desktop Environment (DE), similar to GNOME or KDE. It runs directly on the hardware, managing the entire display and input stack.

---

## 1. The Native Stack

Unlike the SaaS model, the traditional implementation prioritizes direct communication between the UI and the kernel.

### A. The Core Compositor (`tos-comp`)
*   **Role**: A standalone Wayland compositor.
*   **Technology**: Built in Rust using the **Smithay** library for low-level Wayland protocol handling.
*   **Direct Rendering Manager (DRM)**: The compositor talks directly to the Linux kernel via the DRM/KMS subsystem, bypassing any intermediaries for maximum performance.

### B. The Embedded WebView interface
*   **Role**: Renders the LCARS UI layers.
*   **Technology**: Uses `wry` (the library behind Tauri) to embed a lightweight webview directly into the compositor's rendering loop.
*   **Composition**: The webview's output is treated as a high-level texture overlay, and the compositor handles the "Recursive Zoom" transformations of application windows beneath it.

---

## 2. System Level Components

A traditional DE requires several "supporting" services to be functional for daily use:

| Component | Responsibility |
|-----------|----------------|
| **Display Manager** | Support for SDDM, GDM, or Ly to handle user login and session startup. |
| **D-Bus Integration** | Implementing standard Freedesktop.org portals for screen sharing, notifications, and file opening. |
| **Systemd Units** | Managing the lifecycle of the compositor, shell modules, and background daemons (Voice, Input). |
| **NetworkManager/PipeWire** | Using native LCARS widgets to control the underlying system services directly via D-Bus. |

---

## 3. The Local Shell Loop

In this mode, the **TOS Shell API** operates with zero network latency.

*   **Native PTY**: The compositor spawns a local PTY for the Fish shell.
*   **Direct Memory Access**: Metadata injected via OSC sequences (for file thumbnails and previews) is passed via shared memory between the shell process and the compositor, allowing for instantaneous Level N directory updates.
*   **Hardware Keyboard/Mouse**: Input is handled via `libinput`, allowing for custom gesture mappings that feel as responsive as core OS functions.

---

## 4. Advantages of Traditional Deployment

1.  **Zero Latency**: No network overhead means the "Morphing Transitions" can run at the monitor's native refresh rate (144Hz+).
2.  **Full Hardware Access**: Native access to specialized hardware (Styli, high-end GPUs, Multi-monitors) without passthrough configuration.
3.  **Offline Capability**: The OS is fully functional without an internet connection, including local file management and terminal access.
4.  **Security**: Data never leaves the physical machine, and user sessions are managed by the local Linux kernel.
