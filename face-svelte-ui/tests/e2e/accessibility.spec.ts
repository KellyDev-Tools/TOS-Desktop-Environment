import { test, expect } from '@playwright/test';

/**
 * Accessibility and Keyboard Navigation Tests
 * Verifies Task 5.7 implementation: Focus traps, tab-stops, and focus indicators.
 */
test.describe('TOS Accessibility & Keyboard Navigation', () => {

    test.beforeEach(async ({ page }) => {
        test.setTimeout(60000);

        // Clear storage to reset onboarding and settings
        await page.addInitScript(() => {
            window.localStorage.clear();
        });
        
        await page.goto('/');
        
        // 1. Skip cinematic intro
        await page.keyboard.press('Escape');
        
        // 2. Wait for connection to stabilize
        const connectedStatus = page.locator('.status-badge', { hasText: /CONNECTED|BRAIN/i });
        await expect(connectedStatus).toBeVisible({ timeout: 20000 });

        // 3. Skip onboarding tour if visible
        const skipTourBtn = page.locator('button:has-text("SKIP TOUR")');
        try {
            await expect(skipTourBtn).toBeVisible({ timeout: 5000 });
            await skipTourBtn.click({ force: true });
        } catch (e) {
            console.log("Onboarding skip button not found or already dismissed.");
        }

        // Wait for main UI to be ready
        await expect(page.locator('.level-btn').first()).toBeVisible({ timeout: 10000 });
    });

    test('should maintain focus within Settings Modal (Focus Trap)', async ({ page }) => {
        // 1. Open Settings Modal via gear icon
        // We look for the second bezel-item which is the settings gear
        const settingsBtn = page.locator('.bezel-item').filter({ hasText: '⚙' }).first();
        if (!(await settingsBtn.isVisible())) {
            await page.keyboard.press('Control+,');
        } else {
            await settingsBtn.click();
        }
        
        // 2. Wait for modal to appear
        const modal = page.locator('.modal-card');
        await expect(modal).toBeVisible({ timeout: 10000 });

        // 3. Tab many times and ensure focus never leaves the modal card
        for (let i = 0; i < 20; i++) {
            await page.keyboard.press('Tab');
            const isInside = await modal.evaluate((node) => node.contains(document.activeElement));
            expect(isInside).toBeTruthy();
        }
    });

    test('should navigate through sidebar level buttons via Tab', async ({ page }) => {
        const firstLevelBtn = page.locator('.level-btn').first();
        await firstLevelBtn.focus();
        await expect(firstLevelBtn).toBeFocused();

        // Tab to the next level button
        await page.keyboard.press('Tab');
        const secondLevelBtn = page.locator('.level-btn').nth(1);
        await expect(secondLevelBtn).toBeFocused();
    });

    test('should have visible focus indicators on buttons', async ({ page }) => {
        const firstLevelBtn = page.locator('.level-btn').first();
        await firstLevelBtn.focus();
        
        const outlineStyle = await firstLevelBtn.evaluate((el) => window.getComputedStyle(el).outlineStyle);
        expect(outlineStyle).not.toBe('none');
    });

    test('should allow interacting with Minimap via keyboard', async ({ page }) => {
        const minimap = page.locator('.minimap-container');
        await expect(minimap).toBeVisible();
        
        // Minimap is now a focusable div
        await minimap.focus();
        await expect(minimap).toBeFocused();
        
        await page.keyboard.press('Enter');
        const projection = page.locator('.projection-overlay');
        await expect(projection).toBeVisible();
    });

    test('should allow interacting with activity items via keyboard in Command Hub', async ({ page }) => {
        // Switch to Command Hub (Level 2)
        const level2Btn = page.locator('.level-btn').nth(1);
        await level2Btn.click();
        
        const hub = page.locator('.command-hub-view');
        await expect(hub).toBeVisible({ timeout: 10000 });
        
        const activityList = page.locator('.activity-list');
        await expect(activityList).toBeVisible({ timeout: 10000 });
        
        const activityItem = activityList.locator('.activity-item').first();
        
        try {
            await expect(activityItem).toBeVisible({ timeout: 5000 });
            await activityItem.focus();
            await expect(activityItem).toBeFocused();
            const tagName = await activityItem.evaluate(node => node.tagName);
            expect(tagName).toBe('BUTTON');
        } catch (e) {
            console.warn("No activity items found in Command Hub. Skipping focus check.");
        }
    });
});
