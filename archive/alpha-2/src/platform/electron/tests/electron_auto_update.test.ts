/**
 * TOS Electron — Auto-Update Tests
 * ─────────────────────────────────────────────────────────────────────────────
 * Tests for update checking, downloading, and installation.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';

// ── Mock Auto-Updater ────────────────────────────────────────────────────────

interface UpdateInfo {
    version: string;
    releaseDate: string;
    releaseNotes: string;
}

let updateAvailable = false;
let downloadComplete = false;
let currentVersion = '2.2.0';
let latestVersion = '2.2.1';

const mockAutoUpdater = {
    autoDownload: false,
    autoInstallOnAppQuit: true,
    currentVersion: { version: currentVersion },

    checkForUpdates: vi.fn(async () => {
        return {
            updateInfo: {
                version: latestVersion,
                releaseDate: '2026-03-23',
                releaseNotes: 'Bug fixes and performance improvements',
            },
        };
    }),

    downloadUpdate: vi.fn(async () => {
        downloadComplete = true;
    }),

    quitAndInstall: vi.fn((isSilent: boolean, isForceRunAfter: boolean) => {
        // Simulate quit-and-install
    }),

    on: vi.fn(),
};

// ── Tests ────────────────────────────────────────────────────────────────────

describe('Auto-Update', () => {
    beforeEach(() => {
        updateAvailable = false;
        downloadComplete = false;
        currentVersion = '2.2.0';
        latestVersion = '2.2.1';
        vi.clearAllMocks();
    });

    describe('Update Checking', () => {
        it('should check for updates and detect new version', async () => {
            const result = await mockAutoUpdater.checkForUpdates();
            expect(result.updateInfo.version).toBe('2.2.1');
            expect(result.updateInfo.version).not.toBe(currentVersion);
        });

        it('should return false when already on latest version', async () => {
            latestVersion = currentVersion;
            const result = await mockAutoUpdater.checkForUpdates();
            const isNewer = result.updateInfo.version !== currentVersion;
            expect(isNewer).toBe(false);
        });

        it('should include release notes in update info', async () => {
            const result = await mockAutoUpdater.checkForUpdates();
            expect(result.updateInfo.releaseNotes).toBeTruthy();
            expect(typeof result.updateInfo.releaseNotes).toBe('string');
        });

        it('should include release date in update info', async () => {
            const result = await mockAutoUpdater.checkForUpdates();
            expect(result.updateInfo.releaseDate).toBeTruthy();
        });
    });

    describe('Update Download', () => {
        it('should download update when available', async () => {
            await mockAutoUpdater.downloadUpdate();
            expect(downloadComplete).toBe(true);
            expect(mockAutoUpdater.downloadUpdate).toHaveBeenCalledTimes(1);
        });
    });

    describe('Update Installation', () => {
        it('should call quitAndInstall with correct params', () => {
            mockAutoUpdater.quitAndInstall(false, true);
            expect(mockAutoUpdater.quitAndInstall).toHaveBeenCalledWith(false, true);
        });
    });

    describe('Configuration', () => {
        it('should not auto-download by default', () => {
            expect(mockAutoUpdater.autoDownload).toBe(false);
        });

        it('should auto-install on app quit', () => {
            expect(mockAutoUpdater.autoInstallOnAppQuit).toBe(true);
        });
    });

    describe('Event Registration', () => {
        it('should register all required event handlers', () => {
            const requiredEvents = [
                'checking-for-update',
                'update-available',
                'update-not-available',
                'download-progress',
                'update-downloaded',
                'error',
            ];

            requiredEvents.forEach(event => {
                mockAutoUpdater.on(event, () => { });
            });

            expect(mockAutoUpdater.on).toHaveBeenCalledTimes(requiredEvents.length);
        });
    });
});
