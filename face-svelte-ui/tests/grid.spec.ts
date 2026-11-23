import { test, expect } from '@playwright/test';
import { spawn, type ChildProcess } from 'node:child_process';
import * as fs from 'node:fs';
import * as net from 'node:net';
import * as os from 'node:os';
import * as path from 'node:path';
import WebSocket from 'ws';

/**
 * The Brain-owned grid, verified headlessly through the display
 * protocol (Architecture §26.3).
 *
 * The Brain parses the PTY stream once, into a per-session grid;
 * faces receive display state and send raw input. This spec
 * speaks that protocol directly over WSS against a real Brain
 * and asserts the properties the model exists for:
 *
 *  - subscribing answers with a snapshot of a live shell;
 *  - typing arrives as raw key events, the Brain encodes them,
 *    and the shell's in-place input-line redraws stay on ONE
 *    grid row — the staircase fault has no mechanism here;
 *  - a submitted command's output appears exactly once;
 *  - sessions are addressed by id: the primary answers to its
 *    own id, an unknown id is an error, and input routes by id;
 *  - scrollback is a window the Brain serves from the grid.
 */

function brainBinary(): string {
    if (process.env.TOS_BRAIN_BIN) return process.env.TOS_BRAIN_BIN;
    return path.resolve(process.cwd(), '..', 'target', 'debug', 'tos-brain');
}

interface Snapshot {
    seq: number;
    display_offset: number;
    lines: { text: string }[][];
}

function parseDisplay(message: string): { session: string; snap: Snapshot } | null {
    const jsonStart = message.indexOf('{');
    if (jsonStart < 1) return null;
    try {
        return {
            session: message.substring(0, jsonStart - 1).split(':').slice(1).join(':'),
            snap: JSON.parse(message.substring(jsonStart)) as Snapshot
        };
    } catch {
        return null;
    }
}

function snapText(snap: Snapshot): string {
    return snap.lines.map((row) => row.map((s) => s.text).join('')).join('\n');
}

class GridClient {
    private ws: WebSocket | null = null;
    private seq = 0;
    private pending = new Map<string, (res: string | null) => void>();
    latest: Snapshot | null = null;
    session = '';

    async connect(): Promise<void> {
        this.ws = new WebSocket('wss://localhost:7001', { rejectUnauthorized: false });
        this.ws.on('message', (data) => {
            const msg = data.toString();
            if (msg.startsWith('display_update:')) {
                const parsed = parseDisplay(msg);
                if (parsed) this.latest = parsed.snap;
                return;
            }
            if (!msg.startsWith('res:')) return;
            const rest = msg.substring(4);
            const sep = rest.indexOf(':');
            if (sep < 0) return;
            const id = rest.substring(0, sep);
            const resolve = this.pending.get(id);
            if (resolve) {
                this.pending.delete(id);
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
            const id = `grid${++this.seq}`;
            this.pending.set(id, resolve);
            this.ws!.send(`cmd:${id}:${command}`);
            setTimeout(() => {
                if (this.pending.delete(id)) resolve(null);
            }, 10_000);
        });
    }

    async subscribe(session = ''): Promise<{ session: string; snap: Snapshot }> {
        const res = await this.send(`display_subscribe:${session}`);
        expect(res, 'subscribe response').not.toBeNull();
        expect(res!).toContain('display_snapshot:');
        const parsed = parseDisplay(res!);
        expect(parsed, 'snapshot parses').not.toBeNull();
        this.latest = parsed!.snap;
        if (!session) this.session = parsed!.session;
        return parsed!;
    }

    async key(key: string, extra: Record<string, unknown> = {}): Promise<void> {
        const res = await this.send(
            `terminal_key:${JSON.stringify({ key, code: '', ...extra })}`
        );
        expect(res).toBe('OK');
    }

    async typeText(text: string): Promise<void> {
        for (const ch of text) {
            await this.key(ch);
        }
    }
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

test.describe('Brain-owned grid — the display protocol, headless', () => {
    test.describe.configure({ mode: 'serial' });
    test.setTimeout(120_000);

    let brain: ChildProcess | undefined;
    let stateDir: string | undefined;
    let fixtureDir: string | undefined;
    const client = new GridClient();

    test.beforeAll(async () => {
        const bin = brainBinary();
        if (!fs.existsSync(bin)) {
            throw new Error(
                `tos-brain binary not found at ${bin}. Build it first: ` +
                    '`cargo build -p tos-brain` (or set TOS_BRAIN_BIN).'
            );
        }
        stateDir = fs.mkdtempSync(path.join(os.tmpdir(), 'tos-grid-e2e-'));
        fixtureDir = fs.mkdtempSync(path.join(os.tmpdir(), 'tos-grid-fixture-'));
        const env = { ...process.env };
        // Simulate headless systemd --user where TERM=dumb: the Brain
        // itself must override TERM=xterm-256color for its PTY shell.
        env.TERM = 'dumb';
        delete env.COLORTERM;
        brain = spawn(bin, [], {
            env: {
                ...env,
                HOME: fixtureDir,
                XDG_CONFIG_HOME: path.join(stateDir, 'config'),
                XDG_DATA_HOME: path.join(stateDir, 'data')
            },
            stdio: 'ignore'
        });
        await waitForPort(7001);
        await client.connect();
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
                new Promise<void>((r) => setTimeout(r, 5_000)).then(() => {
                    brainProc.kill('SIGKILL');
                })
            ]);
        }
        const rmOpts = { recursive: true, force: true, maxRetries: 5, retryDelay: 200 } as const;
        if (stateDir) fs.rmSync(stateDir, rmOpts);
        if (fixtureDir) fs.rmSync(fixtureDir, rmOpts);
    });

    test('subscribe answers with a snapshot of a live shell', async () => {
        await expect
            .poll(
                async () => {
                    const { snap } = await client.subscribe();
                    return snapText(snap);
                },
                { timeout: 20_000 }
            )
            .toMatch(/[#$>]/);
        expect(client.session).toContain(':');
    });

    test('raw key events type, redraws stay on one row, output lands once', async () => {
        await client.typeText('echo TOS_GRID_OK');
        // Settled: the whole command sits on exactly one grid row.
        await expect
            .poll(() => (client.latest ? snapText(client.latest) : ''), {
                timeout: 15_000
            })
            .toContain('echo TOS_GRID_OK');
        const rows = snapText(client.latest!).split('\n');
        expect(rows.filter((r) => r.includes('echo TOS_GRID'))).toHaveLength(1);

        await client.key('Enter');
        // Wait for the OUTPUT row (the command row already
        // contains the marker, so it cannot be the predicate).
        await expect
            .poll(
                () =>
                    client.latest
                        ? snapText(client.latest)
                              .split('\n')
                              .some((r) => r.trim() === 'TOS_GRID_OK')
                        : false,
                { timeout: 15_000 }
            )
            .toBe(true);
        const after = snapText(client.latest!).split('\n');
        // The command row plus exactly one output row.
        expect(after.filter((r) => r.includes('TOS_GRID_OK'))).toHaveLength(2);
        expect(
            after.filter((r) => r.trim() === 'TOS_GRID_OK')
        ).toHaveLength(1);
    });

    test('sessions are addressed by id', async () => {
        // The primary session answers to its own id.
        const { snap } = await client.subscribe(client.session);
        expect(snap.seq).toBeGreaterThan(0);
        // An unknown id is a loud error, not a silent fallback.
        const bogus = await client.send('display_subscribe:no-such-session');
        expect(bogus).toContain('ERROR: Unknown session');
        // Input routes by id: a key for the real session lands,
        // and Ctrl+U clears the line again.
        const routed = await client.send(
            `terminal_key:${JSON.stringify({ session: client.session, key: 'x', code: 'KeyX' })}`
        );
        expect(routed).toBe('OK');
        const wrong = await client.send(
            `terminal_key:${JSON.stringify({ session: 'no-such-session', key: 'x', code: 'KeyX' })}`
        );
        expect(wrong).toContain('ERROR: Unknown session');
        await client.key('u', { ctrl: true });
    });

    test('scrollback is a window the Brain serves from the grid', async () => {
        const res = await client.send(
            `terminal_text:${JSON.stringify({ text: 'seq -f gridline%.0f 1 40' })}`
        );
        expect(res).toBe('OK');
        await client.key('Enter');
        await expect
            .poll(() => (client.latest ? snapText(client.latest) : ''), {
                timeout: 20_000
            })
            .toContain('gridline40');

        const scrolled = await client.send('display_scroll:;10');
        expect(scrolled).toContain('display_snapshot:');
        const parsed = parseDisplay(scrolled!);
        expect(parsed!.snap.display_offset).toBeGreaterThan(0);
        // Older lines are in this window.
        expect(snapText(parsed!.snap)).toContain('gridline');

        const back = await client.send('display_scroll:;-10');
        const parsedBack = parseDisplay(back!);
        expect(parsedBack!.snap.display_offset).toBe(0);
    });
});
