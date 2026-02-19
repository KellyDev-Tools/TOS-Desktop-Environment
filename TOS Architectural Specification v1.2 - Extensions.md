# TOS Architectural Specification v1.2 - Extensions

This addendum documents features, refinements, and structural changes introduced during development of the TOS application, relative to the original v1.0 document. It is organized to mirror the original section numbering, with new sections added where appropriate. All details from the later versions are preserved and consolidated here.

---

## 1. Core Philosophy (Additions)

The core philosophy is extended with explicit mention of **priority visual indicators**, **unified search**, **AI assistance**, **detailed logging**, and **multi‑sensory feedback** as integral parts of the tactical environment.

In v1.2, the architecture is made **modular and multi‑platform**, targeting:
- **Linux Wayland** – Full desktop experience.
- **Android XR (OpenXR)** – Immersive spatial computing.
- **Android Phone** – Traditional 2D touch interface.

All platform‑specific adaptations are achieved through a set of core traits, enabling a single Rust codebase to serve all targets.

---

## 2. The Three‑Level Hierarchy (Changes)

The hierarchy is extended with two deeper inspection levels:

| Level | Name                 | Description (Additions) |
|-------|----------------------|--------------------------|
| **4** | **Detail**           | Structured metadata view for any surface: CPU, memory, uptime, event history, and configuration. |
| **5** | **Buffer**           | Raw memory / hex dump of the application’s process space. **Privileged access** – requires explicit elevation (see §11.6). On Android, this level may be limited or omitted due to platform restrictions. |

Level 1 (Global Overview) is now reserved for sector setup, configuration, remote management, and global settings.

---

## 3. Command Hub: Four Modes

The Command Hub now provides **four modes**, with the addition of **Search Mode**. The prompt area is extended with a multi‑mode selector and a stop button.

### 3.4 Search Mode (New)

Search Mode transforms the Command Hub into a unified search interface across multiple domains:

| Domain | Examples |
|--------|----------|
| Surfaces | Window titles, application IDs, process names, surface UUIDs, content (if exposed via Application Model). |
| Files and Directories | File names, paths, metadata. |
| TOS Log Events | Commands executed, lifecycle events, inspections, notifications, alerts, collaboration events. |
| Commands and History | Shell command history, favourite commands, aliases. |
| Settings and Preferences | Setting names, direct links to config panels. |
| Sectors and Viewports | Sector names, sector types, viewport positions. |
| Help and Documentation | Built‑in help topics, man pages. |
| Contacts and Collaboration | User names, active collaborators, shared sectors. |
| Marketplace Packages | Installed and available modules (AI backends, sector types, etc.). |
| Notifications | Active and recent notifications. |

**Search Behaviour**
- Federated by default; filters can be used (e.g., `files:budget`).
- Ranked results with relevance scoring; priority indicators shown on result tiles.
- Results appear as a grid at Level 2, replacing the normal Command Hub layout temporarily.
- Clicking a result triggers an **Automated Vertical Transition** to the target’s exact location.

**External Search Providers**
- Tiles for external search engines (Google, Bing, Wikipedia, GitHub, etc.) appear alongside local matches.
- Clicking a tile sends the query to the provider, opening results in the default web browser.
- Providers are user‑configurable via URL templates.

### 3.5 Multi‑Mode Prompt with AI Assistant (New)

The Persistent Unified Prompt now features a three‑way mode selector (`CMD`, `SEARCH`, `AI`) and a stop button.

- **CMD Mode**: Standard shell and TOS command input.
- **SEARCH Mode**: Triggers unified search; typing updates live results; Enter selects top result.
- **AI Mode**: Accepts natural language queries for an AI assistant. Responses appear in a dedicated output area and can include suggested commands.
- **Stop Button (⏹️)**: Immediately interrupts the current operation (SIGINT in CMD, cancels search, stops AI generation). Keyboard shortcut: `Ctrl+Shift+C` or double Esc.

**AI Backend Framework**
- Pluggable architecture with a default **Ollama** integration (local, private).
- Additional backends (OpenAI, Anthropic, Google Gemini, etc.) available as Marketplace modules.
- Each backend declares capabilities (chat, function calling, vision) and required permissions.
- On Android, Gemini is the default; on Linux, Ollama is the default.
- Function calling for system control requires explicit user grant; dangerous commands still require tactile confirmation (see §11.4).

**Platform Notes**
- On Linux, commands execute in a native shell (Fish reference). On Android, commands run in a bundled lightweight shell or are forwarded to a remote host.
- Directory Mode on Android uses the Storage Access Framework.
- Activity Mode on Android lists TOS‑native apps and optionally system apps.

---

## 4. Tactical Bezel (Additions)

- When AI mode is active, the stop button may also appear in the collapsed bezel (if not already in the prompt area).
- Platform‑specific rendering: Wayland compositor overlay on Linux, OpenXR layer on Android XR, Compose overlay on Android phone.

---

## 5. Sectors and the Tree Model

### 5.1 Priority‑Weighted Layouts (Visual Indicators) – *New*

To convey relative importance without altering size or position, TOS uses non‑intrusive visual indicators on tiles at all levels.

**Indicator Types**
| Indicator | Description |
|-----------|-------------|
| Border Chips | Small, pill‑shaped coloured accents along the border. Number reflects priority score. |
| Chevrons | LCARS‑style arrow shapes; pulsing indicates a pending notification or critical status. |
| Glow / Luminance | Subtle inner/outer glow; intensity varies with priority. |
| Status Dots | Small circles in a corner; colour‑coded (blue=normal, yellow=caution, red=critical). Multiple dots can indicate multiple factors. |

**Priority Scoring**
Weighted factors (user‑configurable):
- Recency of focus (40%)
- Frequency of use (20%)
- Activity level (CPU, memory, I/O) (15%)
- Notification priority (10%)
- User pinning (override)
- Collaboration focus (10%)
- Sector‑specific rules (sector‑defined)
- AI suggestion (5%)

Scores map to indicator configurations (e.g., low = no chips, critical = four chips + pulsing chevron + red alert colour).

**Behaviour by Depth**
- Level 1: Sector tiles display overall sector activity.
- Level 2: Application tiles show individual surface priority.
- Level 3: In split viewports, indicators appear along shared borders.
- Level 4/5: Inspection panels can show priority indicators for the inspected surface and a mini‑map of siblings.

**Configuration**
- Master toggle, customisation of indicator types, colours per priority level/factor, sensitivity, hover tooltips.

---

## 6. Split Viewports (Additions)

Split logic is platform‑independent; the platform renderer arranges viewports (tiled quads in OpenXR, resizable panes in Wayland, split‑screen areas on phone).

---

## 7. Remote Sectors (Additions)

- The TOS Remote Server uses WebSocket/TLS for control, WebRTC for video/audio streaming, and WebDAV for file transfer.
- Live Feed Testing (7.4.1) details are unchanged from v1.0.

---

## 8. Collaboration (Additions)

### 8.5 TOS Log Integration
All guest actions within a shared sector are recorded in the **host's TOS Log** (see §14). Entries include guest identity, action type, timestamp, outcome. Guest actions are never written to the guest's local log. A privacy notice is shown upon joining.

### 8.6 AI Assistant for Collaboration
The AI can:
- Summarize recent activity.
- Translate commands/chat between languages.
- Suggest collaboration actions.
- Explain guest intent.
- Mediate role changes.
- Optionally, an AI‑driven guest can be invited.

AI processing of guest actions uses the host's configured AI backend; guests are notified if their actions may be processed by AI.

### 8.7 Collaboration Alerts
Key events trigger a **Yellow Alert**:
- User joins/leaves a sector.
- Guest role changes.
- Guest requests attention (“raise hand” button).
- Guest shares cursor or enters following mode.

Visual, auditory, and haptic feedback configurable; all events recorded in TOS Log.

---

## 9. Input Abstraction Layer (Additions)

New semantic events:
- **AI**: `ai_submit`, `ai_stop`, `ai_mode_toggle`
- **Collaboration**: `raise_hand`
- **Stop operation**: `stop_operation` (mapped from stop button)

Platform‑specific input sources:
- Linux: evdev/libinput
- Android XR: OpenXR actions (gaze, pinch, hand tracking)
- Android Phone: Android touch events, hardware keys, voice (Gemini API)

---

## 10. Platform Abstraction & Rendering (New in v1.2)

The core TOS logic is platform‑independent and interacts with the platform through three core traits.

### 10.1 Renderer Trait
```rust
pub trait Renderer {
    fn create_surface(&mut self, config: SurfaceConfig) -> SurfaceHandle;
    fn update_surface(&mut self, handle: SurfaceHandle, content: &dyn SurfaceContent);
    fn composite(&mut self);
    // ... bezel overlay, viewport management
}
```

### 10.2 InputSource Trait

```rust
pub trait InputSource {
    fn poll_events(&mut self) -> Vec<RawInputEvent>;
    fn map_to_semantic(&self, raw: RawInputEvent) -> Option<SemanticEvent>;
}
```

### 10.3 SystemServices Trait

```rust
pub trait SystemServices {
    fn spawn_process(&self, cmd: &str, args: &[&str]) -> Result<ProcessHandle>;
    fn read_dir(&self, path: &Path) -> Result<Vec<DirEntry>>;
    fn get_system_metrics(&self) -> SystemMetrics;
    fn open_url(&self, url: &str);
    // ... notifications, clipboard
}
```

### 10.4 Platform Implementations
- **Linux Wayland**: Custom Wayland compositor, libinput, POSIX.
- **Android XR**: OpenXR renderer, OpenXR action mapping, JNI for Android APIs.
- **Android Phone**: egui with Android backend or JNI to Jetpack Compose; touch events from Android view system.

---

## 11. Security Model (Additions)

### 11.6 Deep Inspection Privilege (New)
- Access to Level 5 (raw memory) is **disabled by default**.
- Activation requires explicit elevation (`sudo tos enable-deep-inspection` or Polkit dialog).
- When enabled, a 🔓 indicator appears in the Tactical Bezel; clicking it disables deep inspection immediately.
- All enable/disable events and Level 5 usage are audited.
- Applications may opt out via their Application Model manifest.
- On Android, Level 5 is generally unavailable due to platform restrictions.

**Platform Comparison Table**

| Aspect | Linux Wayland | Android XR / Phone |
|--------|---------------|---------------------|
| Authentication | PAM, SSH keys | Android Keystore, biometric prompt |
| Authorization | RBAC (local users) | Android permissions + TOS roles |
| Process Isolation | Flatpak/Firejail optional | Android sandbox (each app isolated) |
| Dangerous Commands | Tactile confirmation | Biometric prompt for sensitive actions |
| Deep Inspection | Level 5 via sudo | Not available |

---

## 12. Application Models and Sector Types (Additions)

Application Models can now provide:
- **Priority factor definitions** (custom weights for priority scoring)
- **Opt‑out from deep inspection**
- **Searchable content** (expose content for Unified Search)

Modules are platform‑specific plugins:
- Linux: shared objects (`.so`)
- Android: Android library plugins (`.apk` or dynamic feature modules)

---

## 13. Shell API (Additions)

- **Linux**: Full shell integration via OSC escapes; shell provider architecture (Fish reference, Bash/Zsh support).
- **Android**: Bundled lightweight shell (based on `mksh`) or optional integration with Termux. Remote shell execution via TOS Remote Server.

---

## 14. TOS Log (New Section)

Every surface maintains its own event history, collectively forming a system‑wide timeline.

### 14.1 Recorded Events
| Event Type | Examples |
|------------|----------|
| Lifecycle | Creation, focus, move, close |
| Commands | Executed commands with exit status and duration |
| Inspections | Level 4/5 views accessed |
| Telemetry | Periodic resource snapshots (if enabled) |
| Collaboration | User joins/leaves, role changes, guest actions |
| System Events | Notifications, alerts, security events |
| Priority Changes | Changes in priority score and indicators |
| AI Interactions | Queries and responses (if enabled) |

### 14.2 Access Methods
- **Per‑Surface (Level 4)**: Scrollable timeline for that surface.
- **Global TOS Log Sector**: Dedicated sector aggregating logs from all surfaces.
- **Prompt Queries**: Commands like `log --surface browser --since 10min`.

### 14.3 Privacy & User Control
- Master toggle in Settings to enable/disable logging.
- Granular controls: per‑surface opt‑out, retention policy, exclude sensitive patterns.
- Logs stored locally in `~/.local/share/tos/logs/` (Linux) or app‑private storage (Android) in a structured format (JSON Lines or SQLite).
- Critical security events may be recorded in a separate, non‑disableable audit log.

### 14.4 OpenSearch Compatibility
- **OpenSearch Protocol**: TOS provides an OpenSearch description document; browser address bar can query logs (e.g., `tos log failed command`).
- **OpenSearch (Elastic) Compatibility**: Logs can be forwarded to an OpenSearch cluster with explicit user consent.

---

## 15. Tactical Reset (No change from v1.0)

---

## 16. Sector Templates and Marketplace (Additions)

### 16.4 AI Backend Modules (New)
- Package type `.tos-ai` providing an AI backend.
- Manifest example (`module.toml`) with fields: name, version, type, description, icon, capabilities (chat, function_calling, vision, streaming), connection details (protocol, default_endpoint, auth_type), permissions, configuration options.

Example manifest:
```toml
name = "OpenAI GPT-4"
version = "1.0.0"
type = "ai-backend"
description = "Connect to OpenAI's GPT-4 model for AI assistance."
icon = "openai.svg"

[capabilities]
chat = true
function_calling = true
vision = false
streaming = true

[connection]
protocol = "https"
default_endpoint = "https://api.openai.com/v1/chat/completions"
auth_type = "api-key"  # or "oauth2", "none"

[permissions]
network = ["api.openai.com"]
filesystem = false

[configuration]
model = { type = "string", default = "gpt-4", options = ["gpt-4", "gpt-3.5-turbo"] }
temperature = { type = "float", default = 0.7, min = 0, max = 2 }
```

**Installation Flow**
- User browses Marketplace, finds an AI module, clicks Install.
- TOS displays the module's requested permissions (e.g., network access to specific domains).
- User confirms (or rejects) installation.
- Module is placed in `~/.local/share/tos/ai-backends/` (Linux) or equivalent on Android and appears in the AI Assistant settings panel.

**Module Isolation**
- Modules run in a sandbox with limited access (using the same module API as Application Models).
- Network permissions are enforced via the host's firewall or a dedicated proxy.
- If a module requires dangerous capabilities (e.g., local file access for context), it must declare them and obtain explicit user consent.

### 16.5 Marketplace Discovery Enhancements
- **Search Mode Integration**: Typing a query shows relevant modules as tiles.
- **AI‑Assisted Discovery**: AI can help find modules based on natural language queries.
- **Update Alerts**: Yellow Alert when an installed module has an update.

Packages are platform‑specific (`.so` for Linux, `.apk` for Android).

---

## 17. Accessibility (Additions)

- Priority indicators can be enlarged or replaced with high‑contrast variants.
- TOS leverages platform accessibility services:
  - Linux: AT‑SPI (Orca), high contrast themes.
  - Android: TalkBack, Switch Access, system‑wide font scaling, color inversion.

---

## 18. Tactical Mini‑Map (Additions)

### 18.5 Monitoring Layer (New)
- Optional layer showing live resource usage of processes relevant to the current depth.
- Toggled via an icon on the mini‑map.
- **Level 1**: Aggregated CPU/memory per sector.
- **Level 2**: All applications in current sector with CPU%, memory%, sparkline.
- **Level 3**: Detailed stats for focused app plus compact usage of other viewports.
- Data updates throttled (1–2 Hz).

---

## 19. Auditory and Haptic Interface (New Section)

### 19.1 Three‑Layer Audio Model
| Layer | Purpose | Characteristics |
|-------|---------|-----------------|
| Ambient | Atmosphere | Continuous, depth‑varying background. |
| Tactical | Action confirmation | Discrete earcons for zoom, commands, notifications, splits, alerts, collaboration. |
| Voice | Speech synthesis | TTS for announcements, screen reader, AI responses. |

Each layer has independent volume control and can be enabled/disabled.

### 19.2 Context Adaptation
- Audio changes with zoom level and alert state.
- **Green (Normal)**: All layers as configured.
- **Yellow Alert**: Ambient shifts, tactical adds periodic pulse, voice more verbose.
- **Red Alert**: Ambient replaced by repeating tone; tactical suppresses non‑critical earcons; voice prioritises critical messages.

### 19.3 Spatial Audio (VR/AR)
- Sounds positioned in 3D space (notifications from a sector to the left appear from that direction).

### 19.4 Theming and Extensibility
- Audio themes (`.tos-audio`) installable via Marketplace.
- Applications may contribute custom tactical sounds via Application Model.

### 19.5 Accessibility Integration
- Voice layer foundation for screen reader.
- Verbosity levels adjust spoken feedback.

### 19.6 Haptic Feedback
Haptics parallel the tactical audio layer.

**Device Support**
- Game controllers, VR controllers, haptic touchpads, mobile devices, accessibility switches.

**Haptic Event Taxonomy** (mapped from semantic events)
| Category | Examples | Pattern Suggestions |
|----------|----------|---------------------|
| Navigation | zoom_in, zoom_out | Ascending/descending pulses |
| Selection | select | Quick click |
| Mode Control | cycle_mode | Mode‑specific pulse sequences |
| Bezel Control | toggle_bezel_expanded | Light buzz / soft thud |
| System Commands | tactical_reset | Distinctive long vibration |
| Text Input | text_input | Subtle keystroke feedback |
| Voice | voice_command_start | Short “listening” pulse |
| Collaboration | user_joined | Gentle ping‑like vibration |
| Dangerous Actions | dangerous_command | Sharp, insistent buzz |
| Alerts | red_alert | Pulsing vibration escalating with alert level |

**Spatial Haptics (VR/AR)**
- Notifications from left trigger left controller vibration.
- Zooming feels like “pulling” hands.
- Moving a surface causes drag‑and‑release vibration.

**Configuration**
- Global toggle, master intensity, per‑category toggles, test patterns.
- Hearing‑impaired mode: route tactical audio to haptics.
- Motor‑impaired mode: haptics confirm switch input.

**Platform Implementation**
- Linux: ALSA/PulseAudio, evdev haptics.
- Android: AudioManager, Vibrator, OpenXR haptic extensions.

---

## 20. Implementation Roadmap (Extended)

The original roadmap is extended with additional phases (in order):

8. **Deep Inspection (L4/L5)** – Level 4 structured metadata, Level 5 raw memory with privilege elevation.
9. **TOS Log** – Per‑surface logging, global log sector, privacy controls, OpenSearch compatibility.
10. **Search Mode** – Unified search across domains with external providers, integration with prompt.
11. **AI Assistant** – Multi‑mode prompt with CMD/SEARCH/AI toggle, pluggable backends, stop button.
12. **Priority‑Weighted Layouts** – Visual indicators, configurable factors and appearance.
13. **Auditory and Haptic Feedback** – Three‑layer audio, context adaptation, haptic feedback for supported devices.
14. **Accessibility and Polish** – Screen reader integration, high contrast, mini‑map monitoring layer.

In v1.2, the roadmap is reframed modularly:
1. Core Library (platform‑agnostic)
2. Linux Wayland Backend
3. Android Phone Backend
4. Android XR Backend
5. Remote Sectors (unified protocol)
6. Marketplace & Modules
7. Polish & Testing

---

## 21. Platform Abstraction Summary

The core TOS logic is platform‑independent and interacts with the platform through three core traits: `Renderer`, `InputSource`, and `SystemServices`. This enables a single Rust codebase to target Linux Wayland, Android XR, and Android phones.

---

## 22. Conclusion

This addendum consolidates development changes and updates to date February 17 2026



