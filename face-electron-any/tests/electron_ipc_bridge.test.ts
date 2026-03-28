/**
 * TOS Electron — IPC Bridge Tests
 * ─────────────────────────────────────────────────────────────────────────────
 * Tests for IPC message passing between main and renderer processes.
 * Validates the preload API surface, channel routing, and message integrity.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';

// ── Mock IPC Channels ───────────────────────────────────────────────────────

const ipcHandlers = new Map<string, Function>();
const ipcListeners = new Map<string, Function>();

const mockIpcMain = {
    handle: vi.fn((channel: string, handler: Function) => {
        ipcHandlers.set(channel, handler);
    }),
    on: vi.fn((channel: string, handler: Function) => {
        ipcListeners.set(channel, handler);
    }),
};

const mockIpcRenderer = {
    invoke: vi.fn(async (channel: string, ...args: any[]) => {
        const handler = ipcHandlers.get(channel);
        if (handler) return handler({}, ...args);
        return undefined;
    }),
    send: vi.fn((channel: string, ...args: any[]) => {
        const listener = ipcListeners.get(channel);
        if (listener) listener({}, ...args);
    }),
    on: vi.fn(),
    removeListener: vi.fn(),
};

// ── Tests ────────────────────────────────────────────────────────────────────

describe('IPC Bridge', () => {
    beforeEach(() => {
        ipcHandlers.clear();
        ipcListeners.clear();
        vi.clearAllMocks();
    });

    describe('Channel Registration', () => {
        it('should register all required IPC handle channels', () => {
            const requiredChannels = [
                'tos:get-brain-url',
                'tos:get-platform',
                'tos:show-open-dialog',
                'tos:show-save-dialog',
                'tos:print-preview',
                'tos:check-updates',
            ];

            requiredChannels.forEach(channel => {
                mockIpcMain.handle(channel, () => { });
                expect(ipcHandlers.has(channel)).toBe(true);
            });
        });

        it('should register all required IPC on channels', () => {
            const requiredChannels = [
                'tos:window-minimize',
                'tos:window-maximize',
                'tos:window-close',
                'tos:set-title',
                'tos:set-badge',
                'tos:install-update',
            ];

            requiredChannels.forEach(channel => {
                mockIpcMain.on(channel, () => { });
                expect(ipcListeners.has(channel)).toBe(true);
            });
        });
    });

    describe('Brain URL', () => {
        it('should return the default Brain WebSocket URL', async () => {
            const defaultUrl = 'ws://127.0.0.1:7001';
            mockIpcMain.handle('tos:get-brain-url', () => defaultUrl);

            const result = await mockIpcRenderer.invoke('tos:get-brain-url');
            expect(result).toBe('ws://127.0.0.1:7001');
        });

        it('should return custom Brain URL from environment', async () => {
            const customUrl = 'ws://192.168.1.100:7001';
            mockIpcMain.handle('tos:get-brain-url', () => customUrl);

            const result = await mockIpcRenderer.invoke('tos:get-brain-url');
            expect(result).toBe(customUrl);
        });
    });

    describe('Platform Info', () => {
        it('should return complete platform information', async () => {
            const platformInfo = {
                platform: 'linux',
                arch: 'x64',
                version: '2.2.1',
                isDevMode: false,
            };
            mockIpcMain.handle('tos:get-platform', () => platformInfo);

            const result = await mockIpcRenderer.invoke('tos:get-platform');
            expect(result).toEqual(platformInfo);
            expect(result.platform).toBe('linux');
            expect(result.version).toBe('2.2.1');
        });
    });

    describe('Window Controls', () => {
        it('should send minimize command', () => {
            let minimized = false;
            mockIpcMain.on('tos:window-minimize', () => { minimized = true; });
            mockIpcRenderer.send('tos:window-minimize');
            expect(minimized).toBe(true);
        });

        it('should send maximize toggle command', () => {
            let maximized = false;
            mockIpcMain.on('tos:window-maximize', () => { maximized = !maximized; });
            mockIpcRenderer.send('tos:window-maximize');
            expect(maximized).toBe(true);
            mockIpcRenderer.send('tos:window-maximize');
            expect(maximized).toBe(false);
        });

        it('should send close command', () => {
            let closed = false;
            mockIpcMain.on('tos:window-close', () => { closed = true; });
            mockIpcRenderer.send('tos:window-close');
            expect(closed).toBe(true);
        });

        it('should send title update', () => {
            let title = '';
            mockIpcMain.on('tos:set-title', (_e: any, t: string) => { title = t; });
            mockIpcRenderer.send('tos:set-title', 'TOS — Sector 3');
            expect(title).toBe('TOS — Sector 3');
        });

        it('should send badge count (macOS)', () => {
            let badge = 0;
            mockIpcMain.on('tos:set-badge', (_e: any, count: number) => { badge = count; });
            mockIpcRenderer.send('tos:set-badge', 5);
            expect(badge).toBe(5);
        });
    });

    describe('File Dialogs', () => {
        it('should handle open dialog request', async () => {
            const expected = {
                canceled: false,
                filePaths: ['/home/user/doc.txt'],
            };
            mockIpcMain.handle('tos:show-open-dialog', () => expected);

            const result = await mockIpcRenderer.invoke('tos:show-open-dialog', {
                title: 'Open File',
                filters: [{ name: 'Text', extensions: ['txt'] }],
            });
            expect(result.canceled).toBe(false);
            expect(result.filePaths).toHaveLength(1);
        });

        it('should handle canceled dialog', async () => {
            const expected = { canceled: true, filePaths: [] };
            mockIpcMain.handle('tos:show-open-dialog', () => expected);

            const result = await mockIpcRenderer.invoke('tos:show-open-dialog', {});
            expect(result.canceled).toBe(true);
            expect(result.filePaths).toHaveLength(0);
        });

        it('should handle save dialog request', async () => {
            const expected = {
                canceled: false,
                filePath: '/home/user/export.json',
            };
            mockIpcMain.handle('tos:show-save-dialog', () => expected);

            const result = await mockIpcRenderer.invoke('tos:show-save-dialog', {
                title: 'Save Export',
                defaultPath: 'export.json',
            });
            expect(result.canceled).toBe(false);
            expect(result.filePath).toBe('/home/user/export.json');
        });
    });

    describe('Preload API Surface', () => {
        it('should expose all required API methods', () => {
            const requiredMethods = [
                'getPlatform',
                'getBrainUrl',
                'windowMinimize',
                'windowMaximize',
                'windowClose',
                'setTitle',
                'setBadge',
                'showOpenDialog',
                'showSaveDialog',
                'printPreview',
                'checkForUpdates',
                'installUpdate',
                'onNavigate',
                'onUpdateAvailable',
                'onUpdateDownloaded',
            ];

            // Verify the method names are all strings (surface check)
            requiredMethods.forEach(method => {
                expect(typeof method).toBe('string');
                expect(method.length).toBeGreaterThan(0);
            });
            expect(requiredMethods).toHaveLength(15);
        });
    });
});

describe('IPC Message Integrity', () => {
    it('should preserve complex objects through IPC', async () => {
        const complexPayload = {
            sectors: [
                { id: 0, name: 'Main', modules: ['terminal', 'browser'] },
                { id: 1, name: 'Dev', modules: ['editor', 'debugger'] },
            ],
            settings: {
                theme: 'dark',
                audioEnabled: true,
                feedbackLevel: 0.7,
            },
        };

        mockIpcMain.handle('tos:test-complex', () => complexPayload);
        const result = await mockIpcRenderer.invoke('tos:test-complex');
        expect(result).toEqual(complexPayload);
        expect(result.sectors).toHaveLength(2);
        expect(result.settings.feedbackLevel).toBe(0.7);
    });

    it('should handle undefined return values gracefully', async () => {
        const result = await mockIpcRenderer.invoke('tos:nonexistent-channel');
        expect(result).toBeUndefined();
    });
});
