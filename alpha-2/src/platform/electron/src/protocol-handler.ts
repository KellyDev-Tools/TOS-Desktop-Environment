/**
 * TOS Electron — Custom Protocol Handler
 * ─────────────────────────────────────────────────────────────────────────────
 * Registers and handles the `tos://` custom protocol.
 * Enables deep-linking: opening `tos://sector/3` will navigate
 * the Face to sector 3, for example.
 *
 * Protocol format:
 *   tos://[action]/[...params]
 *
 * Supported actions:
 *   tos://sector/<index>      → Switch to sector
 *   tos://settings            → Open settings
 *   tos://hub                 → Navigate to Command Hub
 *   tos://global              → Navigate to Global Overview
 *   tos://module/<id>         → Open module by ID
 */

import { app, BrowserWindow, protocol } from 'electron';

const PROTOCOL_NAME = 'tos';

/**
 * Register the `tos://` custom protocol handler.
 * Must be called before app.whenReady().
 */
export function registerProtocolHandler(): void {
    // Register as default protocol handler for tos://
    if (process.defaultApp) {
        if (process.argv.length >= 2) {
            app.setAsDefaultProtocolClient(PROTOCOL_NAME, process.execPath, [
                '--',
                ...process.argv.slice(1),
            ]);
        }
    } else {
        app.setAsDefaultProtocolClient(PROTOCOL_NAME);
    }

    // Handle protocol URLs received when app is already running
    app.on('open-url', (event, url) => {
        event.preventDefault();
        handleCustomProtocol(url);
    });

    // Handle protocol URLs on Windows/Linux (single instance lock)
    app.on('second-instance', (_event, argv) => {
        const protocolUrl = argv.find(arg => arg.startsWith(`${PROTOCOL_NAME}://`));
        if (protocolUrl) {
            handleCustomProtocol(protocolUrl);
        }

        // Focus the existing window
        const win = BrowserWindow.getAllWindows()[0];
        if (win) {
            if (win.isMinimized()) win.restore();
            win.focus();
        }
    });

    console.log(`[Protocol] Registered handler for ${PROTOCOL_NAME}://`);
}

/**
 * Handle a `tos://` protocol URL.
 */
export async function handleCustomProtocol(url: string): Promise<void> {
    console.log(`[Protocol] Handling: ${url}`);

    try {
        const parsed = new URL(url);
        const action = parsed.hostname;
        const pathParts = parsed.pathname.split('/').filter(Boolean);

        const win = BrowserWindow.getAllWindows()[0];
        if (!win) {
            console.warn('[Protocol] No window available to handle URL');
            return;
        }

        switch (action) {
            case 'sector':
                if (pathParts[0]) {
                    win.webContents.send('tos:navigate', `sector-${pathParts[0]}`);
                }
                break;

            case 'settings':
                win.webContents.send('tos:navigate', 'settings');
                break;

            case 'hub':
                win.webContents.send('tos:navigate', 'navigate-hub');
                break;

            case 'global':
                win.webContents.send('tos:navigate', 'navigate-global');
                break;

            case 'module':
                if (pathParts[0]) {
                    win.webContents.send('tos:navigate', `module-${pathParts[0]}`);
                }
                break;

            default:
                console.warn(`[Protocol] Unknown action: ${action}`);
        }

        // Bring window to front
        if (win.isMinimized()) win.restore();
        win.focus();

    } catch (err) {
        console.error('[Protocol] Failed to parse URL:', url, err);
    }
}
