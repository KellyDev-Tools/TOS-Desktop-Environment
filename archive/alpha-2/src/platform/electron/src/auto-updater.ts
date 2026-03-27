/**
 * TOS Electron — Auto-Updater
 * ─────────────────────────────────────────────────────────────────────────────
 * Handles checking for updates, downloading, and applying them
 * using electron-updater (backed by electron-builder publish config).
 */

import { autoUpdater, UpdateInfo } from 'electron-updater';
import { BrowserWindow, ipcMain, dialog } from 'electron';

let mainWindow: BrowserWindow | null = null;

/**
 * Initialize the auto-update system.
 * Should only be called in production builds.
 */
export function setupAutoUpdater(window: BrowserWindow): void {
    mainWindow = window;

    // Configure updater
    autoUpdater.autoDownload = false;
    autoUpdater.autoInstallOnAppQuit = true;

    // ── Events ───────────────────────────────────────────────────────────

    autoUpdater.on('checking-for-update', () => {
        console.log('[AutoUpdate] Checking for updates...');
    });

    autoUpdater.on('update-available', (info: UpdateInfo) => {
        console.log('[AutoUpdate] Update available:', info.version);
        mainWindow?.webContents.send('tos:update-available', {
            version: info.version,
            releaseDate: info.releaseDate,
            releaseNotes: info.releaseNotes,
        });

        // Auto-download the update
        autoUpdater.downloadUpdate();
    });

    autoUpdater.on('update-not-available', (info: UpdateInfo) => {
        console.log('[AutoUpdate] Current version is up to date:', info.version);
    });

    autoUpdater.on('download-progress', (progress) => {
        console.log(`[AutoUpdate] Download: ${progress.percent.toFixed(1)}%`);
        mainWindow?.webContents.send('tos:update-progress', {
            percent: progress.percent,
            bytesPerSecond: progress.bytesPerSecond,
            total: progress.total,
            transferred: progress.transferred,
        });
    });

    autoUpdater.on('update-downloaded', (info: UpdateInfo) => {
        console.log('[AutoUpdate] Update downloaded:', info.version);
        mainWindow?.webContents.send('tos:update-downloaded', {
            version: info.version,
            releaseDate: info.releaseDate,
            releaseNotes: info.releaseNotes,
        });
    });

    autoUpdater.on('error', (err) => {
        console.error('[AutoUpdate] Error:', err.message);
    });

    // ── IPC Handlers ─────────────────────────────────────────────────────

    ipcMain.handle('tos:check-updates', async (): Promise<boolean> => {
        return checkForUpdates();
    });

    ipcMain.on('tos:install-update', () => {
        installUpdate();
    });

    // Check for updates on launch (after a delay)
    setTimeout(() => {
        checkForUpdates();
    }, 10_000);

    // Check periodically (every 4 hours)
    setInterval(() => {
        checkForUpdates();
    }, 4 * 60 * 60 * 1000);
}

/**
 * Check for available updates.
 * Returns true if an update is available.
 */
export async function checkForUpdates(): Promise<boolean> {
    try {
        const result = await autoUpdater.checkForUpdates();
        return result?.updateInfo?.version !== autoUpdater.currentVersion?.version;
    } catch (err) {
        console.error('[AutoUpdate] Check failed:', err);
        return false;
    }
}

/**
 * Install a downloaded update and restart the application.
 */
export function installUpdate(): void {
    autoUpdater.quitAndInstall(false, true);
}
