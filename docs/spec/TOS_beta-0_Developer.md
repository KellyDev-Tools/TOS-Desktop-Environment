# TOS Developer Reference

**Purpose:** Everything a contributor needs to build, test, and extend TOS — covering project structure, development workflow, the third-party SDK, TDD protocols, and cross-system dependency constraints.

---

## Table of Contents

1. [Project Structure](#1-project-structure)
2. [Development Workflow](#2-development-workflow)
3. [Third-Party Module SDK](#3-third-party-module-sdk)
4. [Testing Strategy & TDD Protocols](#4-testing-strategy--tdd-protocols)
5. [Cross-System Dependencies Map](#5-cross-system-dependencies-map)

---

## 1. Project Structure

### 1.1 Source Directory (`src/`)

#### 1.1.1 Core Processes

- **`main.rs`** — System entry point. Initializes the IPC channel and spawns the Brain and Face threads or processes.
- **`brain/`** — Authoritative state. All logic, command execution, and sector management. No rendering code lives here.
- **`face/`** — Representative layer. Handles input capture and visual rendering. Receives state snapshots from the Brain.

#### 1.1.2 Support Systems

- **`common/`** — Shared data structures (e.g., `Sector`, `CommandHub`) and IPC serialization logic used by both Brain and Face.
- **`services/`** — Independent daemons: Settings, Logging, AI, Heuristic, etc.
- **`modules/`** — The runtime environment for plugins. Includes sandbox logic and the SDK for building standard modules.
- **`platform/`** — Concrete implementations of the `Renderer`, `InputSource`, and `SystemServices` traits for different operating systems.
- **`tos-protocol/`** — Shared Rust crate defining the authoritative IPC schema and state structures used by Brain, Face, and all services.

### 1.2 External Assets

- **`svelte_ui/`** — Svelte 5 web-based Face (built to `svelte_ui/build/`). HTML/CSS/JS for the primary graphical interface.
- **`modules/`** — Local repository for installed `.tos-terminal`, `.tos-theme`, and `.tos-appmodel` files.
- **`scripts/`** — Integration files that users source in their external shells to enable OSC communication.

### 1.3 Quality Assurance

- **`tests/`** — Tiered test suites:
  - `test-core`: Brain logic unit tests.
  - `test-shell`: PTY/OSC integration tests.
  - `test-sec`: Sandbox and permission verification.
  - `test-brain`: Brain component tests.
  - `test-ui`: Playwright-based frontend/bridge validation.

### 1.4 System Components & Ports

| Component | Binary / Directory | Port | Protocol | Description |
|:---|:---|:---|:---|:---|
| **Brain Core** | `tos-brain` | `7000` (anchor, configurable) | TCP | Main logic, IPC handler, & service registry |
| **Brain Socket** | `tos-brain` | — | Unix | Local registration & discovery (`brain.sock`) |
| **Brain UI Sync** | `tos-brain` | Ephemeral | WS | WebSocket for UI state synchronization |
| **Settings Daemon** | `tos-settingsd` | Ephemeral | TCP | Persistent configuration storage |
| **Log Daemon** | `tos-loggerd` | Ephemeral | TCP | Unified system logging |
| **Marketplace** | `tos-marketplaced` | Ephemeral | TCP | Module discovery & verification |
| **Priority Engine** | `tos-priorityd` | Ephemeral | TCP | Tactical priority scoring |
| **Session Service** | `tos-sessiond` | Ephemeral | TCP | Session persistence, workspace memory, and offline AI queue |
| **Web Face** | `svelte_ui/` | Ephemeral | HTTP | Svelte 5 LCARS interface |

**Editor & LSP:** The TOS Editor runs as a pane type within the Web Face — it is not a separate daemon. LSP servers are spawned on-demand by the Face when a `.tos-language` module is active and a matching file is opened. LSP server processes are owned by the sector's process tree and terminated when the sector closes.

To view actual live port assignments, use `tos ports` (queries the Brain's registry). See the [Ecosystem Specification §4](./TOS_beta-0_Ecosystem.md) for the full registration and discovery protocol.

---

## 2. Development Workflow

### 2.1 Starting the Full Stack

The easiest way to start the entire environment is using the provided `Makefile`.

#### Unified Orchestration (Recommended)

```bash
make run-web
```

Builds the Svelte Face (`svelte_ui/build/`), spawns the background services, initializes the Brain, and starts a Python-based HTTP server for the UI.

For development with hot-reload:

```bash
make run-web-dev
```

Starts the Vite dev server (with HMR) alongside the Brain. Changes to `.svelte` files reflect instantly in the browser.

Other web targets:
- `make build-web` — Build the Svelte Face only
- `make dev-web` — Start the Svelte dev server only (no Brain)

#### Manual Component Launch

If you need to debug specific components, start them individually. **The Brain must start first** so that daemons can register with it.

```bash
# Step 1: The Brain Core
cargo run --bin tos-brain

# Step 2: Auxiliary Daemons (any order, after Brain)
cargo run --bin tos-settingsd
cargo run --bin tos-loggerd
cargo run --bin tos-marketplaced
cargo run --bin tos-priorityd
cargo run --bin tos-sessiond

# Step 3a: Web Face (Production Build)
make build-web
python3 -m http.server 8080 -d svelte_ui/build

# Step 3b: Web Face (Development w/ HMR)
export NVM_DIR="$HOME/.nvm" && . "$NVM_DIR/nvm.sh" && nvm use 20
cd svelte_ui && npm run dev -- --port 8080
```

### 2.2 SSH Remote Scenario

When starting TOS over SSH (no local Wayland compositor), the Brain automatically detects the environment and falls back to `Headless` or `Remote` rendering mode.

To explicitly force headless mode:

```bash
# On remote Linux box
ssh user@linux-box
cd ~/path/to/tos

# Start Brain in headless mode
TOS_HEADLESS=1 cargo run --bin tos-brain

# In another window on Windows/local machine:
# (Coming soon: tos-face CLI)
# For now, use the Web Face:
# http://linux-box:8080
```

**What happens:**
1. Brain detects `TOS_HEADLESS=1` (or missing `WAYLAND_DISPLAY`) and initializes `HeadlessRenderer`.
2. Brain binds to anchor port 7000 and advertises via mDNS.
3. Face (Web or Remote) connects and receives state updates and buffer streams.

See [Architecture §15.6](../spec/TOS_beta-0_Architecture.md#156-renderer-mode-detection--fallback) for technical details.

### 2.3 Building & Checking

```bash
make build          # Build all
make check          # Fast check (cargo check equivalent)
make lint           # Run linter
cargo check         # Run after any Rust file changes
cargo test          # Run after any Rust file changes
cd svelte_ui && npm run build   # Run after any Svelte/TS file changes
```

> **Pipeline Rule:** Do not commit code that breaks the compilation pipeline. AI Agents must run `cargo check` and `cargo test` after any Rust file changes, and `cd svelte_ui && npm run build` after any Svelte/TS changes.

### 2.3 Testing Tiers (make targets)

| Target | What It Tests |
|---|---|
| `make test-core` | Brain logic unit tests (state machine verification) |
| `make test-shell` | PTY & OSC sequence handling |
| `make test-ai` | Intent extraction and staging |
| `make test-ui-component` | Playwright-based browser tests |
| `make test-health` | Orchestration health check (diagnostic reachability) |

### 2.4 Package Management

```bash
# Interact with the marketplace via CLI
cargo run --bin tos-pkg -- discover ./modules/example
cargo run --bin tos-pkg -- verify ./modules/example
```

### 2.5 Logs & Debugging

System logs are aggregated in the `logs/` directory:
- `logs/tos-brain.log` — Output from the core logic process.
- `logs/web_ui.log` — HTTP server access logs (or `logs/svelte_dev.log` when using `run-web-dev`).
- `logs/system_test.log` — Results from the comprehensive integration suite.
- `logs/settingsd.log`, `logs/loggerd.log`, `logs/marketplaced.log`, `logs/priorityd.log`, `logs/sessiond.log`

### 2.6 Dynamic Port Management

Every TOS process utilizes ephemeral port assignment by requesting Port 0 from the OS. The Brain's in-memory service registry is the single source of truth for all port information.

**Boot sequence:**

```
tos-brain starts     → creates $XDG_RUNTIME_DIR/tos/brain.sock
                     → binds anchor port 7000 (TCP)
                     → binds 0.0.0.0:0 (WS)  → OS assigns port 52314
                     → registers itself: brain_tcp=7000, brain_ws=52314
tos-settingsd starts → binds 0.0.0.0:0 → OS assigns port 49152
                     → connects to brain.sock → sends register(settingsd, 49152)
                     → Brain ACKs → settingsd now discoverable
local Face           → connects to brain.sock → sends get_port_map
                     → receives { brain_tcp: 7000, brain_ws: 52314, settingsd: 49152, ... }
remote Face          → connects to 192.168.1.5:7000 → sends get_port_map
                     → receives full service map → upgrades to WS on port 52314
```

See [Ecosystem Specification §4](./TOS_beta-0_Ecosystem.md) for the full registration protocol, health monitoring, and remote discovery details.

### 2.7 Resources & Templates

- **Rust AI Adapter:** `src/brain/module_manager.rs`
- **CSS Theme Baseline:** `svelte_ui/src/app.css`
- **OSC Script Examples:** `etc/tos-init.fish`

---

## 3. Third-Party Module SDK

### 3.1 Introduction

The TOS Ecosystem is built on a "Local First" philosophy. Every extension — from theme to artificial intelligence — runs in its own localized process space and communicates with the Brain over structured IPC or the "Module Contract" protocol.

**Supported module types:**
- **Themes (`.tos-theme`):** CSS layouts, icons, and typography.
- **AI Backends (`.tos-ai`):** LLM adapters using the JSON Boundary Protocol.
- **AI Skills (`.tos-skill`):** Pluggable co-pilot interaction patterns.
- **Shells (`.tos-shell`):** PTY environments with OSC telemetry.
- **Terminal Output (`.tos-terminal`):** Custom rendering logic for terminal canvases.
- **Application Models (`.tos-appmodel`):** Metadata for deep Level 3 integration.
- **Bezel Components (`.tos-bezel`):** Dockable bezel slot components.
- **Sector Types (`.tos-sector`):** Workspace presets and specialized sector logic.
- **Audio Themes (`.tos-audio`):** Earcon sets and ambient audio layers.

### 3.2 HeadlessRenderer API

For modules that need to render in headless contexts (testing, CI, SSH):

- Buffers are stored in CPU RAM (`Vec<u8>`).
- No GPU calls — all operations succeed even without hardware.
- Useful for unit testing without a running compositor.

```rust
let renderer = HeadlessRenderer::new();
let handle = renderer.create_surface(config);
// Buffer is now allocated in memory; ready for updates
```

### 3.3 Package Anatomy

Every TOS module must adhere to the following directory structure:

```text
package-id/
├── module.toml         # Mandatory Manifest (§3.3)
├── signature.sig       # Cryptographic signature (for verified distribution)
├── bin/                # Executables (AI engines, shell wrappers)
├── assets/             # Themes, CSS, Icons, Fonts
└── etc/                # Default configuration files
```

### 3.3 The Manifest (`module.toml`)

The `module.toml` is the source of truth for the marketplace and the Brain.

#### 3.3.1 Common Fields

```toml
id = "com.community.tactical-amber"
name = "Tactical Amber"
version = "1.2.0"
type = "theme" # Options: appmodel, terminal, theme, shell, ai, skill, bezel, audio, sector
author = "Sovereign Engineering"
description = "High-contrast tactical theme inspired by deep-space sensors."
icon = "assets/icon.png"
```

#### 3.3.2 Type-Specific Sections

**Shell Configuration:**
```toml
[executable]
path = "bin/zsh"
args = ["--login"]

[integration]
osc_directory = true      # Supports OSC 7 or 1337
osc_command_result = true # Supports OSC 9002
```

**Theme Configuration:**
```toml
[assets]
css = "assets/theme.css"
icons = "assets/icons/"
fonts = ["assets/fonts/Inter.ttf"]
```

**AI Backend Capabilities:**
```toml
[capabilities]
chat             = true
streaming        = true
function_calling = true
vision           = false
latency_profile  = "fast_remote"   # local | fast_remote | slow_remote
```

**AI Skill:**
```toml
[behavior]
trigger    = "passive"      # passive | prompt_input | mode_switch | manual
ui_surface = "chips"        # chips | ghost_text | chat_panel | thought_bubble
chip_color = "secondary"
runs_always = true

[permissions]
terminal_read = true
prompt_read   = true
prompt_write  = false
network       = false
```

### 3.4 Module Contracts

#### 3.4.1 The AI Boundary (JSON over Stdin/Stdout)

AI modules are executed as child processes. The Brain communicates via a strict JSON protocol over standard I/O.

**Input (Stdin):**
```json
{
  "prompt": "list all files",
  "context": ["sector:Primary", "path:/home/user"],
  "stream": false
}
```

**Output (Stdout):**
```json
{
  "id": "uuid-123",
  "choice": {
    "role": "assistant",
    "content": "{\"command\": \"ls -la\", \"explanation\": \"Listing files in long format.\"}"
  },
  "status": "complete"
}
```

#### 3.4.2 Shell Telemetry (OSC Sequences)

Shells must emit standard OSC sequences to synchronize with the desktop environment:

- **OSC 7 (Current Directory):** `\x1b]7;file://hostname/path\x07`
- **OSC 1337 (Current Directory):** `\x1b]1337;CurrentDir=/path\x07`
- **OSC 9002 (Command Result):** `\x1b]9002;<command>;<status>;<base64_output>\x07`
- **OSC 9012 (Line Priority):** `\x1b]9012;<level>\x07` (0=Normal, 1=Low, 2=Notice, 3=Warning, 4=Critical, 5=Urgent)

### 3.5 Development Workflow

#### 3.5.1 Discovery & Verification

```bash
# Verify the manifest structure
tos-pkg verify ./my-theme-module

# Dry-run loading the module in a mock brain
tos-pkg load ./my-theme-module
```

#### 3.5.2 Signing

Modules distributed via the Marketplace must be signed.

```bash
# Generate a new developer key pair
tos-pkg gen-key --output ./dev-key.pem

# Sign your module
tos-pkg sign --key ./dev-key.pem --module ./dist/my-module.tos-theme
```

---

## 4. Testing Strategy & TDD Protocols

### 4.1 Test Taxonomy & Definitions

Testing in TOS is strictly categorized into four tiers. No feature code should be written without a failing test being written and executed first.

#### 4.1.1 Unit Tests

- **Definition:** Validates a single, isolated function, struct, or pure logic sequence. No side effects, no file-system access, no network, no global state.
- **Location:** Inline alongside the code within `#[cfg(test)]` modules (Rust) or adjacent `.spec.ts` files (Svelte logic).
- **Execution:** Must execute in microseconds.

**Key Components:**
- **Settings Resolver:** Test cascading resolution (Global → Sector → App).
- **OSC Parser Algorithm:** Verify extraction of 9000-series sequences from dirty string inputs.
- **Priority Math:** Verify priority scores based on mock activity factors.
- **Trust Classifier:** Test regex/rules engines for `privilege_escalation` and `recursive_bulk` detection.

#### 4.1.2 Integration Tests

- **Definition:** Validates the interaction between multiple subsystems natively. Uses realistic but headless state (e.g., verifying an IPC string mutates the Brain and generates the correct JSON delta).
- **Location:** The `tests/` directory at the workspace root (e.g., `tests/headless_brain.rs`).
- **Execution:** Spins up local memory structures, completely bypassing UI/renderers.

**Key Components:**
- **IPC Protocol Matcher:** Validate `prefix:payload` parsing and state mutation (`tests/headless_brain.rs`).
- **Sector Tree Lifecycle:** Test creation, cloning, and destruction of active sectors and PTY backends.
- **TTY Buffer Wraparound:** Test the 500-line FIFO limit when pushing live data to `system_log`.
- **Remote ICE Teardown:** Mock socket drops and verify the graceful 500ms auto-close teardown sequence.

#### 4.1.3 Component Tests

- **Definition:** Validates an individual sub-system, daemon, or UI module completely in isolation, independent of the rest of the system. Given input/state X, it must produce output/state Y.
- **Location:** `svelte_ui/tests/` (for UI components), `tests/` (for isolated Brain modules like the `TrustService`), or standalone service tests.
- **Execution:** Fast execution using mocks for any external dependency (e.g., mocking the `brain.sock` or `SystemServices` trait).

**Key Components:**
- **Isolated Daemons:** Verify `tos-marketplaced` API responses without a running Brain by mocking the `brain.sock`.
- **Brain Subsystems:** Test the `TrustService` decision logic by injecting state JSON independently of the `IpcHandler`.
- **Web Face (Svelte/Playwright):** Assert LCARS `.lcars-bar` and `.glass-panel` layout rules, DOM presence, and interaction state changes in isolation.
- **Native Face (Wayland/OpenXR):** Use string-buffer testing stubs (`tests/face_visual_states.rs`) to validate state representations headlessly without a Compositor.

#### 4.1.4 Modular Sandbox Tests (Security)

- **Goal:** Verify permissions boundary enforcement within the module loader.

**Key Components:**
- **Capability Enforcement:** Ensure "Standard" modules cannot access restricted system traits.
- **Manifest Validation:** Verify module initialization safely rejects malformed or unsigned declarations.
- **Theme Injection Security:** Test that CSS variables from Theme Modules are safely deserialized and cannot perform XSS or arbitrary code execution.

### 4.2 TDD Workflow by Target

#### For Brain/Core (Rust)

1. **Write the Test:** Add a test case to `tests/` (e.g., `tests/headless_brain.rs` or `tests/settings_schema.rs`).
2. **Verify Failure:** Run the test using `cargo test --test <name>` to prove it fails exactly as expected.
3. **Implement Subsystem:** Write the minimal Rust code required to pass the test.
4. **Verify Success:** Run the test again to prove it passes.

> **Rule:** Never use the Face (UI) to manually verify Brain state. Always use headless testing bypassing the IPC socket or explicitly checking state JSON.

#### For Web Face/Frontend (Svelte)

1. **Write the Test:** Add a Playwright component/E2E test in `svelte_ui/tests/`.
2. **Verify Failure:** Run the test using `cd svelte_ui && npx playwright test` to observe failure.
3. **Implement UI:** Build the Svelte component or logic.
4. **Verify Success:** Run the Playwright test again.

> **Rule:** Playwright tests must assert visual state, DOM presence, and CSS classes — not just logic.

#### For Native Faces (Wayland/OpenXR/Android)

1. **Write the Test:** Add a test case to the visual states suite (e.g., `tests/face_visual_states.rs`).
2. **Verify Failure:** Run the test using `cargo test --test face_visual_states` to observe failure.
3. **Implement Render Logic:** Update the platform-specific drawing or layout code.
4. **Verify Success:** Run the test again to prove it correctly simulates the rendering.

> **Rule:** Native faces must provide a testing stub or string-buffer renderer so that visual states, dimensions, and text rendering logic can be validated headlessly in CI without requiring an active Compositor, Spatial runtime, or Handheld hardware.

### 4.3 Mocking Strategy

| Component | Mocking Approach |
|---|---|
| **PTY / Shell Backend** | Use virtual pipes (e.g., `tests/stimulator.rs`) to inject known OSC byte streams and capture responses. |
| **Settings File I/O** | Use an in-memory `HashMap` overlay during testing to prevent polluting `~/.config/tos/`. |
| **Face Input Engine** | Use headless IPC socket writes or direct function calls via the `test-protocol` harness. |
| **Native Renderers** | Stub the `Renderer` traits (`src/platform/mod.rs`) to dump output as parseable text string layout trees. |

### 4.4 Specific Test Protocols

#### 4.4.1 Integration: IPC Standardization Test

1. **Define Test:** `test_ipc_semicolon_parsing`
2. **Input:** `set_setting:theme;lcars-dark`
3. **Expected:** `state.settings.global.get("theme") == "lcars-dark"`
4. **Input:** `signal_app:uuid-123;SIGKILL`
5. **Expected:** `internal_signal_event(uuid-123, SIGKILL)`

#### 4.4.2 Terminal Buffer FIFO Test

1. **Define Test:** `test_terminal_buffer_wrap`
2. **Setup:** Set `terminal_buffer_limit` to 5.
3. **Operation:** Push 6 lines of text.
4. **Expected:** Buffer contains lines 2–6; line 1 is discarded.

#### 4.4.3 Remote Session Failure Test

1. **Define Test:** `test_remote_disconnect_timer`
2. **Setup:** Established remote sector.
3. **Action:** Kill the mock remote socket.
4. **Assert:** Sector state changes to `Disconnected`.
5. **Wait 5.1s:** Verify the sector is removed from `state.sectors`.

#### 4.4.4 Action-Identifier Enforcement

1. **Define Test:** `test_bezel_label_rejection`
2. **Input:** `click:ZOOM OUT` (label instead of identifier)
3. **Expected:** Log warning; no state change.
4. **Input:** `click:zoom_out` (identifier)
5. **Expected:** `state.level` decrements.

### 4.5 Success Criteria

- 100% pass rate on Tier 1 & 2 tests.
- All Dangerous Commands (Architecture §17.2) require verified confirmation logic.
- Zero bypass of Standard Tier sandbox permissions in Tier 4 tests.
- Latency for IPC round-trips (Face → Brain → Face) stays below 16ms in local testing.

### 4.6 Pipeline Verification

- AI Agents must run `cargo check` and `cargo test` after any Rust file changes.
- AI Agents must run `cd svelte_ui && npm run build` after any Svelte/TS file changes.
- Do not commit code that breaks the compilation pipeline.

---

## 5. Cross-System Dependencies Map

This section maps the hard execution blocks across the Ecosystem, Brain, and Face. Tasks must be executed bottom-up according to this dependency tree to prevent development gridlock and orphaned UI states.

### 5.1 Ecosystem Blocks (Data & Services Foundation)

The Ecosystem's background services and IPC integrations are the bedrock of the system. Their omission directly blocks major logic routing and UI rendering features.

**Settings Daemon** (JSON persistence layer and cascading state resolution)
- **Blocks [FACE]:** Settings UI Panel. The Face cannot map dual-sided chips or read theme configurations without the daemon persisting and returning the data.

**Global Search & Indexing Service** (Daemon indexing the file system, apps, and logs)
- **Blocks [BRAIN]:** Natural Language Search. The Brain cannot implement LLM semantic embedding routing without the underlying database index to query.

**Universal OSC Scripts & JSON Context Export** (Shell hooks)
- **Blocks [FACE]:** Directory Context Previews. The Face cannot render inline file or image previews without the shell physically emitting the JSON context metadata upon `ls` or `cd`.

### 5.2 Brain Blocks (Logic & Hardware Transports)

The Brain's system-level hardware APIs and core connection protocols must be initialized before the UI can visualize them or the Ecosystem can sync them securely.

**Wayland DMABUF Logic & Compositing Pipelines** (Zero-copy surface attachment)
- **Blocks [FACE]:** Activity Context Live Thumbnails. The UI cannot render 10Hz live application previews on process chips without the backend compositor extracting and routing the DMABUF handles.

**Multi-Sensory Audio Pipeline** (Initialization of OS audio sinks via `cpal`/`rodio`)
- **Blocks [FACE]:** Multi-Sensory Audio Hooks. The Svelte frontend cannot trigger earcons upon zooming/mode-switching if the backend Rust audio sink is not open.

**Remote WebRTC Auto-Close & Remote Desktop Protocol** (Connection teardown and stability)
- **Blocks [ECOSYSTEM]:** Multi-User Presence API. The ecosystem cannot map cursor sharing, follow modes, or active viewport syncs if the underlying WebRTC socket transport drops randomly or fails to close properly.

**ServiceManager State Decoupling** (Removing `Arc<Mutex<TosState>>`)
- **Blocks [ECOSYSTEM]:** Auxiliary Services. The Ecosystem cannot transition the TOS Log Service, Settings Daemon, or AI Engine into true independent background processes until the core Brain releases its `TosState` ownership lock over them.

### 5.3 Recommended Execution Priority (Bottom-Up)

To safely navigate these blockers, development MUST proceed in this order:

1. **The Bedrock:** Build out the Ecosystem Auxiliary Services (Settings Daemon, Global Search) and Brain Hardware APIs (Wayland Compositor, Audio Sinks, WebRTC).
2. **The Translators:** Build the Ecosystem Shell Scripts (OSC emission) and Brain AI Routing (Natural Language integrations).
3. **The Interface:** Build the Face UI Overlays (Settings Panel, Live Thumbnails, Directory Previews, Audio Hooks) which simply consume the structured data pipelines established in steps 1 and 2.

### 5.4 Editor & AI System Dependencies

**Editor Pane (`pane_type: "editor"`)** (Features §6)
- **Blocks [EDITOR]:** All editor features require the `hub_layout` pane type system (Architecture §11.2) to be implemented first — editor panes are inserted into the existing split layout, not a separate surface.
- **Blocks [AI CONTEXT]:** The Editor Context Object (Features §6.5.1) cannot be included in AI queries until the Brain's `AIService` is updated to accept and merge the editor context delta alongside the standard rolling context.

**LSP Integration** (Features §6.9)
- **Blocks [LSP]:** LSP servers are not managed by TOS — they must exist in the sector's PATH. Development cannot test LSP features without a valid LSP server installed (e.g., `rust-analyzer` for Rust files).
- **Not a blocker for core editor:** Viewer Mode, Diff Mode, AI annotations, and session persistence all function without LSP. LSP is an enhancement layer, not a foundation.

**AI Skill Tool Registry** (Ecosystem §1.4.3)
- **Blocks [VIBE CODER]:** The Vibe Coder skill (Features §4.8) cannot issue `write_file` or `read_file` tool calls until the Brain Tool Registry is implemented and the trust chip system is extended to handle AI-initiated file writes.
- **Blocks [MULTI-FILE EDITS]:** Multi-file edit chip sequences (Features §6.6.3) require the session persistence layer to store `pending_edit_proposal_id` in the editor pane schema (Features §2.9).

**Session Handoff** (Features §2.10)
- **Blocks [HANDOFF]:** Cross-device handoff requires the `face_register` capability profile (Architecture §3.3.5) to be implemented first — the Brain must know the connecting Face profile before it can serve the appropriate session context.
