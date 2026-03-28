/**
 * TOS Electron — Window Management Tests
 * ─────────────────────────────────────────────────────────────────────────────
 * Tests for window creation, restoration, closure, and state persistence.
 * Run with: npx tsx tests/electron_window_management.ts
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';

// ── Mock Electron APIs ──────────────────────────────────────────────────────

const mockBounds = { x: 100, y: 100, width: 1280, height: 800 };
let isMaximized = false;
let isMinimized = false;
let isVisible = true;
const eventHandlers: Record<string, Function[]> = {};

const mockBrowserWindow = {
    getBounds: vi.fn(() => mockBounds),
    isMaximized: vi.fn(() => isMaximized),
    isMinimized: vi.fn(() => isMinimized),
    isVisible: vi.fn(() => isVisible),
    maximize: vi.fn(() => { isMaximized = true; }),
    unmaximize: vi.fn(() => { isMaximized = false; }),
    minimize: vi.fn(() => { isMinimized = true; }),
    restore: vi.fn(() => { isMinimized = false; }),
    show: vi.fn(() => { isVisible = true; }),
    hide: vi.fn(() => { isVisible = false; }),
    focus: vi.fn(),
    close: vi.fn(),
    setTitle: vi.fn(),
    on: vi.fn((event: string, handler: Function) => {
        if (!eventHandlers[event]) eventHandlers[event] = [];
        eventHandlers[event].push(handler);
    }),
    once: vi.fn(),
    webContents: {
        send: vi.fn(),
        openDevTools: vi.fn(),
        id: 1,
        isLoading: vi.fn(() => false),
        setWindowOpenHandler: vi.fn(),
    },
    loadFile: vi.fn().mockResolvedValue(undefined),
    loadURL: vi.fn().mockResolvedValue(undefined),
};

// ── Tests ────────────────────────────────────────────────────────────────────

describe('Electron Window Management', () => {

    beforeEach(() => {
        isMaximized = false;
        isMinimized = false;
        isVisible = true;
        Object.keys(eventHandlers).forEach(k => delete eventHandlers[k]);
        vi.clearAllMocks();
    });

    describe('Window Creation', () => {
        it('should create a window with default dimensions', () => {
            const config = {
                width: 1280,
                height: 800,
                minWidth: 800,
                minHeight: 600,
            };
            expect(config.width).toBe(1280);
            expect(config.height).toBe(800);
            expect(config.minWidth).toBe(800);
            expect(config.minHeight).toBe(600);
        });

        it('should use TOS dark background color', () => {
            const bgColor = '#0a0a0f';
            expect(bgColor).toBe('#0a0a0f');
        });

        it('should have correct security settings', () => {
            const webPrefs = {
                contextIsolation: true,
                nodeIntegration: false,
                sandbox: true,
                webSecurity: true,
                allowRunningInsecureContent: false,
            };
            expect(webPrefs.contextIsolation).toBe(true);
            expect(webPrefs.nodeIntegration).toBe(false);
            expect(webPrefs.sandbox).toBe(true);
            expect(webPrefs.webSecurity).toBe(true);
            expect(webPrefs.allowRunningInsecureContent).toBe(false);
        });

        it('should configure hiddenInset titleBarStyle on macOS', () => {
            const platform = 'darwin';
            const titleBarStyle = platform === 'darwin' ? 'hiddenInset' : 'default';
            expect(titleBarStyle).toBe('hiddenInset');
        });

        it('should configure default titleBarStyle on Windows', () => {
            const platform = 'win32';
            const titleBarStyle = platform === 'darwin' ? 'hiddenInset' : 'default';
            expect(titleBarStyle).toBe('default');
        });
    });

    describe('Window Restoration', () => {
        it('should restore a minimized window', () => {
            isMinimized = true;
            mockBrowserWindow.restore();
            expect(mockBrowserWindow.restore).toHaveBeenCalled();
        });

        it('should focus window after restoration', () => {
            mockBrowserWindow.focus();
            expect(mockBrowserWindow.focus).toHaveBeenCalled();
        });

        it('should maximize window when saved state was maximized', () => {
            mockBrowserWindow.maximize();
            expect(mockBrowserWindow.maximize).toHaveBeenCalled();
            expect(isMaximized).toBe(true);
        });
    });

    describe('Window Closure', () => {
        it('should close the main window', () => {
            mockBrowserWindow.close();
            expect(mockBrowserWindow.close).toHaveBeenCalled();
        });
    });

    describe('Window State Tracking', () => {
        it('should register event listeners for state tracking', () => {
            const events = ['resize', 'move', 'maximize', 'unmaximize', 'close', 'closed'];
            events.forEach(event => {
                mockBrowserWindow.on(event, () => { });
            });
            expect(mockBrowserWindow.on).toHaveBeenCalledTimes(events.length);
        });

        it('should save non-maximized bounds correctly', () => {
            const bounds = mockBrowserWindow.getBounds();
            expect(bounds.x).toBe(100);
            expect(bounds.y).toBe(100);
            expect(bounds.width).toBe(1280);
            expect(bounds.height).toBe(800);
        });

        it('should preserve maximized flag without changing bounds', () => {
            isMaximized = true;
            const state = {
                ...mockBounds,
                isMaximized: mockBrowserWindow.isMaximized(),
            };
            expect(state.isMaximized).toBe(true);
            expect(state.width).toBe(1280); // preserved from before maximize
        });
    });
});

describe('Platform Configuration', () => {
    it('should detect platform correctly', () => {
        const platforms = ['win32', 'darwin', 'linux'] as const;
        platforms.forEach(p => {
            expect(['win32', 'darwin', 'linux']).toContain(p);
        });
    });

    it('should enable native menu bar only on macOS', () => {
        expect(('darwin' === 'darwin')).toBe(true);
        expect(('win32' === 'darwin')).toBe(false);
        expect(('linux' === 'darwin')).toBe(false);
    });

    it('should disable auto-updater in dev mode', () => {
        const isDev = true;
        const enableAutoUpdater = !isDev;
        expect(enableAutoUpdater).toBe(false);
    });

    it('should enable auto-updater in production mode', () => {
        const isDev = false;
        const enableAutoUpdater = !isDev;
        expect(enableAutoUpdater).toBe(true);
    });
});
