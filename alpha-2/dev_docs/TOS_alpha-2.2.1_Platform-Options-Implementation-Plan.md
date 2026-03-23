# Implementation Plan

## [Overview]

Implement Electron-based cross-platform Face containers for Windows, macOS, and Linux as Phase 5 Priority 1 in Alpha-2.2.1, alongside existing Wayland (Linux), OpenXR (VR/AR), and Android options.

This implementation adds Electron as a first-class platform option to satisfy the requirement for Windows/macOS support by Beta-0. Electron provides the fastest path to multiplatform with a single Svelte codebase, existing WebSocket IPC protocol support, and browser DevTools for rapid debugging.

**Scope:** Electron wrapper with platform-native shells, shared Svelte UI renderer, WebSocket IPC bridge to Brain, auto-update integration, and packaging for Windows/macOS/Linux.

---

## [Types]

No type system changes required. This implementation leverages existing IPC message types from `tos-protocol` and adds Electron-specific window management types.

### Electron-specific IPC Types (new)

| Type Name | Fields | Purpose |
|----------|--------|---------|
| `ElectronWindowCreate` | `width: number`, `height: number`, `title: string`, `useIncognito: boolean`, `isMenuVisible: boolean`, `isMenuBarVisible: boolean` | Create Face window with platform-specific menu configuration |
| `ElectronWindowRestore` | `windowId: string` | Restore previously minimized window |
| `ElectronWindowClose` | `windowId: string` | Close Face window |
| `ElectronTrayShow` | `visible: boolean` | Toggle tray icon visibility |
| `ElectronTrayContextMenu` | `x: number`, `y: number`, `menuItems: string[]` | Show tray context menu |
| `ElectronFilePicker` | `title: string`, `filters: string[]` | Platform-specific file dialog |
| `ElectronPrintPreview` | `webContentsId: number` | Show platform-native print preview |

### Platform Configuration Types

| Type Name | Fields | Purpose |
|----------|--------|---------|
| `PlatformConfig` | `platform: 'win32' | 'darwin' | 'linux'`, `rendererBuildPath: string`, `useNativeMenuBar: boolean`, `enableAutoUpdater: boolean` | Platform-specific configuration |
| `ElectronSettings` | `titleBarStyle: string`, `showTrafficLights: boolean`, `backgroundColor: string` | Electron settings for platform customization |

---

## [Files]

### New Files to Create

| Path | Purpose |
|------|---------|
| `alpha-2/src/platform/electron/main.ts` | Electron main process: window management, IPC bridge, tray handling |
| `alpha-2/src/platform/electron/preload.ts` | Preload script: secure IPC between main and renderer |
| `alpha-2/src/platform/electron/index.html` | Electron host window template |
| `alpha-2/src/platform/electron/platform-menu-win.ts` | Windows native menu configuration |
| `alpha-2/src/platform/electron/platform-menu-macos.ts` | macOS AppKit menu configuration |
| `alpha-2/src/platform/electron/platform-menu-linux.ts` | GTK/Linux menu configuration |
| `alpha-2/src/platform/electron/auto-updater.ts` | Auto-update logic using `electron-updater` |
| `alpha-2/src/platform/electron/window-state-manager.ts` | Window state persistence (minimize, maximize, position) |
| `alpha-2/src/platform/electron/file-dialog-handler.ts` | Platform-specific file dialogs |
| `alpha-2/src/platform/electron/print-handler.ts` | Platform-specific print preview |
| `alpha-2/src/platform/electron/protocol-handler.ts` | Custom protocol handler for `tos://` |
| `alpha-2/package.json` (update) | Add Electron and build dependencies |
| `alpha-2/electron-builder.json` (or `alpha-2/config/electron-builder.json`) | Electron-builder configuration for packaging |

### Existing Files to Modify

| Path | Changes |
|------|---------|
| `alpha-2/Cargo.toml` | Add `tokio-tungstenite` if not present (for WebSocket IPC) |
| `alpha-2/Makefile` | Add Electron build targets: `build-electron-win`, `build-electron-macos`, `build-electron-linux` |
| `alpha-2/svelte_ui/package.json` | Ensure build output is ready for Electron renderer load |
| `alpha-2/svelte_ui/src/main.ts` | Add preload script injection |
| `alpha-2/dev_docs/TOS_alpha-2.2_Production-Roadmap.md` | Update Phase 5 to include Electron priority |

### Files to Delete

None at this time.

---

## [Functions]

### New Functions

| Function Name | Signature | File Path | Purpose |
|--------------|-----------|-----------|---------|
| `createFaceWindow()` | `async (config: PlatformConfig): Promise<void>` | `main.ts` | Create platform-specific Face window |
| `restoreWindow()` | `async (windowId: string): Promise<void>` | `window-state-manager.ts` | Restore minimized window |
| `closeWindow()` | `async (windowId: string): Promise<void>` | `main.ts` | Close window |
| `handleWindowState()` | `async (): Promise<void>` | `main.ts` | Save/restore window state |
| `createTray()` | `async (): Promise<ToyApp> | undefined` | `main.ts` | Create platform tray icon |
| `showTrayMenu()` | `(x: number, y: number): Promise<void>` | `main.ts` | Show tray context menu |
| `showFilePicker()` | `(title: string, filters: string[]): Promise<string[]> | undefined` | `file-dialog-handler.ts` | Platform-specific file dialog |
| `showPrintPreview()` | `(webContentsId: number): Promise<void>` | `print-handler.ts` | Show print preview |
| `handleCustomProtocol()` | `(url: string): Promise<void>` | `protocol-handler.ts` | Handle `tos://` protocol |
| `checkForUpdates()` | `async (): Promise<boolean>` | `auto-updater.ts` | Check for updates |
| `installUpdate()` | `async (): Promise<void>` | `auto-updater.ts` | Install pending update |

### Modified Functions

None at this time.

---

## [Classes]

No class modifications required. This implementation uses Electron's built-in modules (`BrowserWindow`, ` Tray`, `ipcMain`) without creating custom classes.

---

## [Dependencies]

### New Dependencies (via `package.json`)

| Package | Purpose | Version |
|---------|---------|---------|
| `electron` | Main framework | `28.0.0` |
| `electron-updater` | Auto-update | `6.1.7` |
| `electron-store` | Settings persistence | `8.1.0` |
| `native-keymap` | Keyboard shortcuts | `3.1.5` |
| `winstore` | Windows app installation | `0.19.8` (optional) |
| `dmg` | macOS DMG creation | (optional) |

### Platform-Specific Dependencies

| Platform | Additional Packages |
|----------|---------------------|
| Windows | `win-store` for app installation |
| macOS | Entitlements file for notarization |
| Linux | GTK dependencies via system packages |

---

## [Testing]

### Test Files to Create

| Path | Purpose |
|------|---------|
| `alpha-2/tests/electron_window_management.ts` | Window creation, restoration, closure |
| `alpha-2/tests/electron_ipc_bridge.ts` | IPC message passing between main and renderer |
| `alpha-2/tests/electron_platform_menu.ts` | Platform-specific menu configuration |
| `alpha-2/tests/electron_auto_update.ts` | Update checking and installation |
| `alpha-2/tests/electron_file_dialog.ts` | File dialog behavior |
| `alpha-2/tests/electron_tray.ts` | Tray icon and context menu |
| `alpha-2/tests/electron_protocol_handler.ts` | Custom protocol handler |
| `alpha-2/tests/electron_window_state.ts` | State persistence |

### Existing Test Modifications

- `alpha-2/tests/app_model.ts`: Add Electron renderer state tests
- `alpha-2/tests/shell_api.ts`: Add Electron IPC message tests

---

## [Implementation Order]

1. **Phase 5.1 — Setup Electron Environment**
   - Add `electron` and build dependencies to `package.json`
   - Create `electron-builder.json` configuration
   - Create `main.ts` skeleton

2. **Phase 5.2 — Implement Main Process**
   - Create `createFaceWindow()` function
   - Implement `handleWindowState()` for persistence
   - Create tray icon handling
   - Set up IPC bridge to WebSocket brain client

3. **Phase 5.3 — Implement Platform-Specific Menus**
   - `platform-menu-win.ts`: WinUI 3 style menu
   - `platform-menu-macos.ts`: AppKit menu with dock integration
   - `platform-menu-linux.ts`: GTK menu

4. **Phase 5.4 — Implement File Dialogs and Print**
   - Platform-specific file dialogs
   - Print preview handlers

5. **Phase 5.5 — Implement Auto-Update**
   - Update checking logic
   - Installation handling

6. **Phase 5.6 — Create Renderer Integration**
   - Inject preload script into Svelte app
   - Share UI state model with web renderer
   - Load from `svelte_ui/build`

7. **Phase 5.7 — Implement Protocol Handler**
   - Handle `tos://` custom protocol
   - File activation

8. **Phase 5.8 — Build and Package**
   - Create build targets in `Makefile`
   - Package for Windows, macOS, Linux
   - Configure auto-update

9. **Phase 5.9 — Testing**
   - Run all Electron tests
   - Integration testing with Brain

10. **Phase 5.10 — Documentation**
    - Update user manual for Electron app usage
    - Document platform-specific features