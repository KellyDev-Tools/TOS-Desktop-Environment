# TOS (Tactical Operating System) – Unified Multi‑Platform Architectural Specification

## Version 3.0 (Modular, Multi‑Platform)

---

## 1. Core Philosophy

TOS (Tactical Operating System) is a reimagining of the personal computer interface, inspired by the LCARS interface from Star Trek. It replaces traditional window management with a **recursive zoom hierarchy** centered on a **command‑first** philosophy. The environment is **input‑agnostic**, supporting touch, mouse, keyboard, voice, game controllers, VR/AR controllers, hand tracking, eye tracking, and accessibility switches.

This specification defines a **modular architecture** that allows TOS to be built for multiple platforms from a single core codebase. The three primary targets are:

- **Linux Wayland** – Full desktop experience with native Linux application support.
- **Android XR (OpenXR)** – Immersive spatial computing on VR/AR headsets.
- **Android Phone** – Traditional 2D touch interface on mobile devices.

All interactions are organized into a strictly vertical, multi‑level structural hierarchy, with a tree‑like organization of **sectors**, **command hubs**, and **applications**. Navigation is achieved by zooming through these layers, with a persistent focus on terminal‑driven intent and graphical augmentation.

---

## 2. The Three‑Level Hierarchy

The core hierarchy consists of three primary levels, extended with two deeper inspection levels.

| Level | Name                 | Description |
|-------|----------------------|-------------|
| **1** | **Global Overview**  | Bird’s‑eye view of all sectors (local and remote). Sectors appear as zoomable tiles with priority indicators. |
| **2** | **Command Hub**      | Central control point for a sector. Contains the **Persistent Unified Prompt** and four toggleable modes. |
| **3** | **Application Focus**| Full‑screen (or tiled) application surface, wrapped in the **Tactical Bezel**. |
| **4** | **Detail**           | Structured metadata view for any surface. |
| **5** | **Buffer**           | Raw memory / hex dump (privileged access). On Android, this level may be limited or omitted due to platform restrictions. |

Navigation is consistent across platforms: pinch/scroll/trigger to zoom, tap/click/select to choose.

---

## 3. Command Hub: Four Modes

The Command Hub (Level 2) is the exclusive home of the **Persistent Unified Prompt**. It provides four modes, switchable via a four‑way toggle. Each mode is designed to help the user interact with the system, and all can affect the entire TOS environment.

### 3.1 Command Mode (CLI‑Centric)
- **Prompt** at the bottom (always visible).
- **Suggestion area**: eval‑help chips, command history, favourites.
- **Auto‑complete dropdown**.
- On Linux, commands execute in a native shell. On Android, commands run in a bundled lightweight shell or are forwarded to a remote host.

### 3.2 Directory Mode (File Manager)
- **Path bar** (breadcrumb style).
- **Grid/list view** of files and folders (respects platform‑specific storage permissions).
- **Selection controls**.
- **Action toolbar** (New Folder, Copy, Paste, etc.).
- On Android, file access is mediated through the Storage Access Framework.

### 3.3 Activity Mode (Process/App Manager)
- **Tactical grid** of all running applications within the current sector.
- Each tile shows icon, title, status indicators (CPU/memory if available).
- On Linux, this lists native processes; on Android, it lists TOS‑native apps and optionally system apps.

### 3.4 Search Mode
- **Unified search** across surfaces, files, commands, logs, settings, contacts, and external providers (web search).
- Results grid with priority indicators.

### 3.5 Multi‑Mode Prompt with AI Assistant
- Mode selector: `CMD` / `SEARCH` / `AI`.
- Stop button to interrupt operations.
- AI backend pluggable (Gemini on Android, Ollama on Linux, etc.).

---

## 4. Tactical Bezel

The Tactical Bezel is an **immutable system overlay** rendered by the platform‑specific UI layer at Level 3. It provides guaranteed navigation escape and unified window decoration.

- **Collapsed state**: Thin strip with Zoom Out, app icon/title, expand handle.
- **Expanded state**: Reveals navigation controls, window controls, application‑specific actions, and collaboration indicators.

The bezel’s appearance and behavior are consistent across platforms, but the rendering technology varies (Wayland compositor overlay on Linux, OpenXR layer on Android XR, Compose overlay on Android phone).

---

## 5. Sectors and the Tree Model

A **sector** is a self‑contained workspace with its own identity, settings, and (if remote) connection. Internally, a sector follows a tree structure:


```

SECTOR
├── Command Hub A (Level 2)
│   ├── Application 1 (Level 3)
│   └── Application 2 (Level 3)
├── Command Hub B (Level 2)
│   └── Application 3 (Level 3)
└── Command Hub C (Level 2)
└── Application 4 (Level 3)

```


Priority‑weighted visual indicators (border chips, chevrons, glows) are rendered by the platform‑specific UI layer.

---

## 6. Split Viewports

Splitting allows a sector to display multiple viewports simultaneously, each with independent depth and content. The split logic is platform‑independent; the platform renderer is responsible for arranging viewports (e.g., as tiled quads in OpenXR, as resizable panes in Wayland, as split‑screen areas on phone).

---

## 7. Remote Sectors

Remote sectors are enabled by the **TOS Remote Server** running on a target machine (Linux, Windows, macOS). The client connects via WebSocket/TLS, streams video/audio via WebRTC, and transfers files via WebDAV. This functionality is identical across all platforms.

---

## 8. Collaboration

Collaboration is host‑owned and platform‑agnostic. Guests connect via the TOS Remote Server protocol. Features include independent viewports, following mode, roles (Viewer, Commenter, Operator, Co‑owner), avatars, and auditory/haptic cues.

---

## 9. Input Abstraction Layer

All physical input devices are normalized into **semantic events** (e.g., `zoom_in`, `select`, `cycle_mode`). The mapping from raw input to semantic events is platform‑specific and configurable by the user.

- **Linux Wayland**: evdev/libinput, keyboard, mouse, game controllers.
- **Android XR**: OpenXR actions (gaze, pinch, hand tracking), voice via Gemini API.
- **Android Phone**: Android touch events, hardware keys, voice.

---

## 10. Platform Abstraction & Rendering

The core TOS logic is platform‑independent and interacts with the platform through three core traits.

### 10.1 Renderer Trait
```rust
pub trait Renderer {
    fn create_surface(&mut self, config: SurfaceConfig) -> SurfaceHandle;
    fn update_surface(&mut self, handle: SurfaceHandle, content: &dyn SurfaceContent);
    fn composite(&mut self);
    // ... bezel overlay, viewport management, etc.
}
### 10.2 InputSource Trait
```rust
pub trait InputSource {
    fn poll_events(&mut self) -> Vec<RawInputEvent>;
    fn map_to_semantic(&self, raw: RawInputEvent) -> Option<SemanticEvent>;
}
### 10.3 SystemServices Trait
```rust
pub trait SystemServices {
    fn spawn_process(&self, cmd: &str, args: &[&str]) -> Result<ProcessHandle>;
    fn read_dir(&self, path: &Path) -> Result<Vec<DirEntry>>;
    fn get_system_metrics(&self) -> SystemMetrics;
    fn open_url(&self, url: &str);
    // ... notifications, clipboard, etc.
}
### 10.4 Platform Implementations
- **Linux Wayland**: Custom Wayland compositor implementing `Renderer`. `InputSource` uses `libinput`. `SystemServices` uses standard POSIX calls.
- **Android XR**: OpenXR‑based renderer using `openxr` crate. `InputSource` maps OpenXR actions to semantic events. `SystemServices` uses JNI to call Android APIs (Storage Access Framework, ActivityManager, etc.).
- **Android Phone**: Renderer using `egui` with Android backend, or JNI to Jetpack Compose. `InputSource` receives touch events from Android view system. `SystemServices` same JNI layer as XR.

---

## 11. Performance and Compositing

Performance strategies are platform‑specific but guided by common principles:
- Depth‑based rendering (only focused level receives full frame rate).
- Texture caching and GPU memory pruning.
- Hardware acceleration where available.
- On Android XR: foveated rendering, space warp.
- On Linux: direct scanout for full‑screen apps.

---

## 12. Security Model

The security model adapts to each platform while maintaining consistent principles.

| Aspect | Linux Wayland | Android XR / Phone |
|--------|---------------|---------------------|
| Authentication | PAM, SSH keys | Android Keystore, biometric prompt |
| Authorization | RBAC (local users) | Android permissions + TOS roles |
| Process Isolation | Flatpak/Firejail optional | Android sandbox (each app isolated) |
| Dangerous Commands | Tactile confirmation | Biometric prompt for sensitive actions |
| Deep Inspection | Level 5 via sudo | Not available (platform restriction) |

---

## 13. Application Models and Sector Types

Modules (Application Models, Sector Types, AI Backends) are distributed as platform‑specific plugins:

- **Linux**: Shared objects (`.so`) loaded dynamically.
- **Android**: Android library plugins (`.apk` or dynamic feature modules) installed via the Marketplace.

Each module declares capabilities and required permissions. The module system is platform‑agnostic in core, but loading and sandboxing differ.

---

## 14. Shell API

- **Linux**: Full shell integration via OSC escapes (Fish/Zsh/Bash). The shell provider architecture allows pluggable shells.
- **Android**: Bundled lightweight shell (based on `mksh`) or integration with Termux if installed. Remote shell execution via TOS Remote Server.

---

## 15. TOS Log

Per‑surface event history stored in a platform‑agnostic format (e.g., SQLite). The log viewer is a dedicated sector. Privacy controls allow users to disable logging or clear data. On Android, logs are stored in app‑private storage.

---

## 16. Tactical Reset

Two‑level emergency recovery:
- **Sector Reset**: Closes all apps in current sector, returns to fresh hub.
- **System Reset**: Closes all sectors, returns to Global Overview.

Confirmation via tactile hold (biometric optional on Android).

---

## 17. Sector Templates and Marketplace

- **Marketplace**: In‑app store for downloading modules. Packages are platform‑specific (`.so` for Linux, `.apk` for Android). Repository indices configurable.
- **AI Backend Modules**: Pluggable via the same module system (Gemini, Ollama, OpenAI, etc.).

---

## 18. Accessibility

TOS leverages platform accessibility services:
- **Linux**: AT‑SPI (Orca), high contrast themes.
- **Android**: TalkBack, Switch Access, system‑wide font scaling, color inversion.
- All platforms: Auditory and haptic feedback as per §20.

---

## 19. Tactical Mini‑Map

An ephemeral overlay providing spatial awareness. Rendered by the platform‑specific UI layer. On Android XR, it can be a 3D panel; on phone, a 2D overlay.

---

## 20. Auditory and Haptic Interface

Three‑layer audio (ambient, tactical, voice) and haptic feedback are implemented using platform APIs:
- **Linux**: ALSA/PulseAudio, evdev haptics (where available).
- **Android XR/Phone**: Android `AudioManager`, `Vibrator`, OpenXR haptic extensions for controllers.

---

## 21. Implementation Roadmap (Modular Approach)

1. **Core Library** – Platform‑agnostic hierarchy, command hub, collaboration, log, file sync.
2. **Linux Wayland Backend** – Compositor, input, system services.
3. **Android Phone Backend** – `egui`‑based UI, touch input, JNI system services.
4. **Android XR Backend** – OpenXR rendering, gaze/pinch input, Gemini integration.
5. **Remote Sectors** – Unified protocol implementation for all platforms.
6. **Marketplace & Modules** – Plugin system per platform.
7. **Polish & Testing** – Cross‑platform validation, performance tuning.

---

## 22. Conclusion

This modular specification enables TOS to become a truly multi‑platform spatial command environment. By separating core logic from platform‑specific rendering and system services, a single Rust codebase can target Linux Wayland, Android XR, and Android phones, delivering a consistent user experience while leveraging each platform’s unique capabilities.