import { test, expect } from '@playwright/test';
import { spawn, type ChildProcess } from 'node:child_process';
import * as fs from 'node:fs';
import * as net from 'node:net';
import * as os from 'node:os';
import * as path from 'node:path';
import WebSocket from 'ws';

/**
 * App placement, headless (Architecture §26.4).
 *
 * The Brain routes prompt submissions from desktop-entry
 * catalogues: its own machine's, read from the XDG data dirs
 * (this spec plants fixture entries there before spawning the
 * Brain), and the face host's, synced with
 * `placement_sync_apps`. Assertions, in order:
 *
 *  - a server-only app resolves to the server, a host-only app
 *    to the host, an app on both machines to the host;
 *  - typed arguments ride along into the launch exec;
 *  - a submission naming no app takes the terminal path,
 *    unchanged, and registers nothing;
 *  - `here:` / `server:` overrides force a side, and fail
 *    loudly when that side lacks the app or the app is unknown;
 *  - prominence `explicit` parks the launch as a pending
 *    placement until `placement_choose` answers it;
 *  - `placement_relaunch` re-runs a registered launch in the
 *    other location, and refuses loudly when that machine
 *    does not have the app.
 */

function brainBinary(): string {
    if (process.env.TOS_BRAIN_BIN) return process.env.TOS_BRAIN_BIN;
    return path.resolve(process.cwd(), '..', 'target', 'debug', 'tos-brain');
}

interface State {
    placement_launches: { id: string; app: string; exec: string; placement: string }[];
    placement_pending: { id: string; app_name: string } | null;
    placement_directive: { id: string; exec: string; placement: string } | null;
}

class BrainClient {
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

    async close(): Promise<void> {
        this.ws?.close();
    }

    send(command: string): Promise<string | null> {
        return new Promise((resolve) => {
            const id = `pl${++this.seq}`;
            this.pending.set(id, resolve);
            this.ws!.send(`cmd:${id}:${command}`);
            setTimeout(() => {
                if (this.pending.delete(id)) resolve(null);
            }, 10_000);
        });
    }

    async state(): Promise<State> {
        const res = await this.send('get_state:');
        expect(res, 'get_state responds').not.toBeNull();
        return JSON.parse(res!) as State;
    }
}

/** One command over the Brain's local UDS path — the full,
 * unnarrowed surface. Settings live there: the settings
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

const HOST_APPS = [
    { id: 'tos-test-editor', name: 'TosTestEditor', exec: 'tos-test-editor %u' },
    { id: 'tos-test-host-app', name: 'TosTestHostApp', exec: 'tos-test-host-app' }
];

test.describe('App placement — the routing protocol, headless', () => {
    test.describe.configure({ mode: 'serial' });
    test.setTimeout(120_000);

    let brain: ChildProcess | undefined;
    let stateDir: string | undefined;
    let sockPath = '';
    const client = new BrainClient();

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
        stateDir = fs.mkdtempSync(path.join(os.tmpdir(), 'tos-placement-e2e-'));
        // The Brain's own catalogue: two fixture apps planted in
        // its XDG_DATA_HOME; XDG_DATA_DIRS points at an empty
        // fixture so the host machine's real apps stay out.
        const appsDir = path.join(stateDir, 'data', 'applications');
        fs.mkdirSync(appsDir, { recursive: true });
        fs.writeFileSync(
            path.join(appsDir, 'tos-test-editor.desktop'),
            '[Desktop Entry]\nType=Application\nName=TosTestEditor\nExec=tos-test-editor %u\n'
        );
        fs.writeFileSync(
            path.join(appsDir, 'tos-test-viewer.desktop'),
            '[Desktop Entry]\nType=Application\nName=TosTestViewer\nExec=tos-test-viewer\n'
        );
        const emptyDataDirs = path.join(stateDir, 'sysdata');
        fs.mkdirSync(path.join(emptyDataDirs, 'applications'), { recursive: true });
        fs.mkdirSync(path.join(stateDir, 'home'), { recursive: true });
        // A fixture config puts the Brain's UDS inside the
        // isolated state dir (and nothing else changes).
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
        await client.connect();
        const sync = await client.send(
            `placement_sync_apps:${JSON.stringify(HOST_APPS)}`
        );
        expect(sync).toBe('OK');
    });

    test.afterAll(async () => {
        await client.close();
        if (brain) {
            const brainProc = brain;
            const exited = new Promise<void>((resolve) => {
                brainProc.once('exit', () => resolve());
            });
            brainProc.kill('SIGTERM');
            await Promise.race([
                exited,
                new Promise((r) => setTimeout(r, 5_000))
            ]);
        }
        if (stateDir) fs.rmSync(stateDir, { recursive: true, force: true });
    });

    test('server-only, host-only, and both resolve to their machines', async () => {
        expect(await client.send('prompt_submit:tos-test-viewer')).toBe(
            'LAUNCHED:server'
        );
        expect(await client.send('prompt_submit:tos-test-host-app')).toBe(
            'LAUNCHED:host'
        );
        expect(await client.send('prompt_submit:tos-test-editor')).toBe(
            'LAUNCHED:host'
        );
        const state = await client.state();
        const placements = state.placement_launches.map((l) => l.placement);
        expect(placements).toEqual(['server', 'host', 'host']);
        expect(state.placement_directive?.placement).toBe('host');
    });

    test('typed arguments ride into the launch exec', async () => {
        expect(
            await client.send('prompt_submit:tos-test-host-app --flag')
        ).toBe('LAUNCHED:host');
        const state = await client.state();
        expect(state.placement_directive?.exec).toContain('--flag');
    });

    test('prompt_launch routes launches and declines the rest to the terminal', async () => {
        expect(await client.send('prompt_launch:tos-test-viewer')).toBe(
            'LAUNCHED:server'
        );
        expect(await client.send('prompt_launch:echo hello')).toBe('SESSION');
    });

    test('a non-app submission takes the terminal path and registers nothing', async () => {
        const before = (await client.state()).placement_launches.length;
        expect(await client.send('prompt_submit:tos-test-not-an-app')).toBe(
            'SUBMITTED'
        );
        const after = (await client.state()).placement_launches.length;
        expect(after).toBe(before);
    });

    test('overrides force a side, and fail loudly when they cannot hold', async () => {
        expect(await client.send('prompt_submit:server: tos-test-editor')).toBe(
            'LAUNCHED:server'
        );
        const noHost = await client.send('prompt_submit:here: tos-test-viewer');
        expect(noHost).toContain('ERROR');
        expect(noHost).toContain('not available on the host');
        const unknown = await client.send('prompt_submit:here: nosuchapp');
        expect(unknown).toContain('ERROR');
        expect(unknown).toContain('not a known app');
    });

    test('explicit prominence parks the launch until the face chooses', async () => {
        await setProminence('explicit');
        const res = await client.send('prompt_submit:tos-test-editor');
        expect(res).toMatch(/^PLACEMENT_PENDING:/);
        const id = res!.split(':')[1];
        let state = await client.state();
        expect(state.placement_pending?.app_name).toBe('TosTestEditor');
        expect(await client.send(`placement_choose:${id};server`)).toBe(
            'LAUNCHED:server'
        );
        state = await client.state();
        expect(state.placement_pending).toBeNull();
        await setProminence('glyph');
    });

    test('relaunch runs a registered launch in the other location', async () => {
        const state = await client.state();
        const hostLaunch = state.placement_launches.find(
            (l) => l.app === 'TosTestHostApp' && l.placement === 'host'
        )!;
        // The server does not have the host-only app: loud refusal.
        const refused = await client.send(
            `placement_relaunch:${hostLaunch.id}`
        );
        expect(refused).toContain('ERROR');
        expect(refused).toContain('not available on the server');
        // The editor is on both machines: its host launch
        // relaunches onto the server.
        const editorLaunch = state.placement_launches.find(
            (l) => l.app === 'TosTestEditor' && l.placement === 'host'
        )!;
        expect(await client.send(`placement_relaunch:${editorLaunch.id}`)).toBe(
            'LAUNCHED:server'
        );
    });
});
