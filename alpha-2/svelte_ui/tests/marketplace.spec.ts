import { test, expect } from '@playwright/test';

const mockHomeData = {
    featured: [
        { id: 'mod-1', name: 'Aurora Theme', module_type: 'Theme', author: 'TOS Core', icon: '🎨', rating: 4.8, price: 'FREE', installed: false }
    ],
    categories: [
        { id: 'cat-1', name: 'Themes', icon: '🎨', module_count: 10 }
    ]
};

const mockDetailData = {
    summary: mockHomeData.featured[0],
    description: 'A beautiful aurora-inspired theme for your TOS desktop.',
    screenshots: ['https://example.com/ss1.png'],
    permissions: ['filesystem_read', 'theme_control'],
    reviews: [
        { author: 'Tim', rating: 5, comment: 'Amazing visuals!', date: '2026-03-01' }
    ]
};

test.describe('Marketplace Integration Tests', () => {
    test.beforeEach(async ({ page }) => {
        // Mock WebSocket for IPC
        await page.routeWebSocket('ws://127.0.0.1:7001', (route) => {
            route.onMessage((message) => {
                const msg = message.toString();
                if (msg.startsWith('get_state:')) {
                    route.send(JSON.stringify({
                        active_sector_index: 0,
                        sectors: [{ name: 'Test Sector' }],
                        system_log: [],
                        settings: { global: { 'tos.onboarding.first_run_complete': 'true' } }
                    }));
                } else if (msg.startsWith('marketplace_home:')) {
                    route.send(JSON.stringify(mockHomeData));
                } else if (msg.startsWith('marketplace_detail:mod-1')) {
                    route.send(JSON.stringify(mockDetailData));
                } else if (msg.toLowerCase().includes('marketplace_search_ai')) {
                    route.send(JSON.stringify([mockHomeData.featured[0]]));
                } else if (msg.startsWith('marketplace_install:mod-1')) {
                    route.send('INSTALLING');
                } else if (msg.startsWith('marketplace_status:mod-1')) {
                    route.send(JSON.stringify({
                        module_id: 'mod-1',
                        progress: 0.5,
                        status: 'Downloading'
                    }));
                } else {
                    route.send('OK');
                }
            });
        });

        // Mark onboarding as complete to avoid interference
        await page.addInitScript(() => {
            window.localStorage.setItem('tos.onboarding.first_run_complete', 'true');
            window.localStorage.setItem('tos.onboarding.wizard_complete', 'true');
        });

        await page.goto('/');
        await page.waitForLoadState('domcontentloaded');
    });

    test('should navigate to marketplace via hotkey', async ({ page }) => {
        await page.keyboard.press('Control+m');

        const marketTitle = page.locator('.market-title');
        await expect(marketTitle).toContainText('SYSTEM // MARKETPLACE', { timeout: 10000 });

        const featuredStrip = page.locator('.featured-strip');
        await expect(featuredStrip).toBeVisible({ timeout: 10000 });

        const featuredCards = featuredStrip.locator('.featured-card');
        await expect(featuredCards).toHaveCount(1);
    });

    test('should perform AI search and show results', async ({ page }) => {
        await page.keyboard.press('Control+m');

        const searchInput = page.locator('.glass-input');
        await searchInput.waitFor({ state: 'visible' });
        await searchInput.fill('theme');
        await page.keyboard.press('Enter');

        const moduleGrid = page.locator('.module-grid');
        // Results might have a transition
        await expect(moduleGrid.locator('.module-card')).toHaveCount(1, { timeout: 15000 });
        await expect(moduleGrid.locator('.module-card').first()).toContainText('Aurora Theme');
    });

    test('should open module detail and initiate install', async ({ page }) => {
        await page.keyboard.press('Control+m');

        // Click the first featured module
        const firstModule = page.locator('.featured-card').first();
        await firstModule.click();

        // Detail overlay should appear
        const detailOverlay = page.locator('.detail-overlay');
        await expect(detailOverlay).toBeVisible();
        await expect(detailOverlay).toContainText('A beautiful aurora-inspired theme');

        // Check for "INSTALL MODULE" button
        const installBtn = detailOverlay.locator('button:has-text("INSTALL MODULE")').first();
        await expect(installBtn).toBeVisible();
        await installBtn.click();

        // Permission modal should appear
        const permissionModal = page.locator('.modal-overlay:has-text("REVIEW PERMISSIONS")');
        await expect(permissionModal).toBeVisible();

        // Confirm install
        const confirmBtn = permissionModal.locator('button:has-text("ACCEPT & INSTALL")');
        await confirmBtn.click();

        // Permission modal should disappear
        await expect(permissionModal).not.toBeVisible();

        // Progress bar should appear in the detail footer
        const progressBar = detailOverlay.locator('.install-progress-bar');
        await expect(progressBar).toBeVisible();
        await expect(progressBar).toContainText('DOWNLOADING... 50%');
    });
});
