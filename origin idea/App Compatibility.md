# App Compatibility (Wayland/X11 Integration)

A core requirement of TOS is seamless compatibility with existing Linux applications. These external applications are integrated into the **Modular Zoom Hierarchy** as children of the App Launcher (**Level 2**).

---

## Integration into the Hierarchy

### Level 1 & 2: Structural Representation
At the Overview and Sector levels, applications exist as **Nodes** in the state stack.
- **Visual**: The compositor provides an icon or a texture-buffer thumbnail of the application.
- **State**: The application may be running in the background, but its visual state is frozen or throttled (see `Performance.md`).

### Level 3: The Active Surface
When the user zooms into an application (**Level 3: The Focus**), the compositor hands off the rendering surface to the Wayland/X11 client.

*   **LCARS Frame Wrapping**: The compositor wraps the application texture in an **LCARS-styled window decorator**. This frame is the functional evolution of the App Button border from Level 2, morphed during the zoom transition.
*   **Window Decorations**: TOS uses **Server-Side Decorations (SSD)**. The compositor draws the frame, buttons, and "Split" controls.

---

## Technical Layers

### Wayland & XWayland
The system is built on **Wayland**, using **XWayland** as a bridge for legacy X11 applications.

1.  **Framebuffer Capture**: The compositor receives the window's content as a texture.
2.  **Protocol Mediation**: The compositor intercepts standard window management requests (maximize, close, tile) and translates them into the TOS vertical zoom or split-screen actions.

### Legacy Mode
For apps that require their own decorations, a "Legacy Mode" can be toggled in settings. This disables the modular LCARS frame but still forces the window to participate in the zoom hierarchy (appearing as a thumbnail at Level 2).
