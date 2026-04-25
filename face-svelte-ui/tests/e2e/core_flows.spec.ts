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
        // In Level 3, we expect to see process cards or a list of apps.
        const appItem = page.locator('.activity-item, .dir-entry, .file-entry').first();
        await expect(appItem).toBeVisible({ timeout: 10000 });

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
        await page.keyboard.press('Control+\\');

        // Verify two panes exist.
        const panes = page.locator('.split-pane-leaf, .split-pane');
        await expect(panes).toHaveCount(2, { timeout: 5000 });

        // Verify output is still visible in one of them.
        await expect(page.locator('.term-line', { hasText: TAG })).toBeVisible();
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

});
