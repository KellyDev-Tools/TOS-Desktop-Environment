# TOS Electron Face Container (`face-electron-any`)

This container houses the Electron-based desktop Face for the **TOS Desktop Environment**. It encapsulates the Svelte-based Web UI renderer and bridges it with local or remote TOS Brain processes.

## Architecture

```
Electron Main Process ── preload ──► Svelte Renderer ── WebSocket ──► TOS Brain (7001)
                └── native APIs (tray, menus, dialogs, protocols)
```

- **Electron Main (`src/main.ts`)**: Manages window lifecycle, system tray, platform menus, and IPC bridges.
- **IPC Bridge**: Relays Svelte window operations (maximize, minimize, close), system-level dialogs, and auto-update processes.
- **Brain Discovery (`src/brain-discovery.ts`)**: Scans for active TOS Brain instances via UDP mDNS/DNS-SD, environment variables, saved configuration files, or local socket probing.

---

## Tooling & Command Guide

### 1. Installation

Ensure Node.js v20+ is installed, then install dependencies:
```bash
npm install
```

### 2. Development

Compile TS files and launch the Electron application in **development mode** (detached DevTools and local server fallbacks):
```bash
npm run dev
```

### 3. Production Boot

Compile and launch in standard mode:
```bash
npm run start
```

### 4. Running Tests

Run the full Vitest suite (includes window management, IPC bridge, tray, protocol handling, and mDNS discovery tests):
```bash
npm run test
```

---

## Connecting to a Remote Brain (e.g., Linux Host)

By default, the Electron app connects to the Brain running locally (`ws://127.0.0.1:7001`). To connect it to a remote Brain (like a Linux server on the LAN):

### Option A: Using the Remote PowerShell Script (Recommended)

Run the included PowerShell helper to probe the host, check Svelte UI build assets, set the connection environment variable, and boot Electron in one command:

```powershell
# Connect to Linux host (192.168.68.77) in Dev mode
.\run-face-electron-remote.ps1 -BrainHost "192.168.68.77" -BrainPort 7001

# Connect in standard Production mode
.\run-face-electron-remote.ps1 -BrainHost "192.168.68.77" -BrainPort 7001 -DevMode $false
```

### Option B: Manual Environment Variable

Set the `TOS_BRAIN_WS` environment variable before running the launch commands:

**Windows (PowerShell):**
```powershell
$env:TOS_BRAIN_WS="ws://192.168.68.77:7001"
npm run dev
```

**Windows (CMD):**
```cmd
set TOS_BRAIN_WS=ws://192.168.68.77:7001
npm run dev
```

**Linux / macOS:**
```bash
export TOS_BRAIN_WS="ws://192.168.68.77:7001"
npm run dev
```

### Option C: Remote Hosts File

Add the remote host under the TOS configs path (`~/.config/tos/remote-hosts.toml` or `C:\Users\<user>\.config\tos\remote-hosts.toml`):

```toml
[[remote]]
name = "Linux Brain Server"
host = "192.168.68.77"
port = 7001
```

The Electron app's **Brain Discovery Service** will automatically load, TCP-probe, and present this host in the connection selector.
