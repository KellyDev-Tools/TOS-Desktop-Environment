import { test, expect } from '@playwright/test';
import { spawn, type ChildProcess } from 'node:child_process';
import * as fs from 'node:fs';
import * as net from 'node:net';
import * as os from 'node:os';
import * as path from 'node:path';
import WebSocket from 'ws';

/**
 * Phase 2 — three-stream narrowing, verified headlessly.
 *
 * The Brain's remote transports (WebSocket + TCP) may only carry
 * three streams — terminal, chip state, prompt. This spec speaks
 * the face protocol directly over WSS (same pattern as the
 * chip-lifecycle spec, no browser) against a real Brain and asserts:
 *
 *  - an allowed command (get_state) answers normally;
 *  - an allowed terminal command actually reaches the shell
 *    (the echo marker comes back in display_update grid state);
 *  - the retired raw-stream command (terminal_input_hex) is
 *    refused like everything else outside the gate — no raw
 *    stream survives alongside the display-state protocol;
 *  - parked commands are refused loudly: marketplace (parked
 *    subsystem), ai_submit (parked AI), and set_mode (a grey-area
 *    command — parked over remote by explicit ruling, see the
 *    allowlist in tos-common/src/platform/remote_server.rs);
 *  - the refusal keeps the cmd:<id> framing, so a caller sees the
 *    error instead of hanging;
 *  - the local UDS path is NOT narrowed: the same marketplace
 *    command over the Unix socket gets a real answer, not a refusal.
 *
 * The gate is per-transport, not per-bind: it applies to the WS
 * listener even on loopback, which is what this spec exercises.
 */

function brainBinary(): string {
    if (process.env.TOS_BRAIN_BIN) return process.env.TOS_BRAIN_BIN;
    return path.resolve(process.cwd(), '..', 'target', 'debug', 'tos-brain');
}

/** The text rows of a display message (snapshot or update). */
function displayText(message: string): string {
    const jsonStart = message.indexOf('{');
    if (jsonStart < 0) return '';
    try {
        const snap = JSON.parse(message.substring(jsonStart));
        return (snap.lines ?? [])
            .map((row: { text: string }[]) => row.map((s) => s.text).join(''))
            .join('\n');
    } catch {
        return '';
    }
}

class RemoteClient {
    private ws: WebSocket | null = null;
    private seq = 0;
    private pending = new Map<string, (res: string | null) => void>();
    /** The text of the latest display state received. */
    terminalText = '';

    async connect(): Promise<void> {
        this.ws = new WebSocket('wss://localhost:7001', { rejectUnauthorized: false });
        this.ws.on('message', (data) => {
            const msg = data.toString();
            if (msg.startsWith('display_update:')) {
                this.terminalText = displayText(msg);
                return;
            }
            if (!msg.startsWith('res:')) return; // other broadcasts are not responses
            const rest = msg.substring(4);
            const sep = rest.indexOf(':');
            if (sep < 0) return;
            const id = rest.substring(0, sep);
            const resolve = this.pending.get(id);
            if (resolve) {
                this.pending.delete(id);
                const payload = rest.substring(sep + 1);
                if (payload.startsWith('display_snapshot:')) {
                    this.terminalText = displayText(payload);
                }
                resolve(payload);
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
            const id = `narrow${++this.seq}`;
            this.pending.set(id, resolve);
            this.ws!.send(`cmd:${id}:${command}`);
            setTimeout(() => {
                if (this.pending.delete(id)) resolve(null);
            }, 10_000);
        });
    }
}

/** One request/response round-trip over the Brain's Unix socket. */
function udsRequest(udsPath: string, command: string): Promise<string> {
    return new Promise((resolve, reject) => {
        const socket = net.connect(udsPath);
        let buffer = '';
        socket.setTimeout(10_000, () => {
            socket.destroy();
            reject(new Error('UDS request timed out'));
        });
        socket.on('connect', () => socket.write(command + '\n'));
        socket.on('data', (chunk) => {
            buffer += chunk.toString();
            const nl = buffer.indexOf('\n');
            if (nl >= 0) {
                socket.end();
                resolve(buffer.substring(0, nl));
            }
        });
        socket.on('error', reject);
    });
}

test.describe('Phase 2 — three-stream narrowing', () => {
    test.describe.configure({ mode: 'serial' });
    test.setTimeout(120_000);

    let brain: ChildProcess | undefined;
    let stateDir: string | undefined;
    let fixtureDir: string | undefined;
    let udsPath = '';
    const client = new RemoteClient();

    test.beforeAll(async () => {
        const bin = brainBinary();
        if (!fs.existsSync(bin)) {
            throw new Error(
                `tos-brain binary not found at ${bin}. Build it first: ` +
                    '`cargo build -p tos-brain` (or set TOS_BRAIN_BIN).'
            );
        }
        stateDir = fs.mkdtempSync(path.join(os.tmpdir(), 'tos-narrow-e2e-'));
        fixtureDir = fs.mkdtempSync(path.join(os.tmpdir(), 'tos-narrow-fixture-'));
        udsPath = path.join(stateDir, 'brain.sock');
        const configPath = path.join(stateDir, 'tos.toml');
        fs.writeFileSync(
            configPath,
            [
                '[remote]',
                'anchor_port = 7000',
                'ws_port = 7001',
                'bind_address = "127.0.0.1"',
                `uds_path = "${udsPath}"`,
                'mdns_enabled = false',
                'remote_access = false',
                ''
            ].join('\n')
        );
        brain = spawn(bin, [], {
            env: {
                ...process.env,
                TOS_CONFIG: configPath,
                HOME: fixtureDir,
                XDG_CONFIG_HOME: path.join(stateDir, 'config'),
                XDG_DATA_HOME: path.join(stateDir, 'data')
            },
            stdio: 'ignore'
        });
        await expect.poll(() => fs.existsSync(udsPath), { timeout: 30_000 }).toBe(true);
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

    test('an allowed command answers normally over remote', async () => {
        const res = await client.send('get_state:');
        expect(res).not.toBeNull();
        expect(res).not.toContain('not available over a remote connection');
        const state = JSON.parse(res!);
        expect(state).toHaveProperty('sectors');
    });

    test('terminal input reaches the shell over remote', async () => {
        const sub = await client.send('display_subscribe:');
        expect(sub).toContain('display_snapshot:');
        const textRes = await client.send(
            `terminal_text:${JSON.stringify({ text: 'echo TOS_NARROW_OK' })}`
        );
        expect(textRes).toBe('OK');
        const keyRes = await client.send(
            `terminal_key:${JSON.stringify({ key: 'Enter', code: 'Enter' })}`
        );
        expect(keyRes).toBe('OK');
        await expect
            .poll(() => client.terminalText, { timeout: 15_000 })
            .toContain('TOS_NARROW_OK');
    });

    test('the retired raw-stream command is refused over remote', async () => {
        const res = await client.send('terminal_input_hex:6869');
        expect(res).toContain(
            'ERROR: terminal_input_hex is not available over a remote connection'
        );
    });

    test('the chips dotfiles switch is allowed over remote (chip-state stream)', async () => {
        // Phase 1.7: chips_set_show_hidden belongs to the chip-state
        // stream — the panel's switch must keep working on a remote
        // face, and the flag must land in Brain state.
        const res = await client.send('chips_set_show_hidden:true');
        expect(res).toBe('OK');
        const stateRes = await client.send('get_state:');
        const state = JSON.parse(stateRes!);
        const sector = state.sectors[state.active_sector_index];
        const hub = sector.hubs[sector.active_hub_index];
        expect(hub.chips_show_hidden).toBe(true);
        // Leave the flag as found for the tests that follow.
        await client.send('chips_set_show_hidden:false');
    });

    test('parked subsystems are refused loudly over remote', async () => {
        const marketplace = await client.send('marketplace_home:');
        expect(marketplace).toContain(
            'ERROR: marketplace_home is not available over a remote connection'
        );

        const ai = await client.send('ai_submit:hello');
        expect(ai).toContain('ERROR: ai_submit is not available over a remote connection');

        const process = await client.send('process_signal:1;9');
        expect(process).toContain(
            'ERROR: process_signal is not available over a remote connection'
        );
    });

    test('grey-area commands are parked over remote by ruling', async () => {
        // set_mode / session_save are not one of the three streams.
        // They keep working locally (UDS, below) but not remotely.
        const mode = await client.send('set_mode:CommandHub');
        expect(mode).toContain('ERROR: set_mode is not available over a remote connection');

        const save = await client.send('session_save:checkpoint');
        expect(save).toContain(
            'ERROR: session_save is not available over a remote connection'
        );
    });

    test('the local UDS path keeps the full surface', async () => {
        const res = await udsRequest(udsPath, 'marketplace_home:');
        expect(res.length).toBeGreaterThan(0);
        expect(res).not.toContain('not available over a remote connection');
    });
});
