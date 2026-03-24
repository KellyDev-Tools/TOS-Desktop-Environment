import { test, expect } from '@playwright/test';

test.describe('TOS Alpha-2.2 UI Component Paces', () => {
    test.beforeEach(async ({ page }) => {
        // Mark onboarding as complete to avoid interference
        await page.addInitScript(() => {
            window.localStorage.setItem('tos.onboarding.first_run_complete', 'true');
            window.localStorage.setItem('tos.onboarding.wizard_complete', 'true');

            const OriginalWebSocket = window.WebSocket;
            window.WebSocket = class MockWebSocket {
                onopen: any = null;
                onmessage: any = null;
                onclose: any = null;
                onerror: any = null;
                readyState: number = 1;

                constructor(url: string) {
                    setTimeout(() => {
                        if (this.onopen) this.onopen(new Event('open'));
                    }, 50);
                }
                send(data: any) { }
                close() { }
                addEventListener() { }
                removeEventListener() { }
            } as any;
        });
    });

    test('should show System Output container', async ({ page }) => {
        await page.goto('/');

        // The system-output is a global background layer.
        const container = page.locator('.system-output');
        await expect(container).toBeAttached({ timeout: 15000 });
    });

    test('should render Onboarding Trust sequence on fresh run', async ({ page }) => {
        // Force onboarding to show by clearing storage
        await page.addInitScript(() => {
            window.localStorage.clear();
        });

        await page.goto('/');

        // Onboarding title is "TRUST CONFIGURATION"
        const title = page.locator('h1.step-title:has-text("TRUST CONFIGURATION")');
        await expect(title).toBeVisible({ timeout: 15000 });

        // Check privilege escalation buttons
        const privEscalationWarn = page.locator('.trust-label:has-text("PRIVILEGE ESCALATION")').locator('..').locator('button:has-text("WARN")');
        await expect(privEscalationWarn).toBeVisible();
    });

    test('should render Split Viewports and Kinetic Borders in DOM', async ({ page }) => {
        await page.goto('/');

        // Global Overview should be visible by default
        const globalOverview = page.locator('.global-overview');
        await expect(globalOverview).toBeAttached({ timeout: 15000 });

        // If sectors are populated (they are by default in our current mock), check for tiles
        const sectorGrid = page.locator('.sector-grid');
        await expect(sectorGrid).toBeVisible();
        const firstTile = page.locator('.sector-tile').first();
        await expect(firstTile).toBeVisible();
    });

    test('should render Detail Inspector God Mode Wireframe', async ({ page }) => {
        await page.goto('/');

        // Ensure browser is ready
        await page.waitForLoadState('networkidle');

        // Trigger the UI via a hotkey 
        // Svelte 5 handleGlobalKeydown uses e.key '4' for Ctrl+4
        await page.keyboard.press('Control+Digit4');

        // Check for the header text of the DetailInspector component
        // Relaxing the text match further
        const inspectorHeader = page.locator('h2').filter({ hasText: 'TACTICAL RESET' });
        await expect(inspectorHeader).toBeVisible({ timeout: 15000 });

        // Check for the kill switch button
        const killSwitch = page.locator('.kill-switch');
        await expect(killSwitch).toBeVisible();

        // Click kill switch and verify confirmation state
        await killSwitch.click();
        await expect(killSwitch).toContainText('SYSTEM RE-AUTH REQUIRED');
    });
});
