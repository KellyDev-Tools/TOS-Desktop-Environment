# Performance (Spatial Hierarchy Optimization)

Ensuring a smooth experience depends on optimizing the **Recursive Zoom Hierarchy**.

---

## Level-Based Performance Strategies

### Level 1 & 2: Overview/Launcher
At these levels, the user sees many potential children (Sectors or Apps).

*   **Image Buffer Rendering**: Instead of live windows, apps at Level 2 are rendered as **static textures** captured from their last state.
*   **Update Throttling**: These textures are updated at a low frequency (e.g., 300ms) or only when the window gains focus.
*   **Level of Detail (LOD)**: Level 1 (Overview) uses low-resolution mipmaps of each Sector.

### Level 3: Focus
At this level, the user is interacting with a single application or window.

*   **Live Injection**: The texture is replaced with the live application framebuffer for zero-latency interaction.
*   **Background Suspension**: Components at Level 2 (other app buttons) are not rendered to save VRAM and GPU cycles.

---

## Technical Implementation

### GPU Acceleration: Vulkan (wgpu)
We use `wgpu` to manage the spatial transitions. The compositor treats the WebView-rendered UI (HTML/CSS) as a **Live Texture**.

*   **Texture-Pass Architecture**: The WebView renders the LCARS interface to an off-screen buffer. This buffer is then fed into the `wgpu` graphics pipeline.
*   **Scale/Opacity Interpolation**: Spatial transitions (Zoom In/Out) are performed on these textures via vertex shaders. This ensures that even heavy CSS layouts remain fluid during rapid zooming.
*   **Texture Atlases**: Thumbnails for Level 2 and Level 3a (Window Picker) are stored in shared atlases to minimize draw calls.

### VRAM Management
*   **Depth-Based Pruning**: Any node in the hierarchy further than 2 steps from the current `currentDepth` has its pixel data purged from VRAM, keeping only metadata in the stack.
