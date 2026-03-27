# Project Plan: LCARS Spatial Desktop Environment (SDE)
## Version 2.0: Rust-WPE Integration

## 1. Executive Summary
A Linux Desktop Environment merging the **LCARS** aesthetic with an **Infinite Canvas** navigation system. The system uses a high-performance Rust engine to manage spatial window positioning, while utilizing **WPE WebKit** to render the LCARS interface via React/WASM for a cinematic, hardware-accelerated user experience.

---

## 2. Technical Stack
* **Bottom Layer (The Engine):** **Rust** using the **Smithay** library to build a custom Wayland Compositor.
* **Rendering Layer (The UI):** **WPE WebKit**. This allows the React-based LCARS UI to be rendered as a high-performance Wayland surface without the overhead of a full browser (like Chrome/Electron).
* **Communication Bridge:** **WPEBackend-fdo**. This allows the Rust compositor to "grab" the rendered UI frames from WPE and place them on the canvas.
* **Logic/Data:** **Nushell** for structured data, piped into the UI via a custom IPC (Inter-Process Communication) bridge.

---

## 3. Revised Architecture (The "WPE" Way)

### A. The Compositor (Rust/Smithay)
- Handles the $x, y, z$ coordinate system for the "Prezi-style" zooming.
- Manages standard Wayland clients (Firefox, LibreOffice, etc.) as sub-textures.
- Runs the **WPE view** as the primary "Shell" layer.

### B. The UI Shell (React/WASM via WPE)
- **Footer:** A persistent React component rendered by WPE, anchored to the bottom using the `layer-shell` protocol.
- **Overlays:** Touch-friendly buttons that send command strings to the underlying Nushell instance.
- **Performance:** WPE provides hardware-accelerated CSS transforms, ensuring LCARS animations stay at 60fps.

---

## 4. Development Roadmap

### Phase 1: The Core Compositor
- [ ] Initialize a Smithay-based Wayland compositor.
- [ ] Implement a 3D transformation matrix for "Surfaces" to allow infinite zooming and panning.

### Phase 2: WPE Integration
- [ ] Embed **WPE WebKit** using the `fdo` backend.
- [ ] Route input events (touch/gestures) from Rust to the WPE surface.
- [ ] Implement transparency support so the LCARS "Elbows" can sit over the zooming windows.

### Phase 3: The Command Link
- [ ] Build a JSON-RPC bridge between the **React UI** and the **Rust Compositor**.
- [ ] **Action:** User touches a file icon → React sends `OPEN_FILE` to Rust → Rust spawns the app and "Zooms" the canvas to that new window's coordinates.

---

## 5. Why WPE WebKit?
1. **Low Memory Footprint:** Significantly lighter than Electron or standard WebKit.
2. **Wayland Native:** Designed to be a Wayland client from the ground up.
3. **No Window Decorations:** It renders only the web content, which is essential for a "full-screen" OS feel.
4. **Hardware Acceleration:** Direct access to the GPU via GStreamer and OpenGL/Vulkan.

---

## 6. Definitions & Goals
* **Spatial Navigation:** Windows are managed in depth, not just layers.
* **Direct Manipulation:** Touching a command in the overlay immediately populates the persistent terminal prompt.
* **Open Source Acceleration:** Leverage the `cog` (WPE launcher) source code to accelerate the WPE-to-Wayland integration.
