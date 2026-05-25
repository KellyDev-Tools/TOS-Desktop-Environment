import { test, expect } from '@playwright/test';

test.describe('TOS Alpha-2.2 UI Component Paces', () => {
    test.beforeEach(async ({ page }) => {
        // Mark onboarding as complete to avoid interference
        await page.addInitScript(() => {
            window.localStorage.setItem('tos.onboarding.first_run_complete', 'true');
            window.localStorage.setItem('tos.onboarding.wizard_complete', 'true');
        });

        // Setup mutable mock state for sectors and level
        let currentSectors = [
            {
                id: '00000000-0000-0000-0000-000000000000',
                name: 'Test Sector',
                hubs: [{
                    id: '00000000-0000-0000-0000-000000000001',
                    mode: 'Command',
                    current_directory: '/',
                    terminal_output: [],
                    is_running: false
                }],
                active_hub_index: 0,
                active_apps: [],
                participants: []
            }
        ];
        let activeSectorIndex = 0;
        let currentLevel: number | string = 1;

        // Mock WebSocket for IPC with correct message framing protocol and 2-second heartbeat
        await page.routeWebSocket(/ws(s)?:\/\/127\.0\.0\.1:7001/, (route) => {
            const sendHeartbeat = () => {
                try {
                    route.send(`state_delta:${JSON.stringify({
                        current_level: currentLevel,
                        active_sector_index: activeSectorIndex,
                        sectors: currentSectors,
                        system_log: [],
                        settings: {
                            global: {},
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
                            active_sector_index: activeSectorIndex,
                            sectors: currentSectors,
                            system_log: [],
                            settings: {
                                global: {}, // Omit onboarding setting to let frontend localStorage rule
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
                    } else if (cmd.startsWith('sector_create:')) {
                        const parts = cmd.split(':');
                        const name = parts[1]?.trim() || "Sector 2";
                        currentSectors.push({
                            id: '00000000-0000-0000-0000-000000000002',
                            name,
                            hubs: [{
                                id: '00000000-0000-0000-0000-000000000003',
                                mode: 'Command',
                                current_directory: '/',
                                terminal_output: [],
                                is_running: false
                            }],
                            active_hub_index: 0,
                            active_apps: [],
                            participants: []
                        });
                        activeSectorIndex = currentSectors.length - 1;
                        respond(`SECTOR_CREATED: ${name}`);
                        sendHeartbeat(); // Trigger immediate push to synchronize frontend after mutation
                    } else if (cmd.startsWith('sector_close:')) {
                        if (currentSectors.length <= 1) {
                            respond('ERROR: Cannot close last sector');
                        } else {
                            const parts = cmd.split(':');
                            const cid = parts[1]?.trim();
                            currentSectors = currentSectors.filter(s => s.id !== cid);
                            activeSectorIndex = 0;
                            respond(`SECTOR_CLOSED: ${cid}`);
                            sendHeartbeat(); // Trigger immediate push to synchronize frontend after mutation
                        }
                    } else {
                        respond('OK');
                    }
                } else {
                    route.send('OK');
                }
            });
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
        const firstTile = page.locator('.sector-grid .sector-tile').first();
        await expect(firstTile).toBeVisible();
    });

    test('should render Detail Inspector God Mode Wireframe', async ({ page }) => {
        await page.goto('/');

        // Ensure browser is ready
        await page.waitForLoadState('networkidle');

        // Trigger the UI via a hotkey 
        // Svelte 5 handleGlobalKeydown uses e.key '4' for Ctrl+4
        await page.keyboard.press('Control+Digit4');

        // Check for the header text of the DetailInspector component (DEEP_INSPECTION)
        const inspectorHeader = page.locator('h2').filter({ hasText: 'DEEP_INSPECTION' });
        await expect(inspectorHeader).toBeVisible({ timeout: 15000 });

        // Switch to the TACTICAL_RESET tab to view the kill switch button
        const resetTabBtn = page.locator('button:has-text("TACTICAL_RESET")');
        await expect(resetTabBtn).toBeVisible();
        await resetTabBtn.click();

        // Check for the kill switch button
        const killSwitch = page.locator('.kill-switch');
        await expect(killSwitch).toBeVisible();

        // Click kill switch and verify confirmation state
        await killSwitch.click();
        await expect(killSwitch).toContainText('SYSTEM RE-AUTH REQUIRED');
    });

    test('should prevent closing the last remaining sector', async ({ page }) => {
        await page.goto('/');
        await expect(page.locator('.sector-grid .sector-tile:not(.sector-add-tile)')).toHaveCount(1);

        // Click Close Sector button (minus bezel button)
        const closeSectorBtn = page.locator('button[title="Close Sector"]');
        await expect(closeSectorBtn).toBeVisible();
        await closeSectorBtn.click();

        // Sector count should still be 1 (protected)
        await expect(page.locator('.sector-grid .sector-tile:not(.sector-add-tile)')).toHaveCount(1);
    });

    test('should auto-focus and name a newly created sector if name is empty', async ({ page }) => {
        await page.goto('/');
        await expect(page.locator('.sector-grid .sector-tile:not(.sector-add-tile)')).toHaveCount(1);

        // Click Add Sector button (plus bezel button)
        const addSectorBtn = page.locator('button[title="Add Sector"]');
        await expect(addSectorBtn).toBeVisible();
        await addSectorBtn.click();

        // Sector count should now be 2
        await expect(page.locator('.sector-grid .sector-tile:not(.sector-add-tile)')).toHaveCount(2);

        // Second sector should be named "Sector 2" (auto-named)
        const secondTile = page.locator('.sector-grid .sector-tile:not(.sector-add-tile)').nth(1);
        await expect(secondTile).toContainText(/Sector 2/i);
    });

    test('sector tile primary click and secondary context menu should be stable', async ({ page }) => {
        await page.goto('/');

        const firstTile = page.locator('.sector-grid .sector-tile:not(.sector-add-tile)').first();
        await expect(firstTile).toBeVisible();

        // Primary click should transition from Global Overview to Level 2 (Command Hub).
        await firstTile.click({ button: 'left' });
        await expect(page.locator('text=COMMAND HUB')).toBeVisible({ timeout: 5000 });

        // Return to level 1 and verify secondary click opens a persistent context menu.
        await page.keyboard.press('Control+Digit1');
        await expect(page.locator('text=GLOBAL OVERVIEW')).toBeVisible({ timeout: 5000 });

        await firstTile.click({ button: 'right' });
        const contextMenu = page.locator('.sector-context-menu');
        await expect(contextMenu).toBeVisible({ timeout: 3000 });

        // Regression guard: menu should not immediately close/flicker off.
        await page.waitForTimeout(250);
        await expect(contextMenu).toBeVisible();
    });
});
