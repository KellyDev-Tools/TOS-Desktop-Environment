# Development Discoveries

An evaluation of the **Traditional App** implementation against the **Origin Idea** documents, cataloguing what was built, what diverged, what concepts emerged during development, and what extensions should be formalized.

---

## 1. Alignment Matrix: Origin Idea → Implementation

| Origin Idea Concept | Status | Traditional App Location | Notes |
|---|---|---|---|
| Recursive Zoom Hierarchy (L1→L3) | ✅ Implemented + Extended | `navigation/zoom.rs` | Extended beyond origin to **L4 (Detail)** and **L5 (Buffer)** |
| Morphing SSD Frames | ✅ Structural | `ui/decorations.rs` | `MorphPhase` enum exists; visual CSS morph not yet live |
| Persistent Unified Prompt | ✅ Implemented | `system/status.rs` | Rendered as `<input>` in the status bar HTML |
| Window Picker (Level 3a) | ✅ Implemented | `navigation/zoom.rs` | Multi-window detection drives picker vs. direct zoom-out |
| Split View (Tiling) | ✅ Implemented | `navigation/zoom.rs`, `lib.rs` | `Level3Split` + `swap_split()` + secondary_app_id tracking |
| Shell Integration (OSC 1337) | ✅ Implemented | `system/shell.rs` | `CurrentDir`, `ZoomLevel`, `SetLayout` commands parsed |
| Spatial File Browser (Level N) | ⚠️ Partial | `system/files.rs` | VirtualFileSystem only; no real inotify, no MIME metadata injection |
| Dashboard & Widgets | ✅ Implemented | `ui/dashboard.rs` | Clock, SystemMonitor, ProcessManager, Settings widgets |
| Notifications (LCARS Alerts) | ✅ Implemented | `system/notifications.rs` | Priority enum (Low/Normal/Critical), FIFO queue |
| Audio Feedback (Chirps/Ambient) | ✅ Implemented | `system/audio.rs` | Ambient timer loop, effect filtering, queue-based dispatch |
| Compositor / Surface Management | ✅ Implemented | `compositor/mod.rs` | `SurfaceManager`, `SpatialMapper` with asymmetric grid layouts |
| Input-Agnostic Design | ⚠️ Partial | `system/input.rs`, `ui/window.rs` | Keyboard mapped; touch/voice/switch not yet wired |
| GPU Acceleration (wgpu) | ❌ Not Started | — | Origin describes Vulkan texture-pass pipeline; app uses WebView only |
| Wayland/XWayland Integration | ❌ Not Started | — | The compositor is fully simulated; no real Smithay backend |
| Multi-Monitor / Viewport Independence | ❌ Not Started | — | Origin describes per-viewport `path` stacks; app has single viewport |
| Accessibility (High Contrast, Scaling) | ⚠️ Stub | `lib.rs` (`AppSettings`) | `high_contrast` field exists but no CSS themes toggle |
| Context-Aware Prompt Suggestions | ❌ Not Started | — | Origin describes overlay suggestions based on `cwd` + history |
| Automated Vertical Transitions | ⚠️ Partial | `system/commands.rs` | `zoom` command chains zoom_in calls, but no shortest-path algorithm |
| Tactical Mini-Map | ❌ Not Started | — | Origin describes translucent wireframe navigator |
| Voice & Speech Synthesis | ❌ Not Started | — | Origin describes PipeWire-linked speech daemon |
| D-Bus / XDG Portals | ❌ Not Started | — | Origin describes file picker and notification portal integration |
| LCARS Standard CSS Library | ⚠️ External | `ui/assets/css/lcars.css` | Exists as external file but not evaluated here |

---

## 2. Concept Extensions: Ideas Born During Development

These are features or patterns that **do not appear** in any origin idea document but **emerged** during the implementation process. They should be evaluated for formal adoption into the project vision.

### 2.1 Deep Inspection Levels (Level 4 + Level 5)

**Origin says**: Levels 1–3 + "Level N" for sub-app deep context (vaguely defined).

**Implementation discovered**: Two concrete deep levels were built:
- **Level 4 (Detail / Node Inspector)**: A structured diagnostic view showing CPU load, memory usage, uptime, and event history for a specific surface. This is a *system-level* tool, not an app-level sub-view.
- **Level 5 (Buffer / Hex Dump)**: A raw memory buffer viewer rendered as hex lines with ASCII decode. This pushes the "Zoom Deeper" metaphor to its logical extreme — you're literally inspecting the *bytes* of a process.

**Concept Extension**: The origin's "Level N: Deep Context" was intentionally vague to allow apps to define their own depth. The implementation reveals that **system-level diagnostics themselves** form natural deep levels. This creates a pattern:

> **Any surface in the hierarchy can be "inspected" at Level 4 (structured metadata) and Level 5 (raw data), regardless of app type.**

This should be formalized as a **Universal Inspection Protocol** — every node in the hierarchy exposes at minimum:
1. A structured metadata view (telemetry, history, permissions)
2. A raw data view (memory buffer, log dump, network packets)

### 2.2 Task Orchestration (Cross-Sector Movement)

**Origin says**: "Drag window onto Sector B's panel" at Level 1 for reparenting.

**Implementation discovered**: The `move`/`orchestrate` command in `commands.rs` enables moving surfaces between sectors *from any zoom level*, not just Level 1. The command `move [id] [sector]` works regardless of current depth, which is actually **more powerful** than the origin's drag-based model.

**Concept Extension**: Formalize two orchestration modes:
1. **Visual Orchestration** (Origin): Drag-and-drop at Level 1 — intuitive, spatial.
2. **Command Orchestration** (Discovery): Prompt-based move from any depth — efficient, scriptable. This enables automation: a script or shell alias could reorganize windows across sectors without any UI interaction.

### 2.3 Global Search Across Hierarchy Boundaries

**Origin says**: No explicit cross-hierarchy search is described. The prompt is for commands; the file browser is for files.

**Implementation discovered**: The `find`/`search` command in `commands.rs` performs a **cross-domain search** that simultaneously queries:
1. Surface titles and app classes (running processes)
2. File names in the VirtualFileSystem

Results are rendered as a **flat search grid at Level 1**, temporarily overriding the sector layout. This is a significant UX pattern — the search acts as a **temporary dimension collapse**, flattening the entire hierarchy into a single ranked list.

**Concept Extension**: Formalize **Unified Search** as a first-class navigational mode:
- Entering search mode always returns to Level 1 visually
- Results span surfaces, files, notifications, and command history
- Selecting a result triggers an **Automated Vertical Transition** to the target's exact location in the hierarchy
- The search itself should be a **Temporary Overlay** (per origin Notifications.md) rather than replacing the Level 1 view

### 2.4 Surface Event History (Telemetry Log per Node)

**Origin says**: Terminal history is a scrollable list. No per-surface history is mentioned.

**Implementation discovered**: Every `TosSurface` maintains a `history: Vec<String>` that accumulates events: creation, telemetry ticks, split-view entries, orchestration moves, inspection events. This is exposed in the Level 4 Detail view as "NODE HISTORY."

**Concept Extension**: This is an emergent **Audit Trail** / **Flight Recorder** pattern. Each node in the hierarchy keeps its own event log. This should be formalized:
- Every surface records lifecycle events (spawn, focus, blur, move, terminate)
- History is viewable at Level 4 and queryable via the Persistent Unified Prompt
- Combined histories across all surfaces form the **Ship's Log** — a system-wide event timeline accessible as a dedicated Sector (aligns with origin's "Security/Logs" Sector concept)

### 2.5 Red Alert State (System-Wide Visual Override)

**Origin says**: Notifications use color coding (Blue=Info, Orange=Warning, Red=Critical). The LCARS Standard Library handles "Alert" state turning panels red.

**Implementation discovered**: The `is_red_alert` flag in `DesktopEnvironment` is auto-triggered when **any** notification has `Priority::Critical`. This flag propagates to the UI thread and sets `document.body.className` to include `red-alert`, enabling a **system-wide CSS visual override**.

**Concept Extension**: This is an emergent **System Alert Mode** that goes beyond per-notification color coding. The entire environment visually transforms. This should be formalized as a tiered alert system:
- **Green (Default)**: Standard LCARS palette
- **Yellow Alert**: Elevated — amber tint on panels, heightened audio cues (e.g., "system under load")
- **Red Alert**: Critical — full visual override, priority audio, possible notification lockout for non-critical items

Triggers should include: critical notifications, high CPU/memory sustained load, low battery, disk space warnings, security events.

### 2.6 Audio Sequencer Pattern (Ambient + Tactile Separation)

**Origin says**: "LCARS chirps and beeps provide tactile feedback" and "Voice Status" for navigation.

**Implementation discovered**: The `AudioFeedback` system cleanly separates three audio layers with independent enable flags:
1. **Master** (`enabled`): Global kill switch
2. **Effects** (`effects_enabled`): Tactical chirps/beeps for user actions
3. **Ambient** (`ambient_enabled`): Background bridge hum + console pulse loop

The ambient system uses a timer-based sequencer pattern with different frequencies (tick % 100 for hum, tick % 47 for pulses).

**Concept Extension**: This three-layer separation should be formalized and extended:
- **Layer 1 — Ambient**: Continuous background atmosphere. Should vary by zoom level (e.g., deeper hum at Level 1, quieter at Level 3 focus).
- **Layer 2 — Tactical**: Discrete event sounds (zoom transitions, command confirmations, errors).
- **Layer 3 — Voice**: Speech synthesis for navigation confirmations and system alerts.
- Each layer independently configurable via Settings panel.
- The ambient sequencer could be **context-adaptive**: bridge sounds at Level 1, working-station ambience at Level 3, alert klaxon during Red Alert.

### 2.7 Clone/Duplicate Surface Pattern

**Origin says**: No concept of duplicating/cloning an application instance is described.

**Implementation discovered**: The `clone`/`duplicate` command creates a copy of the currently focused surface in the same sector. This is a power-user feature that has no analog in traditional desktop environments.

**Concept Extension**: Formalize **Surface Cloning** as a sector management tool:
- Cloning creates a new surface with the same app class and title in the current sector
- In a real compositor, this would spawn a new window of the same application
- Could be extended to support cloning across sectors (clone a terminal into a different workspace)

### 2.8 Asymmetric Grid Layout (Spatial Mapper)

**Origin says**: The Level 2 launcher shows "a grid of LCARS buttons." No specific layout algorithm is described.

**Implementation discovered**: The `SpatialMapper::get_layout()` implements a context-sensitive asymmetric grid:
- **1 app**: Full 3×3 span (hero layout)
- **2 apps**: 2:1 horizontal split
- **3 apps**: L-shaped layout (2×2 + 1×2 + 3×1)
- **4+ apps**: Featured item (2×2) + side column + bottom row overflow

This creates a visually weighted layout where the first/most important item gets the most screen real estate.

**Concept Extension**: Formalize **Priority-Weighted Layouts** as a core UI principle:
- The most recently focused or most active surface gets the largest tile
- Layout weight could be influenced by: recency, CPU activity, user-pinned priority
- This aligns with the "Tactical" aesthetic — the most important information gets the most screen real estate, like a bridge officer's primary display

---

## 3. Emergent Architectural Patterns

### 3.1 The Brain/Face Split

The `main.rs` architecture emerged as a clean **two-thread model**:
- **Brain** (logic thread): Owns `DesktopEnvironment`, processes inputs, generates HTML
- **Face** (UI thread): Owns the WebView, injects HTML, captures IPC

This pattern wasn't explicitly described in the origin but is a natural consequence of the WebView architecture. It should be maintained and documented as the **canonical threading model** for TOS.

### 3.2 HTML-as-Viewport-Protocol

The Brain generates **full HTML strings** for each viewport update, and the Face injects them via `innerHTML`. This is a deliberately simple rendering protocol that trades efficiency for correctness — the entire view is always consistent.

**Discovery**: This works well for the prototype but will not scale. The origin's Performance.md describes texture-based rendering with GPU compositing. The transition path is:
1. **Current**: Full HTML regeneration per tick
2. **Next**: Diffed HTML updates (only changed elements)
3. **Target**: Live texture injection via wgpu, with HTML only for LCARS chrome

### 3.3 Command Parser as Universal Interface

The `CommandParser` in `commands.rs` has become the **central nervous system** of the app. Every action — zoom, spawn, kill, move, split, search, settings — flows through text command parsing. This validates the origin's "Persistent Unified Prompt" concept: **the command line is not just an input method, it is the primary API**.

**Discovery**: This means the prompt is not just "a terminal" — it's the **RPC interface**. Shell scripts, automation tools, and even the UI buttons themselves (`sendCommand('terminal:...')`) all speak the same command language. This is a powerful architectural discovery that should influence the real compositor design.

---

## 4. Gap Analysis: What the Origin Describes but Was Not Built

### 4.1 High Priority Gaps

| Gap | Origin Source | Status | Implementation |
|---|---|---|---|
| **Wayland Compositor (Smithay)** | Native Desktop Architecture | ✅ **Foundation Built** | `compositor/wayland.rs` — Full surface lifecycle, xdg_toplevel/XWayland roles, SSD/CSD decoration modes, seat with keyboard/pointer/touch, click-to-focus, sector assignment, event queue. Needs Smithay protocol wiring. |
| **GPU Pipeline (wgpu)** | Performance.md | ✅ **Foundation Built** | `compositor/gpu.rs` — Level-based render strategies, VRAM cache with depth-based eviction, zoom transition animations (LCARS-snap easing), direct scanout detection, DMA-BUF zero-copy support. Needs wgpu device/queue binding. |
| **Real Shell PTY** | Dream.md, File Management.md | ✅ **Foundation Built** | `system/pty.rs` — Real forkpty with exec, robust OSC 1337 parser (split-across-read), TOS Shell API hook injection for Fish/Zsh/Bash, per-surface session management via PtyManager. |
| **Multi-Viewport / Multi-Monitor** | Multi-Monitor Support.md | ✅ **Foundation Built** | `navigation/viewport.rs` — Independent ZoomPath stacks per viewport, output hotplug, horizontal/vertical split, Automated Vertical Transitions with common-ancestor pathfinding. |

### 4.2 Medium Priority Gaps  

| Gap | Origin Source | Impact |
|---|---|---|
| **Automated Vertical Transitions** | Spatial Model Reconciliation.md | ✅ **Implemented** in `viewport.rs` via `navigate_to()` — computes common ancestor depth and generates ZoomOut/ZoomIn step sequence |
| **MIME-Based File Rendering** | File Management.md | File browser tiles should show thumbnails and type-appropriate icons |
| **XDG Portal Integration** | System Integration & Packaging.md | Native file pickers, share dialogs, screenshot portals |
| **Gesture Prediction** | Performance & Hardware Optimization.md | Velocity-based zoom snapping |

### 4.3 Low Priority Gaps (Polish)

| Gap | Origin Source | Impact |
|---|---|---|
| **Tactical Mini-Map** | Dashboard and UI Components.md | Visual hierarchy position indicator |
| **Directory Peek Overlay** | Dashboard and UI Components.md | Preview without full zoom |
| **Transporter Buffer (Clipboard UI)** | Dashboard and UI Components.md | Touch-optimized clipboard panel |
| **Voice Synthesis** | Dashboard and UI Components.md | Audible navigation confirmations |
| **Dwell Click** | Accessibility.md | Hover-to-click for motor impairments |

---

## 5. C++ Concept vs. Rust Implementation

The `concepts/cpp_nav/` prototype was an earlier proof-of-concept. Comparing it reveals the evolution:

| Aspect | C++ Concept | Rust Implementation |
|---|---|---|
| Zoom Levels | 4 (L1, L2, L3, L3a) | 7 (L1, L2, L3, L3a, L3Split, L4, L5) |
| Multi-Window Detection | Hardcoded (`appIndex % 2 == 0`) | Runtime query of `SurfaceManager` |
| Split View | Logged but not stateful | Full `Level3Split` state + `secondary_app_id` + swap |
| Surface Management | None | Full `SurfaceManager` with create/remove/move/search |
| Shell Integration | None | OSC 1337 parsing with channel-based IPC |
| File System | None | `VirtualFileSystem` with CRUD and search |
| Audio | None | Three-layer audio system with queue dispatch |
| UI Rendering | Console `cout` only | HTML generation pipeline → WebView injection |

The Rust implementation is a **substantial evolution**, taking the C++ concept from a state-machine demo to a near-functional desktop environment prototype.

---

## 6. Recommendations

### 6.1 Formalize Concept Extensions

The following discoveries should be written back into the origin idea as new documents or amendments:
1. **Universal Inspection Protocol** (L4/L5) → New origin doc: `Deep Inspection.md`
2. **Unified Search Mode** → Amend `Dashboard and UI Components.md`
3. **System Alert Mode (Red/Yellow/Green)** → Amend `Notifications.md`
4. **Priority-Weighted Layouts** → Amend `Spatial Model Reconciliation.md`
5. **Command Orchestration** → Amend `Multi-Monitor Support.md`
6. **Audio Layer Architecture** → New origin doc: `Audio Design.md`
7. **Surface Event History / Ship's Log** → New origin doc: `Telemetry & Logging.md`

### 6.2 Next Implementation Priorities

Based on this evaluation, the highest-impact next steps are:

#### Completed (Foundation Phase)
- ~~**Real PTY Integration**~~ → `system/pty.rs` ✅
- ~~**Multi-Viewport State**~~ → `navigation/viewport.rs` ✅
- ~~**GPU Rendering Pipeline**~~ → `compositor/gpu.rs` ✅
- ~~**Wayland Backend**~~ → `compositor/wayland.rs` ✅

#### Next Phase: Wiring & Integration
1. **Wire ViewportManager into DesktopEnvironment** — Replace the single `SpatialNavigator` with multi-viewport support
2. **Wire WaylandBackend into the main loop** — Process compositor events in `tick()`, create surfaces from real Wayland clients
3. **Wire PtyManager into surface creation** — Auto-spawn PTY when a terminal surface is created
4. **Wire GpuPipeline into the render path** — Replace `generate_viewport_html` with GPU-composited frames when `--features gpu` is enabled
5. **Integrate Smithay protocol handlers** — Connect `WaylandBackend` to a real `calloop` event loop with Smithay's `CompositorHandler`, `XdgShellHandler`, etc.

#### Future Phase: Polish
6. **Real LCARS CSS System**: Get the WebView rendering a polished LCARS interface
7. **Diff-based HTML Updates**: Stop regenerating the full viewport every tick
8. **Gesture Prediction**: Velocity-based zoom level snapping

---

*Last evaluated: 2026-02-10*
*Scope: `/8TB/tos/traditional/src/` vs. `/8TB/tos/origin idea/`*
*Module test count: 109 total (53 from new foundation modules)*
