# TOS Alpha-2 Development Guide

This guide describes how to start and orchestrate the various components of the TOS (Tactical Operating System) environment for development and testing.

## System Components & Ports

TOS is a distributed system consisting of a central logic core (the Brain), a visual interface (the Face), and several auxiliary daemons.

| Component | Binary / Directory | Port | Protocol | Description |
| :--- | :--- | :--- | :--- | :--- |
| **Brain Core** | `tos-brain` | 7000 | TCP | Main logic & IPC handler |
| **Brain UI Sync** | `tos-brain` | 7001 | WS | WebSocket for UI state synchronization |
| **Settings Daemon** | `tos-settingsd` | 7002 | TCP | Persistent configuration storage |
| **Log Daemon** | `tos-loggerd` | 7003 | TCP | Unified system logging |
| **Marketplace** | `tos-marketplaced`| 7004 | TCP | Module discovery & verification |
| **Priority Engine** | `tos-priorityd` | 7005 | TCP | Tactical priority scoring |
| **Web UI Server** | `web_ui/` | 8080 | HTTP | Serves the LCARS interface |

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
