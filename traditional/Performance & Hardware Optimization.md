# Performance & Hardware Optimization (Native)

Implementing TOS natively allows for aggressive optimization that isn't possible in a SaaS or containerized environment. This document focuses on leveraging local hardware for the "Recursive Zoom" metaphor.

---

## 1. Zero-Copy Window Management

In a native environment, the compositor can optimize how application windows are displayed within the LCARS frame.

*   **Direct Scan-out**: When a single application is zoomed to **Level 3 (Focus)**, the compositor can utilize hardware planes to "scan out" the application's surface directly to the monitor. This bypasses the composition step entirely, reducing input lag to the absolute minimum of the hardware.
*   **DMA-BUF Sharing**: Use of Linux DMA-BUF allows the compositor to share textures between the UI rendering (Rust/WebView) and the application rendering without copying data through the CPU.

---

## 2. Advanced Input Handling

Direct access to `libinput` enables a more "tactical" feel for the interface:

*   **High-Resolution Scroll/Pinch**: Smooth, sub-pixel zooming through the hierarchy levels.
*   **Gesture Prediction**: Using movement velocity to "snap" to the next level (e.g., a fast pinch-out automatically transitions from Level 3 to Level 1, skipping Level 2 if the velocity is high enough).
*   **Switch Device Support**: Native drivers for assistive switches, allowing the LCARS interface to be navigated by any hardware trigger.

---

## 3. Storage Performance (Local Filesystem)

The **Spatial File Browser** benefits from local NVMe speeds:

*   **Inotify Sync**: Real-time updates of directory thumbnails. If a file is downloaded in a background terminal, the Level N view updates the visual tile instantly.
*   **Aggressive Caching**: TOS maintains a local cache of high-resolution thumbnails for all files in recently visited directories, enabling the "Vertical Depth" navigation to feel instantaneous.

---

## 4. Resource Allocation

As the primary DE, TOS has control over system resources:

*   **Cgroups Integration**: TOS assigns desktop-critical processes (The compositor and active Level 3 app) to high-priority CGroups, ensuring that a background compile or heavy render doesn't cause the UI zoom transitions to stutter.
*   **GPU Priority**: Leveraging Vulkan's priority queues to ensure the LCARS UI is always rendered first, maintaining a consistent 60+ FPS even under heavy system load.
