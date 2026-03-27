/**
 * TOS Electron — Print Handler
 * ─────────────────────────────────────────────────────────────────────────────
 * Handles platform-native print and print preview via IPC.
 */

import { ipcMain, BrowserWindow } from 'electron';

/**
 * Register IPC handlers for printing.
 */
export function registerPrintHandlers(): void {
    ipcMain.handle('tos:print-preview', async () => {
        const win = BrowserWindow.getFocusedWindow();
        if (!win) return;

        // Ensure content is ready before printing
        if (win.webContents.isLoading()) {
            await new Promise<void>((resolve) => {
                win.webContents.once('did-finish-load', () => resolve());
            });
        }

        // Use the web contents print method (shows system print dialog)
        win.webContents.print({ silent: false, printBackground: true });
    });
}

/**
 * Show print preview for a specific webContents.
 */
export async function showPrintPreview(webContentsId: number): Promise<void> {
    const allWindows = BrowserWindow.getAllWindows();
    for (const win of allWindows) {
        if (win.webContents.id === webContentsId) {
            win.webContents.print({ silent: false, printBackground: true });
            return;
        }
    }
    console.warn(`[Print] WebContents ${webContentsId} not found`);
}
