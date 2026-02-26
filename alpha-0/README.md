# TOS Traditional (Alpha-0) - Heritage Implementation

This directory contains the **Alpha-0 (Heritage)** Rust implementation of the TOS "Origin Idea" concepts. It represents the early foundational work on spatial navigation and native desktop integration before the pivot to the `alpha-1` production architecture.

## 🛠 Legacy Restoration (Feb 2026)
This project has been restored to a functional state. It was previously broken due to breaking changes in `wry`, `tao`, and `smithay`. It now uses a hybrid configuration that allows it to compile alongside modern dependencies while preserving its original logic.

**Key Fixes:**
- Upgraded `wry` to `0.40` and `tao` to `0.27` to resolve `raw-window-handle` conflicts.
- Refactored `src/ui/window.rs` to support the modern `wry` IPC handler and `tao` event loop.
- Updated `src/main.rs` channel management for compiler compatibility.

## 🏗 Architecture Overview

The system runs two concurrent threads:

1.  **The Brain (Logic Thread)**:
    -   Manages **Spatial Navigation** state (Recursive Zoom Levels).
    -   Handles **Shell Integration** (parsing legacy OSC 1337 sequences).
    -   Simulates user input and system events for the UI.

2.  **The Face (UI Thread)**:
    -   Runs the `wry` (WebView) Event Loop.
    -   Renders the **Legacy LCARS CSS Design System**.
    -   Injects state updates into the DOM via JavaScript.

## 🚀 Running the Legacy System

### Build Commands

```bash
# Build the core library
cargo build

# Run the standard desktop demo (requires GUI)
cargo run --features gui

# Run the full infrastructure demo (with Dev Monitor)
cargo run --features dev-monitor --bin demo-backend
```

### Dev Monitor
The `alpha-0` version includes a specialized **Development Monitor** that provides a browser-based view into the system's internals (GPU stats, Wayland surfaces, and PTY sessions).

To use the monitor:
1. Start the server: `cargo run --features dev-monitor --bin dev-server`
2. Open `http://127.0.0.1:3000` in your browser.
3. Run the demo: `cargo run --features dev-monitor --bin demo-backend`

## 📂 Project Structure

- `src/compositor/`: Early Wayland compositor and GPU rendering tests.
- `src/navigation/`: Original recursive zoom implementation.
- `src/ui/`: `wry` window management and dashboard widgets.
- `src/system/`: Legacy PTY handles, notification manager, and audio feedback.

## ⚠️ Disclaimer
`alpha-0` is preserved for **historical and conceptual reference**. Active development has shifted to the `alpha-1` directory, which features a more robust module system and multi-platform support.
