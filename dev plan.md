# TOS Development Plan & Environment Status

**Environment**: WSL2 (Ubuntu 24.04 via WSLg) on Windows
**Architecture**: Rust (Tao/Wry) with Web-based UI (LCARS)
**Current Version**: TOS Dream Complete (v1.2 Extension)

## 🛠️ Phase 1: Environment & Core (COMPLETED)
*   [x] **WSL2 Configuration**: Environment active.
*   [x] **Rust Toolchain**: `tos-dream` crate implementation.
*   [x] **Platform Abstraction**: `tao` windowing and `wry` webview integration.
*   [x] **Input Handling**: Keyboard event mapping and Semantic Event system.

## 🏗️ Phase 2: Architecture & Systems (COMPLETED)
*   [x] **Hierarchy**: Global Overview, Command Hub, Application Focus, Inspector Views.
*   [x] **Command Hub**: PTY integration (Fish shell), Directory Mode, Activity Mode.
*   [x] **Tactical Bezel**: Responsive overlay with priority/gain controls.
*   [x] **Security Module**:
    *   [x] Dangerous command detection (Regex patterns).
    *   [x] Tactile Confirmation (Slider, Hold, Pattern, Voice).
    *   [x] Audit Logging.
    *   [x] **Deep Inspection (Level 5)**: Privilege separation and visual indicators.

## 🧠 Phase 3: Intelligence & Collaboration (ACTIVE)
*   [x] **AI Integration**: `OllamaBackend` implementation.
    *   [ ] **Streaming**: Implement async streaming for AI responses (Currently blocking/buffered).
*   [x] **Unified Search**: File system, Log, and Web search integration.
*   [ ] **Collaboration**: Real-time sector syncing (Refine `CollaborationManager`).

## 🐛 Debugging & Refinement (CURRENT FOCUS)
*   [ ] **Input Responsiveness**: Investigate IPC/Input latency in user's WSLg environment.
    *   *Issue*: "Command input not being responded to".
    *   *Investigation*: Verifying `wry` IPC handler and JS event loop interaction.
*   [ ] **Performance**: Monitor `Dropped Escape call` warnings from WSLg.

## 🔱 Phase 4: Native OS Backends (ACTIVE)
*   [x] **Feature Flags**: Added `wayland` and `openxr` features to `Cargo.toml`.
*   [x] **Wayland Compositor**: Implemented native compositor skeleton using `smithay` v0.7.0. Handles socket creation, client acceptance, and event dispatching (§10.1).
*   [x] **OpenXR Immersive Mode**: Implemented `android_xr` backend stub with spatial session initialization (§21).
*   [x] **CLI Flag Selection**: Refactored `main.rs` to support `--wayland` and `--xr` for native mode selection.

## 📦 Phase 5: Distribution & Polish (PLANNED)
*   [ ] **Packaging**: Debian package creation.
*   [ ] **Themes**: Additional LCARS color schemes.
*   [ ] **Documentation**: Finalize operator manual.

---
**Note**: This plan is dynamically updated based on the codebase functionality.
