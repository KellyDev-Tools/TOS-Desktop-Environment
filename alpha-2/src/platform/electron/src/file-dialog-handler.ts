/**
 * TOS Electron — File Dialog Handler
 * ─────────────────────────────────────────────────────────────────────────────
 * Handles platform-native file open/save dialogs via IPC.
 */

import { dialog, ipcMain, BrowserWindow } from 'electron';

/**
 * Register IPC handlers for file dialogs.
 */
export function registerFileDialogHandlers(): void {
    ipcMain.handle('tos:show-open-dialog', async (_event, options: {
        title?: string;
        filters?: Array<{ name: string; extensions: string[] }>;
        properties?: Array<'openFile' | 'openDirectory' | 'multiSelections' | 'showHiddenFiles'>;
    }) => {
        const win = BrowserWindow.getFocusedWindow();
        if (!win) return { canceled: true, filePaths: [] };

        const result = await dialog.showOpenDialog(win, {
            title: options.title ?? 'Open File',
            filters: options.filters ?? [
                { name: 'All Files', extensions: ['*'] },
            ],
            properties: (options.properties ?? ['openFile']) as any[],
        });

        return {
            canceled: result.canceled,
            filePaths: result.filePaths,
        };
    });

    ipcMain.handle('tos:show-save-dialog', async (_event, options: {
        title?: string;
        defaultPath?: string;
        filters?: Array<{ name: string; extensions: string[] }>;
    }) => {
        const win = BrowserWindow.getFocusedWindow();
        if (!win) return { canceled: true, filePath: undefined };

        const result = await dialog.showSaveDialog(win, {
            title: options.title ?? 'Save File',
            defaultPath: options.defaultPath,
            filters: options.filters ?? [
                { name: 'All Files', extensions: ['*'] },
            ],
        });

        return {
            canceled: result.canceled,
            filePath: result.filePath,
        };
    });
}

/**
 * Show a file picker dialog directly (for use from main process).
 */
export async function showFilePicker(
    title: string,
    filters: string[]
): Promise<string[] | undefined> {
    const win = BrowserWindow.getFocusedWindow();
    if (!win) return undefined;

    const dialogFilters = filters.length > 0
        ? [{ name: 'Filtered', extensions: filters }]
        : [{ name: 'All Files', extensions: ['*'] }];

    const result = await dialog.showOpenDialog(win, {
        title,
        filters: dialogFilters,
        properties: ['openFile'],
    });

    if (result.canceled) return undefined;
    return result.filePaths;
}
