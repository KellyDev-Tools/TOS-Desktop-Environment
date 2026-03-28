import { test, expect } from '@playwright/test';
import fs from 'fs';

// In E2E tests, the real Rust Brain connects locally, and we assert UI changes
test.describe('TOS Full Stack E2E Infrastructure', () => {

    test('should connect to actual Rust Brain and hydrate initial Sector representation', async ({ page }) => {
        // Start by clearing storage to force onboarding if we want.
        // We actually want to clear everything so the real Brain is entirely responsible for state
        await page.addInitScript(() => {
            window.localStorage.clear();
            // We intentionally DO NOT mock window.WebSocket here!
        });

        await page.goto('/');

        // 1. Wait for connection state to resolve
        // Wait for the "BRAIN: CONNECTED" banner to appear
        const connectedStatus = page.locator('.status-badge', { hasText: /CONNECTED|BRAIN/i });
        await expect(connectedStatus).toBeVisible({ timeout: 15000 });

        // 2. We should bypass or complete the Onboarding logic
        // Skip cinematic intro by pressing a key
        await page.keyboard.press('Escape');

        // If the Onboarding module is covering the screen, we can just skip it
        const skipTourBtn = page.locator('button:has-text("SKIP TOUR")');
        await expect(skipTourBtn).toBeVisible({ timeout: 5000 });
        await skipTourBtn.click();

        // 3. The true test of full stack hydration: Does the actual Rust generated Sector Tile exist?
        // Wait for the sector-tile to populate driven entirely by an IPC parsed message

        console.log("[E2E] Waiting for sector tile...");
        const html = await page.content();
        fs.writeFileSync('dom-e2e-dump.html', html);
        console.log("[E2E] Wrote DOM to dom-e2e-dump.html");

        const sectorTile = page.locator('.sector-tile').first();
        await expect(sectorTile).toBeVisible({ timeout: 10000 });

        // Assert it has a valid name (the default sector is named 'Primary' by the Rust backend, not 'TESTING')
        const sectorName = sectorTile.locator('.sector-name');
        await expect(sectorName).not.toHaveText('TESTING'); // TESTING is the hardcoded mock name
        await expect(sectorName).toContainText('PRIMARY', { ignoreCase: true }); // The real Rust backend defaults to Primary

        // Success! Real data is hydrating the Face!
    });
});
