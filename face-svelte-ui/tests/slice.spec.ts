import { test, expect, type Page } from '@playwright/test';
import { spawn, type ChildProcess } from 'child_process';
import fs from 'fs';
import net from 'net';
import os from 'os';
import path from 'path';

/**
 * Phase 1 slice E2E — the ambient face against a REAL Brain.
 *
 * Roadmap Phase 1 ("Frozen First Slice") scope: Brain + the three
 * layer-shell surfaces (background terminal, chips, prompt) on one
 * machine, local only. The layer-shell host itself is compositor
 * territory and is smoke-tested manually; this suite covers the web
 * layer of the slice end to end: the built face served over http,
 * talking over WSS to a real `tos-brain` binary spawned per run with
 * isolated XDG state. No WebSocket mocking — unlike the older specs,
 * if the Brain or the connection breaks, these tests go red.
 *
 * The Brain's face socket (7001) uses a self-signed certificate the
 * Brain generates itself (rcgen), so the browser must not enforce
 * certificate validation for it. Playwright's `ignoreHTTPSErrors`
 * does not cover WebSocket TLS in Chromium; the
 * `--ignore-certificate-errors` launch flag does. The native
 * layer-shell host handles the same problem via WebKit's
 * TLSErrorsPolicy on its network session.
 */

const FACE_URL = 'http://localhost:4173';
const MARKER_CMD = 'TOS_SLICE_E2E_A1';
const MARKER_RECONNECT = 'TOS_SLICE_E2E_B2';
const MARKER_PROMPT = 'TOS_SLICE_E2E_C3';

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

async function expectTerminalContains(page: Page, text: string): Promise<void> {
    await expect
        .poll(() => terminalText(page), { timeout: 15_000 })
        .toContain(text);
}

async function runInTerminal(page: Page, command: string): Promise<void> {
    await page.locator('.xterm').click();
    await page.keyboard.type(command);
    await page.keyboard.press('Enter');
}

test.describe('Phase 1 slice — ambient face against a real Brain', () => {
    test.describe.configure({ mode: 'serial' });
    test.setTimeout(90_000);

    let brain: ChildProcess | undefined;
    let stateDir: string | undefined;

    test.beforeAll(async () => {
        const bin = brainBinary();
        if (!fs.existsSync(bin)) {
            throw new Error(
                `tos-brain binary not found at ${bin}. Build it first: ` +
                    '`cargo build -p tos-brain` (or set TOS_BRAIN_BIN).'
            );
        }
        stateDir = fs.mkdtempSync(path.join(os.tmpdir(), 'tos-slice-e2e-'));
        brain = spawn(bin, [], {
            env: {
                ...process.env,
                XDG_CONFIG_HOME: path.join(stateDir, 'config'),
                XDG_DATA_HOME: path.join(stateDir, 'data')
            },
            stdio: 'ignore'
        });
        await waitForPort(7001);
    });

    test.afterAll(async () => {
        brain?.kill('SIGTERM');
        if (stateDir) fs.rmSync(stateDir, { recursive: true, force: true });
    });

    test('background surface connects and shows a live shell', async ({ page }) => {
        await page.goto(`${FACE_URL}/?surface=background`);
        await expect(page.locator('.surface-background .xterm')).toBeVisible();
        // A real shell prompt (user@host … # or $) proves the PTY is live,
        // not just painted.
        await expect
            .poll(() => terminalText(page), { timeout: 20_000 })
            .toMatch(/@.*[#$]/);
    });

    test('runs a command in the background terminal', async ({ page }) => {
        await page.goto(`${FACE_URL}/?surface=background`);
        await expect(page.locator('.surface-background .xterm')).toBeVisible();
        await runInTerminal(page, `echo ${MARKER_CMD}`);
        await expectTerminalContains(page, MARKER_CMD);
    });

    test('prompt surface renders the hub with a live terminal', async ({ page }) => {
        await page.goto(`${FACE_URL}/?surface=prompt`);
        await expect(page.locator('.surface-prompt')).toBeVisible();
        await expect(page.locator('.surface-prompt .xterm')).toBeVisible();
        await runInTerminal(page, `echo ${MARKER_PROMPT}`);
        await expectTerminalContains(page, MARKER_PROMPT);
    });

    test('chips surface mounts its overlay components', async ({ page }) => {
        // Both chips components are state-conditional: ExpandedBezel
        // renders only when the Brain reports bezel_expanded, and
        // WarningChip only when trust warnings exist. On a fresh Brain
        // the strip is therefore empty at rest — this test pins that
        // the surface itself mounts and loads cleanly, and that the
        // bezel appears once state expands it is covered by the mock
        // specs. If the slice grows an always-on collapsed strip, this
        // test should assert its content instead.
        await page.goto(`${FACE_URL}/?surface=chips`);
        await expect(page.locator('.surface-chips')).toBeVisible();
    });

    test('terminal session persists across a face restart', async ({ page }) => {
        // Phase 1.5: the shell is Brain-owned and outlives any face;
        // the Brain retains the raw output (ScrollbackBuffer) and
        // replays it to a face when it (re)connects, so the terminal
        // repaints the ongoing session — output from before the
        // restart is still on screen after it.
        await page.goto(`${FACE_URL}/?surface=background`);
        await expect(page.locator('.surface-background .xterm')).toBeVisible();
        await runInTerminal(page, `echo ${MARKER_CMD}`);
        await expectTerminalContains(page, MARKER_CMD);
        await page.reload();
        await expect(page.locator('.surface-background .xterm')).toBeVisible();
        await expectTerminalContains(page, MARKER_CMD);
    });

    test('face reconnects after a reload and runs new commands', async ({ page }) => {
        await page.goto(`${FACE_URL}/?surface=background`);
        await expect(page.locator('.surface-background .xterm')).toBeVisible();
        await page.reload();
        await expect(page.locator('.surface-background .xterm')).toBeVisible();
        await runInTerminal(page, `echo ${MARKER_RECONNECT}`);
        await expectTerminalContains(page, MARKER_RECONNECT);
    });
});
