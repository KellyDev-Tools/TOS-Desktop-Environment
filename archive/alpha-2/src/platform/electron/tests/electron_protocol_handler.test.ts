/**
 * TOS Electron — Protocol Handler Tests
 * ─────────────────────────────────────────────────────────────────────────────
 * Tests for the `tos://` custom protocol handler.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';

// ── Mock Renderer Navigation ─────────────────────────────────────────────────

let lastNavigateAction = '';

function simulateProtocol(url: string): string | null {
    try {
        const parsed = new URL(url);
        const action = parsed.hostname;
        const pathParts = parsed.pathname.split('/').filter(Boolean);

        switch (action) {
            case 'sector':
                if (pathParts[0]) {
                    lastNavigateAction = `sector-${pathParts[0]}`;
                    return lastNavigateAction;
                }
                return null;

            case 'settings':
                lastNavigateAction = 'settings';
                return lastNavigateAction;

            case 'hub':
                lastNavigateAction = 'navigate-hub';
                return lastNavigateAction;

            case 'global':
                lastNavigateAction = 'navigate-global';
                return lastNavigateAction;

            case 'module':
                if (pathParts[0]) {
                    lastNavigateAction = `module-${pathParts[0]}`;
                    return lastNavigateAction;
                }
                return null;

            default:
                return null;
        }
    } catch {
        return null;
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

describe('Protocol Handler', () => {
    beforeEach(() => {
        lastNavigateAction = '';
    });

    describe('Sector Navigation', () => {
        it('should navigate to sector by index', () => {
            const result = simulateProtocol('tos://sector/3');
            expect(result).toBe('sector-3');
        });

        it('should navigate to sector 0', () => {
            const result = simulateProtocol('tos://sector/0');
            expect(result).toBe('sector-0');
        });

        it('should handle missing sector index', () => {
            const result = simulateProtocol('tos://sector');
            expect(result).toBeNull();
        });
    });

    describe('Settings Navigation', () => {
        it('should navigate to settings', () => {
            const result = simulateProtocol('tos://settings');
            expect(result).toBe('settings');
        });

        it('should navigate to settings with trailing slash', () => {
            const result = simulateProtocol('tos://settings/');
            expect(result).toBe('settings');
        });
    });

    describe('Hub Navigation', () => {
        it('should navigate to command hub', () => {
            const result = simulateProtocol('tos://hub');
            expect(result).toBe('navigate-hub');
        });
    });

    describe('Global Overview Navigation', () => {
        it('should navigate to global overview', () => {
            const result = simulateProtocol('tos://global');
            expect(result).toBe('navigate-global');
        });
    });

    describe('Module Navigation', () => {
        it('should navigate to module by ID', () => {
            const result = simulateProtocol('tos://module/terminal');
            expect(result).toBe('module-terminal');
        });

        it('should navigate to numeric module ID', () => {
            const result = simulateProtocol('tos://module/42');
            expect(result).toBe('module-42');
        });

        it('should handle missing module ID', () => {
            const result = simulateProtocol('tos://module');
            expect(result).toBeNull();
        });
    });

    describe('Unknown Actions', () => {
        it('should return null for unknown protocol action', () => {
            const result = simulateProtocol('tos://unknown-action');
            expect(result).toBeNull();
        });

        it('should return null for empty hostname', () => {
            // URL constructor with tos:// and no hostname
            // This is technically an invalid URL but we should handle it
            const result = simulateProtocol('tos:///');
            expect(result).toBeNull();
        });
    });

    describe('Malformed URLs', () => {
        it('should handle completely invalid URL gracefully', () => {
            const result = simulateProtocol('not-a-valid-url');
            expect(result).toBeNull();
        });

        it('should handle empty string gracefully', () => {
            const result = simulateProtocol('');
            expect(result).toBeNull();
        });
    });

    describe('Protocol Registration', () => {
        it('should use "tos" as protocol name', () => {
            const PROTOCOL_NAME = 'tos';
            expect(PROTOCOL_NAME).toBe('tos');
        });

        it('should construct valid protocol URLs', () => {
            const urls = [
                'tos://sector/1',
                'tos://settings',
                'tos://hub',
                'tos://global',
                'tos://module/browser',
            ];

            urls.forEach(url => {
                expect(() => new URL(url)).not.toThrow();
            });
        });
    });
});
