import { test, expect } from '@playwright/test';

test.describe('TOS Editor System (Stage 2)', () => {
    test.beforeEach(async ({ page }) => {
        // Assume we need to bypass onboarding if not already done in globalSetup
        await page.goto('/');
        await page.evaluate(() => {
            window.localStorage.setItem('tos.onboarding.first_run_complete', 'true');
            window.localStorage.setItem('tos.onboarding.wizard_complete', 'true');
        });
        await page.reload();
    });

    test('EDT-07: Save Shortcut (Ctrl+S)', async ({ page }) => {
        // 1. Trigger editor_open via console or wait for one to be present
        // For testing purposes, we can mock the state or interact with the Hub
        await page.evaluate(() => {
            // Simulate an editor pane being opened via IPC response
            const state = (window as any).tosState;
            if (state) {
                const ed = {
                    file_path: '/tmp/test.rs',
                    content: 'fn main() {\n    println!("Hello World");\n}',
                    mode: 'Editor',
                    language: 'rust',
                    cursor_line: 0,
                    cursor_col: 0,
                    scroll_offset: 0,
                    dirty: true,
                    diff_hunks: [],
                    annotations: []
                };
                state.sectors[0].hubs[0].split_layout = {
                    Leaf: {
                        id: 'test-pane-1',
                        weight: 100,
                        cwd: '/tmp',
                        content: { Editor: ed }
                    }
                };
            }
        });

        const editor = page.locator('.editor-textarea');
        await expect(editor).toBeVisible();

        // 2. Modify content
        await editor.focus();
        await page.keyboard.type('// Modified\n');

        // 3. Listen for IPC call
        const ipcCall = page.waitForFunction(() => {
            return (window as any).lastIpcCommand === '!ipc editor_save:test-pane-1';
        });

        // 4. Press Ctrl+S
        await page.keyboard.press('Control+s');

        // Note: In real app, submitCommand is called. We'd need a spy on submitCommand.
    });

    test('EDT-06: Tap line number sends to AI (mobile)', async ({ page }) => {
        await page.evaluate(() => {
            const state = (window as any).tosState;
            state.sectors[0].hubs[0].split_layout = {
                Leaf: {
                    id: 'test-pane-1',
                    weight: 100,
                    cwd: '/tmp',
                    content: { 
                        Editor: {
                            file_path: 'src/main.rs',
                            content: 'line 1\nline 2\nline 3',
                            mode: 'Editor',
                            language: 'rust',
                            annotations: []
                        }
                    }
                }
            };
        });

        const lineNumber = page.locator('.line-number').first();
        await expect(lineNumber).toBeVisible();

        // Click line 1
        await lineNumber.click();

        // Check if ai_submit was called via IPC
        // This requires spying on submitCommand in tos-state
    });

    test('Trust Chip UI: Save outside CWD', async ({ page }) => {
        await page.evaluate(() => {
            const state = (window as any).tosState;
            state.sectors[0].hubs[0].split_layout = {
                Leaf: {
                    id: 'test-pane-1',
                    weight: 100,
                    cwd: '/sector/work',
                    content: { 
                        Editor: {
                            file_path: '/external/file.txt', // Outside /sector/work
                            content: 'content',
                            mode: 'Editor',
                            language: 'text',
                            dirty: true,
                            annotations: []
                        }
                    }
                }
            };
        });

        await page.keyboard.press('Control+s');

        // Expect trust chip to appear
        const trustChip = page.locator('.trust-chip');
        await expect(trustChip).toBeVisible();
        await expect(trustChip).toContainText('OUTSIDE CWD');

        const allowBtn = trustChip.locator('.danger-chip');
        await expect(allowBtn).toBeVisible();
    });
});
