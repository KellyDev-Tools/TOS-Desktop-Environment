/**
 * TOS Electron — Brain Discovery Tests
 * ─────────────────────────────────────────────────────────────────────────────
 * Tests for Brain instance discovery via local, saved hosts, mDNS, and manual.
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import * as fs from 'fs';
import * as net from 'net';
import * as dgram from 'dgram';

// ── Mocks ────────────────────────────────────────────────────────────────────

vi.mock('electron', () => ({
    ipcMain: { handle: vi.fn(), on: vi.fn() }
}));

vi.mock('fs', () => ({
    existsSync: vi.fn(),
    readFileSync: vi.fn(),
    writeFileSync: vi.fn(),
    mkdirSync: vi.fn()
}));

vi.mock('net', () => {
    return {
        Socket: vi.fn().mockImplementation(function () {
            return {
                setTimeout: vi.fn(),
                on: vi.fn(),
                connect: vi.fn(),
                destroy: vi.fn(),
            };
        })
    };
});

vi.mock('dgram', () => {
    return {
        createSocket: vi.fn().mockImplementation(function () {
            return {
                on: vi.fn(),
                bind: vi.fn(),
                addMembership: vi.fn(),
                send: vi.fn(),
                close: vi.fn(),
            };
        })
    };
});

// Import after mocks
import { loadSavedHosts, runFullDiscovery, discoverLocalBrain, scanMdns } from '../src/brain-discovery';

// ── Tests ────────────────────────────────────────────────────────────────────

describe('Brain Discovery', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    afterEach(() => {
        vi.restoreAllMocks();
    });

    describe('Saved Hosts (loadSavedHosts)', () => {
        it('should parse TOML-format saved hosts correctly', () => {
            const tomlContent = `[[remote]]
name = "Work Server"
host = "192.168.1.50"
port = 7001

[[remote]]
name = "Home Server"
host = "10.0.0.10"
port = 7002`;

            vi.mocked(fs.existsSync).mockReturnValue(true);
            vi.mocked(fs.readFileSync).mockReturnValue(tomlContent);

            const hosts = loadSavedHosts();

            expect(hosts).toHaveLength(2);
            expect(hosts[0].name).toBe('Work Server');
            expect(hosts[0].host).toBe('192.168.1.50');
            expect(hosts[0].port).toBe(7001);
            expect(hosts[1].name).toBe('Home Server');
            expect(hosts[1].host).toBe('10.0.0.10');
            expect(hosts[1].port).toBe(7002);
        });

        it('should handle empty saved hosts file', () => {
            vi.mocked(fs.existsSync).mockReturnValue(true);
            vi.mocked(fs.readFileSync).mockReturnValue('');

            const hosts = loadSavedHosts();
            expect(hosts).toHaveLength(0);
        });

        it('should handle missing file gracefully', () => {
            vi.mocked(fs.existsSync).mockReturnValue(false);

            const hosts = loadSavedHosts();
            expect(hosts).toHaveLength(0);
        });
    });

    describe('Local Brain Detection (discoverLocalBrain)', () => {
        it('should return null if socket connection fails/times out', async () => {
            // Mock Socket.on to simulate a timeout
            vi.mocked(net.Socket).mockImplementation(function () {
                return {
                    setTimeout: vi.fn(),
                    on: vi.fn((event, cb) => {
                        if (event === 'timeout') {
                            setTimeout(cb, 5); // trigger timeout
                        }
                    }),
                    connect: vi.fn(),
                    destroy: vi.fn(),
                } as any;
            } as any);

            const result = await discoverLocalBrain();
            expect(result).toBeNull();
        });

        it('should return BrainInstance if socket connection succeeds', async () => {
            // Mock Socket.on to simulate a successful connection
            const mockDestroy = vi.fn();
            let connectAttempt = 0;
            vi.mocked(net.Socket).mockImplementation(function () {
                return {
                    setTimeout: vi.fn(),
                    on: vi.fn((event, cb) => {
                        if (event === 'connect' && connectAttempt === 0) {
                            connectAttempt++;
                            setTimeout(cb, 5); // trigger successful connect
                        } else if (event === 'error') {
                            setTimeout(cb, 5);
                        }
                    }),
                    connect: vi.fn(),
                    destroy: mockDestroy,
                } as any;
            } as any);

            const result = await discoverLocalBrain();
            expect(result).not.toBeNull();
            expect(result?.host).toBe('127.0.0.1');
            expect(result?.wsPort).toBe(7001);
            expect(result?.reachable).toBe(true);
            expect(mockDestroy).toHaveBeenCalled();
        });
    });

    describe('mDNS Discovery (scanMdns)', () => {
        it('should setup dgram socket and resolve after duration', async () => {
            const mockBind = vi.fn((port, cb) => {
                if (cb) cb();
            });
            const mockClose = vi.fn();

            vi.mocked(dgram.createSocket).mockImplementation(function () {
                return {
                    on: vi.fn(),
                    bind: mockBind,
                    addMembership: vi.fn(),
                    send: vi.fn(),
                    close: mockClose,
                } as any;
            } as any);

            // Fast forward time to quicken test (vitest fake timers could be used, but we'll manually set a short duration or just await)
            const resultPromise = scanMdns(10); // 10ms timeout

            const result = await resultPromise;

            expect(mockBind).toHaveBeenCalled();
            expect(mockClose).toHaveBeenCalled();
            expect(result).toEqual([]);
        });
    });
});
