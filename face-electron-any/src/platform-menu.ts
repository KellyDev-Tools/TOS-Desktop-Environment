/**
 * TOS Electron — Platform Menu Factory
 * ─────────────────────────────────────────────────────────────────────────────
 * Creates platform-appropriate menus for Windows, macOS, and Linux.
 * macOS uses the native AppKit menu bar; Windows/Linux use custom menus.
 */

import { Menu, MenuItem, shell, app, BrowserWindow, dialog } from 'electron';

/**
 * Create a platform-specific application menu.
 */
export function createPlatformMenu(platform: string): Menu {
    switch (platform) {
        case 'darwin':
            return createMacOSMenu();
        case 'win32':
            return createWindowsMenu();
        default:
            return createLinuxMenu();
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// macOS Menu (AppKit style with dock integration)
// ─────────────────────────────────────────────────────────────────────────────

function createMacOSMenu(): Menu {
    const appName = app.getName();

    return Menu.buildFromTemplate([
        {
            label: appName,
            submenu: [
                { role: 'about' },
                { type: 'separator' },
                {
                    label: 'Preferences…',
                    accelerator: 'Cmd+,',
                    click: () => sendToRenderer('settings'),
                },
                { type: 'separator' },
                { role: 'services' },
                { type: 'separator' },
                { role: 'hide' },
                { role: 'hideOthers' },
                { role: 'unhide' },
                { type: 'separator' },
                { role: 'quit' },
            ],
        },
        {
            label: 'File',
            submenu: [
                {
                    label: 'New Sector',
                    accelerator: 'Cmd+N',
                    click: () => sendToRenderer('new-sector'),
                },
                { type: 'separator' },
                { role: 'close' },
            ],
        },
        {
            label: 'Edit',
            submenu: [
                { role: 'undo' },
                { role: 'redo' },
                { type: 'separator' },
                { role: 'cut' },
                { role: 'copy' },
                { role: 'paste' },
                { role: 'selectAll' },
            ],
        },
        {
            label: 'View',
            submenu: [
                {
                    label: 'Global Overview',
                    accelerator: 'Cmd+1',
                    click: () => sendToRenderer('navigate-global'),
                },
                {
                    label: 'Command Hub',
                    accelerator: 'Cmd+2',
                    click: () => sendToRenderer('navigate-hub'),
                },
                { type: 'separator' },
                { role: 'reload' },
                { role: 'forceReload' },
                { role: 'toggleDevTools' },
                { type: 'separator' },
                { role: 'resetZoom' },
                { role: 'zoomIn' },
                { role: 'zoomOut' },
                { type: 'separator' },
                { role: 'togglefullscreen' },
            ],
        },
        {
            label: 'Window',
            submenu: [
                { role: 'minimize' },
                { role: 'zoom' },
                { type: 'separator' },
                { role: 'front' },
            ],
        },
        {
            label: 'Help',
            submenu: [
                {
                    label: 'TOS Documentation',
                    click: () => shell.openExternal('https://docs.tos.dev'),
                },
                {
                    label: 'Report Issue',
                    click: () => shell.openExternal('https://github.com/tos-project/issues'),
                },
            ],
        },
    ]);
}

// ─────────────────────────────────────────────────────────────────────────────
// Windows Menu
// ─────────────────────────────────────────────────────────────────────────────

function createWindowsMenu(): Menu {
    return Menu.buildFromTemplate([
        {
            label: 'File',
            submenu: [
                {
                    label: 'New Sector',
                    accelerator: 'Ctrl+N',
                    click: () => sendToRenderer('new-sector'),
                },
                { type: 'separator' },
                {
                    label: 'Settings',
                    accelerator: 'Ctrl+,',
                    click: () => sendToRenderer('settings'),
                },
                { type: 'separator' },
                { role: 'quit' },
            ],
        },
        {
            label: 'Edit',
            submenu: [
                { role: 'undo' },
                { role: 'redo' },
                { type: 'separator' },
                { role: 'cut' },
                { role: 'copy' },
                { role: 'paste' },
                { role: 'selectAll' },
            ],
        },
        {
            label: 'View',
            submenu: [
                {
                    label: 'Global Overview',
                    accelerator: 'Ctrl+1',
                    click: () => sendToRenderer('navigate-global'),
                },
                {
                    label: 'Command Hub',
                    accelerator: 'Ctrl+2',
                    click: () => sendToRenderer('navigate-hub'),
                },
                { type: 'separator' },
                { role: 'reload' },
                { role: 'forceReload' },
                { role: 'toggleDevTools' },
                { type: 'separator' },
                { role: 'resetZoom' },
                { role: 'zoomIn' },
                { role: 'zoomOut' },
                { type: 'separator' },
                { role: 'togglefullscreen' },
            ],
        },
        {
            label: 'Help',
            submenu: [
                {
                    label: 'Documentation',
                    click: () => shell.openExternal('https://docs.tos.dev'),
                },
                {
                    label: 'Report Issue',
                    click: () => shell.openExternal('https://github.com/tos-project/issues'),
                },
                { type: 'separator' },
                {
                    label: 'About TOS',
                    click: () => {
                        dialog.showMessageBox({
                            title: 'About TOS',
                            message: `TOS Desktop Environment\nVersion ${app.getVersion()}`,
                            type: 'info',
                        });
                    },
                },
            ],
        },
    ]);
}

// ─────────────────────────────────────────────────────────────────────────────
// Linux Menu (GTK-style)
// ─────────────────────────────────────────────────────────────────────────────

function createLinuxMenu(): Menu {
    // Linux menu is identical to Windows in structure
    return createWindowsMenu();
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper
// ─────────────────────────────────────────────────────────────────────────────

function sendToRenderer(action: string): void {
    const win = BrowserWindow.getFocusedWindow();
    win?.webContents.send('tos:navigate', action);
}
