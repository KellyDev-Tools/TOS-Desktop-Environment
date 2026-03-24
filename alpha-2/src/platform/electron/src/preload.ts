/**
 * TOS Electron Preload Script
 * ─────────────────────────────────────────────────────────────────────────────
 * Secure bridge between Electron main process and the Svelte renderer.
 * Exposes a `window.tosElectron` API using contextBridge.
 *
 * Security model:
 *   - contextIsolation = true
 *   - nodeIntegration  = false
 *   - sandbox          = true
 *   - Only whitelisted IPC channels are exposed
 */

import { contextBridge, ipcRenderer } from 'electron';

/**
 * The TOS Electron API exposed to the renderer process.
 * Available as `window.tosElectron` in the Svelte UI.
 */
const tosElectronAPI = {
    // ── Platform Info ────────────────────────────────────────────────────
    getPlatform: () => ipcRenderer.invoke('tos:get-platform'),
    getBrainUrl: () => ipcRenderer.invoke('tos:get-brain-url'),

    // ── Window Controls ──────────────────────────────────────────────────
    windowMinimize: () => ipcRenderer.send('tos:window-minimize'),
    windowMaximize: () => ipcRenderer.send('tos:window-maximize'),
    windowClose: () => ipcRenderer.send('tos:window-close'),
    setTitle: (title: string) => ipcRenderer.send('tos:set-title', title),
    setBadge: (count: number) => ipcRenderer.send('tos:set-badge', count),

    // ── File Dialogs ─────────────────────────────────────────────────────
    showOpenDialog: (options: {
        title?: string;
        filters?: Array<{ name: string; extensions: string[] }>;
        properties?: string[];
    }) => ipcRenderer.invoke('tos:show-open-dialog', options),

    showSaveDialog: (options: {
        title?: string;
        defaultPath?: string;
        filters?: Array<{ name: string; extensions: string[] }>;
    }) => ipcRenderer.invoke('tos:show-save-dialog', options),

    // ── Print ────────────────────────────────────────────────────────────
    printPreview: () => ipcRenderer.invoke('tos:print-preview'),

    // ── Auto-Update ──────────────────────────────────────────────────────
    checkForUpdates: () => ipcRenderer.invoke('tos:check-updates'),
    installUpdate: () => ipcRenderer.send('tos:install-update'),

    // ── Event Listeners ──────────────────────────────────────────────────
    onNavigate: (callback: (route: string) => void) => {
        const handler = (_event: Electron.IpcRendererEvent, route: string) => callback(route);
        ipcRenderer.on('tos:navigate', handler);
        return () => ipcRenderer.removeListener('tos:navigate', handler);
    },

    onUpdateAvailable: (callback: (info: any) => void) => {
        const handler = (_event: Electron.IpcRendererEvent, info: any) => callback(info);
        ipcRenderer.on('tos:update-available', handler);
        return () => ipcRenderer.removeListener('tos:update-available', handler);
    },

    onUpdateDownloaded: (callback: (info: any) => void) => {
        const handler = (_event: Electron.IpcRendererEvent, info: any) => callback(info);
        ipcRenderer.on('tos:update-downloaded', handler);
        return () => ipcRenderer.removeListener('tos:update-downloaded', handler);
    },
};

// Expose the API to the renderer in a secure namespace
contextBridge.exposeInMainWorld('tosElectron', tosElectronAPI);

// Type declaration for the renderer side
export type TosElectronAPI = typeof tosElectronAPI;
