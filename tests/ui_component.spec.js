const { test, expect } = require('@playwright/test');

/**
 * TOS UI Component Test
 * Paces the LCARS interface through Hierarchical Navigation and IPC flow.
 */

test.describe('TOS Alpha-2.1 UI Component Paces', () => {

    test.beforeEach(async ({ page }) => {
        // Assume the test server is running on TOS_FACE_PORT (default 8080)
        const port = process.env.TOS_FACE_PORT || '8080';
        await page.goto(`http://localhost:${port}`);
    });

    test('should initialize in Global Overview with hidden footer', async ({ page }) => {
        const title = page.locator('#view-title');
        await expect(title).toHaveText('GLOBAL OVERVIEW');

        // Footer should be hidden in Global Overview (Spec 6)
        const footer = page.locator('.lcars-footer');
        await expect(footer).not.toBeVisible();

        // Check for sector tiles
        const tiles = page.locator('.sector-tile');
        await expect(tiles).toHaveCount(3); // Based on mock state count
    });

    test('should transition to Hub View on sector selection', async ({ page }) => {
        // Click the first sector tile
        const firstTile = page.locator('.sector-tile').first();
        await firstTile.click();

        // Title should update
        const title = page.locator('#view-title');
        await expect(title).toHaveText(/HUB VIEW/);

        // Footer (Unified Prompt) should now be visible
        const footer = page.locator('.lcars-footer');
        await expect(footer).toBeVisible();
    });

    test('should transmit commands and receive ACK latency', async ({ page }) => {
        await page.locator('.sector-tile').first().click();

        const input = page.locator('#cmd-input');
        await input.fill('sys_diagnostic --full');
        await input.press('Enter');

        // Check telemetry bar (mini-log) for ACK
        const miniLog = page.locator('#mini-log');
        await expect(miniLog).toHaveText(/ACK \/\//);

        // Verify color change to success (green)
        // Note: CSS variable evaluation can be tricky, but we check text first
        await expect(miniLog).toHaveCSS('color', /rgb\(153, 204, 153\)/); // var(--lcars-green)
    });

    test('should toggle bezel commands', async ({ page }) => {
        const terminalToggle = page.locator('#bezel-term-toggle');
        await expect(terminalToggle).toBeVisible();
        await terminalToggle.click();

        // Verify layer ordering changes (Spec 6)
        const area = page.locator('#system-output-area');
        await expect(area).toHaveClass(/brought-to-front/);

        const target = page.locator('#state-render-target');
        await expect(target).toHaveClass(/dimmed/);
    });

    test('should render system log in background', async ({ page }) => {
        const area = page.locator('#system-output-area');
        await expect(area).toBeVisible();
        // Since it's behind, we check if it has content (mock state has one entry)
        await expect(area).toContainText(/SYS_BRAIN/);
    });
});
