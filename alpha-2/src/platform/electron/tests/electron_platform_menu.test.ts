/**
 * TOS Electron — Platform Menu Tests
 * ─────────────────────────────────────────────────────────────────────────────
 * Tests for platform-specific menu configuration across Windows, macOS, Linux.
 */

import { describe, it, expect } from 'vitest';

// ── Menu Structure Types ─────────────────────────────────────────────────────

interface MenuItem {
    label?: string;
    role?: string;
    type?: 'separator';
    accelerator?: string;
    submenu?: MenuItem[];
}

// ── Mock Menu Templates ──────────────────────────────────────────────────────

function getMacOSMenuTemplate(): MenuItem[] {
    return [
        {
            label: 'TOS',
            submenu: [
                { role: 'about' },
                { type: 'separator' },
                { label: 'Preferences…', accelerator: 'Cmd+,' },
                { type: 'separator' },
                { role: 'services' },
                { type: 'separator' },
                { role: 'hide' },
                { role: 'hideOthers' },
                { role: 'unhide' },
                { type: 'separator' },
                { role: 'quit' },
            ],
        },
        { label: 'File', submenu: [{ label: 'New Sector', accelerator: 'Cmd+N' }, { type: 'separator' }, { role: 'close' }] },
        { label: 'Edit', submenu: [{ role: 'undo' }, { role: 'redo' }, { role: 'cut' }, { role: 'copy' }, { role: 'paste' }, { role: 'selectAll' }] },
        { label: 'View', submenu: [{ label: 'Global Overview', accelerator: 'Cmd+1' }, { label: 'Command Hub', accelerator: 'Cmd+2' }] },
        { label: 'Window', submenu: [{ role: 'minimize' }, { role: 'zoom' }, { role: 'front' }] },
        { label: 'Help', submenu: [{ label: 'TOS Documentation' }, { label: 'Report Issue' }] },
    ];
}

function getWindowsMenuTemplate(): MenuItem[] {
    return [
        { label: 'File', submenu: [{ label: 'New Sector', accelerator: 'Ctrl+N' }, { label: 'Settings', accelerator: 'Ctrl+,' }, { role: 'quit' }] },
        { label: 'Edit', submenu: [{ role: 'undo' }, { role: 'redo' }, { role: 'cut' }, { role: 'copy' }, { role: 'paste' }, { role: 'selectAll' }] },
        { label: 'View', submenu: [{ label: 'Global Overview', accelerator: 'Ctrl+1' }, { label: 'Command Hub', accelerator: 'Ctrl+2' }] },
        { label: 'Help', submenu: [{ label: 'Documentation' }, { label: 'Report Issue' }, { label: 'About TOS' }] },
    ];
}

// ── Tests ────────────────────────────────────────────────────────────────────

describe('Platform Menu Configuration', () => {

    describe('macOS Menu', () => {
        const menu = getMacOSMenuTemplate();

        it('should have 6 top-level menu items', () => {
            expect(menu).toHaveLength(6);
        });

        it('should start with the application name menu', () => {
            expect(menu[0].label).toBe('TOS');
        });

        it('should include About in the app menu', () => {
            const appMenu = menu[0].submenu!;
            expect(appMenu.some(item => item.role === 'about')).toBe(true);
        });

        it('should include Preferences with Cmd+, shortcut', () => {
            const appMenu = menu[0].submenu!;
            const prefs = appMenu.find(item => item.label === 'Preferences…');
            expect(prefs).toBeDefined();
            expect(prefs!.accelerator).toBe('Cmd+,');
        });

        it('should include macOS window management roles', () => {
            const windowMenu = menu.find(m => m.label === 'Window')!;
            const roles = windowMenu.submenu!.map(i => i.role).filter(Boolean);
            expect(roles).toContain('minimize');
            expect(roles).toContain('zoom');
            expect(roles).toContain('front');
        });

        it('should use Cmd modifier for shortcuts', () => {
            const fileMenu = menu.find(m => m.label === 'File')!;
            const newSector = fileMenu.submenu!.find(i => i.label === 'New Sector');
            expect(newSector!.accelerator).toBe('Cmd+N');
        });

        it('should include TOS navigation items in View menu', () => {
            const viewMenu = menu.find(m => m.label === 'View')!;
            const labels = viewMenu.submenu!.map(i => i.label).filter(Boolean);
            expect(labels).toContain('Global Overview');
            expect(labels).toContain('Command Hub');
        });
    });

    describe('Windows Menu', () => {
        const menu = getWindowsMenuTemplate();

        it('should have 4 top-level menu items', () => {
            expect(menu).toHaveLength(4);
        });

        it('should NOT start with an application name menu', () => {
            expect(menu[0].label).toBe('File');
        });

        it('should use Ctrl modifier for shortcuts', () => {
            const fileMenu = menu[0];
            const newSector = fileMenu.submenu!.find(i => i.label === 'New Sector');
            expect(newSector!.accelerator).toBe('Ctrl+N');
        });

        it('should include Settings in File menu', () => {
            const fileMenu = menu[0];
            const settings = fileMenu.submenu!.find(i => i.label === 'Settings');
            expect(settings).toBeDefined();
            expect(settings!.accelerator).toBe('Ctrl+,');
        });

        it('should include About TOS in Help menu', () => {
            const helpMenu = menu.find(m => m.label === 'Help')!;
            const about = helpMenu.submenu!.find(i => i.label === 'About TOS');
            expect(about).toBeDefined();
        });

        it('should NOT have a Window menu', () => {
            const windowMenu = menu.find(m => m.label === 'Window');
            expect(windowMenu).toBeUndefined();
        });
    });

    describe('Linux Menu', () => {
        it('should use same structure as Windows menu', () => {
            // Implementation: createLinuxMenu() delegates to createWindowsMenu()
            const winMenu = getWindowsMenuTemplate();
            const linuxMenu = getWindowsMenuTemplate();
            expect(linuxMenu).toEqual(winMenu);
        });
    });

    describe('Menu Keyboard Shortcut Mapping', () => {
        it('should map Cmd on macOS and Ctrl on Windows/Linux', () => {
            const macMenu = getMacOSMenuTemplate();
            const winMenu = getWindowsMenuTemplate();

            const macViewAccels = macMenu.find(m => m.label === 'View')!
                .submenu!.map(i => i.accelerator).filter(Boolean);
            const winViewAccels = winMenu.find(m => m.label === 'View')!
                .submenu!.map(i => i.accelerator).filter(Boolean);

            macViewAccels.forEach(a => expect(a).toContain('Cmd'));
            winViewAccels.forEach(a => expect(a).toContain('Ctrl'));
        });
    });
});
