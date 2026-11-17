/**
 * Phase 2 — the Brain honours its [remote] config.
 *
 * Before Phase 2 the Brain ignored RemoteConfig entirely: it bound
 * [::] on fixed ports whatever tos.toml said. These tests spawn a
 * real Brain with a TOS_CONFIG file and probe it over the network:
 *
 *  - remote_access = true + bind_address = this machine's LAN IP:
 *    the Brain answers WSS on that address (a get_state round-trip)
 *    and is NOT reachable on loopback — the bind is real.
 *  - remote_access = false + the same bind_address: the configured
 *    address is ignored (fail closed) — loopback answers, the LAN
 *    address refuses.
 *
 * Headless: speaks the face protocol directly over WSS, like the
 * chip-lifecycle spec. mDNS is disabled in the test config.
 *
 * Environment note: these are real non-loopback TCP connections, so
 * the spec needs an environment that permits them (the CI runner
 * does). Sandboxes that intercept direct TCP to non-loopback
 * addresses will fail the probes even though the Brain bound
 * correctly — verify the bind there with `ss -tln` instead.
 */
import { test, expect } from '@playwright/test';
import { spawn, type ChildProcess } from 'node:child_process';
import * as fs from 'node:fs';
import * as net from 'node:net';
import * as os from 'node:os';
import * as path from 'node:path';
import WebSocket from 'ws';

function brainBinary(): string {
    if (process.env.TOS_BRAIN_BIN) return process.env.TOS_BRAIN_BIN;
    return path.resolve(process.cwd(), '..', 'target', 'debug', 'tos-brain');
}

/** This machine's first non-loopback IPv4 — the "remote" address. */
function lanAddress(): string | null {
    for (const infos of Object.values(os.networkInterfaces())) {
        for (const info of infos ?? []) {
            if (info.family === 'IPv4' && !info.internal) return info.address;
        }
    }
    return null;
}

interface RunningBrain {
    proc: ChildProcess;
    dirs: string[];
}

async function startBrain(remoteAccess: boolean, bindAddress: string): Promise<RunningBrain> {
    const bin = brainBinary();
    if (!fs.existsSync(bin)) {
        throw new Error(
            `tos-brain binary not found at ${bin}. Build it first: ` +
                '`cargo build -p tos-brain` (or set TOS_BRAIN_BIN).'
        );
    }
    const stateDir = fs.mkdtempSync(path.join(os.tmpdir(), 'tos-remote-e2e-'));
    const fixtureDir = fs.mkdtempSync(path.join(os.tmpdir(), 'tos-remote-fixture-'));
    const udsPath = path.join(stateDir, 'brain.sock');
    const configPath = path.join(stateDir, 'tos.toml');
    fs.writeFileSync(
        configPath,
        [
            '[remote]',
            'anchor_port = 7000',
            'ws_port = 7001',
            `bind_address = "${bindAddress}"`,
            `uds_path = "${udsPath}"`,
            'mdns_enabled = false',
            `remote_access = ${remoteAccess}`,
            ''
        ].join('\n')
    );
    const proc = spawn(bin, [], {
        env: {
            ...process.env,
            TOS_CONFIG: configPath,
            HOME: fixtureDir,
            XDG_CONFIG_HOME: path.join(stateDir, 'config'),
            XDG_DATA_HOME: path.join(stateDir, 'data')
        },
        stdio: 'ignore'
    });
    // The UDS socket appears once run() has bound the listeners.
    await expect
        .poll(() => fs.existsSync(udsPath), { timeout: 30_000 })
        .toBe(true);
    return { proc, dirs: [stateDir, fixtureDir] };
}

async function stopBrain(brain: RunningBrain): Promise<void> {
    const exited = new Promise<void>((resolve) => {
        brain.proc.once('exit', () => resolve());
    });
    brain.proc.kill('SIGTERM');
    await Promise.race([
        exited,
        new Promise<void>((r) => setTimeout(r, 5_000)).then(() => {
            brain.proc.kill('SIGKILL');
        })
    ]);
    const rmOpts = { recursive: true, force: true, maxRetries: 5, retryDelay: 200 } as const;
    for (const dir of brain.dirs) fs.rmSync(dir, rmOpts);
}

function tcpReachable(host: string, port: number, timeoutMs = 3_000): Promise<boolean> {
    return new Promise((resolve) => {
        const socket = net.connect(port, host);
        const done = (ok: boolean) => {
            socket.destroy();
            resolve(ok);
        };
        socket.setTimeout(timeoutMs, () => done(false));
        socket.on('connect', () => done(true));
        socket.on('error', () => done(false));
    });
}

/** A real face-protocol round-trip: connect WSS, ask get_state. */
function wsGetState(host: string, timeoutMs = 10_000): Promise<boolean> {
    return new Promise((resolve) => {
        const ws = new WebSocket(`wss://${host}:7001`, { rejectUnauthorized: false });
        const timer = setTimeout(() => {
            ws.terminate();
            resolve(false);
        }, timeoutMs);
        ws.on('open', () => ws.send('cmd:1:get_state:'));
        ws.on('message', (data) => {
            if (data.toString().startsWith('res:1:')) {
                clearTimeout(timer);
                ws.close();
                resolve(true);
            }
        });
        ws.on('error', () => {
            clearTimeout(timer);
            resolve(false);
        });
    });
}

test.describe('Phase 2 — remote config', () => {
    test.describe.configure({ mode: 'serial' });
    test.setTimeout(120_000);

    const lan = lanAddress();

    test('remote access on: answers on the configured bind address only', async () => {
        test.skip(!lan, 'no non-loopback IPv4 on this machine');
        const brain = await startBrain(true, lan!);
        try {
            // The tailnet-style address answers a real protocol round-trip…
            await expect.poll(() => wsGetState(lan!), { timeout: 30_000 }).toBe(true);
            // …and loopback does not: the configured bind is the only door.
            expect(await tcpReachable('127.0.0.1', 7001)).toBe(false);
            expect(await tcpReachable('127.0.0.1', 7000)).toBe(false);
        } finally {
            await stopBrain(brain);
        }
    });

    test('remote access off: the configured bind is ignored, loopback only', async () => {
        test.skip(!lan, 'no non-loopback IPv4 on this machine');
        const brain = await startBrain(false, lan!);
        try {
            // Fail closed: loopback answers…
            await expect.poll(() => wsGetState('127.0.0.1'), { timeout: 30_000 }).toBe(true);
            // …and the configured LAN address refuses.
            expect(await tcpReachable(lan!, 7001)).toBe(false);
            expect(await tcpReachable(lan!, 7000)).toBe(false);
        } finally {
            await stopBrain(brain);
        }
    });
});
