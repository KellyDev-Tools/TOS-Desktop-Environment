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
const MARKER_PROMPT_INPUT = 'TOS_SLICE_E2E_D4';

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
    let fixtureDir: string | undefined;

    test.beforeAll(async () => {
        const bin = brainBinary();
        if (!fs.existsSync(bin)) {
            throw new Error(
                `tos-brain binary not found at ${bin}. Build it first: ` +
                    '`cargo build -p tos-brain` (or set TOS_BRAIN_BIN).'
            );
        }
        stateDir = fs.mkdtempSync(path.join(os.tmpdir(), 'tos-slice-e2e-'));
        // Phase 1.6 fixture: a known directory the chips can list.
        // A fresh Brain starts its shell hub in the user's home
        // directory, so pointing HOME at the fixture makes the
        // initial listing deterministic: one dir + one file.
        fixtureDir = fs.mkdtempSync(path.join(os.tmpdir(), 'tos-chips-fixture-'));
        fs.mkdirSync(path.join(fixtureDir, 'alpha-dir'));
        fs.writeFileSync(path.join(fixtureDir, 'notes.txt'), 'fixture\n');
        // Phase 1.7 fixture: a dotfile for the chips panel switch.
        fs.writeFileSync(path.join(fixtureDir, '.secret.txt'), 'secret\n');
        brain = spawn(bin, [], {
            env: {
                ...process.env,
                HOME: fixtureDir,
                XDG_CONFIG_HOME: path.join(stateDir, 'config'),
                XDG_DATA_HOME: path.join(stateDir, 'data')
            },
            stdio: 'ignore'
        });
        await waitForPort(7001);
    });

    test.afterAll(async () => {
        // Wait for the Brain to actually exit before removing its
        // dirs: on shutdown the shell writes history into HOME (the
        // fixture dir), and removing it mid-write races to ENOTEMPTY
        // — which is how this hook failed in CI while all 8 tests
        // passed. maxRetries covers any straggler writes.
        if (brain) {
            const brainProc = brain;
            const exited = new Promise<void>((resolve) => {
                brainProc.once('exit', () => resolve());
            });
            brainProc.kill('SIGTERM');
            await Promise.race([
                exited,
                new Promise<void>((r) => setTimeout(r, 5_000)).then(() => {
                    brainProc.kill('SIGKILL');
                })
            ]);
        }
        const rmOpts = { recursive: true, force: true, maxRetries: 5, retryDelay: 200 } as const;
        if (stateDir) fs.rmSync(stateDir, rmOpts);
        if (fixtureDir) fs.rmSync(fixtureDir, rmOpts);
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

    test('chips surface shows accelerator chips for the current directory', async ({ page }) => {
        // Phase 1.6: the strip is never empty at rest. The Brain
        // populates the shell hub's listing at startup (the fixture
        // home: alpha-dir + notes.txt), and the chips surface renders
        // it as tappable accelerator chips plus cwd breadcrumbs.
        await page.goto(`${FACE_URL}/?surface=chips`);
        await expect(page.locator('.surface-chips')).toBeVisible();
        await expect(page.getByTestId('accel-chips')).toBeVisible();
        await expect(page.getByTestId('accel-chip-dir').filter({ hasText: 'alpha-dir' })).toBeVisible();
        await expect(page.getByTestId('accel-chip-file').filter({ hasText: 'notes.txt' })).toBeVisible();
        await expect(page.getByTestId('accel-crumb').first()).toBeVisible();
        // Phase 2: the chips status truthfully scopes the connection —
        // this Brain is loopback, so LOCAL.
        await expect(page.getByTestId('accel-scope')).toHaveText('LOCAL');
    });

    test('chips narrow as you type and the dotfiles switch works', async ({ page }) => {
        // Phase 1.7 — dynamic chips through the real face: typing
        // in the (background) terminal publishes a filter, and the
        // chips surface narrows to matches; dotfiles stay hidden
        // until the panel's own `.*` switch flips them on.
        const chipsPage = await page.context().newPage();
        try {
            await page.goto(`${FACE_URL}/?surface=background`);
            await expect(page.locator('.surface-background .xterm')).toBeVisible();
            await expect
                .poll(() => terminalText(page), { timeout: 20_000 })
                .toMatch(/@.*[#$]/);

            await chipsPage.goto(`${FACE_URL}/?surface=chips`);
            await expect(
                chipsPage.getByTestId('accel-chip-file').filter({ hasText: 'notes.txt' })
            ).toBeVisible();
            // Dotfiles are hidden by default.
            await expect(
                chipsPage.getByTestId('accel-chip-file').filter({ hasText: '.secret.txt' })
            ).toHaveCount(0);

            // Type the file's name: the strip narrows to it.
            await page.locator('.xterm').click();
            await page.keyboard.type('notes');
            await expect(chipsPage.getByTestId('accel-filter')).toHaveText('⌕ notes');
            await expect(
                chipsPage.getByTestId('accel-chip-dir').filter({ hasText: 'alpha-dir' })
            ).toHaveCount(0);
            await expect(
                chipsPage.getByTestId('accel-chip-file').filter({ hasText: 'notes.txt' })
            ).toBeVisible();

            // Clear the line (Ctrl+U): the full strip comes back.
            await page.keyboard.press('Control+u');
            await expect(chipsPage.getByTestId('accel-filter')).toHaveCount(0);
            await expect(
                chipsPage.getByTestId('accel-chip-dir').filter({ hasText: 'alpha-dir' })
            ).toBeVisible();

            // The panel's own switch shows dotfiles; tapping again
            // hides them. (Leaves the flag off and the terminal
            // line empty for the tests that follow.)
            await chipsPage.getByTestId('accel-toggle-hidden').click();
            await expect(
                chipsPage.getByTestId('accel-chip-file').filter({ hasText: '.secret.txt' })
            ).toBeVisible();
            await chipsPage.getByTestId('accel-toggle-hidden').click();
            await expect(
                chipsPage.getByTestId('accel-chip-file').filter({ hasText: '.secret.txt' })
            ).toHaveCount(0);
        } finally {
            await chipsPage.close();
        }
    });

    test('prompt surface input runs commands in the terminal', async ({ page }) => {
        // Phase 1.6: the prompt input is independent of the terminal
        // view — submitting types the command into the real terminal.
        await page.goto(`${FACE_URL}/?surface=prompt`);
        const input = page.getByTestId('prompt-input');
        await expect(input).toBeVisible();
        await input.click();
        await page.keyboard.type(`echo ${MARKER_PROMPT_INPUT}`);
        await page.keyboard.press('Enter');
        await expectTerminalContains(page, MARKER_PROMPT_INPUT);
    });

    test('tapping a directory chip cds the terminal', async ({ page }) => {
        // Phase 1.6, the accelerator contract: a chip tap does the
        // typing. Tapping alpha-dir on the chips surface types
        // `cd 'alpha-dir'` into the Brain-owned shell; the terminal
        // (re)opened afterwards must show the new real cwd in pwd's
        // output — <fixture>/alpha-dir only ever appears there if
        // the shell itself moved.
        await page.goto(`${FACE_URL}/?surface=chips`);
        const chip = page.getByTestId('accel-chip-dir').filter({ hasText: 'alpha-dir' });
        await expect(chip).toBeVisible();
        await chip.click();

        await page.goto(`${FACE_URL}/?surface=background`);
        await expect(page.locator('.surface-background .xterm')).toBeVisible();
        await runInTerminal(page, 'pwd');
        await expect
            .poll(() => terminalText(page), { timeout: 15_000 })
            .toMatch(/tos-chips-fixture-[^\s]*\/alpha-dir/);
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
