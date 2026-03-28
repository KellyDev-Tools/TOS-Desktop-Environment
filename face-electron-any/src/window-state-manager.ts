/**
 * TOS Electron — Window State Manager
 * ─────────────────────────────────────────────────────────────────────────────
 * Persists and restores window position, size, and maximized state
 * across application restarts using electron-store.
 */

import Store from 'electron-store';
import { BrowserWindow, screen } from 'electron';

export interface WindowState {
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

export class WindowStateManager {
    private store: Store<Record<string, WindowState>>;
    private storeKey: string;
    private window: BrowserWindow | null = null;
    private saveTimeout: ReturnType<typeof setTimeout> | null = null;

    constructor(windowName: string) {
        this.storeKey = `window-state-${windowName}`;
        this.store = new Store<Record<string, WindowState>>({
            name: 'tos-window-states',
            defaults: {
                [this.storeKey]: DEFAULT_STATE,
            },
        });
    }

    /** Get the persisted window state, validated against current displays. */
    getState(): WindowState {
        const state = this.store.get(this.storeKey) as WindowState;

        // Validate that the stored position is still on a visible display
        if (state.x !== undefined && state.y !== undefined) {
            const displays = screen.getAllDisplays();
            const isOnScreen = displays.some(display => {
                const bounds = display.bounds;
                return (
                    state.x! >= bounds.x &&
                    state.y! >= bounds.y &&
                    state.x! < bounds.x + bounds.width &&
                    state.y! < bounds.y + bounds.height
                );
            });

            if (!isOnScreen) {
                // Reset position if window would be off-screen
                delete state.x;
                delete state.y;
            }
        }

        return state;
    }

    /** Begin tracking a BrowserWindow for state changes. */
    track(window: BrowserWindow): void {
        this.window = window;

        const events = ['resize', 'move', 'maximize', 'unmaximize'] as const;
        for (const event of events) {
            window.on(event as any, () => this.debouncedSave());
        }

        window.on('close', () => {
            this.saveImmediate();
        });

        window.on('closed', () => {
            this.window = null;
        });
    }

    /** Restore window to its previously saved state. */
    async restoreWindow(windowId: string): Promise<void> {
        if (!this.window) return;

        const state = this.getState();

        if (this.window.isMinimized()) {
            this.window.restore();
        }

        if (state.isMaximized) {
            this.window.maximize();
        }

        this.window.focus();
    }

    // ── Private ──────────────────────────────────────────────────────────

    private debouncedSave(): void {
        if (this.saveTimeout) {
            clearTimeout(this.saveTimeout);
        }
        this.saveTimeout = setTimeout(() => this.saveImmediate(), 300);
    }

    private saveImmediate(): void {
        if (!this.window) return;

        const isMaximized = this.window.isMaximized();

        // Only save bounds when not maximized (to preserve the "normal" size)
        if (!isMaximized) {
            const bounds = this.window.getBounds();
            this.store.set(this.storeKey, {
                x: bounds.x,
                y: bounds.y,
                width: bounds.width,
                height: bounds.height,
                isMaximized: false,
            });
        } else {
            // Preserve the previous non-maximized bounds, just update the flag
            const current = this.store.get(this.storeKey) as WindowState;
            this.store.set(this.storeKey, {
                ...current,
                isMaximized: true,
            });
        }
    }
}
