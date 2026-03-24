import { test, expect } from '@playwright/test';

test.describe('Secondary Select Infrastructure', () => {
    test.beforeEach(async ({ page }) => {
        // Mark onboarding as complete to avoid interference
        await page.addInitScript(() => {
            window.localStorage.setItem('tos.onboarding.first_run_complete', 'true');
            window.localStorage.setItem('tos.onboarding.wizard_complete', 'true');
        });
        await page.goto('/');
        await page.waitForLoadState('domcontentloaded');
        // Await global readiness
        await expect(page.locator('.lcars-container')).toBeVisible({ timeout: 15000 });
    });

    test('Sector Tile Right Click - Should open SectorContextMenu', async ({ page }) => {
        // The global overview should be open by default
        const sectorTile = page.locator('.sector-tile').first();
        await expect(sectorTile).toBeVisible({ timeout: 10000 });

        // Right click the sector tile
        await sectorTile.click({ button: 'right' });

        // Wait for Context Menu to appear
        const contextMenu = page.locator('.sector-context-menu');
        await expect(contextMenu).toBeVisible();

        // Verify Save and Load Session buttons exist
        const saveBtn = contextMenu.locator('button.menu-btn:has-text("[SAVE]")');
        const loadBtn = contextMenu.locator('button.menu-btn:has-text("[LOAD]")');
        await expect(saveBtn).toBeVisible();
        await expect(loadBtn).toBeVisible();
    });

    test('Sector Tile Long Press - Should open SectorContextMenu', async ({ page }) => {
        const sectorTile = page.locator('.sector-tile').first();
        await expect(sectorTile).toBeVisible({ timeout: 10000 });

        // Simulate Long press via touch (longpress.ts acts on mousedown/touchstart)
        const box = await sectorTile.boundingBox();
        if (box) {
            await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
            await page.mouse.down();
            await page.waitForTimeout(650); // longer than 600ms threshold
            await page.mouse.up();
        }

        const contextMenu = page.locator('.sector-context-menu');
        await expect(contextMenu).toBeVisible();
    });

    test('Command Hub Activity Item Right Click - Should open TacticalContextMenu', async ({ page }) => {
        // Navigate to Command Hub (Level 2)
        await page.keyboard.press('Control+Digit2');

        // Find activity item
        const activityItem = page.locator('.activity-item').first();
        await expect(activityItem).toBeVisible({ timeout: 10000 });

        // Right click
        await activityItem.click({ button: 'right' });

        // Verify Tactical Context Menu appeared
        const contextMenu = page.locator('.tactical-context-menu');
        await expect(contextMenu).toBeVisible();

        // Verify required IPC handler actions are present visually
        const inspectBtn = contextMenu.locator('button.menu-btn:has-text("[INSPECT]")');
        const killBtn = contextMenu.locator('button.critical:has-text("[SIGNAL] Force Kill")');
        await expect(inspectBtn).toBeVisible();
        await expect(killBtn).toBeVisible();
    });

    test('Command Hub Activity Item Long Press - Should open TacticalContextMenu', async ({ page }) => {
        // Navigate to Command Hub (Level 2)
        await page.keyboard.press('Control+Digit2');

        // Find activity item
        const activityItem = page.locator('.activity-item').first();
        await expect(activityItem).toBeVisible({ timeout: 10000 });

        // Simulate Long press via mouse
        const box = await activityItem.boundingBox();
        if (box) {
            await page.mouse.move(box.x + 10, box.y + 10);
            await page.mouse.down();
            // longpress threshold is 600ms
            await page.waitForTimeout(650);
            await page.mouse.up();
        }

        // Verify Tactical Context Menu appeared
        const contextMenu = page.locator('.tactical-context-menu');
        await expect(contextMenu).toBeVisible();
    });
});
