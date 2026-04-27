import { expect, type Page } from '@playwright/test';

/**
 * Robustly navigate past onboarding to Level 2 Command Hub.
 */
export async function bootToCommandHub(page: Page) {
    console.log('[bootToCommandHub] Setting localStorage flags...');
    await page.addInitScript(() => {
        window.localStorage.clear(); // Start fresh
        window.localStorage.setItem('tos.onboarding.first_run_complete', 'true');
        window.localStorage.setItem('tos.onboarding.wizard_complete', 'true');
    });

    console.log('[bootToCommandHub] Navigating to /...');
    await page.goto('/');

    // Wait for Brain connection badge.
    console.log('[bootToCommandHub] Waiting for Brain connection...');
    const status = page.locator('.brain-status .status-badge');
    await expect(status).toBeVisible({ timeout: 15000 });
    // Wait until it doesn't say DISCONNECTED anymore
    await expect(status).not.toContainText('DISCONNECTED', { timeout: 15000 });

    // Skip cinematic intro overlay if it appears.
    console.log('[bootToCommandHub] Pressing Escape to skip intro...');
    await page.keyboard.press('Escape');
    await page.waitForTimeout(400);

    // Skip onboarding overlay if it still appears.
    console.log('[bootToCommandHub] Checking for Skip Tour button...');
    const skipBtn = page.locator('button:has-text("SKIP TOUR")');
    const skipVisible = await skipBtn.isVisible().catch(() => false);
    if (skipVisible) {
        console.log('[bootToCommandHub] Clicking Skip Tour...');
        await skipBtn.click();
    }

    // Wait for the Global Overview sector tile to appear, then click to zoom in.
    console.log('[bootToCommandHub] Waiting for sector tile...');
    const sectorTile = page.locator('.sector-grid .sector-tile').first();
    await expect(sectorTile).toBeVisible({ timeout: 10000 });
    
    // Wait for scaleIn animation to finish
    await page.waitForTimeout(600);
    
    console.log('[bootToCommandHub] Clicking sector tile...');
    await sectorTile.click({ force: true });

    // Wait for mode transition to be reflected in title
    console.log('[bootToCommandHub] Waiting for COMMAND HUB title...');
    const title = page.locator('.viewport-title');
    await expect(title).toContainText('COMMAND HUB', { timeout: 10000 });

    // After clicking a sector tile we should be in Level 2 — wait for #cmd-input.
    console.log('[bootToCommandHub] Waiting for #cmd-input...');
    const cmdInput = page.locator('input#cmd-input');
    await expect(cmdInput).toBeVisible({ timeout: 10000 });

    // Wait for footer to expand (mode 'hubs' makes it expanded)
    const footer = page.locator('.lcars-footer');
    await expect(footer).not.toHaveClass(/collapsed-locked/, { timeout: 5000 });
    await page.waitForTimeout(400); // Animation buffer

    // Ensure CMD prompt mode is active.
    console.log('[bootToCommandHub] Ensuring CMD mode...');
    const cmdPill = page.locator('.pill-btn', { hasText: 'CMD' });
    const cmdPillVisible = await cmdPill.isVisible().catch(() => false);
    if (cmdPillVisible) {
        console.log('[bootToCommandHub] Clicking CMD pill...');
        await cmdPill.click({ force: true });
    }
    console.log('[bootToCommandHub] Done.');
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
    const exactMode = new RegExp(`^\\s*${mode}\\s*$`);
    const levelBtn = page.locator('.level-btn, .market-btn, .bezel-item', { hasText: exactMode });
    if (await levelBtn.first().isVisible().catch(() => false)) {
        await levelBtn.first().click();
        await page.waitForTimeout(500);
        return;
    }

    // Try prompt pills
    const pill = page.locator('.pill-btn', { hasText: exactMode });
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
