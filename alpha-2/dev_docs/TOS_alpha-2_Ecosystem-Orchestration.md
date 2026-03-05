# TOS Alpha-2 Ecosystem Orchestration

This document details the service-oriented architecture (SOA) of the **TOS** (**Terminal On Steroids**) Alpha-2 environment and the orchestration protocols used for system-wide boot synchronization.

## Architecture Overview

TOS is designed as a constellation of independent processes communicating over a local bus (TCP/WebSocket). This decoupled approach ensures that AI failures, log overflows, or marketplace timeouts do not compromise the core Brain logic.

### 1. The Core Constellation
- **`tos-brain`**: The central authority. It manages global state, hierarchical navigation (Levels 1-4), coordinates IPC between all components, and maintains the **Service Registry** — the single source of truth for all daemon endpoints. The Brain exposes a local Unix domain socket for daemon registration and local discovery, an **always-on anchor TCP port** (default 7000, configurable in Settings) for remote access, and an ephemeral WebSocket port for state synchronization.
- **`tos-face`**: The visual layer (Web UI or native client). It discovers the Brain via the local socket (local) or via the anchor port / mDNS / manual entry (remote), then queries the Brain's service registry for the full port map.

### 2. Auxiliary Daemons (§4)
Daemons are specialized services that extend the Brain's capabilities. They are launched as background processes, bind ephemeral TCP ports (Port 0), and **register themselves with the Brain** on startup:

- **Settings Service (`tos-settingsd`)**: Handles the persistence of global and sector-specific configurations.
- **Log Service (`tos-loggerd`)**: Aggregates structured JSONL logs from all system components.
- **Marketplace Service (`tos-marketplaced`)**: Performs cryptographic verification of installable modules.
- **Priority Service (`tos-priorityd`)**: Calculates tactical priority scores based on system telemetry.
- **Session Service (`tos-sessiond`)**: Handles session persistence and workspace memory (live state auto-save, named sessions).

## Orchestration Flow

The boot sequence is managed by the `Makefile`. **The Brain starts first** so that daemons can register with it immediately.

### Phase 1: Brain Core (`make run-brain`)
1. `mkdir -p logs`
2. Cleanup of lingering processes (`pkill`) and stale socket files.
3. Launch `tos-brain`:
   - Creates the registration socket at `$XDG_RUNTIME_DIR/tos/brain.sock` (fallback: `~/.local/share/tos/brain.sock`).
   - Binds the **anchor port** (resolved from: `TOS_ANCHOR_PORT` env var → `tos.network.anchor_port` setting → default `7000`).
   - Binds an ephemeral WebSocket port for Face state synchronization.
   - Begins accepting daemon registrations and client queries immediately.
   - Advertises itself via mDNS (`_tos-brain._tcp`) if Avahi is available.
   - Writes the active anchor port value back to `tos.network.anchor_port` in the Settings Daemon (once `tos-settingsd` registers).

### Phase 2: Service Initialization (`make run-services`)
Each daemon starts in the background, binds its own ephemeral TCP port, and registers with the Brain:
1. Daemon calls `bind(0.0.0.0:0)` → OS assigns an ephemeral port.
2. Daemon connects to `brain.sock` and sends a registration message:
   ```json
   { "type": "register", "name": "settingsd", "port": 49152 }
   ```
3. Brain ACKs the registration and adds the daemon to its in-memory service registry.
4. Daemon is now discoverable by any client querying the Brain.

If the Brain socket is not yet available (race condition), the daemon retries with exponential backoff (100ms → 200ms → 400ms, max 5 retries). If registration fails, the daemon logs a fatal error and exits.

### Phase 3: Face Orchestration (`make run-web`)
1. Spawns `python3 -m http.server` to serve the Web UI (it also registers with the Brain).
2. Sets up a signal trap (INT/TERM) to ensure that killing the Brain also terminates the UI server and backgrounded services.

### Full Stack: `make run-web`
Runs all three phases in sequence: Brain → Services → Face.

## The Service Registry

The Brain maintains an in-memory service registry. There are **no port files on disk** — the Brain is the single source of truth.

### Registration Protocol (Unix Socket)

Daemons register over the local Unix domain socket at `$XDG_RUNTIME_DIR/tos/brain.sock`:

| Message | Direction | Effect |
| :--- | :--- | :--- |
| `{ "type": "register", "name": "<service>", "port": <port> }` | daemon → Brain | Registers a service endpoint |
| `{ "type": "ack", "name": "<service>" }` | Brain → daemon | Confirms registration |
| `{ "type": "deregister", "name": "<service>" }` | daemon → Brain | Removes service on graceful shutdown |
| `{ "type": "get_port_map" }` | client → Brain | Requests the full service map |
| `{ "type": "port_map", "services": { ... } }` | Brain → client | Returns all registered services |

### Service Map Response

When any client (local or remote) sends `get_port_map`, the Brain responds with:

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

### Health Monitoring

The Brain periodically probes registered services (TCP connect, 30s interval). If a service is unreachable for 3 consecutive probes, it is marked as `offline` in the registry (but not removed — the daemon may restart and re-register).

`make test-health` queries the Brain's registry and displays reachability status for all services.

## Manual Status Check

- **Port Map**: `tos ports` — queries the Brain's service registry and displays a formatted table.
- **Process Status**: `ps aux | grep tos-`
- **Socket Awareness**: `ss -ltnp | grep tos-` — shows all TOS listeners regardless of port number.
- **Tactical Health Check**: `make test-health` (queries Brain registry with reachability probes).
- **Orchestration Logs**:
  - `logs/settingsd.log`
  - `logs/loggerd.log`
  - `logs/marketplaced.log`
  - `logs/priorityd.log`
  - `logs/sessiond.log`
  - `logs/tos-brain.log`

## Communication Protocol

- **Brain-to-Daemon**: The Brain initiates tactical queries to daemons as needed (e.g., fetching a setting or verifying a signature), using the port map from its in-memory registry.
- **Face-to-Brain**: Local Faces connect to `brain.sock`; remote Faces connect to the Brain's TCP port. Once connected, the Face establishes a persistent WebSocket for state synchronization on the Brain's ephemeral WS port.

## Dynamic Port Management

Auxiliary daemons utilize ephemeral port assignment by requesting Port 0 from the operating system. The Brain's in-memory service registry is the single source of truth for all port information.

The Brain itself always binds **two ports**:
1. **Anchor Port** — a stable, predictable TCP port for remote access and IPC (default: `7000`).
2. **Ephemeral WS Port** — an OS-assigned WebSocket port for Face state synchronization.

### Anchor Port Resolution

The Brain **always** creates an anchor port. The value is resolved in priority order:

| Priority | Source | Example |
| :--- | :--- | :--- |
| 1 (highest) | `TOS_ANCHOR_PORT` environment variable | `TOS_ANCHOR_PORT=7777` |
| 2 | `tos.network.anchor_port` setting (persisted) | Set via **Settings → Network** |
| 3 (default) | Hardcoded fallback | `7000` |

**Behavior:**
1. The Brain attempts to bind the resolved anchor port.
2. If the port is **available** → binds it, registers in its own service map, writes the value back to `tos.network.anchor_port` in the Settings Daemon.
3. If the port is **occupied** → the Brain scans upward (port + 1, + 2, ..., capped at + 10). If a nearby port is found, binds it and updates `tos.network.anchor_port` with the actual value. Logs a notice:
   ```
   NOTICE: Anchor port 7000 occupied, bound to 7002 instead.
           Updated tos.network.anchor_port setting. View in Settings → Network.
   ```
4. If **all 10 fallback ports are occupied** → the Brain binds Port 0 (fully ephemeral) and logs a warning. Remote clients without mDNS will not be able to connect.

The env var override (`TOS_ANCHOR_PORT`) is intended for CI, Docker, and scripted environments. For interactive use, the **Settings → Network** panel is the primary configuration surface.

### Settings → Network

The Settings app displays the current anchor port under **Settings → Network**:

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

- Changing the **Remote Access Port** writes to `tos.network.anchor_port` and takes effect on next Brain restart.
- The **mDNS Advertisement** toggle controls whether the Brain broadcasts `_tos-brain._tcp`.
- **View Port Map** opens the full `tos ports` table inline.

## Port Discovery for Remote Connections

Remote clients (native Faces, Web Portals, Horizon OS clients) need to discover the Brain without access to the local Unix socket. Three mechanisms are provided, in order of preference:

### Method 1: Anchor Port (Default)

Since the Brain always binds an anchor port (default `7000`), remote clients can connect directly if they know the host:

1. **Connect**: The remote client connects to `<host>:<anchor_port>` TCP.
2. **Query**: The client sends `get_port_map` over the TCP connection.
3. **Response**: The Brain replies with the full service map (including the `anchor_port` field).
4. **Upgrade**: The client uses the returned `brain_ws` port to establish a WebSocket for state synchronization.
5. **Re-discover**: If a connection drops, the client reconnects to the anchor port and re-queries the map.

The anchor port is visible in **Settings → Network** on the host machine, and is included in the mDNS TXT records.

### Method 2: mDNS / DNS-SD (Zero-Config LAN)

The Brain advertises itself on the local network using mDNS (Avahi on Linux):

- **Service Type**: `_tos-brain._tcp`
- **TXT Records**: `brain_tcp=<port>`, `brain_ws=<port>`
- **Name**: `TOS-<hostname>`

Remote clients on the same LAN can discover the Brain automatically:

```bash
# Discover TOS instances on the LAN
avahi-browse -rt _tos-brain._tcp

# Or from the tos CLI
tos discover
```

### Method 2: Manual Entry (Fallback)

If mDNS is unavailable (e.g., cross-subnet, WAN tunnels, restrictive firewalls), the user can manually specify the Brain's host and port. The default anchor port (`7000`) means users often only need to enter the host — the port is predictable.

**UI:** Remote Face clients present a connection dialog on startup:

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

The dialog simultaneously runs mDNS discovery, showing any auto-detected instances above the manual fields. The user can either tap a discovered instance or type a host and port directly.

**CLI:** Remote Faces also accept connection details via command-line arguments or environment variables:

```bash
# Direct arguments
tos-face --host 10.0.0.42 --port 49300

# Environment variables
TOS_REMOTE_HOST=10.0.0.42 TOS_REMOTE_PORT=49300 tos-face
```

**How it works:** The manual host:port connects directly to the Brain's TCP socket. From there, the client sends `get_port_map` to discover all other services. The user obtains the port from the host machine via `tos ports` (run locally or over SSH).

**Saved connections:** Successfully connected remote hosts are saved to `~/.config/tos/remote-hosts.toml` for quick reconnection:

```toml
[[hosts]]
name = "Workstation"
host = "10.0.0.42"
port = 49300          # last-known Brain TCP port
last_connected = "2026-03-04T18:00:00Z"
```

On next launch, saved hosts are probed automatically alongside mDNS results. If the last-known port is stale, the entry is shown as "offline" and the user can update the port manually.

### Discovery Priority

Remote clients attempt discovery in this order:

| Priority | Method | When to use |
| :--- | :--- | :--- |
| 1 | **mDNS** | Same LAN, zero config |
| 2 | **Anchor Port** (default `7000`) | Cross-subnet, known host — port is always available |
| 3 | **Saved Hosts** | Reconnecting to a previously used Brain |
| 4 | **Manual Entry** | New host, no mDNS — enter host (port defaults to 7000) |

### `tos ports` CLI

For local and remote inspection, the `tos ports` command queries the Brain's service registry:

```bash
$ tos ports
SERVICE            PORT    STATUS
tos-brain (anchor)    7000   ✓ reachable
tos-brain (ws)     52314   ✓ reachable
tos-settingsd      49152   ✓ reachable
tos-loggerd        49153   ✓ reachable
tos-marketplaced   49155   ✓ reachable
tos-priorityd      49157   ✓ reachable
tos-sessiond       49160   ✓ reachable
web_ui             49161   ✓ reachable
```

Flags:
- `tos ports --json` — machine-readable JSON output (same format as `get_port_map` response).
- `tos ports --wait` — blocks until all expected services are registered and reachable (useful in scripts and CI).
- `tos ports --remote <host>[:<port>]` — queries a remote Brain's port map. If port is omitted, attempts mDNS discovery first.
