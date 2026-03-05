# TOS Alpha-2 Development Guide

This guide describes how to start and orchestrate the various components of the **TOS** (**Terminal On Steroids**) environment for development and testing.

## System Components & Ports

TOS is a distributed system consisting of a central logic core (the Brain), a visual interface (the Face), and several auxiliary daemons. **Every** TOS process binds an ephemeral port (Port 0) by default — there are no hardcoded port numbers. For remote access, the Brain's TCP port can be pinned via `TOS_ANCHOR_PORT`.

| Component | Binary / Directory | Port | Protocol | Description |
| :--- | :--- | :--- | :--- | :--- |
| **Brain Core** | `tos-brain` | Ephemeral (or `TOS_ANCHOR_PORT`) | TCP | Main logic, IPC handler, & port map server |
| **Brain UI Sync** | `tos-brain` | Ephemeral | WS | WebSocket for UI state synchronization |
| **Settings Daemon** | `tos-settingsd` | Ephemeral | TCP | Persistent configuration storage |
| **Log Daemon** | `tos-loggerd` | Ephemeral | TCP | Unified system logging |
| **Marketplace** | `tos-marketplaced`| Ephemeral | TCP | Module discovery & verification |
| **Priority Engine** | `tos-priorityd` | Ephemeral | TCP | Tactical priority scoring |
| **Session Service** | `tos-sessiond` | Ephemeral | TCP | Session persistence & workspace memory |
| **Web UI Server** | `web_ui/` | Ephemeral | HTTP | Serves the LCARS interface |

To view actual live port assignments, use `tos ports`. See [Ecosystem Orchestration](./TOS_alpha-2_Ecosystem-Orchestration.md) for the full discovery protocol.

## Starting the Full Stack

The easiest way to start the entire environment is using the provided `Makefile`.

### 1. Unified Orchestration (Recommended)
You can start all components, including the background daemons, the Brain core, and the Web UI server, with a single command:

```bash
make run-web
```

*Note: This will spawn the background services, initialize the Brain, and start a Python-based HTTP server for the UI.*

### 2. Manual Component Launch
If you need to debug specific components, you can start them individually in separate terminals.

#### Step 1: Auxiliary Daemons
```bash
cargo run --bin tos-settingsd
cargo run --bin tos-loggerd
cargo run --bin tos-marketplaced
cargo run --bin tos-priorityd
cargo run --bin tos-sessiond
```

#### Step 2: The Brain Core
```bash
cargo run --bin tos-brain
```

#### Step 3: Web UI Face
```bash
python3 -m http.server 8080 -d web_ui
```

Access the interface at: `http://localhost:8080`

## Development Workflow

### Building & Checking
- **Build All**: `make build`
- **Fast Check**: `make check`
- **Linting**: `make lint`

### Testing Tier
TOS uses a multi-tier testing strategy:
1. **Logic Tests**: `make test-core` (State machine verification)
2. **Integration Tests**: `make test-shell` (PTY & OSC sequence handling)
3. **AI Tests**: `make test-ai` (Intent extraction and staging)
4. **UI Tests**: `make test-ui-component` (Playwright-based browser tests)
5. **Orchestration Health**: `make test-health` (Diagnostic reachability check)

### Package Management
Use the `tos-pkg` utility to interact with the marketplace via the CLI:
```bash
cargo run --bin tos-pkg -- discover ./modules/example
cargo run --bin tos-pkg -- verify ./modules/example
```

## Logs & Debugging
System logs are aggregated in the `logs/` directory:
- `logs/tos-brain.log`: Output from the core logic process.
- `logs/web_ui.log`: Access logs from the HTTP server.
- `logs/system_test.log`: Results from the comprehensive integration suite.

## Dynamic Port Management

**Every** TOS process — including the Brain — utilizes ephemeral port assignment by requesting Port 0 from the operating system. There are no hardcoded port numbers by default.

### Strategy

1. **Bind Port 0:** Each daemon (including the Brain) calls `bind()` with port 0. The OS assigns an available ephemeral port automatically.
2. **Registration:** On successful bind, the daemon writes the system-assigned port to `~/.local/share/tos/ports/<daemon-name>.port` (e.g., `tos-settingsd.port`, `tos-brain.port`, `tos-brain-ws.port`). This file contains a single integer.
3. **Anchor Override:** If `TOS_ANCHOR_PORT` is set, the Brain attempts that port first. If occupied, it falls back to Port 0 and logs a warning. See [Ecosystem Orchestration](./TOS_alpha-2_Ecosystem-Orchestration.md) for details.
4. **Discovery (Local):** The Brain reads all port files from `~/.local/share/tos/ports/` during initialization to build its service discovery map. If a port file is missing, the Brain retries with exponential backoff (100ms → 200ms → 400ms, max 3 retries).
5. **Discovery (Remote):** Remote clients find the Brain via pinned anchor port (`TOS_ANCHOR_PORT`) or mDNS (`_tos-brain._tcp`), then send `get_port_map` to get the full service map.
6. **CLI:** `tos ports` reads port files and displays a live table with reachability status. `tos ports --json` for machine output. `tos ports --remote <host>` to query a remote Brain.
7. **Health Check:** `make test-health` reads the port files and verifies TCP reachability for each registered daemon.
8. **Cleanup:** Port files are deleted on graceful shutdown. Stale port files (from crashes) are detected by the Brain via a TCP connect probe; unreachable ports trigger a re-discovery cycle.

### Example Flow

```
tos-settingsd starts → binds 0.0.0.0:0 → OS assigns port 49152
                     → writes "49152" to ~/.local/share/tos/ports/tos-settingsd.port
tos-brain starts     → binds 0.0.0.0:0 (TCP) → OS assigns port 49300
                     → binds 0.0.0.0:0 (WS)  → OS assigns port 52314
                     → writes "49300" to tos-brain.port, "52314" to tos-brain-ws.port
                     → reads tos-settingsd.port → connects to 127.0.0.1:49152
local Face           → reads tos-brain.port → connects to 127.0.0.1:49300
remote Face          → avahi-browse _tos-brain._tcp → finds 192.168.1.5:49300
                     → sends get_port_map → receives full service map
```

This ensures TOS starts reliably regardless of what other services are running on the host, and remote clients can discover the full constellation without any hardcoded ports.
