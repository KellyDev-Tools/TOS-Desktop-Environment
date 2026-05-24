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
        // Log browser console
        page.on('console', msg => {
            console.log(`BROWSER [${msg.type()}]: ${msg.text()}`);
        });

        // Mock WebSocket for IPC
        await page.routeWebSocket(/ws(s)?:\/\/127\.0\.0\.1:7001/, (route) => {
            let currentLevel: number | string = 1;

            const sendHeartbeat = () => {
                try {
                    route.send(`state_delta:${JSON.stringify({
                        current_level: currentLevel,
                        active_sector_index: 0,
                        sectors: [{
                            id: '00000000-0000-0000-0000-000000000000',
                            name: 'Test Sector',
                            hubs: [{ mode: 'command', current_directory: '/', terminal_output: [] }],
                            active_hub_index: 0,
                            active_apps: [],
                            participants: []
                        }],
                        system_log: [],
                        settings: {
                            global: { 'tos.onboarding.first_run_complete': 'true' },
                            sectors: {},
                            applications: {}
                        },
                        sys_prefix: 'TOS',
                        sys_title: 'TEST',
                        sys_status: 'OK',
                        brain_time: '12:00:00',
                        active_terminal_module: 'tos-standard-rect',
                        available_modules: [],
                        active_ai_module: 'tos-ai-standard',
                        available_ai_modules: [],
                        ai_behaviors: [],
                        bezel_expanded: false,
                        ai_default_backend: 'tos-ai-standard',
                        active_theme: 'tos-classic-lcars',
                        available_themes: [],
                        version: 1
                    })}`);
                } catch {}
            };

            const heartbeatInterval = setInterval(sendHeartbeat, 4500);

            route.onClose(() => {
                clearInterval(heartbeatInterval);
            });

            route.onMessage((message) => {
                const rawMsg = message.toString();
                const match = rawMsg.match(/^cmd:([^:]+):(.*)$/);
                if (match) {
                    const id = match[1];
                    const cmd = match[2];

                    const respond = (data) => {
                        route.send(`res:${id}:${data}`);
                    };

                    if (cmd.startsWith('get_state:')) {
                        respond(JSON.stringify({
                            current_level: currentLevel,
                            active_sector_index: 0,
                            sectors: [{
                                id: '00000000-0000-0000-0000-000000000000',
                                name: 'Test Sector',
                                hubs: [{ mode: 'command', current_directory: '/', terminal_output: [] }],
                                active_hub_index: 0,
                                active_apps: [],
                                participants: []
                            }],
                            system_log: [],
                            settings: {
                                global: { 'tos.onboarding.first_run_complete': 'true' },
                                sectors: {},
                                applications: {}
                            },
                            sys_prefix: 'TOS',
                            sys_title: 'TEST',
                            sys_status: 'OK',
                            brain_time: '12:00:00',
                            active_terminal_module: 'tos-standard-rect',
                            available_modules: [],
                            active_ai_module: 'tos-ai-standard',
                            available_ai_modules: [],
                            ai_behaviors: [],
                            bezel_expanded: false,
                            ai_default_backend: 'tos-ai-standard',
                            active_theme: 'tos-classic-lcars',
                            available_themes: [],
                            version: 1
                        }));
                    } else if (cmd.startsWith('set_mode:')) {
                        const newMode = cmd.substring(9).trim();
                        const modeMap: Record<string, number | string> = {
                            'global': 1,
                            'hubs': 2,
                            'sectors': 3,
                            'detail': 4,
                            'buffer': 5,
                            'logs': 'logs',
                            'marketplace': 'marketplace'
                        };
                        currentLevel = modeMap[newMode] || newMode;
                        respond(`MODE_SET: ${newMode}`);
                        sendHeartbeat();
                    } else if (cmd.startsWith('marketplace_home:')) {
                        respond(JSON.stringify(mockHomeData));
                    } else if (cmd.startsWith('marketplace_detail:mod-1')) {
                        respond(JSON.stringify(mockDetailData));
                    } else if (cmd.toLowerCase().includes('marketplace_search_ai')) {
                        respond(JSON.stringify([mockHomeData.featured[0]]));
                    } else if (cmd.startsWith('marketplace_install:mod-1')) {
                        respond('INSTALLING');
                    } else if (cmd.startsWith('marketplace_status:mod-1')) {
                        respond(JSON.stringify({
                            module_id: 'mod-1',
                            progress: 0.5,
                            status: 'Downloading'
                        }));
                    } else {
                        respond('OK');
                    }
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
        await expect(page.locator('.lcars-container')).toBeVisible();
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
        // Wait for connection to be logged
        await expect(page.locator('.lcars-container')).toBeVisible();

        await page.keyboard.press('Control+m');

        const searchInput = page.locator('.glass-input');
        await searchInput.waitFor({ state: 'visible' });
        await searchInput.fill('theme');
        await searchInput.press('Enter');

        const moduleGrid = page.locator('.module-grid');
        // Results might have a transition
        await expect(moduleGrid.locator('.module-card')).toHaveCount(1, { timeout: 20000 });
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

        // Scroll the permissions review container to the bottom to unlock the install button
        const permReview = page.locator('.perm-review');
        await permReview.evaluate(el => {
            el.scrollTop = el.scrollHeight;
            // Dispatch scroll event to make sure Svelte handles it
            el.dispatchEvent(new Event('scroll'));
        });

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
