# Native Roadmap: Building a Hardware-Ready Desktop

This roadmap outlines the steps to move from the "Origin Idea" to a fully functional, installable Linux Desktop Environment.

---

## Phase 1: Core Compositor Development
*   **Goal**: A stable Wayland compositor capable of hosting a WebView.
*   **Key Tasks**:
    - Implement basic DRM/KMS backend using Smithay.
    - Integrate the Wry/WebView as a persistent top-layer overlay.
    - Build the `path` stack logic for basic Zoom In/Out navigation between placeholders.

## Phase 2: Shell & Application Wrapping
*   **Goal**: Running real Linux apps inside the LCARS frame.
*   **Key Tasks**:
    - Implement XWayland support for legacy compatibility.
    - Create the "Morphing SSD" (Server-Side Decoration) logic that transforms buttons into window frames.
    - Develop the Fish shell module for basic metadata injection (CWD updates).

## Phase 3: Spatial UX & System Services
*   **Goal**: A functional desktop experience.
*   **Key Tasks**:
    - Build the **Spatial File Browser** (Level N) with native inotify support.
    - Implement the **Dashboard & Widgets** (System monitors, terminal history).
    - Develop the D-Bus services for standard portals (File Pickers, Notifications).

## Phase 4: Distribution & Polish
*   **Goal**: Public release as an installable DE.
*   **Key Tasks**:
    - Create installation scripts and packages for major distros (Arch, Fedora).
    - Finalize the Voice synthesis and Auditory feedback loop.
    - Perform extensive hardware compatibility testing (Laptops, Multi-monitor setups).

---

## Conclusion

Implementing TOS in the traditional manner provides the most powerful and responsive version of the "Origin Idea." By running directly on the hardware, TOS can deliver the seamless, high-fidelity spatial transitions and tactical "depth" that define the project vision, making it a viable daily-driver desktop environment for Linux power users.
