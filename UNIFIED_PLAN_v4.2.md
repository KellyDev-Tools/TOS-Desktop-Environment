Unified Implementation Plan: LCARS Spatial Desktop Environment (SDE)
Version: 4.2 (Structured & Ready)
Status: Ready for Development
Refined: Added standard toolchain subdirectories and asset paths.

1. Executive Summary
The LCARS SDE is a revolutionary Linux desktop environment that moves away from traditional "stacking windows" toward a Spatial-Command Hybrid. The system treats the workspace as an Infinite Canvas where navigation is performed via geometric transformations (zoom/pan). A persistent Nushell-based command bar at the base provides direct system control via structured JSON data.

2. Core Architecture (The "Four Pillars")
The system is built as a Hybrid Pluggable Micro-Shell, separating high-performance system tasks from the sensory UI layer.

| Layer | Technology | Primary Responsibility |
|---|---|---|
| I/O & Kernel | Linux + libinput | Hardware abstraction and multi-touch gesture parsing. |
| Core (Engine) | Rust + Smithay | Manages the Global Scene Graph and x, y, z transformation matrix. |
| UI Renderer | WPE WebKit (fdo) | Renders the React/WASM shell as a hardware-accelerated Wayland surface. |
| Data Engine | Nushell | Provides structured JSON data and executes terminal logic. |

3. Spatial System Design
A. Viewport & Transformation Logic
 * Coordinate System: The compositor does not use absolute pixels; it uses a Global Scene Graph where windows are textures placed at (x, y, z).
 * Zooming: Pinching gestures adjust the Viewport's z-axis scale.
 * Focusing: Selecting an app triggers a "Focus Zoom," where the compositor calculates a transition until the window bounds match the viewport.

B. The Layer Shell (HUD)
 * The LCARS frame (buttons and terminal) is rendered as a Wayland Layer Shell.
 * While the workspace zooms, the HUD stays at a 1:1 static scale for pixel-perfect legibility.

4. The UI & Theme Module (lcars-official)
The interface is a pluggable React/WASM module that communicates with the Core via JSON-RPC over a Unix Domain Socket.

A. Sonic Interface Cues
 * Location: Assets stored in `public/audio` within the theme module.
 * Function: Triggers high-fidelity "Star Trek" style chirps and pings for UI interactions.
 * Control: Includes an independent Enable/Disable toggle within the system settings area.

B. Settings Registry
 * Persistence: User preferences (Sonic Cues, animation speeds, opacity) are stored in a persistent JSON registry (`src/data/settings_registry.json`).
 * Management: A dedicated "Engineering" panel allows real-time adjustment of UI behaviors without restarting the Rust Core.

5. Build & Orchestration (Updated)
The project uses a centralized Makefile at the root directory to manage the multi-language lifecycle. The structure now adheres to standard Rust and React toolchain requirements.

Directory Context
/Makefile                        # Central
/src                            # code Root Orchestration
├── /core                       # The Micro-Shell Engine (Rust/Smithay)
│   ├── Cargo.toml              # Rust Dependency Manifest
│   └── /src
│       └── main.rs             # Compositor Entry Point
├── /plugins
│   └── /lcars-theme            # React/WPE UI Module
│       ├── package.json        # Node/React Manifest
│       ├── /public
│       │   └── /audio          # Sonic Cues (wav/mp3)
│       └── /src
│           └── App.tsx         # Main UI Component
├── /bridge                     # JSON-RPC & Socket Logic
│   └── /src
│       └── lib.rs              # Library Entry Point
└── /data                       # System Logic & Persistence
    ├── directory_listing.nu    # Nushell Script for Data
    └── settings_registry.json  # Persistence File

Makefile Specifications
 * make setup-deps: Installs system-level dependencies (smithay-devel, wpewebkit-devel, nushell) and toolchain deps (cargo, npm).
 * make ui: Installs npm packages in `plugins/lcars-theme` and builds the React shell (optimizing WASM/Audio assets).
 * make core: Runs `cargo build` inside `src/core`.
 * make run: Launches the environment in a nested Wayland window for testing.

6. Development Roadmap
 * Phase 1 (The Engine): Initialize the Smithay-based compositor and implement the 3D transformation matrix.
 * Phase 2 (WPE Integration): Embed WPE WebKit and route touch gestures from Rust to the UI surface.
 * Phase 3 (Command Link): Build the JSON-RPC bridge and connect Nushell data to the React UI.
 * Phase 4 (Legacy & Polish): Implement XWayland for legacy app support and integrate the modular Sonic Interface settings.
 