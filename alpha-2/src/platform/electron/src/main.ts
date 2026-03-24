/**
 * TOS Electron Main Process
 * ─────────────────────────────────────────────────────────────────────────────
 * Primary entry point for the Electron Face container.
 * Manages window lifecycle, IPC bridge to the Brain (via WebSocket),
 * tray icon, platform-specific menus, and protocol handling.
 *
 * Architecture:
 *   Electron Main ── preload ──► Svelte Renderer ── WebSocket ──► Brain (7001)
 *                └── native APIs (tray, menus, dialogs, protocol)
 */

import { app, BrowserWindow, Tray, Menu, nativeImage, ipcMain, session, shell } from 'electron';
import * as path from 'path';
import * as fs from 'fs';

import { WindowStateManager } from './window-state-manager';
import { setupAutoUpdater } from './auto-updater';
import { createPlatformMenu } from './platform-menu';
import { registerFileDialogHandlers } from './file-dialog-handler';
import { registerPrintHandlers } from './print-handler';
import { registerProtocolHandler } from './protocol-handler';

// ─────────────────────────────────────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────────────────────────────────────

const IS_DEV = process.argv.includes('--dev');
const BRAIN_WS_URL = process.env.TOS_BRAIN_WS ?? 'ws://127.0.0.1:7001';

/** Path to the prebuilt Svelte UI renderer */
function getRendererPath(): string {
    if (IS_DEV) {
        // In dev mode, point to the svelte_ui build output
        return path.resolve(__dirname, '..', '..', '..', 'svelte_ui', 'build');
    }
    // In production, the renderer is bundled as an extra resource
    return path.join(process.resourcesPath, 'renderer');
}

// ─────────────────────────────────────────────────────────────────────────────
// Application State
// ─────────────────────────────────────────────────────────────────────────────

let mainWindow: BrowserWindow | null = null;
let tray: Tray | null = null;
let windowStateManager: WindowStateManager | null = null;

// ─────────────────────────────────────────────────────────────────────────────
// Platform Configuration
// ─────────────────────────────────────────────────────────────────────────────

export interface PlatformConfig {
    platform: 'win32' | 'darwin' | 'linux';
    rendererBuildPath: string;
    useNativeMenuBar: boolean;
    enableAutoUpdater: boolean;
}

function getPlatformConfig(): PlatformConfig {
    const platform = process.platform as 'win32' | 'darwin' | 'linux';
    return {
        platform,
        rendererBuildPath: getRendererPath(),
        useNativeMenuBar: platform === 'darwin',
        enableAutoUpdater: !IS_DEV,
    };
}

// ─────────────────────────────────────────────────────────────────────────────
// Window Management
// ─────────────────────────────────────────────────────────────────────────────

export async function createFaceWindow(config: PlatformConfig): Promise<void> {
    windowStateManager = new WindowStateManager('main-window');
    const savedState = windowStateManager.getState();

    const windowOptions: Electron.BrowserWindowConstructorOptions = {
        width: savedState.width ?? 1280,
        height: savedState.height ?? 800,
        x: savedState.x,
        y: savedState.y,
        minWidth: 800,
        minHeight: 600,
        show: false,                    // Show after ready-to-show
        backgroundColor: '#0a0a0f',    // TOS dark background
        autoHideMenuBar: config.platform !== 'darwin',
        titleBarStyle: config.platform === 'darwin' ? 'hiddenInset' : 'default',
        trafficLightPosition: config.platform === 'darwin' ? { x: 12, y: 12 } : undefined,
        webPreferences: {
            preload: path.join(__dirname, 'preload.js'),
            contextIsolation: true,
            nodeIntegration: false,
            sandbox: true,
            webSecurity: true,
            allowRunningInsecureContent: false,
        },
        icon: getAppIcon(),
    };

    mainWindow = new BrowserWindow(windowOptions);

    // Restore maximized state
    if (savedState.isMaximized) {
        mainWindow.maximize();
    }

    // Setup window state tracking
    windowStateManager.track(mainWindow);

    // Load the Svelte UI renderer
    const indexPath = path.join(config.rendererBuildPath, 'index.html');

    if (fs.existsSync(indexPath)) {
        await mainWindow.loadFile(indexPath);
    } else if (IS_DEV) {
        // In dev, try loading from the Svelte dev server directly
        console.log('[Electron] Loading from Svelte dev server...');
        await mainWindow.loadURL('http://localhost:8080');
    } else {
        console.error('[Electron] ❌ Renderer build not found at:', indexPath);
        app.quit();
        return;
    }

    // Show window once content is ready (avoids flash of white)
    mainWindow.once('ready-to-show', () => {
        mainWindow?.show();
        if (IS_DEV) {
            mainWindow?.webContents.openDevTools({ mode: 'detach' });
        }
    });

    // Handle external links — open in system browser
    mainWindow.webContents.setWindowOpenHandler(({ url }) => {
        shell.openExternal(url);
        return { action: 'deny' };
    });

    // Window close behavior
    mainWindow.on('closed', () => {
        mainWindow = null;
    });

    console.log('[Electron] ✅ Face window created');
}

export async function closeWindow(windowId?: string): Promise<void> {
    if (mainWindow) {
        mainWindow.close();
    }
}

function handleWindowState(): void {
    // State is automatically managed by WindowStateManager.track()
}

// ─────────────────────────────────────────────────────────────────────────────
// Tray
// ─────────────────────────────────────────────────────────────────────────────

export function createTray(): Tray | undefined {
    const icon = getAppIcon();
    if (!icon) return undefined;

    tray = new Tray(icon);
    tray.setToolTip('TOS Desktop Environment');

    const contextMenu = Menu.buildFromTemplate([
        {
            label: 'Show TOS',
            click: () => {
                if (mainWindow) {
                    mainWindow.show();
                    mainWindow.focus();
                } else {
                    createFaceWindow(getPlatformConfig());
                }
            }
        },
        { type: 'separator' },
        {
            label: 'Brain Status',
            sublabel: `Connected to ${BRAIN_WS_URL}`,
            enabled: false,
        },
        { type: 'separator' },
        {
            label: 'Settings',
            click: () => {
                mainWindow?.webContents.send('tos:navigate', 'settings');
            }
        },
        { type: 'separator' },
        {
            label: 'Quit TOS',
            click: () => {
                app.quit();
            }
        }
    ]);

    tray.setContextMenu(contextMenu);

    tray.on('click', () => {
        if (mainWindow) {
            if (mainWindow.isVisible()) {
                mainWindow.focus();
            } else {
                mainWindow.show();
            }
        }
    });

    return tray;
}

export function showTrayMenu(x: number, y: number): void {
    tray?.popUpContextMenu();
}

// ─────────────────────────────────────────────────────────────────────────────
// IPC Bridge (Main ↔ Renderer)
// ─────────────────────────────────────────────────────────────────────────────

function setupIPCBridge(): void {
    // Pass Brain WS URL to renderer via environment query
    ipcMain.handle('tos:get-brain-url', () => {
        return BRAIN_WS_URL;
    });

    // Window control commands from renderer
    ipcMain.on('tos:window-minimize', () => {
        mainWindow?.minimize();
    });

    ipcMain.on('tos:window-maximize', () => {
        if (mainWindow?.isMaximized()) {
            mainWindow.unmaximize();
        } else {
            mainWindow?.maximize();
        }
    });

    ipcMain.on('tos:window-close', () => {
        mainWindow?.close();
    });

    // Platform info
    ipcMain.handle('tos:get-platform', () => {
        return {
            platform: process.platform,
            arch: process.arch,
            version: app.getVersion(),
            isDevMode: IS_DEV,
        };
    });

    // App-level actions
    ipcMain.on('tos:set-title', (_event, title: string) => {
        mainWindow?.setTitle(title);
    });

    ipcMain.on('tos:set-badge', (_event, count: number) => {
        if (process.platform === 'darwin') {
            app.setBadgeCount(count);
        }
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

function getAppIcon(): Electron.NativeImage | undefined {
    const iconPaths = [
        path.join(__dirname, '..', 'resources', 'icon.png'),
        path.join(process.resourcesPath ?? '', 'icon.png'),
    ];

    for (const iconPath of iconPaths) {
        if (fs.existsSync(iconPath)) {
            return nativeImage.createFromPath(iconPath);
        }
    }

    return undefined;
}

// ─────────────────────────────────────────────────────────────────────────────
// Application Lifecycle
// ─────────────────────────────────────────────────────────────────────────────

app.whenReady().then(async () => {
    const config = getPlatformConfig();

    console.log(`[TOS Electron] Platform: ${config.platform}`);
    console.log(`[TOS Electron] Renderer: ${config.rendererBuildPath}`);
    console.log(`[TOS Electron] Brain WS: ${BRAIN_WS_URL}`);
    console.log(`[TOS Electron] Dev Mode: ${IS_DEV}`);

    // Register custom protocol
    registerProtocolHandler();

    // Setup IPC bridge
    setupIPCBridge();

    // Register dialog and print handlers
    registerFileDialogHandlers();
    registerPrintHandlers();

    // Setup platform menu
    const menu = createPlatformMenu(config.platform);
    Menu.setApplicationMenu(menu);

    // Create main window
    await createFaceWindow(config);

    // Create tray icon
    createTray();

    // Setup auto-updater (production only)
    if (config.enableAutoUpdater) {
        setupAutoUpdater(mainWindow!);
    }

    // macOS: re-create window when dock icon is clicked
    app.on('activate', () => {
        if (BrowserWindow.getAllWindows().length === 0) {
            createFaceWindow(config);
        }
    });
});

// Quit when all windows are closed (except on macOS)
app.on('window-all-closed', () => {
    if (process.platform !== 'darwin') {
        app.quit();
    }
});

// Security: prevent new window creation
app.on('web-contents-created', (_event, contents) => {
    contents.setWindowOpenHandler(({ url }) => {
        shell.openExternal(url);
        return { action: 'deny' };
    });
});
