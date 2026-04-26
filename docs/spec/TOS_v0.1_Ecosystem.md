# TOS Ecosystem Specification

**Purpose:** This document defines the plugin architecture, module types, sandboxing rules, marketplace systems, and service orchestration for **TOS** (**Terminal On Steroids**). It covers everything from module manifests to boot sequencing.

**Version:** 0.1

For system architecture, IPC contracts, and visual hierarchy, see the [Architecture Specification](./TOS_v0.1_Architecture.md).

---

## Table of Contents

1. [Modules & Packages](#1-modules--packages)
2. [Sector Templates and Marketplace](#2-sector-templates-and-marketplace)
3. [Service Orchestration & Boot Sequencing](#3-service-orchestration--boot-sequencing)
4. [Dynamic Port Management & Service Registry](#4-dynamic-port-management--service-registry)
5. [Remote Discovery](#5-remote-discovery)

---

## 1. Modules & Packages

Modules are platform-specific plugins (`.so` on Linux, `.apk` or dynamic modules on Android) that extend TOS functionality.

TOS employs a dual-tier trust model for modules:
1. **Standard Tier (Sandboxed):** Most modules run in an isolated environment and must declare required permissions in a manifest (`module.toml`).
2. **System Tier (Trusted):** Shell Modules and native Sector Types are trusted by the user and run without TOS-enforced sandboxing to ensure full local system access.

### 1.0 Package Format & Structure

All TOS modules are distributed as signed archives with a `.tos-<type>` extension. Recognized types: `.tos-appmodel`, `.tos-sector`, `.tos-assistant`, `.tos-curator`, `.tos-agent`, `.tos-terminal`, `.tos-theme`, `.tos-shell`, `.tos-bezel`, `.tos-audio`.

```
package.tos-terminal/
├── module.toml         # Canonical manifest
├── signature.sig       # Ed25519 signature of the manifest + assets
├── bin/                # Compiled binaries (if any)
├── assets/             # CSS, icons, fonts, sounds
├── etc/                # Default configuration files
└── README.md           # Documentation
```

**Signature Scheme:** Modules must be signed by registered developers. The Marketplace Service verifies signatures against a trusted root CA before installation. Users can add custom public keys to allow "sideloading" of community-built modules.

### 1.1 Application Model (`.tos-appmodel`)

Customizes an application's integration at Level 3. Manifest includes: name, version, type = "app-model", icon, permissions, capabilities (bezel actions, searchable content, etc.).

### 1.2 Sector Type (`.tos-sector`)

Defines a sector's default behaviour: command favourites, interesting directories, environment, available hub modes, default guest role, associated Application Models.

### 1.3 Cortex: The Modular Orchestration Layer

The **Cortex** is the Brain’s modular reasoning, context, and behavior layer. It replaces the monolithic AI Service and the separate Skill/Persona systems. All Cortex components are hot‑loaded from `~/.local/share/tos/cortex/` and communicate with the Brain via standardized protocols.

| Extension | Component | Role | Logic Type |
|---|---|---|---|
| **`.tos-assistant`** | **Reasoning** | Manages LLM backend communication. | Service‑based (Ollama, Gemini, OpenAI). |
| **`.tos-curator`** | **Context** | Standardizes data and indexing sources. | **MCP‑based** (Model Context Protocol). |
| **`.tos-agent`** | **Behavior** | Defines canned personas and strategies. | Prompt‑based (Careful‑bot, Vibe‑coder). |

### 1.3.1 Assistants (`.tos-assistant`)

Assistants are providers, not models. They register a specific backend service and expose a list of available models.

*Manifest:*
```toml
[metadata]
id = "ollama-provider"
name = "Ollama Local"
type = "assistant"

[connection]
transport = "http"
endpoint = "http://localhost:11434"
timeout_ms = 10000

[auth]
type = "none" # Credentials injected by Brain from secure store

[trust]
may_request = [] # Assistants usually don't need direct tool access

[capabilities]
streaming = true
function_calling = true
latency_profile = "local"
```

### 1.3.2 Curators (`.tos-curator`)

Curators connect external data sources into a unified Global Knowledge Graph. They are typically MCP servers.

*Manifest:*
```toml
[metadata]
id = "gitnexus-mcp"
name = "GitNexus"
type = "curator"

[connection]
transport = "mcp"
endpoint = "http://localhost:3000/mcp"

[auth]
type     = "api_key"
header   = "Authorization"
prefix   = "Bearer "
env_hint = "GITNEXUS_TOKEN"

[trust]
may_request  = ["write_file"] # Allows the curator to suggest file edits
grant_scope  = "session"
cross_sector = false

[mcp]
command = "npx"
args = ["-y", "@abhigyanpatwari/gitnexus", "mcp"]
```

### 1.3.3 Agents (`.tos-agent`)

Agents encapsulate persona, constraints, and task strategy. They use **Agent Stacking** to merge instructions hierarchically.

*Manifest:*
```toml
[metadata]
id = "careful-bot"
name = "Careful Bot"
type = "agent"

[prompt]
identity = "You are a meticulous Rust developer."
constraints = ["Run `cargo test` after every change."]
efficiency = "Keep responses under 200 words."

[trust]
may_request = ["read_file", "write_file", "exec_cmd", "search_codebase"]
grant_scope = "interaction"
```

### 1.3.4 Credential Injection & Trust Gating

Cortex components run under strict isolation with two additional security layers:

1. **Credential Injection (`[auth]`):** Secrets (API keys, tokens) are never stored in the manifest. The manifest declares the *shape* of the requirement; the Brain injects the actual values from the secure Settings store at request time.
2. **Trust Gating (`[trust]`):** Components must declare which Brain tools they intend to use. The Brain’s Trust Chip system enforces these declarations—even if a user grants broad access, a component can only invoke tools listed in its `may_request` block.

### 1.3.5 Marketplace & Activation Workflow

1. Marketplace drops a template into `cortex/pending/`.
2. Settings UI reads the manifest and renders a configuration form for `[auth]` and connection details.
3. User enters credentials (stored in secure `tos-settingsd`).
4. On activation, the Brain moves the file to `cortex/active/` and initializes the component.

### 1.4 (Reserved for future Cortex expansion)

**Full AI system specification:** See [Features Specification §4 — Ambient AI & Co-Pilot System](./TOS_v0.1_Features.md).

### 1.5 Terminal Output Modules (`.tos-terminal`)

Terminal Output Modules define how terminal output is visually presented within Command Hubs and the System Output Area at Level 1.

#### 1.5.1 Module Interface

A Terminal Output Module must implement a well-defined interface (Rust trait or FFI):

- Initialize a new instance for a given **context** (sector terminal or system output). The context determines whether the instance is interactive or read-only.
- Receive a stream of lines, each with metadata: text content (UTF-8, including ANSI codes), timestamp, exit status, command echo flag, and priority/importance level.
- **Render the output** to a surface provided by the Face. See Architecture §30.1 for rendering and interaction contracts.
- **Provide configuration options** exposed via the Settings Daemon.

```rust
pub trait TerminalOutputModule {
    fn init(&mut self, context: TerminalContext, config: ModuleConfig);
    fn push_lines(&mut self, lines: Vec<TerminalLine>);
    // UI-side rendering and event handling defined in Architecture §30.1
}
```

**Multi-terminal support:** Terminal Output Modules that support multi-terminal layouts must declare `multi_terminal = true` in their manifest and implement the `render_layout(hub_layout)` interface. Modules without this declaration receive the `tabs` fallback automatically.

#### 1.5.2 Built-in and Optional Modules

- **Rectangular Module:** Standard flat full-width scrolling block with uniform text and vertical scrolling.
- **Cinematic Triangular Module:** Adds 3D depth. Lines recede toward a vanishing point at the user's focus, scaling down as they get older. Supports a "pinwheel" multi-terminal layout where inactive panes appear as smaller angled panels flanking the primary pane. Scrolling triggers subtle haptic detents.

#### 1.5.3 Installation and Switching

- Users browse the Marketplace for Terminal Output Modules.
- After installation, the module appears in **Settings → Appearance → Terminal Output**.
- Users can select the active module globally, or per-sector (if the module supports it).
- Switching modules takes effect immediately in all open Command Hubs (existing terminal history is re-rendered by the new module).


---

### 1.8 Theme Modules (`.tos-theme`)

Theme Modules define the visual appearance of TOS across all levels.

**Manifest example:**
```toml
name = "Star Trek: TNG"
version = "1.0.0"
type = "theme"
description = "Classic LCARS color scheme from The Next Generation"
author = "TOS Community"
icon = "tng.png"

[assets]
css = "theme.css"
fonts = ["lcars.ttf"]
icons = "icons/"

[capabilities]
supports_high_contrast = true
supports_reduced_motion = true
```

**Interface:**
- **CSS Injection:** The Face reads `theme.css` and injects its content into a `<style>` block at the root of the UI.
- **Dynamic Updates:** Themes react to system state (Alert Levels) via CSS classes on `<body>` (e.g., `.alert-red`, `.alert-yellow`).
- **Asset Resolution:** Icons are referenced by name and resolved to the module's `icons/` path.

**Color palette:** Use the curated variables in `variables.css` (`--lcars-orange`, `--lcars-blue`, `--lcars-gold`, `--lcars-red`). Use semi-transparent backgrounds with `backdrop-filter: blur()` for glassmorphism overlays.

**Built-in themes:** TOS ships with at least two default themes — a light and a dark variant of the LCARS design, plus a high-contrast accessibility theme.

**Installation and switching:**
- After installation, the theme appears in **Settings → Appearance → Theme**.
- Switching themes takes effect immediately (UI reloads with new styles).

**Permissions:** Typically none — themes are static assets. If a theme includes web fonts, it must declare network permissions.

### 1.9 Shell Modules (`.tos-shell`)

Shell Modules provide different shell implementations. They include the shell executable (or wrapper script), integration scripts to enable OSC communication, and default configuration files.

**Manifest example:**
```toml
name = "Zsh"
version = "5.9"
type = "shell"
description = "Z shell with powerline support"
icon = "zsh.png"

[executable]
path = "bin/zsh"
args = ["--login"]

[integration]
osc_directory = true       # Supports OSC 1337;CurrentDir
osc_command_result = true  # Supports OSC 9002 (with base64)
osc_suggestions = false    # (future) Supports command suggestions

[configuration]
default_env = { LANG = "en_US.UTF-8" }
rc_file = "etc/zshrc"
```

**PTY Lifecycle:**
- `spawn(config)`: Create PTY, set ENV, fork/exec.
- `write(input)`: Forward prompt characters to PTY stdin.
- `resize(cols, rows)`: Send `TIOCSWINSZ` to PTY.
- `signal(sig)`: Send `SIGINT`, `SIGTERM`, etc.

**OSC Requirements:** Shells MUST emit `ESC]1337;CurrentDir=<path>BEL` and `ESC]9002;...BEL` for status reporting.

**Shell Telemetry sequences:**
- **OSC 7 (Current Directory):** `\x1b]7;file://hostname/path\x07`
- **OSC 1337 (Current Directory):** `\x1b]1337;CurrentDir=/path\x07`
- **OSC 9002 (Command Result):** `\x1b]9002;<command>;<status>;<base64_output>\x07`

**Permissions:** Shell modules run as user processes with the same privileges as any shell. They are not sandboxed by TOS.

**Installation and switching:**
- Default shell set in **Settings → System → Default Shell**.
- Per-sector shell selection via Sector Overrides.
- Switching shells for an existing sector requires a sector reset or creating a new hub.

**Built-in shell:** TOS includes a reference Fish shell module with full OSC integration. Additional modules (Bash, Zsh) available via the Marketplace.

### 1.10 Bezel Component Modules (`.tos-bezel`)

Bezel Components are modular UI elements that can be installed via the marketplace and docked into any available Tactical Bezel Slot (Top, Left, or Right). Each component runs as a background process or thread and communicates with the Face via the API defined in Architecture §30.2.

For the complete list of core system components and their default slot assignments, see Architecture §8.1.

### 1.11 Audio Theme Modules (`.tos-audio`)

Audio themes define earcon sets and ambient audio layers for the three-layer audio model (Architecture §23). Can be installed via the Marketplace and selected in **Settings → Interface → Audio Theme**.

### 1.12 Language Modules (`.tos-language`)

Language modules add syntax highlighting and optional LSP configuration for languages not built into TOS. They are used exclusively by the TOS Editor (Features §6).

**Manifest example:**
```toml
name = "Gleam Language Support"
version = "1.0.0"
type = "language"
description = "Syntax highlighting and LSP for the Gleam functional language"
author = "Community"
file_extensions = [".gleam"]
treesitter_grammar = "bin/gleam.so"

[lsp]
command = "gleam"
args = ["lsp"]
```

**Installation and activation:**
- After installation, the language appears in the editor's language detection pool.
- The editor automatically uses the module when it detects a matching file extension.
- LSP servers declared in the module are started on-demand when the editor opens a matching file — the LSP binary must be present in the sector's PATH.

**Permissions:** Language modules require no special permissions. They are static grammar assets plus an optional LSP configuration. They do not run in the sandbox — they are loaded directly into the editor's rendering pipeline.

- **Cortex components** run under the same sandboxing rules as standard modules, with the addition of **Credential Injection** and **Trust Gating** (see §1.3.4).
- Credentials declared in `[auth]` are injected by the Brain at request time.
- Tool access declared in `[trust]` is strictly enforced by the Brain Tool Registry.

### 1.14 Relationship Between Module Types

- **Sector Types** may specify a preferred shell and a default set of AI skills to activate.
- **Application Models** are shell-agnostic; they interact with the Brain, not directly with the shell.
- **Terminal Output Modules** render the shell's output regardless of which shell is used.
- **Theme Modules** affect the appearance of all UI elements, including terminal output and the editor surface. Editor token colors are defined in the theme's CSS via `.tos-editor` class selectors.
- **Assistants** provide inference backends; **Agents** define behavior; **Curators** supply real‑time context. Together they form the **Cortex**.  
- Language Modules are editor‑only and have no interaction with the terminal or the Cortex unless an Agent explicitly requests file content via the Brain Tool Registry.

---

## 2. Sector Templates and Marketplace

### 2.1 Package Types & Manifests

| Sector Template | `.tos-template` | Blueprint for creating pre-configured workspaces. |
| Sector Type | `.tos-sector` | Logic for special sector behavior. |
| Application Model | `.tos-appmodel` | Customizes Level 3 integration. |
| **Assistant** | **`.tos-assistant`** | LLM backend provider; manages model discovery. |
| **Curator** | **`.tos-curator`** | MCP‑based data & context source. |
| **Agent** | **`.tos-agent`** | Prompt‑based persona & strategy stack. |
| Terminal Output Module | `.tos-terminal` | Visual terminal rendering logic. |
| Theme Module | `.tos-theme` | Global CSS and assets. |
| Shell Module | `.tos-shell` | PTY integration and shell binaries. |
| Audio Theme | `.tos-audio` | Earcons and ambient layers. |
| Bezel Component | `.tos-bezel` | Dockable bezel slot components. |
| Language Module | `.tos-language` | Syntax highlighting grammar and LSP configuration for the editor. |

#### 2.1.1 Sector Template Schema

```toml
name = "Rust Development"
type = "template"
description = "Pre-configured for Rust projects with terminal and lsp chips."

[environment]
PATH = "$PATH:$HOME/.cargo/bin"
RUST_LOG = "info"

[hubs.main]
mode = "CMD"
cwd = "~/projects"
shell = "fish"
terminal_module = "cinematic-triangular"

[chips.left]
pinned = ["~/projects", "/etc"]

[chips.right]
actions = ["cargo build", "cargo test", "cargo run"]
```

### 2.2 Installation & Atomic Updates

The **Update Daemon** ensures updates are applied safely:
1. **Download:** New package downloaded to a temporary buffer.
2. **Verification:** Signature and checksum verified.
3. **Staging:** Files extracted to a secondary directory.
4. **Switching:** Symlink in `~/.local/share/tos/modules/active/` updated atomically.
5. **Reload:** Brain receives `reload_module:<id>` signal to hot-swap the logic where possible, or prompts for a Tactical Reset.

### 2.3 Installation Flow & Permissions

1. Discovery (Search, Marketplace, direct file open).
2. Details panel with description, permissions, dependencies.
3. Permission review (user grants/denies; optional session-only grant).
4. Dependency resolution.
5. Installation (files copied to `~/.local/share/tos/` or equivalent).
6. Post-install notification; immediate availability.

### 2.4 Discovery (Search, AI, Updates)

- Search Mode includes packages as a domain.
- AI-assisted discovery ("I need a Git integration").
- Update alerts (Yellow Alert) for installed modules; update details show permission changes.

### 2.5 Creating & Sharing Packages

- Export sector as template.
- Developer tools for packaging modules.
- Submission to repositories (optional signature verification).

**Module signing workflow:**

```bash
# Generate a new developer key pair
tos-pkg gen-key --output ./dev-key.pem

# Sign your module
tos-pkg sign --key ./dev-key.pem --module ./dist/my-module.tos-theme

# Verify the manifest structure
tos-pkg verify ./my-theme-module

# Dry-run loading the module in a mock brain
tos-pkg load ./my-theme-module
```

### 2.6 Marketplace Discovery & Browse Experience

The full Marketplace UI specification — including the Home view, Category Browse, Module Detail Page, and install flow — is documented in [Features Specification §5 — Marketplace Discovery](./TOS_v0.1_Features.md).

---

## 3. Service Orchestration & Boot Sequencing

TOS is designed as a constellation of independent processes communicating over a local bus (TCP/WebSocket). This decoupled approach ensures that AI failures, log overflows, or marketplace timeouts do not compromise the core Brain logic.

### 3.1 The Core Constellation

- **`tos-brain`**: The central authority. Manages global state, hierarchical navigation (Levels 1–4), coordinates IPC between all components, and maintains the **Service Registry** — the single source of truth for all daemon endpoints. The Brain exposes a local Unix domain socket for daemon registration and local discovery, an **always-on anchor TCP port** (default 7000, configurable in Settings) for remote access, and an ephemeral WebSocket port for state synchronization.
- **`tos-face`**: The visual layer (Web UI or native client). Discovers the Brain via the local socket (local) or via the anchor port / mDNS / manual entry (remote), then queries the Brain's service registry for the full port map.

### 3.2 Auxiliary Daemons

Daemons are specialized services that extend the Brain's capabilities. They launch as background processes, bind ephemeral TCP ports (Port 0), and **register themselves with the Brain** on startup:

| Daemon | Binary | Description |
|---|---|---|
| Settings Service | `tos-settingsd` | Handles persistence of global and sector-specific configurations. |
| Log Service | `tos-loggerd` | Aggregates structured JSONL logs from all system components. |
| Marketplace Service | `tos-marketplaced` | Performs cryptographic verification of installable modules. |
| Priority Service | `tos-priorityd` | Calculates tactical priority scores based on system telemetry. |
| Session Service | `tos-sessiond` | Handles session persistence and workspace memory (live state auto-save, named sessions). |

### 3.3 Orchestration Flow

The boot sequence is managed by the `Makefile`. **The Brain starts first** so that daemons can register with it immediately.

#### Phase 1: Brain Core (`make run-brain`)

1. `mkdir -p logs`
2. Cleanup of lingering processes (`pkill`) and stale socket files.
3. Launch `tos-brain`:
   - Creates the registration socket at `$XDG_RUNTIME_DIR/tos/brain.sock` (fallback: `~/.local/share/tos/brain.sock`).
   - Binds the **anchor port** (resolved from: `TOS_ANCHOR_PORT` env var → `tos.network.anchor_port` setting → default `7000`).
   - Binds an ephemeral WebSocket port for Face state synchronization.
   - Begins accepting daemon registrations and client queries immediately.
   - Advertises itself via mDNS (`_tos-brain._tcp`) if Avahi is available.
   - Writes the active anchor port value back to `tos.network.anchor_port` in the Settings Daemon (once `tos-settingsd` registers).

#### Phase 2: Service Initialization (`make run-services`)

Each daemon starts in the background, binds its own ephemeral TCP port, and registers with the Brain:

1. Daemon calls `bind(0.0.0.0:0)` → OS assigns an ephemeral port.
2. Daemon connects to `brain.sock` and sends a registration message:
   ```json
   { "type": "register", "name": "settingsd", "port": 49152 }
   ```
3. Brain ACKs the registration and adds the daemon to its in-memory service registry.
4. Daemon is now discoverable by any client querying the Brain.

If the Brain socket is not yet available (race condition), the daemon retries with exponential backoff (100ms → 200ms → 400ms, max 5 retries). If registration fails, the daemon logs a fatal error and exits.

#### Phase 3: Face Orchestration (`make run-web`)

1. Spawns `python3 -m http.server` to serve the Web UI (it also registers with the Brain).
2. Sets up a signal trap (INT/TERM) to ensure killing the Brain also terminates the UI server and backgrounded services.

**Full Stack:** `make run-web` runs all three phases in sequence: Brain → Services → Face.

---

## 4. Dynamic Port Management & Service Registry

The Brain maintains an in-memory service registry. There are **no port files on disk** — the Brain is the single source of truth.

### 4.1 Registration Protocol (Unix Socket)

Daemons register over the local Unix domain socket at `$XDG_RUNTIME_DIR/tos/brain.sock`:

| Message | Direction | Effect |
|:---|:---|:---|
| `{ "type": "register", "name": "<service>", "port": <port> }` | daemon → Brain | Registers a service endpoint |
| `{ "type": "ack", "name": "<service>" }` | Brain → daemon | Confirms registration |
| `{ "type": "deregister", "name": "<service>" }` | daemon → Brain | Removes service on graceful shutdown |
| `{ "type": "get_port_map" }` | client → Brain | Requests the full service map |
| `{ "type": "port_map", "services": { ... } }` | Brain → client | Returns all registered services |

### 4.2 Service Map Response

```json
{
  "type": "port_map",
  "host": "192.168.1.5",
  "anchor_port": 7000,
  "services": {
    "brain_tcp":    7000,
    "brain_ws":     52314,
    "settingsd":    49152,
    "loggerd":      49153,
    "marketplaced": 49155,
    "priorityd":    49157,
    "sessiond":     49160,
    "web_ui":       49161
  }
}
```

### 4.3 Health Monitoring

The Brain periodically probes registered services (TCP connect, 30s interval). If a service is unreachable for 3 consecutive probes, it is marked as `offline` in the registry (but not removed — the daemon may restart and re-register).

`make test-health` queries the Brain's registry and displays reachability status for all services.

### 4.4 Anchor Port Resolution

The Brain **always** creates an anchor port. The value is resolved in priority order:

| Priority | Source | Example |
|:---|:---|:---|
| 1 (highest) | `TOS_ANCHOR_PORT` environment variable | `TOS_ANCHOR_PORT=7777` |
| 2 | `tos.network.anchor_port` setting (persisted) | Set via **Settings → Network** |
| 3 (default) | Hardcoded fallback | `7000` |

**Behavior:**
1. Brain attempts to bind the resolved anchor port.
2. If **available** → binds it, registers in its own service map, writes the value back to `tos.network.anchor_port`.
3. If **occupied** → Brain scans upward (+1 to +10). If a nearby port is found, binds it and updates the setting. Logs a notice.
4. If **all 10 fallback ports are occupied** → Brain binds Port 0 (fully ephemeral) and logs a warning. Remote clients without mDNS will not be able to connect.

The `TOS_ANCHOR_PORT` env var is intended for CI, Docker, and scripted environments. For interactive use, **Settings → Network** is the primary configuration surface.

### 4.5 Settings → Network

```
┌─────────────────────────────────────────────────┐
│  Network                                        │
│                                                 │
│  Remote Access Port:  [ 7000       ]  ✓ active  │
│  WebSocket Port:      52314          (auto)     │
│  mDNS Advertisement:  [✓] Enabled               │
│                                                 │
│  Service Registry:    8 services online         │
│  [ View Port Map → ]                            │
└─────────────────────────────────────────────────┘
```

Changing the **Remote Access Port** writes to `tos.network.anchor_port` and takes effect on next Brain restart.

### 4.6 Manual Status Check

- **Port Map:** `tos ports` — queries the Brain's service registry and displays a formatted table.
- **Process Status:** `ps aux | grep tos-`
- **Socket Awareness:** `ss -ltnp | grep tos-`
- **Tactical Health Check:** `make test-health`
- **Orchestration Logs:** `logs/settingsd.log`, `logs/loggerd.log`, `logs/marketplaced.log`, `logs/priorityd.log`, `logs/sessiond.log`, `logs/tos-brain.log`

### 4.7 Communication Protocol

- **Brain-to-Daemon:** Brain initiates tactical queries to daemons as needed, using the port map from its in-memory registry.
- **Face-to-Brain (Local):** Faces connect to `brain.sock`.
- **Face-to-Brain (Remote):** Remote Faces connect to the Brain's TCP anchor port. Once connected, the Face establishes a persistent WebSocket for state synchronization on the Brain's ephemeral WS port.

---

## 5. Remote Discovery

Remote clients (native Faces, Web Portals, Horizon OS clients) discover the Brain without access to the local Unix socket. Three mechanisms are provided, in order of preference:

### 5.1 Method 1: Anchor Port (Default)

Since the Brain always binds an anchor port (default `7000`), remote clients can connect directly if they know the host:

1. **Connect:** Remote client connects to `<host>:<anchor_port>` TCP.
2. **Query:** Client sends `get_port_map` over the TCP connection.
3. **Response:** Brain replies with the full service map.
4. **Upgrade:** Client uses the returned `brain_ws` port to establish a WebSocket for state synchronization.
5. **Re-discover:** If a connection drops, the client reconnects to the anchor port and re-queries the map. See [Architecture §3.4.4](./TOS_v0.1_Architecture.md#344-reconnection-logic) for the full retry schedule and visual states during reconnection.

### 5.2 Method 2: mDNS / DNS-SD (Zero-Config LAN)

The Brain advertises itself on the local network using mDNS (Avahi on Linux):
- **Service Type:** `_tos-brain._tcp`
- **TXT Records:** `brain_tcp=<port>`, `brain_ws=<port>`
- **Name:** `TOS-<hostname>`

```bash
# Discover TOS instances on the LAN
avahi-browse -rt _tos-brain._tcp

# Or from the tos CLI
tos discover
```

### 5.3 Method 3: Manual Entry (Fallback)

If mDNS is unavailable, the user can manually specify the Brain's host and port. The default anchor port (`7000`) means users often only need to enter the host.

**Connection dialog:**
```
┌────────────────────────────────────────────┐
│  Connect to TOS Brain                      │
│                                            │
│  [ Scanning LAN... ]       (mDNS)          │
│  [ 192.168.1.5 — TOS-workstation ] ← found  │
│                                            │
│  ── or enter manually ──────────────────   │
│  Host: [ 10.0.0.42_________ ]              │
│  Port: [ 49300_____________ ]              │
│                                            │
│        [ Connect ]    [ Cancel ]           │
└────────────────────────────────────────────┘
```

**CLI:**
```bash
# Direct arguments
tos-face --host 10.0.0.42 --port 49300

# Environment variables
TOS_REMOTE_HOST=10.0.0.42 TOS_REMOTE_PORT=49300 tos-face
```

**Saved connections:** Successfully connected remote hosts are saved to `~/.config/tos/remote-hosts.toml`:

```toml
[[hosts]]
name = "Workstation"
host = "10.0.0.42"
port = 49300
last_connected = "2026-03-04T18:00:00Z"
```

On next launch, saved hosts are probed automatically alongside mDNS results.

### 5.4 Discovery Priority

| Priority | Method | When to use |
|:---|:---|:---|
| 1 | **mDNS** | Same LAN, zero config |
| 2 | **Anchor Port** (default `7000`) | Cross-subnet, known host |
| 3 | **Saved Hosts** | Reconnecting to a previously used Brain |
| 4 | **Manual Entry** | New host, no mDNS — port defaults to 7000 |

### 5.5 `tos ports` CLI

```bash
$ tos ports
SERVICE              PORT    STATUS
tos-brain (anchor)   7000    ✓ reachable
tos-brain (ws)       52314   ✓ reachable
tos-settingsd        49152   ✓ reachable
tos-loggerd          49153   ✓ reachable
tos-marketplaced     49155   ✓ reachable
tos-priorityd        49157   ✓ reachable
tos-sessiond         49160   ✓ reachable
web_ui               49161   ✓ reachable
```

Flags:
- `tos ports --json` — machine-readable JSON output.
- `tos ports --wait` — blocks until all expected services are registered and reachable (useful in CI).
- `tos ports --remote <host>[:<port>]` — queries a remote Brain's port map.
