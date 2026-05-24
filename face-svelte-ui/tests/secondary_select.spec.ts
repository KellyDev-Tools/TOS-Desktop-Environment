import { test, expect } from '@playwright/test';

test.describe('Secondary Select Infrastructure', () => {
    test.beforeEach(async ({ page }) => {
        // Mark onboarding complete
        await page.addInitScript(() => {
            window.localStorage.setItem('tos.onboarding.first_run_complete', 'true');
            window.localStorage.setItem('tos.onboarding.wizard_complete', 'true');
        });

        // Mock WebSocket for IPC with hydrated state delta
        await page.routeWebSocket(/ws(s)?:\/\/127\.0\.0\.1:7001/, (route) => {
            const statePayload = {
                current_level: 1,
                active_sector_index: 0,
                sectors: [{
                    id: '00000000-0000-0000-0000-000000000000',
                    name: 'Test Sector',
                    hubs: [{
                        id: '00000000-0000-0000-0000-000000000001',
                        mode: 'Command',
                        current_directory: '/',
                        terminal_output: [],
                        activity_listing: {
                            processes: [
                                { pid: 1234, name: 'tos-brain', cpu_usage: 1.2, mem_usage: 4.5 }
                            ]
                        },
                        is_running: false
                    }],
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
            };

            const heartbeatInterval = setInterval(() => {
                try {
                    route.send(`state_delta:${JSON.stringify(statePayload)}`);
                } catch {}
            }, 4500);

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
                        respond(JSON.stringify(statePayload));
                    } else {
                        respond('OK');
                    }
                } else {
                    route.send('OK');
                }
            });
        });
        await page.goto('/');
        await page.waitForLoadState('domcontentloaded');
        // Await global readiness
        await expect(page.locator('.lcars-container')).toBeVisible({ timeout: 15000 });
    });

    test('Sector Tile Right Click - Should open SectorContextMenu', async ({ page }) => {
        // The global overview should be open by default
        const sectorTile = page.locator('.sector-grid .sector-tile').first();
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
        const sectorTile = page.locator('.sector-grid .sector-tile').first();
        await expect(sectorTile).toBeVisible({ timeout: 10000 });

        // Simulate Long press via touch (longpress.ts acts on mousedown/touchstart)
        const box = await sectorTile.boundingBox();
        if (box) {
            await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
            await page.mouse.down();
            await page.waitForTimeout(1000); // longer than 600ms threshold
            await page.mouse.up();
        }

        const contextMenu = page.locator('.sector-context-menu');
        await expect(contextMenu).toBeVisible();
    });
});
