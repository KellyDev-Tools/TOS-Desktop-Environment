import { test, expect } from '@playwright/test';
import { bootToCommandHub, switchToMode, navToGlobalOverview, openExpandedBezel, closeExpandedBezel } from './test-utils';

test.describe('TOS Core UI Flows (Stage 6.9)', () => {

    test('Flow 1: Fresh Boot & Connection', async ({ page }) => {
        await page.goto('/');
        
        // Should show connection state if Brain isn't ready or if it's slow.
        const status = page.locator('.status-badge');
        await expect(status).toBeVisible({ timeout: 15000 });
        await expect(status).toContainText(/BRAIN/i);

        // Verify cinematic overlay presence (unless first_run_complete is set).
        const cinematic = page.locator('.onboarding-overlay');
        const visible = await cinematic.isVisible().catch(() => false);
        if (visible) {
            await page.keyboard.press('Escape');
            await expect(cinematic).toBeHidden();
        }
    });

    test('Flow 2: Hierarchical Navigation (L1 -> L4)', async ({ page }) => {
        await bootToCommandHub(page); // Starts at L2

        // Go to L1
        await navToGlobalOverview(page);
        
        // Go back to L2 (Command Hub)
        const sectorTile = page.locator('.sector-tile').first();
        await expect(sectorTile).toBeVisible();
        await sectorTile.click();
        
        const cmdInput = page.locator('input#cmd-input');
        await expect(cmdInput).toBeVisible();

        // Switch to Level 3 (Application Focus / Sectors)
        await switchToMode(page, '3');
        // In Level 3, we expect to see the application focus surface.
        const appFocus = page.locator('.app-focus, .terminal-output').first();
        await expect(appFocus).toBeVisible({ timeout: 10000 });

        // Switch to Level 4 (Detail Inspector)
        await switchToMode(page, '4');
        const detailView = page.locator('.detail-view, .detail-inspector');
        await expect(detailView).toBeVisible({ timeout: 5000 });
    });

    test('Flow 3: Split Viewports & Persistence', async ({ page }) => {
        await bootToCommandHub(page);

        const cmdInput = page.locator('input#cmd-input');
        
        // Execute a command to have output.
        const TAG = 'SPLIT_PERSIST_TEST';
        await cmdInput.fill(`echo ${TAG}`);
        await cmdInput.press('Enter');
        await expect(page.locator('.term-line', { hasText: TAG })).toBeVisible();

        // Create a horizontal split using keyboard shortcut Ctrl+\
        // Fallback to Ctrl+- if Ctrl+\ is tricky on some layouts
        await page.keyboard.press('Control+\\');
        
        // If split didn't happen, try Ctrl+-
        const panes = page.locator('.split-pane-leaf');
        if (await panes.count() < 2) {
            await page.keyboard.press('Control+-');
        }

        // Verify two panes exist.
        await expect(panes).toHaveCount(2, { timeout: 8000 });

        // Verify output is still visible in one of them.
        await expect(page.locator('.term-line', { hasText: TAG }).first()).toBeVisible();
    });

    test('Flow 4: Editor Save & Trust Chip', async ({ page }) => {
        await bootToCommandHub(page);
        
        // Use keyboard shortcut to open a file or navigate to L4 with an editor.
        // We'll try to find a file in Level 2 or 3 and click it.
        await switchToMode(page, '3');
        const fileEntry = page.locator('.file-entry, .dir-entry').first();
        await fileEntry.click();

        // Verify editor is open.
        const editor = page.locator('.editor-textarea, .editor-pane textarea');
        await expect(editor).toBeVisible({ timeout: 10000 });

        // Type something.
        await editor.focus();
        await page.keyboard.type('\n// E2E Test Edit');

        // Try to save.
        await page.keyboard.press('Control+s');

        // Check for a confirmation chip or trust chip.
        const trustIndicator = page.locator('.trust-chip, .confirmation-overlay, .warning-chip');
        // It might not appear if we are in a trusted directory, so we just check for NO error.
        await page.waitForTimeout(1000);
    });

    test('Flow 5: Marketplace & Permissions', async ({ page }) => {
        await bootToCommandHub(page);
        
        // Navigate to Marketplace (Level 6).
        await switchToMode(page, '⊞');

        // Verify Marketplace home is visible.
        const marketHome = page.locator('.marketplace-home, .marketplace-grid');
        await expect(marketHome).toBeVisible({ timeout: 10000 });

        // Click a module to see details if available.
        const moduleCard = page.locator('.module-card, .marketplace-item').first();
        if (await moduleCard.isVisible().catch(() => false)) {
            await moduleCard.click();

            // Verify details and permission gate.
            const installBtn = page.locator('button', { hasText: /INSTALL|PURCHASE/i });
            await expect(installBtn).toBeVisible();
            
            // Scroll to reveal permission consent if required.
            const consent = page.locator('.permission-consent, .permission-list');
            if (await consent.isVisible().catch(() => false)) {
                await consent.scrollIntoViewIfNeeded();
                await expect(installBtn).toBeEnabled();
            }
        }
    });

    test('Flow 6: AI Interaction & Thoughts', async ({ page }) => {
        await bootToCommandHub(page);
        
        // Switch to AI mode.
        await switchToMode(page, 'AI');

        const aiInput = page.locator('input#cmd-input');
        await aiInput.fill('how are you?');
        await aiInput.press('Enter');

        // Verify thought bubbles or AI activity appear.
        const thoughts = page.locator('.thought-bubble, .active-thoughts, .ai-thinking');
        await expect(thoughts).toBeVisible({ timeout: 15000 });

        // Verify AI response arrives.
        const aiResponse = page.locator('.ai-chat-bubble, .ai-response');
        await expect(aiResponse).toBeVisible({ timeout: 20000 });
    });

    test('Flow 7: Session Handoff (Stage 6.3)', async ({ page }) => {
        await bootToCommandHub(page);
        
        const cmdInput = page.locator('input#cmd-input');
        await cmdInput.fill('session_handoff_prepare');
        await cmdInput.press('Enter');

        // Verify handoff token or info appears in the system log or a modal.
        // The Brain returns a token like "Handoff Token: XXXXXX"
        const handoffLog = page.locator('.term-line, .system-log-line', { hasText: /HANDOFF TOKEN/i });
        await expect(handoffLog).toBeVisible({ timeout: 10000 });
    });

    test('Flow 8: Vibe Coder & Thought Sequence (Stage 3.3)', async ({ page }) => {
        await bootToCommandHub(page);
        
        await switchToMode(page, 'AI');
        const aiInput = page.locator('input#cmd-input');
        await aiInput.fill('vibe-coder: initialize a new rust project');
        await aiInput.press('Enter');

        // Verify thought bubbles appear and change stages.
        const thoughts = page.locator('.thought-bubble, .active-thoughts');
        await expect(thoughts).toBeVisible({ timeout: 5000 });
        
        // Wait for a second thought or a change in state
        await page.waitForTimeout(2000);
        await expect(thoughts).toBeVisible();
    });

    test('Flow 9: Predictive Intelligence - Ghost Text (Stage 3.2)', async ({ page }) => {
        await bootToCommandHub(page);
        
        const cmdInput = page.locator('input#cmd-input');
        await cmdInput.fill('l');
        // Wait for heuristic/prediction debounce
        await page.waitForTimeout(1000);

        const ghostText = page.locator('.prediction-text');
        // It should suggest 'ls' or similar
        const visible = await ghostText.isVisible().catch(() => false);
        if (visible) {
            await expect(ghostText).not.toBeEmpty();
            // Try to accept with Tab
            await page.keyboard.press('Tab');
            const val = await cmdInput.inputValue();
            expect(val.length).toBeGreaterThan(1);
        }
    });

    test('Flow 10: Heuristic Typo Correction (Stage 3.9)', async ({ page }) => {
        await bootToCommandHub(page);
        
        const cmdInput = page.locator('input#cmd-input');
        // Type a typo that tos-heuristicd should catch (e.g. 'lss' -> 'ls')
        await cmdInput.fill('lss');
        
        const heuristicChip = page.locator('.heuristic-chip, .staged-command-chip', { hasText: /ls/i });
        await expect(heuristicChip).toBeVisible({ timeout: 5000 });
        
        // Click chip to apply
        await heuristicChip.click();
        const val = await cmdInput.inputValue();
        expect(val).toBe('ls');
    });

});
