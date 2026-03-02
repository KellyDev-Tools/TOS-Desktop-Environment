# TOS Alpha-2.1 Face (Frontend) Roadmap

This roadmap tracks the progress of the visual, rendering, and interaction layers of the TOS Face, separating frontend-specific goals from the backend logic.

## High-Fidelity Rendering & Themes
- [x] **Web Profile DOM Implementation:** Transition the Face layer from terminal-mock to a rich React/Vite-based LCARS interface.
- [x] **Cinematic Modules:** Implement the "Cinematic Triangular Module" with GPU-accelerated transitions.
- [x] **Dynamic Theme Engine:** Implement runtime CSS variable injection and multi-sensory asset swapping (SFX/Haptics).
- [x] **Configurable Bezel Architecture:** Implement omni-directional slots (Top, Left, Right) with modular component docking and downward/lateral slot projection.
- [x] **Top Bezel Evolution:** Partition the Top Bezel into Left/Center/Right zones (Left: Handles, Center: Telemetry, Right: System Controls).

## Context-Aware Augmented UI (Pending)
- [x] **Level 3-5 UI Rendering:** Currently, `ApplicationFocus`, `DetailView`, and `BufferView` fall perfectly flat on the terminal proxy UI. Replace with functional UI Overlays.
- [x] **Settings UI Panel:** Implement the LCARS-themed modal interfaces for configuring global parameters, mapping behaviors, and executing the standardized Settings IPC queries (§3.3.3).
- [x] **Directory Context Previews:** Implement inline file and image previews within the Dual-Sided Chip Layout when `ls` or `cd` is executed.
- [x] **Activity Context Live Thumbnails:** Render 10Hz live, low-resolution frame buffer thumbnails of application surfaces on right-side process chips during `top` or `ps`.
- [x] **Web Portal Sharing UI:** Create the temporary link generation overlay to copy and share secure one-time URL tokens for remote collaboration.

## Hardware Forms & Multi-Sensory
- [x] **OpenXR Spatial UI Shell:** Built CSS 3D-transformed floating panels with Z-layering and perspective for XR sector visualization.
- [x] **Multi-Sensory Audio Hooks:** Fully integrated localized SFX for mode transitions, modal triggers, and data commits using `rodio`.
- [x] **Tactical Vibration Hooks:** Implemented the `HapticService` with IPC haptic pulsing for tactile user feedback.

## Documentation
- [x] **End-to-End User Manual:** Finalized the user guide in `dev_docs/TOS-Alpha-2.1_User-Manual.md` focusing on Face interactions, terminal configurations, and bezel docking.

## Cross-Roadmap Dependencies
- **Settings UI Panel** is blocked by the **Settings Daemon** (Ecosystem Roadmap) handling the JSON persistence layer and cascading state resolution.
- **Activity Context Live Thumbnails** relies on Wayland frame buffers managed by the Backend Compositing (Brain Roadmap).
- **Directory Context Previews** is blocked by **Universal OSC Scripts** and JSON context definitions (Ecosystem Roadmap) driving the shell.
- **Multi-Sensory Audio Hooks** depends on the backend's **Multi-Sensory Audio Pipeline** initialization of OS audio sinks (Brain Roadmap).
