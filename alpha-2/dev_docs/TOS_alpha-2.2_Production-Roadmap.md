# TOS Alpha-2.2 Production-Hardening Roadmap

This roadmap defines the transition from Alpha-2.1 (Experimental/Mocked) to Alpha-2.2 (Production/Stable). Tasks are ordered by dependency layer — each phase must be substantially complete before the next is unblocked. Items marked ~~strikethrough~~ have been superseded by Alpha-2.2 specifications.

---

## Phase 1 — Core Infrastructure
*No external dependencies. These tasks unblock everything else and should begin in parallel.*

- [x] **`tos-protocol` Extraction:** Move all `common/` state and IPC structs into a shared library to stabilize the Face-Brain contract.
    - All subsequent IPC work in this roadmap depends on this contract being stable.
- [x] **Unified Visual Token System:** Create `assets/design_tokens.json` and update both `svelte_ui/src/app.css` and `LinuxRenderer` to consume these values.
    - CSS tokens are partially in place via `svelte_ui/src/app.css` design system; need to extract into standalone JSON consumed by both Face and Wayland renderer.
    - Required before any new Face components are built (Onboarding, Expanded Bezel, Marketplace).
- [x] **Headless Brain Testing:** Build a `test-protocol` suite that validates Brain state deltas without requiring a renderer.
    - Required before adding new Brain services (TrustService, AIService, SessionService) to ensure regressions are caught.
- [x] **Settings Daemon Schema Extensions:** Add new namespaces to the Settings Daemon for Alpha-2.2 features.
    - `tos.onboarding` — state fields: `first_run_complete`, `wizard_complete`, `hints_dismissed`, `hint_suppressed`, `sessions_count`, `commands_run`, `levels_visited`.
    - `tos.trust` — trust level per command class, per-sector overrides.
    - `tos.ai.behaviors` — per-behavior configs and backend overrides.
    - `tos.interface.bezel` — expanded bezel dismiss behaviour preference.
    - `tos.interface.splits` — divider snap preference.
    - `tos.network` — `anchor_port` (default `7000`), `mdns_enabled` (default `true`).
- [x] **OSC-Exclusive Mode Switching:** Deprecate "String Sniffing" in `ipc_handler.rs`.
    - Force all mode transitions (`CMD → DIR`, etc.) to be driven by `OSC 7` and `OSC 9004` sequences emitted by the shell.
    - *Required before onboarding guided demo step detection can rely on shell events.*
- [x] **Service Registry & Port Infrastructure** *(Ecosystem-Orchestration):*
    - [x] Implement Brain Unix domain socket at `$XDG_RUNTIME_DIR/tos/brain.sock` for local daemon registration and client discovery.
    - [x] Implement Brain in-memory service registry: register, deregister, health probe (TCP connect, 30s interval, 3-strike offline marking).
    - [x] Implement daemon registration protocol: daemon binds Port 0, connects to `brain.sock`, sends `{ "type": "register", "name": "<name>", "port": <port> }`, receives ACK. Retry with exponential backoff if socket not yet available.
    - [x] Implement **always-on anchor port**: Brain binds a stable TCP port on every startup (resolved from: `TOS_ANCHOR_PORT` env var → `tos.network.anchor_port` setting → default `7000`). If occupied, scan upward +1 to +10; if all taken, fall back to Port 0 with warning. Write active value back to settings.
    - [x] Implement `get_port_map` IPC message on both Unix socket and Brain TCP socket: returns JSON service map including `anchor_port` field.
    - [x] Implement `tos ports` CLI: queries Brain registry, displays formatted table with reachability probes. Flags: `--json`, `--wait`, `--remote <host>[:<port>]`.
    - [x] Implement mDNS/DNS-SD advertisement via Avahi (`_tos-brain._tcp` service type with `brain_tcp` and `brain_ws` TXT records). Controlled by `tos.network.mdns_enabled` setting.
    - [x] Implement `tos discover` CLI: scans LAN for `_tos-brain._tcp` services.
    - [x] Implement remote Face connection dialog: mDNS scan + saved hosts + manual host:port entry (port defaults to anchor port). Save connections to `~/.config/tos/remote-hosts.toml`.
    - [x] Implement daemon deregistration on graceful shutdown; socket cleanup on Brain exit.
    - [x] Flip `Makefile` startup order: Brain first (`make run-brain`), then services (`make run-services`), then Face (`make run-web` runs all three in sequence).
    - [x] Add **Settings → Network** panel: Remote Access Port field (writes `tos.network.anchor_port`, takes effect on restart), WebSocket Port (read-only, auto), mDNS toggle, service registry status count, [View Port Map] inline table.
    - *Required before any remote Face work (Phase 5) and before Session Service, as all daemons depend on this infrastructure.*

---

## Phase 2 — Brain Services
*Depends on Phase 1. These are new or heavily refactored Brain-side subsystems that Face features are built on top of.*

- [x] **Session Service (`tos-sessiond`)** *(Session-Persistence-Specification):*
    - *Depends on: Service Registry & Port Infrastructure (Phase 1).*
    - [x] Implement `tos-sessiond` daemon (ephemeral port, registers with Brain); add to startup orchestration (`Makefile`, `run-services`).
    - [x] Implement atomic write via temp-file rename (`_live.tos-session.tmp` → `_live.tos-session`) for crash safety.
    - [x] Implement 2-second debounced auto-save triggered by: PTY command submit, `cd`/cwd change, sector create/close/rename, etc.
    - [x] Implement named session CRUD: save, load, delete, list per sector.
    - [x] Implement session file import/export with schema validation and version checking.
    - [x] Implement startup restore sequence: load `_live.tos-session` → reconstruct sectors and hub layouts → spawn shells in restored `cwd` → load terminal histories into output buffers before first prompt.
    - [x] Add IPC messages: `session_save`, `session_load`, `session_delete`, `session_list`, `session_export`, `session_import`, `session_live_write`.
    - *Blocks: Split Viewport pane detach, Expanded Bezel pane promotion, all session restore UI.*

- [x] **Command Trust Service (`TrustService`)** *(Trust-Confirmation-Specification — replaces Architecture Spec §17.2):*
    - [x] Remove confirmation slider UI entirely. Deprecated and removed `update_confirmation_progress` IPC from `ipc_handler.rs`.
    - [x] Implement Brain-side command classifier:
        - Stage 1 — explicit class matching: `privilege_escalation` (`sudo`, `su`, `doas`, `pkexec`) and `recursive_bulk` (`-r`/`-R`/`--recursive` with destructive verbs).
        - Stage 2 — implicit bulk detection: glob expansion estimate, configurable threshold (default: 10 files).
    - [x] Implement `TrustService`: trust config registry, per-sector override resolution, cascade (Sector → Global).
    - [x] Implement non-blocking warning chip IPC emission — fires on prompt stage, does not delay PTY submission.
    - [x] Add IPC messages: `trust_promote`, `trust_demote`, `trust_promote_sector`, `trust_demote_sector`, `trust_clear_sector`, `trust_get_config`.
    - *Blocks: Onboarding Step 0 trust configuration screen.*

- [x] **Production LLM Bridge** *(AI-Copilot-Specification — extends §18.3):*
    - [x] Transition `src/services/ai/mod.rs` from hardcoded string fallbacks to a module-driven system.
    - [x] Support configurable OpenAI / Anthropic / Ollama endpoints via `provider` and `endpoint` manifest fields.
    - [x] Added `capabilities` and `latency_profile` fields to `ModuleManifest` and `.tos-ai` manifest parsing.
    - [x] Implement provider-normalized response via `llm_http_call()` shared in `module_manager.rs`.
    - *Blocks: AIService refactor, all AI behavior modules.*

- [x] **AIService Refactor** *(AI-Copilot-Specification):*
    - *Depends on: Production LLM Bridge.*
    - [x] Implement behavior module registry: register, enable, disable, configure per module.
    - [x] Implement rolling context aggregator — assemble context object (cwd, sector name, shell, terminal buffer tail, last command, exit code, active mode, session stats, env hints); send only fields declared in module manifest.
    - [x] Implement per-behavior backend resolution cascade: behavior-level override → system default.
    - [x] Support multiple backends running simultaneously (one per behavior with an override).
    - [x] Preserve existing `ai_query` and `ai_tool_call` internal messages as the backend protocol.
    - [x] Add IPC messages: `ai_behavior_enable`, `ai_behavior_disable`, `ai_behavior_configure`, `ai_chip_stage`, `ai_chip_dismiss`, `ai_thought_expand`, `ai_thought_dismiss`, `ai_thought_dismiss_permanent`, `ai_context_request`, `ai_backend_set_default`, `ai_backend_set_behavior`, `ai_backend_clear_behavior`.
    - *Blocks: All AI behavior module implementations.*

- [x] **Split Pane Tree (Brain-side)** *(Split-Viewport-Specification — replaces Architecture Spec §11):*
    - [x] Implement pane tree data model in Brain sector state: `SplitPane`, `SplitNode` (recursive tree), `SplitOrientation`, `PaneContent`.
    - [x] Implement aspect-ratio-driven orientation selection: evaluate display aspect ratio on each split; `Vertical` if wider than tall, `Horizontal` if taller than wide.
    - [x] Implement minimum pane size enforcement: larger of (1/6 ratio minimum) or (content-aware minimum: 400×200px terminal); block split with amber chip + earcon when limit reached.
    - [x] Implement pane lifecycle: create, close, focus, focus_direction, resize (weight), equalize, swap.
    - [x] Implement pane detach to new sector: **Fresh Start** (clean shell in same cwd). **Bring Context** stub (requires PTY re-parenting, deferred).
    - [x] Implement split layout serialization into `hub.split_layout` session field via `serde_json`.
    - [x] Add IPC messages: `split_create`, `split_close`, `split_focus`, `split_focus_direction`, `split_resize`, `split_equalize`, `split_fullscreen`, `split_fullscreen_exit`, `split_swap`, `split_detach:context`, `split_detach:fresh`, `split_save_template`.
    - *Blocks: Split Viewport Face rendering, Expanded Bezel split actions, session restore of pane layouts.*

- [x] **Standalone Heuristic Service:** Extract predictive logic from the Brain into a separate `tos-heuristic` service.
    - [x] Implement predictive suggestions and typo correction in `tos-heuristicd`.

---

## Phase 3 — Face Features & Intelligence
*Depends on Phase 2. These are the primary user-facing features of Alpha-2.2.*

- [x] **Onboarding & First-Run Experience** *(Onboarding-Specification):*
    - *Depends on: Settings Daemon schema extensions (Phase 1), TrustService (Phase 2), Brain init log stream (existing).*
    - [x] Implement Face-side cinematic intro renderer:
        - LCARS grid sweep (frames 0–5s); live Brain init log stream (frames 5–9s); kinetic zoom into Level 2 (frames 9–12s).
        - Any keypress/tap skips immediately; Brain init runs in parallel.
        - Register two new earcons: `onboarding_start`, `onboarding_complete`.
    - [x] Implement `<OnboardingOverlay>` component: glassmorphism card, 8-step guided demo, `[NEXT →]` / `[← BACK]` / `[SKIP TOUR]` controls, "Show me" auto-execute per step, event-driven step completion (never enforces sequence, never blocks input).
    - [x] Insert **Step 0 — Trust Configuration** before guided demo: present `privilege_escalation` and `recursive_bulk` toggles with no pre-selection; Skip defers all to WARN.
    - [x] Implement ambient hints system: per-hint `[x]` dismissal, auto-dismissal on independent action, opacity decay tiers (100% → 70% → 40% → auto-suppress), master suppress toggle in Settings → Interface → Onboarding.
    - [x] Add permanent `[?]` Help Badge to Top Bezel Right: Replay Tour / Open Manual / Reset Hints.
    - [x] Implement **Settings → Security → Trust** panel with global policy selectors (Block/Warn/Allow).
    - [x] Implement Sector Overrides list in Trust panel for per-workspace sandboxing.
    - [x] Add **Settings → Sector** panel for renaming and freezing sectors.
    - [x] Add onboarding IPC prefix handler: `onboarding_skip_cinematic`, `onboarding_skip_tour`, `onboarding_advance_step`, `onboarding_hint_dismiss:<id>`, `onboarding_hints_suppress`, `onboarding_replay_tour`, `onboarding_reset_hints`.

- [x] **Session Restore UI** *(Session-Persistence-Specification):*
    - *Depends on: `tos-sessiond` (Phase 2), Split Pane Tree (Phase 2).*
    - [x] Implement silent startup restore — Face receives reconstructed state via WebSocket sync, renders without any notification or animation.
    - [x] Implement sector name chip popover (Top Bezel Left when in Command Hub): LIVE badge, named session list with timestamps, `[Save Current]`, `[Export]` actions.
    - [x] Implement secondary-select sector tile context menu entries: **Save Session As...**, **Load Session**.
    - [ ] Implement session import via drag-and-drop onto sector tile at Level 1.
    - [ ] Implement relaunch chip for Level 3 app panes that cannot be auto-restored.
    - [ ] Add **Settings → Sessions → Import** entry point.

- [x] **Split Viewport Face Rendering** *(Split-Viewport-Specification):*
    - *Depends on: Split Pane Tree (Phase 2).*
    - [x] Implement pane tree renderer: tiled layout, freely draggable dividers with optional 50% snap-assist.
    - [x] Implement focused pane amber border; unfocused panes at full opacity (peers, not backgrounds).
    - [x] Implement `Ctrl+\` / `Ctrl+-` / `Shift+Ctrl+\` split shortcuts; `Ctrl+Arrow` focus navigation; `Ctrl+W` pane close; double-click divider equalize.
    - [x] Implement blocked-split feedback: amber pane border flash + earcon.
    - [x] Implement fullscreen pane promotion: expand focused pane, preserve layout in memory, show `[⊞ Return to Split]` chip in Top Bezel.
    - [x] Wire pane management chips into Expanded Bezel surface (see below): `[⛶ Fullscreen]`, `[⇄ Swap]`, `[⊞ Detach →Sector]`, `[💾 Save Layout]`.
    - [x] Implement detach context chips: `[📦 Bring Context]` / `[✦ Fresh Start]`.
    - [x] Implement layout save as `.tos-template` — prompt for name, write via `tos-marketplaced`.

- [x] **Expanded Bezel Command Surface** *(Expanded-Bezel-Specification — supplements Architecture Spec §7.1 & §8.1):*
    - *Depends on: Split Pane Tree (Phase 2), `tos-sessiond` (Phase 2 — for ephemeral pane promotion).*
    - [x] Implement three triggers: tap bottom bezel anywhere, swipe up from bottom edge, Top Bezel Center split/expand button.
    - [x] Implement spatial zoom-out animation on expansion: current view scales back on z-axis, dimmed but visible; bottom bezel animates upward revealing full prompt + chip columns.
    - [x] Implement Level 3 lateral swipe navigation: while bezel is expanded, `←`/`→` arrows and swipe gestures cycle through open Level 3 apps in the zoomed-out layer.
    - [x] Implement shell context resolution: route to active PTY if idle; if PTY busy, overlay prompt with `[⏹ Stop (Ctrl+C)]` / `[⧉ New Terminal]` / `[⏳ Wait...]` chips — Stop always visible.
    - [x] Implement ephemeral pane: spawns for "New Terminal" path, closes on bezel dismiss, `[⊞ Promote to Split]` chip appears after first command.
    - [x] Implement output panel: renders via active Terminal Output Module, max 40% viewport height, scrollable; amber border on non-zero exit.
    - [x] Implement post-output action chips: `[→ Command Hub]`, `[⊞ Split View]`, `[✕ Dismiss]`, `[⧉ Keep Open]`.
    - [x] Implement configurable dismiss behaviour in **Settings → Interface → Expanded Bezel**: Stay Open / Auto-collapse on complete / Auto-collapse after timeout (default 5s).
    - [x] Disable expansion trigger during Tactical Reset (Level 4 God Mode).
    - [x] Add Brain `bezel_expanded` boolean flag (not persisted to session).
    - [x] Add IPC messages: `bezel_expand`, `bezel_collapse`, `bezel_output_action`, `bezel_pane_promote`, `bezel_swipe`.
    - [x] **Standalone Search Service:** Refactored global search into `tos-searchd` daemon with simulated semantic scoring.

- [x] **AI Behavior Modules — Default Set** *(AI-Copilot-Specification):*
    - *Depends on: AIService refactor (Phase 2).*
    - [x] Implement `.tos-aibehavior` module type, manifest schema, sandbox permission enforcement.
    - [x] Ship **Passive Observer (`tos-observer`)** as built-in removable module:
        - [x] Trigger conditions: exit 127, non-zero exit with stderr, 1.5s idle partial input, long-running command >30s, first `cd` into unseen directory.
        - [x] Secondary-color chip rendering (`✦` prefix glyph, teal/cyan by default).
        - [x] Right chip column placement, below system chips.
        - [x] Settings: trigger sensitivity (Low/Medium/High), chip column preference.
    - [x] Ship **Chat Companion (`tos-chat`)** as built-in removable module:
        - [x] Owns `[AI]` mode panel surface — renders full streaming chat interface.
        - [x] Context: current cwd, last 20 terminal lines, sector name, shell.
        - [x] Code blocks in responses get `[Stage →]` button.
        - [ ] Conversation history persisted to session file; restored via `on_session_restore` callback; capped at 200 messages.
        - [x] `[Clear]` resets history for current sector.
        - [x] Fallback message if no Chat Companion installed: "Install a Chat Companion from the Marketplace."
    - [ ] Add **Settings → AI** panel: Backend section (system default + installed list), Behaviors section (per-behavior toggle + backend override dropdown + config), Global section (chip color, ghost text opacity, master off, context level).

- [x] **Marketplace Discovery Face** *(Marketplace-Discovery-Specification — supplements Ecosystem Spec §2):*
    - *Depends on: existing `tos-marketplaced` (ephemeral port, discovered via Brain service registry), Secondary Select Infrastructure (Phase 3 — Visual).*
    - [x] Implement marketplace as Level 3 Application Focus: registers as standard app, bezel remains visible, `Esc` returns without disruption.
    - [x] Add long-press on Web Portal button in Top Bezel Right as marketplace entry point.
    - [x] Implement home view: horizontally scrollable Featured strip (signed `featured.json` manifest from `tos-marketplaced`); category grid with module type badges and counts.
    - [x] Implement category browse view: scrollable module grid, sort (Most Downloaded / Highest Rated / Newest), filter (All / Free / TOS Team / Compatible), real-time search within category.
    - [x] Implement module detail page: screenshot gallery (min 1 screenshot required), metadata block, human-readable permissions section, ratings and reviews, `[Review Permissions & Install]` CTA.
    - [x] Implement install flow: permission review modal (shows author, signature validity, permission list) → `[Install]` → inline download progress bar → completion notification pushed to TOS Log.
    - [x] Implement install failure state: amber progress bar with reason, button resets to allow retry.
    - [x] Implement installed state in browse: `[Installed ✓]` badge on cards; `[Manage in Settings →]` link on detail page.
    - [x] Add `marketplace_home`, `marketplace_category`, `marketplace_detail`, `marketplace_search_ai`, `marketplace_install_cancel`, `marketplace_install_status` IPC messages (extends existing `marketplace_search`, `marketplace_install`).

- [ ] **Predictive Interaction & Heuristics** *(Architecture Spec §10):*
    - *Depends on: Standalone Heuristic Service (Phase 2), OSC-Exclusive Mode Switching (Phase 1).*
    - [x] Implement **Autocomplete-to-Chip**: real-time shell/path completion resulting in clickable left/right chips.
    - [x] Implement **Implicit Correction Trigger**: hook into shell error state `127` / `command not found` to trigger typo-matching chips.
    - [x] Implement **Heuristic Sector Renaming**: update sector names based on `Cwd` or `ActiveApp`.

- [ ] **Vector Search Engine** *(Architecture Spec §18.3):*
    - Replace "token-overlap" algorithm in `src/services/search.rs` with local vector embedding search (`fastembed` or local vector store).
    - *Requires background indexer generating and caching embeddings on file changes.*

---

## Phase 4 — High-Fidelity Visual Layer
*Depends on Phase 3 being stable. Visual polish and platform-specific rendering.*

- [ ] **Global Console Implementation:** Update `svelte_ui/src/lib/components/SystemOutput.svelte` to fully implement the "System Output Area" (Level 1 Middle Layer).
    - Render Brain terminal log behind sector tiles.
    - Implement the "Bring Terminal to Front" bezel toggle logic.
    - *Note: Basic implementation exists in `SystemOutput.svelte` — needs Level 1 z-layering and spatial integration.*
- [ ] **Kinetic Zoom Transitions:** Implement the z-axis zoom animation between Levels 1 and 2.
    - Animate sector tile borders expanding to become the Tactical Bezel.
    - Apply depth-blur/fade to background layers (Global Map/Brain Console).
    - *Also powers the Expanded Bezel spatial zoom-out — reuse the same animation system.*
- [ ] **Tiered Thumbnailing System:**
    - **Sector Tiles (Level 1):** Render dynamic thumbnails of active hubs/apps within the tile interior.
    - **App Tiles (Level 2 ACT):** Implement 10Hz live thumbnails for running apps.
    - **Inactive Chips (Level 2 ACT):** Fallback to static app icons for non-running applications.
    - **Generic Fallback:** Symbolic placeholder for system processes lacking icons and frame buffers.
- [ ] **Secondary Select Infrastructure:** Implement long-press/right-click trigger for all chip types.
    - Create "Tactical Context Menu" glassmorphism UI component.
    - Implement IPC handlers for `[Signal]`, `[Renice]`, `[Inspect]` actions.
- [ ] **Kinetic Sector Borders:** Implement dynamic CSS border animations for sector tiles.
    - Solid Green/Red for last command exit status.
    - Sliding Gradient for active PTY tasks.
    - *Depends on PTY exit code telemetry being broadcast via IPC versioned state.*
- [ ] **Wayland Frame Captures:** Replace `base64` mock thumbnails in `src/brain/sector/mod.rs` with actual frame buffer fetches.
    - Utilize DMABUF Native Path to share sub-surface textures with UI thread at 10Hz.
    - *Requires Face renderer to support `dmabuf` texture bindings.*
- [ ] **Level 4 Tactical Reset (God Mode) Implementation:**
    - Develop Wireframe Diagnostic Renderer (low-poly, high-performance view).
    - Implement Global Process Kill-Switch in Brain (with re-auth and confirmation).
    - Implement Prompt Interlocking (lock prompt during Tactical Reset; disable Expanded Bezel trigger).
    - Hook Auto-Trigger logic for high-latency or deadlock states.

---

## Phase 5 — Native Platform Faces
*Depends on Phase 4. Requires stable visual layer and all IPC contracts locked.*

**PRIORITY 1 — Native Linux Face (Wayland Shell):**
- [ ] Replace `Face` struct's `println!` simulation with `LinuxRenderer` (Wayland).
- [ ] Implement real `wlr-layer-shell` surface management in `src/platform/linux/`.
- [ ] Native GL/Vulkan composition of Sector tiles and Hub viewports.
- [ ] **Discovery & Connectivity:** Connect to local `brain.sock` first; if not found, scan via mDNS (`_tos-brain._tcp`); probe saved hosts; present manual host:port entry dialog as fallback. Save connections to `~/.config/tos/remote-hosts.toml`.

**PRIORITY 2 — Native OpenXR Face (Quest/VisionPro):**
- [ ] Populate `src/platform/xr/` with OpenXR context initialization.
- [ ] Implement World Space Compositing for the cylindrical "Cockpit" viewport.
- [ ] 3D spatial positioning of sectoral glass panels.
- [ ] **Discovery & Connectivity:** Scan via mDNS (`_tos-brain._tcp`) on startup; show discovered instances and saved hosts in VR connection lobby; manual host:port entry as fallback; direct memory sharing if Brain is running on same device.

**PRIORITY 3 — Native Android Face:**
- [ ] Populate `src/platform/android/` with NDK-based surface rendering logic.
- [ ] Integration with Android choreographer for 90Hz+ smooth persistence.
- [ ] **Discovery & Connectivity:** Background mDNS scan for `_tos-brain._tcp`; probe saved hosts; manual host:port entry; `TOS_REMOTE_HOST`/`TOS_REMOTE_PORT` env var support.

---

## Superseded Items
*These items from the Alpha-2.1 roadmap have been replaced by Alpha-2.2 specifications and should not be implemented.*

- ~~**Tactile Confirmation UI (Confirmation Slider):**~~ Replaced by Command Trust & Confirmation System (Trust-Confirmation-Specification). Remove `update_confirmation_progress` IPC message.
- ~~**Architecture Spec §11 (Split Viewports):**~~ Replaced in full by Split-Viewport-Specification.

---

## Cross-Feature Dependency Map

| Task | Blocked By |
| :--- | :--- |
| All Face components | `tos-protocol` extraction, Visual Token System, Svelte Face (`svelte_ui/`) |
| All new Brain services | Headless Brain Testing |
| Onboarding Step 0 (Trust Screen) | TrustService |
| Onboarding cinematic Brain log stream | Brain init log IPC (existing) |
| AIService refactor | Production LLM Bridge |
| AI behavior modules | AIService refactor |
| AI chat history restore | `tos-sessiond`, Chat Companion module |
| Split Viewport Face | Split Pane Tree (Brain-side) |
| Expanded Bezel split action chip | Split Pane Tree (Brain-side) |
| Expanded Bezel pane promotion | `tos-sessiond` |
| Session restore UI | `tos-sessiond`, Split Pane Tree |
| Pane detach to sector | `tos-sessiond` (writes layout on detach) |
| Predictive Interaction | Standalone Heuristic Service, OSC-Exclusive Mode Switching |
| Vector Search Engine | Background file indexer with embedding cache |
| Kinetic Zoom Transitions | Stable Face component tree (Phase 3) |
| Wayland Frame Captures | `dmabuf` texture binding support in Face renderer |
| Kinetic Sector Borders | PTY exit code telemetry via IPC versioned state |
| Level 4 Tactical Reset Expanded Bezel disable | Expanded Bezel IPC + Tactical Reset prompt lock |
| Session Service | Service Registry & Port Infrastructure |
| `tos ports` CLI | Service Registry & Port Infrastructure |
| mDNS discovery | Service Registry & Port Infrastructure + Avahi integration |
| Remote Face manual entry | `get_port_map` IPC (Service Registry) |
| Native Platform Faces | All Phase 4 items stable, Service Registry & Port Infrastructure |
