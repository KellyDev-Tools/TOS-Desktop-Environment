/**
 * Phase 2 — reconnect behaviour.
 *
 * The face must survive a Brain restart (and, by the same client
 * path, sleep/wake and network drops) by reattaching on its own:
 * exponential-backoff reconnect in the IPC store, a truthful
 * DISCONNECTED state in the chips while the Brain is down, and a
 * terminal reset on reconnect so the Brain's scrollback replay
 * (Phase 1.5) lands as the whole truth instead of appending to the
 * frozen pre-drop content.
 *
 * This spec drives the real stack: a real Brain the test kills and
 * restarts against the same state dirs, and the built face in a
 * browser that is NEVER reloaded — if the face doesn't reattach by
 * itself, the test goes red.
 */
import { test, expect, type Page } from '@playwright/test';
import { spawn, type ChildProcess } from 'child_process';
import fs from 'fs';
import net from 'net';
import os from 'os';
import path from 'path';

const FACE_URL = 'http://localhost:4173';
const MARKER_BEFORE = 'TOS_RECONNECT_BEFORE_R7';
const MARKER_AFTER = 'TOS_RECONNECT_AFTER_S8';

test.use({
    ignoreHTTPSErrors: true,
    launchOptions: { args: ['--ignore-certificate-errors'] }
});

function brainBinary(): string {
    if (process.env.TOS_BRAIN_BIN) return process.env.TOS_BRAIN_BIN;
    return path.resolve(process.cwd(), '..', 'target', 'debug', 'tos-brain');
}

async function waitForPort(port: number, timeoutMs = 30_000): Promise<void> {
    const deadline = Date.now() + timeoutMs;
    for (;;) {
        const open = await new Promise<boolean>((resolve) => {
            const socket = net.connect(port, 'localhost');
            socket.on('connect', () => {
                socket.end();
                resolve(true);
            });
            socket.on('error', () => resolve(false));
        });
        if (open) return;
        if (Date.now() > deadline) throw new Error(`port ${port} never came up`);
        await new Promise((r) => setTimeout(r, 250));
    }
}

async function terminalText(page: Page): Promise<string> {
    return (await page.locator('.xterm-rows').textContent()) ?? '';
}

async function runInTerminal(page: Page, command: string): Promise<void> {
    await page.locator('.xterm').click();
    await page.keyboard.type(command);
    await page.keyboard.press('Enter');
}

test.describe('Phase 2 — reconnect', () => {
    test.describe.configure({ mode: 'serial' });
    test.setTimeout(180_000);

    let brain: ChildProcess | undefined;
    let stateDir: string;
    let fixtureDir: string;

    async function startBrain(): Promise<void> {
        brain = spawn(brainBinary(), [], {
            env: {
                ...process.env,
                HOME: fixtureDir,
                XDG_CONFIG_HOME: path.join(stateDir, 'config'),
                XDG_DATA_HOME: path.join(stateDir, 'data')
            },
            stdio: 'ignore'
        });
        await waitForPort(7001);
    }

    async function stopBrain(): Promise<void> {
        if (!brain) return;
        const proc = brain;
        brain = undefined;
        const exited = new Promise<void>((resolve) => {
            proc.once('exit', () => resolve());
        });
        proc.kill('SIGTERM');
        await Promise.race([
            exited,
            new Promise<void>((r) => setTimeout(r, 5_000)).then(() => {
                proc.kill('SIGKILL');
            })
        ]);
    }

    test.beforeAll(async () => {
        if (!fs.existsSync(brainBinary())) {
            throw new Error(
                `tos-brain binary not found at ${brainBinary()}. Build it first: ` +
                    '`cargo build -p tos-brain` (or set TOS_BRAIN_BIN).'
            );
        }
        stateDir = fs.mkdtempSync(path.join(os.tmpdir(), 'tos-reconnect-e2e-'));
        fixtureDir = fs.mkdtempSync(path.join(os.tmpdir(), 'tos-reconnect-fixture-'));
        await startBrain();
    });

    test.afterAll(async () => {
        await stopBrain();
        const rmOpts = { recursive: true, force: true, maxRetries: 5, retryDelay: 200 } as const;
        fs.rmSync(stateDir, rmOpts);
        fs.rmSync(fixtureDir, rmOpts);
    });

    test('face reattaches after a Brain restart, without a reload', async ({ page }) => {
        // A live session on the background surface.
        await page.goto(`${FACE_URL}/?surface=background`);
        await expect(page.locator('.surface-background .xterm')).toBeVisible();
        await runInTerminal(page, `echo ${MARKER_BEFORE}`);
        await expect
            .poll(() => terminalText(page), { timeout: 15_000 })
            .toContain(MARKER_BEFORE);

        // The chips surface (a second face instance) shows LOCAL.
        const chipsPage = await page.context().newPage();
        await chipsPage.goto(`${FACE_URL}/?surface=chips`);
        await expect(chipsPage.getByTestId('accel-scope')).toHaveText('LOCAL');

        // Kill the Brain. The chips must tell the truth: DISCONNECTED.
        await stopBrain();
        await expect(chipsPage.locator('.accel-note')).toHaveText('DISCONNECTED', {
            timeout: 20_000
        });

        // Restart the Brain against the same state. No reload: the
        // face's own backoff reconnect must reattach it.
        await startBrain();
        await expect
            .poll(
                async () => {
                    // Typing while disconnected is a no-op (nothing
                    // queues), so repeated attempts are safe; the one
                    // that lands after reattach echoes the marker.
                    await runInTerminal(page, `echo ${MARKER_AFTER}`);
                    return terminalText(page);
                },
                { timeout: 60_000, intervals: [2_000] }
            )
            .toContain(MARKER_AFTER);

        // The terminal was reset on reconnect: the pre-kill content
        // is gone (the restarted Brain's scrollback is empty, so its
        // replay contributes nothing) — no stale session illusion.
        expect(await terminalText(page)).not.toContain(MARKER_BEFORE);

        // And the chips scope is back, truthfully LOCAL again.
        await expect(chipsPage.getByTestId('accel-scope')).toHaveText('LOCAL', {
            timeout: 20_000
        });
        await chipsPage.close();
    });
});
