# TOS Alpha-2 Development Guide

This guide describes how to start and orchestrate the various components of the **TOS** (**Terminal On Steroids**) environment for development and testing.

## System Components & Ports

TOS is a distributed system consisting of a central logic core (the Brain), a visual interface (the Face), and several auxiliary daemons. **Every** TOS process binds an ephemeral port (Port 0) by default — there are no hardcoded port numbers. All port information lives in the Brain's in-memory **Service Registry**; there are no port files on disk. For remote access, the Brain's TCP port can be pinned via `TOS_ANCHOR_PORT`.

| Component | Binary / Directory | Port | Protocol | Description |
| :--- | :--- | :--- | :--- | :--- |
| **Brain Core** | `tos-brain` | Ephemeral (or `TOS_ANCHOR_PORT`) | TCP | Main logic, IPC handler, & service registry |
| **Brain Socket** | `tos-brain` | — | Unix | Local registration & discovery (`brain.sock`) |
| **Brain UI Sync** | `tos-brain` | Ephemeral | WS | WebSocket for UI state synchronization |
| **Settings Daemon** | `tos-settingsd` | Ephemeral | TCP | Persistent configuration storage |
| **Log Daemon** | `tos-loggerd` | Ephemeral | TCP | Unified system logging |
| **Marketplace** | `tos-marketplaced`| Ephemeral | TCP | Module discovery & verification |
| **Priority Engine** | `tos-priorityd` | Ephemeral | TCP | Tactical priority scoring |
| **Session Service** | `tos-sessiond` | Ephemeral | TCP | Session persistence & workspace memory |
| **Web UI Server** | `web_ui/` | Ephemeral | HTTP | Serves the LCARS interface |

To view actual live port assignments, use `tos ports` (queries the Brain's registry). See [Ecosystem Orchestration](./TOS_alpha-2_Ecosystem-Orchestration.md) for the full registration and discovery protocol.

## Starting the Full Stack

The easiest way to start the entire environment is using the provided `Makefile`.

### 1. Unified Orchestration (Recommended)
You can start all components, including the background daemons, the Brain core, and the Web UI server, with a single command:

```bash
make run-web
```

*Note: This will spawn the background services, initialize the Brain, and start a Python-based HTTP server for the UI.*

### 2. Manual Component Launch
If you need to debug specific components, you can start them individually in separate terminals. **The Brain must start first** so that daemons can register with it.

#### Step 1: The Brain Core
```bash
cargo run --bin tos-brain
```

#### Step 2: Auxiliary Daemons (any order, after Brain)
```bash
cargo run --bin tos-settingsd
cargo run --bin tos-loggerd
cargo run --bin tos-marketplaced
cargo run --bin tos-priorityd
cargo run --bin tos-sessiond
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

**Every** TOS process — including the Brain — utilizes ephemeral port assignment by requesting Port 0 from the operating system. There are no hardcoded port numbers and **no port files on disk**. The Brain's in-memory service registry is the single source of truth.

### Strategy

1. **Brain starts first:** Creates a Unix domain socket at `$XDG_RUNTIME_DIR/tos/brain.sock`. Binds ephemeral TCP + WS ports (or `TOS_ANCHOR_PORT` for TCP). Advertises via mDNS if available.
2. **Daemons register:** Each daemon calls `bind(0)`, then connects to `brain.sock` and sends `{ "type": "register", "name": "<name>", "port": <port> }`. The Brain ACKs and adds it to the registry.
3. **Anchor Override:** If `TOS_ANCHOR_PORT` is set, the Brain attempts that port first. If occupied, falls back to Port 0 with a warning.
4. **Discovery (Local):** Local clients connect to `brain.sock` and send `get_port_map`.
5. **Discovery (Remote):** Remote clients find the Brain via mDNS (`_tos-brain._tcp`), anchor port, saved hosts, or manual host:port entry — then send `get_port_map` over TCP.
6. **CLI:** `tos ports` queries the Brain's registry and displays a live table. `tos ports --json` for machine output. `tos ports --remote <host>[:<port>]` to query a remote Brain.
7. **Health Check:** `make test-health` queries the Brain's registry and verifies TCP reachability for each registered service.
8. **Cleanup:** Daemons send `deregister` on graceful shutdown. The Brain also probes registered services periodically and marks unreachable ones as offline.

### Example Flow

```
tos-brain starts     → creates $XDG_RUNTIME_DIR/tos/brain.sock
                     → binds 0.0.0.0:0 (TCP) → OS assigns port 49300
                     → binds 0.0.0.0:0 (WS)  → OS assigns port 52314
                     → registers itself: brain_tcp=49300, brain_ws=52314
tos-settingsd starts → binds 0.0.0.0:0 → OS assigns port 49152
                     → connects to brain.sock → sends register(settingsd, 49152)
                     → Brain ACKs → settingsd now discoverable
local Face           → connects to brain.sock → sends get_port_map
                     → receives { brain_tcp: 49300, brain_ws: 52314, settingsd: 49152, ... }
remote Face          → avahi-browse _tos-brain._tcp → finds 192.168.1.5:49300
                     → connects to TCP → sends get_port_map → receives full service map
```

See [Ecosystem Orchestration](./TOS_alpha-2_Ecosystem-Orchestration.md) for the full registration protocol, health monitoring, and remote discovery details.

