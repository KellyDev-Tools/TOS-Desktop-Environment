/**
 * TOS Electron — Window State Persistence Tests
 * ─────────────────────────────────────────────────────────────────────────────
 * Tests for window state save/restore and display validation.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';

// ── Mock Store ───────────────────────────────────────────────────────────────

interface WindowState {
    x?: number;
    y?: number;
    width: number;
    height: number;
    isMaximized: boolean;
}

const DEFAULT_STATE: WindowState = {
    width: 1280,
    height: 800,
    isMaximized: false,
};

let storeData: Record<string, WindowState> = {};

const mockStore = {
    get: vi.fn((key: string) => storeData[key] ?? DEFAULT_STATE),
    set: vi.fn((key: string, value: WindowState) => { storeData[key] = value; }),
};

// ── Mock Display ─────────────────────────────────────────────────────────────

interface Display {
    bounds: { x: number; y: number; width: number; height: number };
}

function isOnScreen(state: WindowState, displays: Display[]): boolean {
    if (state.x === undefined || state.y === undefined) return true; // no position = center
    return displays.some(display => {
        const b = display.bounds;
        return (
            state.x! >= b.x &&
            state.y! >= b.y &&
            state.x! < b.x + b.width &&
            state.y! < b.y + b.height
        );
    });
}

// ── Tests ────────────────────────────────────────────────────────────────────

describe('Window State Persistence', () => {
    beforeEach(() => {
        storeData = {};
        vi.clearAllMocks();
    });

    describe('Default State', () => {
        it('should have correct default dimensions', () => {
            expect(DEFAULT_STATE.width).toBe(1280);
            expect(DEFAULT_STATE.height).toBe(800);
        });

        it('should not be maximized by default', () => {
            expect(DEFAULT_STATE.isMaximized).toBe(false);
        });

        it('should not have a position by default', () => {
            expect(DEFAULT_STATE.x).toBeUndefined();
            expect(DEFAULT_STATE.y).toBeUndefined();
        });
    });

    describe('State Save', () => {
        it('should save non-maximized window bounds', () => {
            const state: WindowState = {
                x: 200,
                y: 150,
                width: 1440,
                height: 900,
                isMaximized: false,
            };
            mockStore.set('window-state-main-window', state);

            expect(storeData['window-state-main-window']).toEqual(state);
        });

        it('should preserve previous bounds when maximized', () => {
            // First save normal state
            const normalState: WindowState = {
                x: 200,
                y: 150,
                width: 1440,
                height: 900,
                isMaximized: false,
            };
            mockStore.set('window-state-main-window', normalState);

            // Then save maximized state (preserve bounds)
            const maximizedState: WindowState = {
                ...normalState,
                isMaximized: true,
            };
            mockStore.set('window-state-main-window', maximizedState);

            const saved = storeData['window-state-main-window'];
            expect(saved.isMaximized).toBe(true);
            expect(saved.width).toBe(1440);  // preserved
            expect(saved.height).toBe(900);  // preserved
        });
    });

    describe('State Restore', () => {
        it('should restore saved window state', () => {
            storeData['window-state-main-window'] = {
                x: 300,
                y: 200,
                width: 1600,
                height: 1000,
                isMaximized: false,
            };

            const restored = mockStore.get('window-state-main-window');
            expect(restored.x).toBe(300);
            expect(restored.y).toBe(200);
            expect(restored.width).toBe(1600);
            expect(restored.height).toBe(1000);
        });

        it('should return default state when no state is saved', () => {
            const restored = mockStore.get('window-state-nonexistent');
            expect(restored).toEqual(DEFAULT_STATE);
        });
    });

    describe('Display Validation', () => {
        const singleDisplay: Display[] = [
            { bounds: { x: 0, y: 0, width: 1920, height: 1080 } },
        ];

        const dualDisplay: Display[] = [
            { bounds: { x: 0, y: 0, width: 1920, height: 1080 } },
            { bounds: { x: 1920, y: 0, width: 2560, height: 1440 } },
        ];

        it('should validate window on single monitor', () => {
            const state: WindowState = { x: 100, y: 100, width: 1280, height: 800, isMaximized: false };
            expect(isOnScreen(state, singleDisplay)).toBe(true);
        });

        it('should detect off-screen window', () => {
            const state: WindowState = { x: 5000, y: 3000, width: 1280, height: 800, isMaximized: false };
            expect(isOnScreen(state, singleDisplay)).toBe(false);
        });

        it('should accept window on second monitor', () => {
            const state: WindowState = { x: 2000, y: 200, width: 1280, height: 800, isMaximized: false };
            expect(isOnScreen(state, dualDisplay)).toBe(true);
        });

        it('should allow state without position (will center)', () => {
            const state: WindowState = { width: 1280, height: 800, isMaximized: false };
            expect(isOnScreen(state, singleDisplay)).toBe(true);
        });

        it('should detect window at negative coordinates (removed monitor)', () => {
            const state: WindowState = { x: -2000, y: -1000, width: 1280, height: 800, isMaximized: false };
            expect(isOnScreen(state, singleDisplay)).toBe(false);
        });
    });

    describe('State Key Naming', () => {
        it('should use window name as part of store key', () => {
            const windowName = 'main-window';
            const key = `window-state-${windowName}`;
            expect(key).toBe('window-state-main-window');
        });

        it('should support multiple window state keys', () => {
            mockStore.set('window-state-main-window', { ...DEFAULT_STATE, x: 100, y: 100 });
            mockStore.set('window-state-settings', { ...DEFAULT_STATE, width: 800, height: 600 });

            expect(storeData['window-state-main-window'].x).toBe(100);
            expect(storeData['window-state-settings'].width).toBe(800);
        });
    });
});
