# TOS Alpha-2 Ecosystem Orchestration

This document details the service-oriented architecture (SOA) of the **TOS** (**Terminal On Steroids**) Alpha-2 environment and the orchestration protocols used for system-wide boot synchronization.

## Architecture Overview

TOS is designed as a constellation of independent processes communicating over a local bus (TCP/WebSocket). This decoupled approach ensures that AI failures, log overflows, or marketplace timeouts do not compromise the core Brain logic.

### 1. The Core Constellation
- **`tos-brain`**: The central authority. It manages the global state, hierarchical navigation (Levels 1-4), and coordinates the IPC flow between all other components. Like all TOS services, the Brain binds an ephemeral port (Port 0) and registers it via a port file.
- **`tos-face`**: The visual layer (Web UI or native client). It discovers the Brain's ports via the local port file (local) or via the discovery protocol (remote), then establishes a persistent WebSocket for high-frequency state synchronization.

### 2. Auxiliary Daemons (§4)
Daemons are specialized services that extend the Brain's capabilities. They are launched as background processes and bind ephemeral ports (Port 0):

- **Settings Service (`tos-settingsd`)**: Handles the persistence of global and sector-specific configurations.
- **Log Service (`tos-loggerd`)**: Aggregates structured JSONL logs from all system components.
- **Marketplace Service (`tos-marketplaced`)**: Performs cryptographic verification of installable modules.
- **Priority Service (`tos-priorityd`)**: Calculates tactical priority scores based on system telemetry.
- **Session Service (`tos-sessiond`)**: Handles session persistence and workspace memory (live state auto-save, named sessions).

## Orchestration Flow

The boot sequence is managed by the `Makefile` using the following hierarchy:

### Phase 1: Service Initialization (`make run-services`)
The system first spawns the auxiliary daemons in the background. Each daemon has its output redirected to `logs/*.log` to keep the primary console focused on Brain telemetry.
1. `mkdir -p logs ~/.local/share/tos/ports`
2. Cleanup of lingering processes (`pkill`) and stale port files.
3. Background launch of all 5 daemons. Each writes its assigned port to `~/.local/share/tos/ports/<name>.port`.

### Phase 2: Face Orchestration (`make run-web`)
Once services are initialized, the UI server and Brain core are launched.
1. Spawns `python3 -m http.server` to serve the Web UI (ephemeral port, registered via port file).
2. Initializes the `tos-brain` (ephemeral port, or pinned via `TOS_ANCHOR_PORT`).
3. The Brain reads all `~/.local/share/tos/ports/*.port` files to build its service discovery map.
4. Sets up a signal trap (INT/TERM) to ensure that killing the Brain also terminates the UI server and backgrounded services.

## Manual Status Check

To verify the health of the constellation, check the following:

- **Port Map**: `tos ports` — displays the live port assignments for all running daemons (reads `~/.local/share/tos/ports/`).
- **Process Status**: `ps aux | grep tos-`
- **Socket Awareness**: `ss -ltnp | grep tos-` — shows all TOS listeners regardless of port number.
- **Tactical Health Check**: `make test-health` (Automated reachability verification using port files).
- **Orchestration Logs**:
  - `logs/settingsd.log`
  - `logs/loggerd.log`
  - `logs/marketplaced.log`
  - `logs/priorityd.log`
  - `logs/sessiond.log`
  - `logs/tos-brain.log`

## Communication Protocol

- **Brain-to-Daemon**: The Brain initiates tactical queries to daemons as needed (e.g., fetching a setting or verifying a signature), using the port map built from port files.
- **Face-to-Brain**: Local Faces read `tos-brain.port` directly; remote Faces use the discovery protocol (see below). Once connected, the Face establishes a persistent WebSocket for state synchronization on the Brain's ephemeral WS port.

## Dynamic Port Management

**Every** TOS process — including the Brain — utilizes ephemeral port assignment by requesting Port 0 from the operating system. Upon successful binding, each daemon writes the system-assigned port to `~/.local/share/tos/ports/<daemon-name>.port`. There are **no hardcoded port numbers** in the system by default. See the [Development Guide](./TOS_alpha-2_Development-Guide.md) for implementation details.

### Anchor Port Override (`TOS_ANCHOR_PORT`)

For environments where a predictable Brain port is required (remote connections without mDNS, firewall rules, CI), the Brain's TCP port can be pinned using the `TOS_ANCHOR_PORT` environment variable:

```bash
# Pin the Brain to port 7000 for remote access
TOS_ANCHOR_PORT=7000 make run-web

# Or export it system-wide
export TOS_ANCHOR_PORT=7000
```

**Behavior:**
1. If `TOS_ANCHOR_PORT` is set, the Brain attempts to bind that port.
2. If the port is **available** → binds it, writes to `tos-brain.port`, proceeds normally.
3. If the port is **occupied** → the Brain falls back to Port 0, writes the actual ephemeral port to `tos-brain.port`, and logs a warning:
   ```
   WARN: Anchor port 7000 occupied, fell back to ephemeral port 49312.
         Remote clients using port 7000 will not be able to connect.
         Set TOS_ANCHOR_PORT to an available port, or use mDNS discovery.
   ```
4. If `TOS_ANCHOR_PORT` is **unset** → the Brain uses Port 0 like every other daemon (the default).

## Port Discovery for Remote Connections

Remote clients (native Faces, Web Portals, Horizon OS clients) need to discover the Brain's service endpoints without access to local port files. Three mechanisms are provided, in order of preference:

### Method 1: Anchor Port Query (Simple)

If `TOS_ANCHOR_PORT` is configured, remote clients can connect directly:

1. **Bootstrap**: The remote client connects to the Brain's pinned port (`<host>:<TOS_ANCHOR_PORT>` TCP).
2. **Query**: The client sends `get_port_map` over the TCP connection.
3. **Response**: The Brain replies with a JSON object mapping service names to their ephemeral ports:

```json
{
  "type": "port_map",
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

4. **Connect**: The remote client uses the returned ports to establish WebSocket and service connections.
5. **Re-discover**: If a connection drops, the client reconnects to the anchor port and re-queries the map, since daemons may have restarted on different ports.

### Method 2: mDNS / DNS-SD (Zero-Config LAN)

For fully ephemeral setups where no anchor port is pinned, the Brain advertises itself on the local network using mDNS (Avahi on Linux):

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

This enables zero-configuration remote Faces — plug in a Meta Quest or another desktop, and it finds the Brain automatically without knowing any port number.

### Method 3: Manual Entry (Ultimate Fallback)

If both mDNS and anchor port are unavailable (e.g., cross-subnet connections, restrictive firewalls, WAN tunnels), the user can manually specify the Brain's host and port.

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

**How it works:** The manual host:port connects directly to the Brain's TCP socket at the specified address. From there, the client sends `get_port_map` to discover all other services, exactly like Method 1. The user obtains the port from the host machine via `tos ports` (run locally or over SSH).

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
| 2 | **Anchor Port** (`TOS_ANCHOR_PORT`) | Known fixed port, cross-subnet |
| 3 | **Saved Hosts** | Reconnecting to a previously used Brain |
| 4 | **Manual Entry** | New host, no mDNS, no anchor port |

### `tos ports` CLI

For local inspection and scripting, the `tos ports` command reads the port files and outputs a formatted table:

```bash
$ tos ports
SERVICE            PORT    STATUS
tos-brain (tcp)    49312   ✓ reachable
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
- `tos ports --remote <host>[:<port>]` — queries a remote Brain's port map via `get_port_map` IPC. If port is omitted, attempts mDNS discovery first.
