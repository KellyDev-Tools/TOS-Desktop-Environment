# TOS Electron Platform — Developer Guide

## Overview

The TOS Electron platform provides a cross-platform desktop application shell for Windows, macOS, and Linux. It wraps the existing Svelte UI (the "Face") in an Electron container, connecting to the Brain via WebSocket IPC — identical to the web-based approach but with native OS integration.

### Architecture

```
┌──────────────────────────────────────────────────────────┐
│  Electron Main Process                                    │
│  ┌─────────────┐  ┌──────────┐  ┌───────────────────┐   │
│  │ Window Mgmt  │  │  Tray    │  │  Protocol Handler │   │
│  │ State Persist│  │  Menu    │  │  tos://            │   │
│  └─────────────┘  └──────────┘  └───────────────────┘   │
│          │                                                │
│  ┌───────┴────────────────────────────────────────────┐  │
│  │  Preload Script (contextBridge)                     │  │
│  │  window.tosElectron API                             │  │
│  └────────────────────────┬───────────────────────────┘  │
│                           │                               │
│  ┌────────────────────────┴───────────────────────────┐  │
│  │  Svelte Renderer (svelte_ui/build)                  │  │
│  │  ┌──────────────────────────────────────┐           │  │
│  │  │  WebSocket IPC → Brain (port 7001)   │           │  │
│  │  └──────────────────────────────────────┘           │  │
│  └────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

---

## Quick Start

### Prerequisites

- **Node.js 20+** (via NVM recommended)
- **Rust toolchain** (for the Brain)
- **Built Svelte UI** (`make build-web`)

### Development Mode

```bash
# From alpha-2 root:
make electron-dev

# Or directly:
cd src/platform/electron
npm install
npm run dev
```

This launches Electron pointed at the Svelte dev server on `localhost:8080`. DevTools open automatically.

### Production Build

```bash
# Build for current platform:
make electron-build

# Package for specific platforms:
make build-electron-win     # Windows (NSIS installer + portable)
make build-electron-macos   # macOS (DMG + zip)
make build-electron-linux   # Linux (AppImage + deb + rpm)
make build-electron-all     # All platforms
```

Packaged output goes to `src/platform/electron/release/`.

---

## Project Structure

```
src/platform/electron/
├── package.json              # Dependencies and scripts
├── tsconfig.json             # TypeScript configuration
├── electron-builder.json     # Packaging configuration
├── .gitignore
├── src/
│   ├── main.ts               # Entry point — window, tray, IPC, lifecycle
│   ├── preload.ts            # Secure renderer bridge (contextBridge)
│   ├── index.html            # Boot loader shell (fallback)
│   ├── platform-menu.ts      # Platform-specific menus (macOS/Win/Linux)
│   ├── window-state-manager.ts  # Position/size persistence
│   ├── auto-updater.ts       # Update checking and installation
│   ├── file-dialog-handler.ts   # Native file open/save dialogs
│   ├── print-handler.ts      # Native print dialog
│   └── protocol-handler.ts   # tos:// deep-link handling
├── resources/
│   └── entitlements.mac.plist   # macOS hardened runtime
├── tests/
│   ├── electron_window_management.test.ts
│   ├── electron_ipc_bridge.test.ts
│   ├── electron_platform_menu.test.ts
│   ├── electron_auto_update.test.ts
│   ├── electron_file_dialog.test.ts
│   ├── electron_tray.test.ts
│   ├── electron_protocol_handler.test.ts
│   └── electron_window_state.test.ts
├── dist/                     # Compiled JS (gitignored)
└── release/                  # Packaged apps (gitignored)
```

---

## Preload API Reference

The Svelte renderer accesses native features through `window.tosElectron`:

| Method | Returns | Description |
|--------|---------|-------------|
| `getPlatform()` | `Promise<{platform, arch, version, isDevMode}>` | Platform info |
| `getBrainUrl()` | `Promise<string>` | Brain WebSocket URL |
| `windowMinimize()` | `void` | Minimize window |
| `windowMaximize()` | `void` | Toggle maximize |
| `windowClose()` | `void` | Close window |
| `setTitle(title)` | `void` | Update window title |
| `setBadge(count)` | `void` | Set dock badge (macOS) |
| `showOpenDialog(opts)` | `Promise<{canceled, filePaths}>` | Native file picker |
| `showSaveDialog(opts)` | `Promise<{canceled, filePath}>` | Native save dialog |
| `printPreview()` | `Promise<void>` | System print dialog |
| `checkForUpdates()` | `Promise<boolean>` | Check for app updates |
| `installUpdate()` | `void` | Install and restart |
| `onNavigate(cb)` | `() => void` | Listen for menu/protocol navigation |
| `onUpdateAvailable(cb)` | `() => void` | Listen for update availability |
| `onUpdateDownloaded(cb)` | `() => void` | Listen for download completion |

### Usage in Svelte

```typescript
// Check if running in Electron
const isElectron = typeof window !== 'undefined' && 'tosElectron' in window;

if (isElectron) {
    const platform = await window.tosElectron.getPlatform();
    console.log(`Running on ${platform.platform} ${platform.arch}`);

    // Override IPC URL from Electron
    const brainUrl = await window.tosElectron.getBrainUrl();
}
```

---

## Custom Protocol

TOS registers the `tos://` deep-link protocol for all platforms:

| URL | Action |
|-----|--------|
| `tos://sector/3` | Navigate to sector 3 |
| `tos://settings` | Open settings |
| `tos://hub` | Navigate to Command Hub |
| `tos://global` | Navigate to Global Overview |
| `tos://module/terminal` | Open terminal module |

### Registering on Each Platform

- **Windows**: Registered in the registry by NSIS installer
- **macOS**: Registered via `Info.plist` / `LSItemContentTypes`
- **Linux**: Registered via `.desktop` file `MimeType` entry

---

## Platform-Specific Behavior

### macOS
- Hidden inset title bar with traffic light controls at (12, 12)
- Full AppKit menu bar (TOS, File, Edit, View, Window, Help)
- `Cmd+,` opens preferences, `Cmd+N` new sector, `Cmd+1/2` navigation
- Dock badge support via `setBadge()`
- App stays running when last window is closed (re-created on dock click)
- DMG packaging with drag-to-Applications layout
- Hardened runtime with notarization support

### Windows
- Standard title bar (auto-hide menu bar)
- `Ctrl+,` settings, `Ctrl+N` new sector, `Ctrl+1/2` navigation
- NSIS installer with start menu/desktop shortcuts
- Portable `.exe` option available
- Custom `tos://` protocol via registry

### Linux
- Standard title bar (auto-hide menu bar)
- GTK menu (same structure as Windows)
- AppImage (universal), `.deb` (Debian/Ubuntu), `.rpm` (Fedora/RHEL)
- System tray support

---

## Auto-Update

The auto-updater uses `electron-updater` with generic provider publishing:

1. **Check schedule**: On launch (10s delay) + every 4 hours
2. **Download**: Automatic after detection, progress reported to renderer
3. **Install**: User-triggered via `installUpdate()` (quit-and-install)

### Publish Configuration

Update releases are fetched from the URL configured in `electron-builder.json`:

```json
"publish": {
    "provider": "generic",
    "url": "https://updates.tos.dev/releases"
}
```

---

## Security

- **Context Isolation**: `true` — renderer cannot access Node.js
- **Node Integration**: `false` — no `require()` in renderer
- **Sandbox**: `true` — renderer runs in sandboxed process
- **Web Security**: `true` — enforces same-origin policy
- **CSP**: Strict Content-Security-Policy in HTML
- **Window Open Handler**: External links open in system browser
- **Preload**: Only whitelisted IPC channels are exposed

---

## Testing

```bash
cd src/platform/electron

# Run all tests
npm test

# Watch mode
npm run test:watch
```

**Test coverage** (105 tests across 8 files):

| File | Tests | Covers |
|------|-------|--------|
| `electron_window_management.test.ts` | 16 | Window creation, restoration, closure, state, platform config |
| `electron_ipc_bridge.test.ts` | 16 | Channel registration, Brain URL, window controls, dialogs, preload API |
| `electron_platform_menu.test.ts` | 15 | macOS/Windows/Linux menu structure and shortcuts |
| `electron_auto_update.test.ts` | 9 | Update checking, download, install, configuration |
| `electron_file_dialog.test.ts` | 10 | Open/save dialogs, filters, multi-select, cancel |
| `electron_tray.test.ts` | 9 | Tray creation, context menu, click behavior |
| `electron_protocol_handler.test.ts` | 16 | All `tos://` actions, edge cases, malformed URLs |
| `electron_window_state.test.ts` | 14 | State persistence, display validation, key naming |

---

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `TOS_BRAIN_WS` | `ws://127.0.0.1:7001` | Brain WebSocket URL |

---

## Troubleshooting

### "Renderer build not found"
Ensure the Svelte UI is built first: `make build-web`

### "Brain connection failed"
Ensure the Brain is running: `make run`

### "Electron won't launch on WSL"
Electron requires a display server. Use `DISPLAY=:0` with an X server, or run natively on Windows/macOS.

### "App icon missing"
Place `icon.png` in `src/platform/electron/resources/`. For Windows, also provide `icon.ico`. For macOS, `icon.icns`.
