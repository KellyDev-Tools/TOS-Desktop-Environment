import { test, expect } from '@playwright/test';
import path from 'path';
import fs from 'fs';

/**
 * Phase 4 — Edge Scenarios & Service Mesh Connect
 *
 * Tests system-level behaviors that depend on multi-daemon interactions:
 *  - Trust Service blocking dangerous commands and showing amber trust chips
 *  - Split-pane detachment writing to _live.tos-session via tos-sessiond
 *  - Heuristic AI resolution surfacing staged command chips from tos-heuristicd
 *
 * Selectors used:
 *  .trust-badge / .trust-chip   — Trust service UI indicator
 *  .confirmation-overlay        — Blocking overlay before dangerous commands execute
 *  .split-pane-leaf             — Individual pane in a split layout
 *  .sector-tile                 — Sector tile in Global Overview
 *  .staged-command-chip         — Heuristic AI suggestion chip
 *  .ai-suggestion               — AI/heuristic overlay element
 */

const LIVE_SESSION_PATH = path.join(
    process.env.HOME || '/root',
    '.local/share/tos/sessions/_live.tos-session'
);

// ---------------------------------------------------------------------------
// Shared boot helper — navigate to Level 2 (Command Hub).
// ---------------------------------------------------------------------------
async function bootToCommandHub(page: any) {
    await page.addInitScript(() => {
        // Use the exact keys the app reads from localStorage.
        window.localStorage.setItem('tos.onboarding.first_run_complete', 'true');
        window.localStorage.setItem('tos.onboarding.wizard_complete', 'true');
    });

    await page.goto('/');

    const status = page.locator('.status-badge', { hasText: /BRAIN/i });
    await expect(status).toBeVisible({ timeout: 15000 });

    await page.keyboard.press('Escape');
    await page.waitForTimeout(400);

    const skipBtn = page.locator('button:has-text("SKIP TOUR")');
    const skipVisible = await skipBtn.isVisible().catch(() => false);
    if (skipVisible) await skipBtn.click();

    // Click the sector tile to zoom into Level 2 (Command Hub).
    const sectorTile = page.locator('.sector-tile').first();
    await expect(sectorTile).toBeVisible({ timeout: 8000 });
    await sectorTile.click();

    const cmdInput = page.locator('input#cmd-input');
    await expect(cmdInput).toBeVisible({ timeout: 5000 });
}

// ---------------------------------------------------------------------------
// 4-A  Trust Confirmation Blockers
//
// NOTE: Tagged 'pending' — triggers once .confirmation-overlay or .trust-chip
// is wired in the UI from the Brain's pending_confirmation IPC payload.
// ---------------------------------------------------------------------------
test.describe('Trust Confirmation Blockers', () => {
    test.slow();

    test('should render trust confirmation overlay before executing sudo su', async ({ page }) => {
        test.info().annotations.push({
            type: 'pending',
            description:
                'Requires .confirmation-overlay or .trust-chip DOM node to be wired. ' +
                'Currently the TrustService emits pending_confirmation on the IPC state ' +
                'channel but the UI does not surface a dedicated overlay element yet.',
        });

        await bootToCommandHub(page);

        const cmdInput = page.locator('input#cmd-input');
        await expect(cmdInput).toBeVisible({ timeout: 5000 });

        await cmdInput.fill('sudo su');
        await cmdInput.press('Enter');

        // Expect a trust/confirmation badge to appear before the command executes.
        const trustIndicator = page
            .locator('[class*="trust"], [class*="confirmation"], [class*="confirm"]')
            .first();

        await expect(trustIndicator).toBeVisible({ timeout: 5000 });
    });
});

// ---------------------------------------------------------------------------
// 4-B  Split Detachment & Session Persistence
// ---------------------------------------------------------------------------
test.describe('Split Detachment & Session Persistence', () => {
    test('should write _live.tos-session after creating a split pane', async ({ page }) => {
        await bootToCommandHub(page);

        const mtimeBefore = fs.existsSync(LIVE_SESSION_PATH)
            ? fs.statSync(LIVE_SESSION_PATH).mtimeMs
            : 0;

        // Ctrl+\ creates a split pane in the active Command Hub.
        await page.keyboard.press('Control+\\');

        // A second split-pane-leaf should become visible in the viewport.
        const panes = page.locator('.split-pane-leaf');
        await expect(panes).toHaveCount(2, { timeout: 5000 });

        // Wait for tos-sessiond to flush the new layout (debounced, up to 5 s).
        await page.waitForTimeout(5500);

        const mtimeAfter = fs.existsSync(LIVE_SESSION_PATH)
            ? fs.statSync(LIVE_SESSION_PATH).mtimeMs
            : 0;

        expect(mtimeAfter).toBeGreaterThan(mtimeBefore);

        const raw = fs.readFileSync(LIVE_SESSION_PATH, 'utf8');
        const session = JSON.parse(raw);
        const activeSector = session.sectors?.[session.active_sector_index ?? 0];
        const activeHub = activeSector?.hubs?.[activeSector?.active_hub_index ?? 0];
        expect(activeHub?.split_layout).not.toBeNull();
    });
});

// ---------------------------------------------------------------------------
// 4-C  Heuristic AI Resolution (staged command chip)
//
// NOTE: Tagged 'pending' — triggers once tos-heuristicd is registered and
// .staged-command-chip /.ai-suggestion nodes are rendered by the Brain.
// ---------------------------------------------------------------------------
test.describe('Heuristic AI Resolution', () => {
    test.slow();

    test('should surface a heuristic AI suggestion chip on hallucinated error', async ({ page }) => {
        test.info().annotations.push({
            type: 'pending',
            description:
                'Requires tos-heuristicd to be registered in the ServiceManager registry ' +
                'and the `.staged-command-chip` or `.ai-suggestion` DOM node to be rendered ' +
                'in the prompt interlock area when the heuristic triggers.',
        });

        await bootToCommandHub(page);

        const cmdInput = page.locator('input#cmd-input');

        await cmdInput.fill('tos-nonexistent-binary --flag-that-does-not-exist');
        await cmdInput.press('Enter');

        const aiChip = page
            .locator('[class*="staged-command"], [class*="ai-suggestion"], [class*="heuristic"]')
            .first();

        await expect(aiChip).toBeVisible({ timeout: 12000 });
    });
});
