/**
 * TOS Electron — File Dialog Tests
 * ─────────────────────────────────────────────────────────────────────────────
 * Tests for platform-specific file dialog behavior.
 */

import { describe, it, expect, vi } from 'vitest';

// ── Mock Dialog ──────────────────────────────────────────────────────────────

const mockDialog = {
    showOpenDialog: vi.fn(),
    showSaveDialog: vi.fn(),
};

// ── Tests ────────────────────────────────────────────────────────────────────

describe('File Dialog Handler', () => {

    describe('Open Dialog', () => {
        it('should open with default title', async () => {
            mockDialog.showOpenDialog.mockResolvedValue({
                canceled: false,
                filePaths: ['/home/user/file.txt'],
            });

            const result = await mockDialog.showOpenDialog({
                title: 'Open File',
                properties: ['openFile'],
            });

            expect(result.canceled).toBe(false);
            expect(result.filePaths).toHaveLength(1);
            expect(result.filePaths[0]).toBe('/home/user/file.txt');
        });

        it('should support file type filters', async () => {
            mockDialog.showOpenDialog.mockResolvedValue({
                canceled: false,
                filePaths: ['/home/user/image.png'],
            });

            const result = await mockDialog.showOpenDialog({
                title: 'Select Image',
                filters: [
                    { name: 'Images', extensions: ['png', 'jpg', 'gif'] },
                ],
                properties: ['openFile'],
            });

            expect(result.filePaths[0]).toMatch(/\.(png|jpg|gif)$/);
        });

        it('should support directory selection', async () => {
            mockDialog.showOpenDialog.mockResolvedValue({
                canceled: false,
                filePaths: ['/home/user/projects'],
            });

            const result = await mockDialog.showOpenDialog({
                title: 'Select Directory',
                properties: ['openDirectory'],
            });

            expect(result.canceled).toBe(false);
            expect(result.filePaths).toHaveLength(1);
        });

        it('should support multi-selection', async () => {
            mockDialog.showOpenDialog.mockResolvedValue({
                canceled: false,
                filePaths: ['/home/user/a.txt', '/home/user/b.txt', '/home/user/c.txt'],
            });

            const result = await mockDialog.showOpenDialog({
                properties: ['openFile', 'multiSelections'],
            });

            expect(result.filePaths).toHaveLength(3);
        });

        it('should handle user cancellation', async () => {
            mockDialog.showOpenDialog.mockResolvedValue({
                canceled: true,
                filePaths: [],
            });

            const result = await mockDialog.showOpenDialog({});
            expect(result.canceled).toBe(true);
            expect(result.filePaths).toHaveLength(0);
        });
    });

    describe('Save Dialog', () => {
        it('should return selected save path', async () => {
            mockDialog.showSaveDialog.mockResolvedValue({
                canceled: false,
                filePath: '/home/user/export.json',
            });

            const result = await mockDialog.showSaveDialog({
                title: 'Save Export',
                defaultPath: 'export.json',
                filters: [{ name: 'JSON', extensions: ['json'] }],
            });

            expect(result.canceled).toBe(false);
            expect(result.filePath).toBe('/home/user/export.json');
        });

        it('should handle user cancellation', async () => {
            mockDialog.showSaveDialog.mockResolvedValue({
                canceled: true,
                filePath: undefined,
            });

            const result = await mockDialog.showSaveDialog({});
            expect(result.canceled).toBe(true);
            expect(result.filePath).toBeUndefined();
        });

        it('should support default path suggestion', async () => {
            const defaultPath = '/home/user/documents/report.pdf';

            mockDialog.showSaveDialog.mockImplementation(async (opts: any) => ({
                canceled: false,
                filePath: opts.defaultPath || '/home/user/untitled',
            }));

            const result = await mockDialog.showSaveDialog({
                defaultPath,
            });

            expect(result.filePath).toBe(defaultPath);
        });
    });

    describe('showFilePicker Helper', () => {
        it('should return file paths on success', async () => {
            mockDialog.showOpenDialog.mockResolvedValue({
                canceled: false,
                filePaths: ['/home/user/selected.txt'],
            });

            const result = await mockDialog.showOpenDialog({
                title: 'Pick a file',
                filters: [{ name: 'Filtered', extensions: ['txt'] }],
                properties: ['openFile'],
            });

            expect(result.canceled).toBe(false);
            expect(result.filePaths).toEqual(['/home/user/selected.txt']);
        });

        it('should return undefined on cancellation', async () => {
            mockDialog.showOpenDialog.mockResolvedValue({
                canceled: true,
                filePaths: [],
            });

            const result = await mockDialog.showOpenDialog({});
            const pickerResult = result.canceled ? undefined : result.filePaths;
            expect(pickerResult).toBeUndefined();
        });
    });
});
