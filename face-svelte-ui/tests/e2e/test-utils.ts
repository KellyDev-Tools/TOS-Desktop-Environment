import { expect, type Page } from '@playwright/test';

/**
 * Robustly navigate past onboarding to Level 2 Command Hub.
 */
export async function bootToCommandHub(page: Page) {
    await page.addInitScript(() => {
        window.localStorage.setItem('tos.onboarding.first_run_complete', 'true');
        window.localStorage.setItem('tos.onboarding.wizard_complete', 'true');
    });

    await page.goto('/');

    // Wait for Brain connection badge.
    const status = page.locator('.status-badge', { hasText: /BRAIN/i });
    await expect(status).toBeVisible({ timeout: 15000 });

    // Skip cinematic intro overlay if it appears.
    await page.keyboard.press('Escape');
    await page.waitForTimeout(400);

    // Skip onboarding overlay if it still appears.
    const skipBtn = page.locator('button:has-text("SKIP TOUR")');
    const skipVisible = await skipBtn.isVisible().catch(() => false);
    if (skipVisible) await skipBtn.click();

    // Wait for the Global Overview sector tile to appear, then click to zoom in.
    const sectorTile = page.locator('.sector-tile').first();
    await expect(sectorTile).toBeVisible({ timeout: 10000 });
    await sectorTile.click();

    // After clicking a sector tile we should be in Level 2 — wait for #cmd-input.
    const cmdInput = page.locator('input#cmd-input');
    await expect(cmdInput).toBeVisible({ timeout: 8000 });

    // Ensure CMD prompt mode is active.
    const cmdPill = page.locator('.pill-btn', { hasText: 'CMD' });
    const cmdPillVisible = await cmdPill.isVisible().catch(() => false);
    if (cmdPillVisible) await cmdPill.click();
}

/**
 * Navigate to Global Overview (Level 1) from any level.
 */
export async function navToGlobalOverview(page: Page) {
    // Zoom out using Ctrl+1 shortcut or Level button
    const level1Btn = page.locator('.level-btn', { hasText: '1' });
    if (await level1Btn.isVisible().catch(() => false)) {
        await level1Btn.click();
    } else {
        await page.keyboard.press('Control+1');
    }
    await page.waitForTimeout(1000);

    // Verify sector tile is visible
    const sectorTile = page.locator('.sector-tile').first();
    await expect(sectorTile).toBeVisible({ timeout: 10000 });
}

/**
 * Switch between prompt modes (CMD, SEARCH, AI) or hierarchy levels (1, 2, 3, 4, ⊞).
 */
export async function switchToMode(page: Page, mode: 'CMD' | 'SEARCH' | 'AI' | '1' | '2' | '3' | '4' | 'L' | 'VISUAL' | 'ACT' | '⊞') {
    // Try level buttons first (sidebar) including marketplace ⊞
    const levelBtn = page.locator('.level-btn, .market-btn, .bezel-item', { hasText: mode });
    if (await levelBtn.first().isVisible().catch(() => false)) {
        await levelBtn.first().click();
        await page.waitForTimeout(500);
        return;
    }

    // Try prompt pills
    const pill = page.locator('.pill-btn', { hasText: mode });
    if (await pill.isVisible().catch(() => false)) {
        await pill.click();
        await page.waitForTimeout(500);
        return;
    }

    // Mode might be in the expanded bezel
    await openExpandedBezel(page);
    const bezelBtn = page.locator('.expanded-bezel-overlay button', { hasText: mode });
    if (await bezelBtn.isVisible().catch(() => false)) {
        await bezelBtn.click();
        await page.waitForTimeout(500);
    } else {
        // Fallback for names like 'ACT' or 'VISUAL' which might be in the bezel
        const genericBtn = page.locator('.expanded-bezel-overlay button', { hasText: new RegExp(mode, 'i') });
        if (await genericBtn.isVisible().catch(() => false)) {
            await genericBtn.click();
            await page.waitForTimeout(500);
        }
    }
}

/**
 * Open the expanded bezel overlay.
 */
export async function openExpandedBezel(page: Page) {
    const bottomBar = page.locator('.lcars-bar.lcars-bar-bottom');
    await bottomBar.click();
    const bezelOverlay = page.locator('.expanded-bezel-overlay');
    await expect(bezelOverlay).toBeVisible({ timeout: 5000 });
}

/**
 * Close the expanded bezel overlay.
 */
export async function closeExpandedBezel(page: Page) {
    const closeBtn = page.locator('.expanded-bezel-overlay button', { hasText: 'CLOSE' });
    if (await closeBtn.isVisible().catch(() => false)) {
        await closeBtn.click();
    }
}
