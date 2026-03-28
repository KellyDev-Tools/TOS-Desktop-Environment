import { test, expect, type Page } from '@playwright/test';

/**
 * Phase 3 — PTY Execution Matrix
 *
 * Verifies that shell commands typed into the Command Hub are executed by the
 * real Rust Brain backend (via PTY) and that both stdout output and process
 * lifecycle events are correctly streamed back up through WebSockets to the UI.
 *
 * Selectors used:
 *  .status-badge        — Brain connection banner
 *  .sector-tile         — Clickable sector tile in Global Overview (navigates to Level 2)
 *  .pill-btn[CMD]       — CMD mode toggle in the bottom bezel
 *  #cmd-input           — Persistent Unified Prompt text input
 *  .term-line           — Individual terminal output line (CommandHub right column)
 *  .activity-item       — Process card in ACT (activity) mode
 *  .proc-name           — Process name span inside an activity-item
 *  .tactical-context-menu  — Floating kill/inspect menu
 */

// ---------------------------------------------------------------------------
// Shared helper: boot the app past onboarding, into Level 2 Command Hub.
// ---------------------------------------------------------------------------
async function bootToCommandHub(page: Page) {
    await page.addInitScript(() => {
        // Use the exact keys the app reads from localStorage.
        window.localStorage.setItem('tos.onboarding.first_run_complete', 'true');
        window.localStorage.setItem('tos.onboarding.wizard_complete', 'true');
    });

    await page.goto('/');

    // Wait for Brain connection badge.
    const status = page.locator('.status-badge', { hasText: /BRAIN/i });
    await expect(status).toBeVisible({ timeout: 15000 });

    // Skip cinematic intro overlay.
    await page.keyboard.press('Escape');
    await page.waitForTimeout(400);

    // Skip onboarding overlay if it still appears (e.g. on first connection).
    const skipBtn = page.locator('button:has-text("SKIP TOUR")');
    const skipVisible = await skipBtn.isVisible().catch(() => false);
    if (skipVisible) await skipBtn.click();

    // Wait for the Global Overview sector tile to appear, then click to zoom in.
    const sectorTile = page.locator('.sector-tile').first();
    await expect(sectorTile).toBeVisible({ timeout: 8000 });
    await sectorTile.click();

    // After clicking a sector tile we should be in Level 2 — wait for #cmd-input.
    const cmdInput = page.locator('input#cmd-input');
    await expect(cmdInput).toBeVisible({ timeout: 5000 });

    // Ensure CMD prompt mode is active (not AI or SEARCH).
    const cmdPill = page.locator('.pill-btn', { hasText: 'CMD' });
    const cmdPillVisible = await cmdPill.isVisible().catch(() => false);
    if (cmdPillVisible) await cmdPill.click();
}

// ---------------------------------------------------------------------------
// 3-A  Command roundtrip — echo output appears in .term-line
// ---------------------------------------------------------------------------
test.describe('PTY Command Roundtrip', () => {
    test('should echo output back into terminal UI', async ({ page }) => {
        await bootToCommandHub(page);

        const cmdInput = page.locator('input#cmd-input');

        const TAG = 'TOS_TEST_E2E_ALIVE';
        await cmdInput.fill(`echo "${TAG}"`);
        await cmdInput.press('Enter');

        // Rust PTY should stream the output back; it appears as a .term-line.
        const termLine = page.locator('.term-line', { hasText: TAG }).first();
        await expect(termLine).toBeVisible({ timeout: 10000 });
    });
});

// ---------------------------------------------------------------------------
// 3-B  Process registration — sleep appears in ACT listing
// ---------------------------------------------------------------------------
test.describe('PTY Process Registration', () => {
    test('should register a long-running process in activity listing', async ({ page }) => {
        await bootToCommandHub(page);

        const cmdInput = page.locator('input#cmd-input');

        // Start a 15-second background job.
        await cmdInput.fill('sleep 15 &');
        await cmdInput.press('Enter');

        // Give the Brain heartbeat loop 2 s to refresh activity listings.
        await page.waitForTimeout(2000);

        // The Brain populates activity_listing on a 1-Hz heartbeat.
        // Navigate to ACT mode via the Expanded Bezel.
        await page.evaluate(() => {
            document.querySelector('.lcars-bar.lcars-bar-bottom')?.dispatchEvent(
                new MouseEvent('click', { bubbles: true })
            );
        });
        const actBtn = page.locator('.expanded-bezel-overlay button', { hasText: 'ACT' });
        await expect(actBtn).toBeVisible({ timeout: 5000 });
        await actBtn.click();
        await page.waitForTimeout(1500);

        // Close the bezel to reveal the activity view
        const closeBtn = page.locator('.expanded-bezel-overlay button', { hasText: 'CLOSE' });
        if (await closeBtn.isVisible()) await closeBtn.click();

        // Wait for the activity-item list to populate.
        const activityItems = page.locator('.activity-item');
        await expect(activityItems.first()).toBeVisible({ timeout: 10000 });

        // At least one entry should contain "sleep".
        const sleepEntry = page.locator('.activity-item .proc-name', { hasText: /sleep/i });
        await expect(sleepEntry).toBeVisible({ timeout: 10000 });
    });
});

// ---------------------------------------------------------------------------
// 3-C  Tactical context-menu kill signal
// ---------------------------------------------------------------------------
test.describe('PTY Tactical Context-Menu Signal', () => {
    test('should send SIGKILL to a process via the tactical context menu', async ({ page }) => {
        await bootToCommandHub(page);

        const cmdInput = page.locator('input#cmd-input');

        // Spawn the target process.
        await cmdInput.fill('sleep 30 &');
        await cmdInput.press('Enter');

        // Give the Brain heartbeat loop time to register the process.
        await page.waitForTimeout(2000);

        // Switch to ACT mode to surface the activity listing from Expanded Bezel.
        await page.evaluate(() => {
            document.querySelector('.lcars-bar.lcars-bar-bottom')?.dispatchEvent(
                new MouseEvent('click', { bubbles: true })
            );
        });
        const actBtn = page.locator('.expanded-bezel-overlay button', { hasText: 'ACT' });
        await expect(actBtn).toBeVisible({ timeout: 5000 });
        await actBtn.click();
        await page.waitForTimeout(1500);

        const closeBtn = page.locator('.expanded-bezel-overlay button', { hasText: 'CLOSE' });
        if (await closeBtn.isVisible()) await closeBtn.click();

        const sleepEntry = page.locator('.activity-item').filter({ hasText: /sleep/i }).first();
        await expect(sleepEntry).toBeVisible({ timeout: 10000 });

        // Right-click to open the tactical context menu.
        await sleepEntry.click({ button: 'right' });

        const contextMenu = page.locator('.tactical-context-menu');
        await expect(contextMenu).toBeVisible({ timeout: 3000 });

        // Click Force Kill.
        const killBtn = contextMenu.locator('button:has-text("[SIGNAL] Force Kill")');
        await expect(killBtn).toBeVisible({ timeout: 2000 });
        await killBtn.click();

        // The menu should dismiss immediately.
        await expect(contextMenu).toBeHidden({ timeout: 3000 });

        // After kill, the sleep entry should disappear on the next Brain sync cycle.
        await expect(sleepEntry).toBeHidden({ timeout: 8000 });
    });
});
