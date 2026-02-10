# TOS Traditional - Rust Implementation

This repository contains the **Rust implementation** of the TOS "Origin Idea" concepts.
Designed for the **Native Desktop Architecture**, it uses a local WebView (`wry`) for the UI and Rust for the core logic/compositor.

## Architecture Overview

The system is split into two concurrent threads communicating via channels (IPC):

1.  **The Brain (Logic Thread)**:
    -   Runs the `DesktopEnvironment` loop.
    -   Manages **Spatial Navigation** state (Zoom Levels).
    -   Handles **Shell Integration** (parsing OSC 1337 sequences).
    -   Simulates User Input and System Events.

2.  **The Face (UI Thread)**:
    -   Runs the `wry` (WebView) Event Loop on the main thread.
    -   Renders the **LCARS CSS Design System** (`ui/assets/css/lcars.css`).
    -   Receives `UiEvent` messages from the Brain to update the DOM.

## Module Structure

-   `src/main.rs`: Entry point. Spawns the Brain thread and launches the UI.
-   `src/lib.rs`: The core library defining the `DesktopEnvironment` struct.
-   `src/navigation/`:
    -   `zoom.rs`: Implements the recursive zoom hierarchy logic.
-   `src/ui/`:
    -   `window.rs`: The `wry` integration (WebView setup and IPC loop).
    -   `dashboard.rs`: The Widget system (Clock, System Monitor) returning HTML fragments.
-   `src/system/`:
    -   `shell.rs`: The **OSC Parser** that translates shell output into UI commands.
    -   `files.rs`: A mock Spatial File Browser.
    -   `notifications.rs`: The priority-based notification manager.
    -   `audio.rs`: Auditory feedback loop.

## Building and Running

### Prerequisites
-   Rust Toolchain (Latest Stable recommended, > 1.75).
-   System dependencies for Wry/Winit:
    -   `libwebkit2gtk-4.0-dev` (or 4.1)
    -   `libgtk-3-dev`
    -   `libayatana-appindicator3-dev`

### Known Issues
-   **Dependency Conflict**: The current environment has an issue with `getrandom v0.4.1` requiring Rust 2024 edition support.
-   **Workaround**: Update your Rust toolchain to the latest stable release:
    ```bash
    rustup update stable
    ```

### Running the Demo
Once dependencies are satisfied:

```bash
cargo run
```

This will launch a window displaying the LCARS interface. In the terminal, you will see logs from the "Brain" thread simulating user actions (Zooming, Shell Commands), which will reflect in the UI window via JavaScript injection.

## Project Files
-   **UI**: `ui/index.html`, `ui/assets/css/lcars.css`
-   **Rust Source**: `src/` directory.
-   **Config**: `Cargo.toml`
