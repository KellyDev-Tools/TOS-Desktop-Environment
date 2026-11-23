import { test, expect } from '@playwright/test';
import { spawn, type ChildProcess } from 'child_process';
import fs from 'fs';
import net from 'net';
import os from 'os';
import path from 'path';
import WebSocket from 'ws';

/**
 * App placement through the face (Architecture §26.4).
 *
 * Against a real Brain with fixture catalogues (a server-side
 * desktop entry planted in its XDG dirs; a host catalogue
 * synced over the protocol, as a face shell would):
 *
 *  - under prominence `explicit`, submitting a GUI app from
 *    the prompt surface raises the placement choice card, and
 *    choosing "On server" completes the launch;
 *  - the chips strip's launches row then shows the launch with
 *    its S glyph — where it ran is visible at a glance;
 *  - a host launch under the default prominence shows its H
 *    glyph the same way.
 */

const FACE_URL = 'http://localhost:4173';

test.use({
    ignoreHTTPSErrors: true,
    launchOptions: {
        args: ['--ignore-certificate-errors'],
        ...(process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH
            ? { executablePath: process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH }
            : {})
    }
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

/** One command over the Brain's local UDS path — settings
 * commands are parked over remote by ruling, so the prominence
 * preference is arranged the way a local client would set it. */
function udsCommand(sockPath: string, command: string): Promise<string> {
    return new Promise((resolve, reject) => {
        const socket = net.connect(sockPath);
        let buf = '';
        socket.on('connect', () => socket.write(`${command}\n`));
        socket.on('data', (chunk) => {
            buf += chunk.toString();
            const nl = buf.indexOf('\n');
            if (nl >= 0) {
                socket.end();
                resolve(buf.substring(0, nl));
            }
        });
        socket.on('error', reject);
        setTimeout(() => {
            socket.destroy();
            reject(new Error(`UDS command timed out: ${command}`));
        }, 10_000);
    });
}

/** Minimal command channel for arranging Brain state. */
class Arrange {
    private ws: WebSocket | null = null;
    private seq = 0;
    private pending = new Map<string, (res: string | null) => void>();

    async connect(): Promise<void> {
        this.ws = new WebSocket('wss://localhost:7001', { rejectUnauthorized: false });
        this.ws.on('message', (data) => {
            const msg = data.toString();
            if (!msg.startsWith('res:')) return;
            const rest = msg.substring(4);
            const sep = rest.indexOf(':');
            if (sep < 0) return;
            const resolve = this.pending.get(rest.substring(0, sep));
            if (resolve) {
                this.pending.delete(rest.substring(0, sep));
                resolve(rest.substring(sep + 1));
            }
        });
        await new Promise<void>((resolve, reject) => {
            this.ws!.once('open', resolve);
            this.ws!.once('error', reject);
        });
    }

    send(command: string): Promise<string | null> {
        return new Promise((resolve) => {
            const id = `ui${++this.seq}`;
            this.pending.set(id, resolve);
            this.ws!.send(`cmd:${id}:${command}`);
            setTimeout(() => {
                if (this.pending.delete(id)) resolve(null);
            }, 10_000);
        });
    }

    async close(): Promise<void> {
        this.ws?.close();
    }
}

const HOST_APPS = [
    { id: 'tos-test-editor', name: 'TosTestEditor', exec: 'tos-test-editor %u' },
    { id: 'tos-test-host-app', name: 'TosTestHostApp', exec: 'tos-test-host-app' }
];

test.describe('App placement — through the face', () => {
    test.describe.configure({ mode: 'serial' });
    test.setTimeout(120_000);

    let brain: ChildProcess | undefined;
    let stateDir: string | undefined;
    let sockPath = '';
    const arrange = new Arrange();

    async function setProminence(value: string): Promise<void> {
        await udsCommand(
            sockPath,
            `set_setting:tos.placement.prominence;${value}`
        );
    }

    test.beforeAll(async () => {
        const bin = brainBinary();
        if (!fs.existsSync(bin)) {
            throw new Error(
                `tos-brain binary not found at ${bin}. Build it first: ` +
                    '`cargo build -p tos-brain` (or set TOS_BRAIN_BIN).'
            );
        }
        stateDir = fs.mkdtempSync(path.join(os.tmpdir(), 'tos-placement-ui-'));
        const appsDir = path.join(stateDir, 'data', 'applications');
        fs.mkdirSync(appsDir, { recursive: true });
        fs.writeFileSync(
            path.join(appsDir, 'tos-test-editor.desktop'),
            '[Desktop Entry]\nType=Application\nName=TosTestEditor\nExec=tos-test-editor %u\n'
        );
        const emptyDataDirs = path.join(stateDir, 'sysdata');
        fs.mkdirSync(path.join(emptyDataDirs, 'applications'), { recursive: true });
        fs.mkdirSync(path.join(stateDir, 'home'), { recursive: true });
        sockPath = path.join(stateDir, 'brain.sock');
        const configPath = path.join(stateDir, 'tos.toml');
        fs.writeFileSync(configPath, `[remote]\nuds_path = "${sockPath}"\n`);
        brain = spawn(bin, [], {
            env: {
                ...process.env,
                HOME: path.join(stateDir, 'home'),
                XDG_CONFIG_HOME: path.join(stateDir, 'config'),
                XDG_DATA_HOME: path.join(stateDir, 'data'),
                XDG_DATA_DIRS: emptyDataDirs,
                TOS_CONFIG: configPath
            },
            stdio: 'ignore'
        });
        await waitForPort(7001);
        await arrange.connect();
        expect(
            await arrange.send(`placement_sync_apps:${JSON.stringify(HOST_APPS)}`)
        ).toBe('OK');
    });

    test.afterAll(async () => {
        await arrange.close();
        if (brain) {
            const brainProc = brain;
            const exited = new Promise<void>((resolve) => {
                brainProc.once('exit', () => resolve());
            });
            brainProc.kill('SIGTERM');
            await Promise.race([exited, new Promise((r) => setTimeout(r, 5_000))]);
        }
        if (stateDir) fs.rmSync(stateDir, { recursive: true, force: true });
    });

    test('explicit prominence: the prompt asks, the chips strip shows where it ran', async ({
        page
    }) => {
        await setProminence('explicit');
        await page.goto(`${FACE_URL}/?surface=prompt`);
        const input = page.getByTestId('prompt-input');
        await expect(input).toBeVisible();
        await input.click();
        await page.keyboard.type('tos-test-editor');
        await page.keyboard.press('Enter');

        const card = page.getByTestId('placement-choice');
        await expect(card).toBeVisible();
        await expect(card).toContainText('TosTestEditor');
        await page.getByTestId('placement-choose-server').click();
        await expect(card).toBeHidden();

        await page.goto(`${FACE_URL}/?surface=chips`);
        const row = page.getByTestId('accel-launches-row');
        await expect(row).toBeVisible();
        const launch = row.getByTestId('accel-launch').filter({ hasText: 'TosTestEditor' });
        await expect(launch).toBeVisible();
        await expect(launch.getByTestId('accel-launch-glyph')).toHaveText('S');
    });

    test('a host launch shows its H glyph on the chips strip', async ({ page }) => {
        await setProminence('glyph');
        expect(await arrange.send('prompt_submit:tos-test-host-app')).toBe(
            'LAUNCHED:host'
        );
        await page.goto(`${FACE_URL}/?surface=chips`);
        const row = page.getByTestId('accel-launches-row');
        await expect(row).toBeVisible();
        const launch = row
            .getByTestId('accel-launch')
            .filter({ hasText: 'TosTestHostApp' });
        await expect(launch).toBeVisible();
        await expect(launch.getByTestId('accel-launch-glyph')).toHaveText('H');
    });
});
