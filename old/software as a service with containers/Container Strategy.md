# Container Strategy: GPU & Environment Virtualization

To implement TOS as a containerized service, the backend must efficiently virtualize the Wayland compositor and the GPU-accelerated rendering pipeline.

---

## 1. The Container Image Construction

The TOS container image is built on a high-performance base (e.g., Arch Linux or Alpine for slimness) and includes:

*   **Compositor**: The custom Rust/Wayland compositor (`tos-comp`).
*   **Webview Bridge**: A headless browser instance (Playwright or custom Chromium build) to render the LCARS UI to a texture buffer.
*   **Virtual Display**: `wayland-vnc` or `kasm-vnc` for immediate fallback, but primary delivery via a custom WebRTC stream of the `wgpu` surface.

---

## 2. GPU Acceleration in Containers

A critical challenge is ensuring the **Recursive Zoom Hierarchy** remains fluid at 60fps within a remote container.

### A. Hardware Passthrough (NVIDIA/AMD)
*   **Technology**: `nvidia-container-toolkit`.
*   **Approach**: Map the host GPU directly into the container. This provides the lowest latency and allows the Rust `wgpu` backend to utilize native Vulkan shaders.

### B. Virtual GPU (vGPU)
*   **Scenario**: High-density multi-user environments.
*   **Approach**: Use technologies like NVIDIA GRID to slice a single physical GPU into multiple virtual instances, ensuring each user has guaranteed VRAM for their Sector thumbnails.

### C. Software Rasterization (Fallback)
*   **Technology**: SwiftShader or LLVMPipe.
*   **Scenario**: Running on CPU-only instances. The LCARS interface will still work, but complex Level 3 apps may experience lag.

---

## 3. Persistent Storage (The Sector Vault)

User data is decoupled from the container to allow for stateless instance management.

*   **Config Storage**: User LCARS settings, CSS theme overrides, and Sector layouts are stored in a distributed Key-Value store (Redis/Etcd).
*   **File Storage**: User home directories are mounted as **Persistent Volumes (PV)** via NFS or CEPH.
*   **Sector Snapshots**: Periodic serializations of the `path` stack are saved to a management DB, enabling "Resume Session" across different physical locations.

---

## 4. Input & Shell Integration

*   **Remote Shell API**: The Fish shell module running inside the container sends OSC escape sequences over a virtual terminal (PTY), which are intercepted by the containerized compositor.
*   **Touch/Gesture Relaying**: User gestures in the browser are captured as JSON events and piped into the container's input management layer, mimicking local Wayland input events.
