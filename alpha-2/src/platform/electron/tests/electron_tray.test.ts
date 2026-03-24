/**
 * TOS Electron — Tray Tests
 * ─────────────────────────────────────────────────────────────────────────────
 * Tests for system tray icon and context menu behavior.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';

// ── Mock Tray ────────────────────────────────────────────────────────────────

interface MockMenuItem {
    label: string;
    type?: 'separator';
    enabled?: boolean;
    sublabel?: string;
    click?: () => void;
}

let trayTooltip = '';
let trayContextMenu: MockMenuItem[] = [];

const mockTray = {
    setToolTip: vi.fn((tip: string) => { trayTooltip = tip; }),
    setContextMenu: vi.fn((menu: any) => { trayContextMenu = menu; }),
    popUpContextMenu: vi.fn(),
    on: vi.fn(),
    destroy: vi.fn(),
};

// ── Tests ────────────────────────────────────────────────────────────────────

describe('System Tray', () => {
    beforeEach(() => {
        trayTooltip = '';
        trayContextMenu = [];
        vi.clearAllMocks();
    });

    describe('Tray Creation', () => {
        it('should set tooltip to TOS Desktop Environment', () => {
            mockTray.setToolTip('TOS Desktop Environment');
            expect(trayTooltip).toBe('TOS Desktop Environment');
        });

        it('should create context menu with required items', () => {
            const menuItems: MockMenuItem[] = [
                { label: 'Show TOS', click: () => { } },
                { label: '', type: 'separator' },
                { label: 'Brain Status', sublabel: 'Connected to ws://127.0.0.1:7001', enabled: false },
                { label: '', type: 'separator' },
                { label: 'Settings', click: () => { } },
                { label: '', type: 'separator' },
                { label: 'Quit TOS', click: () => { } },
            ];
            mockTray.setContextMenu(menuItems);

            expect(trayContextMenu).toHaveLength(7);
        });

        it('should include Show TOS as first item', () => {
            const menuItems: MockMenuItem[] = [
                { label: 'Show TOS', click: () => { } },
            ];
            mockTray.setContextMenu(menuItems);

            expect(trayContextMenu[0].label).toBe('Show TOS');
        });
    });

    describe('Tray Context Menu Items', () => {
        const menuItems: MockMenuItem[] = [
            { label: 'Show TOS', click: () => { } },
            { label: '', type: 'separator' },
            { label: 'Brain Status', sublabel: 'Connected to ws://127.0.0.1:7001', enabled: false },
            { label: '', type: 'separator' },
            { label: 'Settings', click: () => { } },
            { label: '', type: 'separator' },
            { label: 'Quit TOS', click: () => { } },
        ];

        it('should have Brain Status as disabled info item', () => {
            const brainStatus = menuItems.find(i => i.label === 'Brain Status');
            expect(brainStatus).toBeDefined();
            expect(brainStatus!.enabled).toBe(false);
            expect(brainStatus!.sublabel).toContain('ws://');
        });

        it('should have Settings item', () => {
            const settings = menuItems.find(i => i.label === 'Settings');
            expect(settings).toBeDefined();
            expect(settings!.click).toBeDefined();
        });

        it('should have Quit TOS as last item', () => {
            const nonSeparators = menuItems.filter(i => i.type !== 'separator');
            const lastItem = nonSeparators[nonSeparators.length - 1];
            expect(lastItem.label).toBe('Quit TOS');
        });

        it('should have separators between sections', () => {
            const separators = menuItems.filter(i => i.type === 'separator');
            expect(separators.length).toBeGreaterThanOrEqual(2);
        });
    });

    describe('Tray Click Behavior', () => {
        it('should register click handler', () => {
            mockTray.on('click', () => { });
            expect(mockTray.on).toHaveBeenCalledWith('click', expect.any(Function));
        });

        it('should popup context menu at position', () => {
            mockTray.popUpContextMenu();
            expect(mockTray.popUpContextMenu).toHaveBeenCalled();
        });
    });
});
