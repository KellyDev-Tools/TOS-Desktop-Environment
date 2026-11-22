import { test, expect } from '@playwright/test';
import { spawn, type ChildProcess } from 'child_process';
import fs from 'fs';
import net from 'net';
import os from 'os';
import path from 'path';
import WebSocket from 'ws';

/**
 * Phase 1.6 — chip lifecycle, verified with a FAKE HEADLESS CHIP
 * SYSTEM (Timothy's requested test, 2026-11-17).
 *
 * The fake chip system is a headless stand-in for a chip UI layer:
 * no browser, no rendering. It speaks the Brain's face protocol
 * directly over WSS (the same `cmd:`/`res:` framing the real face
 * uses), watches the shell hub's directory listing, and treats
 * every new listing GENERATION as a CHIP REQUEST — a fresh set of
 * chips issued at full priority. Per the lifecycle Timothy
 * specified, when a new request arrives the previous sets are
 * deprecated and lowered in priority, and a set whose priority
 * reaches zero falls off the visible chips.
 *
 * The sequence under test is his:
 *   1. `ls` runs            -> a new chip request comes through.
 *   2. A directory chip is selected
 *                           -> `cd <selected dir>` is sent to the terminal.
 *   3. `ls` again           -> verifies the change (new request, new dir).
 *   4. `pwd`                -> even that results in a chip request,
 *                              and the first set has by now fallen off.
 *
 * This works against a stock shell with no OSC integration: the
 * Brain's PTY completion watcher syncs the hub cwd from the shell
 * process (/proc/<pid>/cwd) and re-issues the listing after every
 * completed command.
 */

const ACTIVE_PRIORITY = 3;

interface Chip {
    label: string;
    kind: 'dir' | 'file';
}

interface ChipSet {
    generation: number;
    dir: string;
    chips: Chip[];
    priority: number;
    deprecated: boolean;
}

class FakeChipSystem {
    private ws: WebSocket | null = null;
    private seq = 0;
    private pending = new Map<string, (res: string | null) => void>();
    private pollTimer: NodeJS.Timeout | null = null;

    /** Every chip request observed, in arrival order. */
    readonly requests: ChipSet[] = [];
    /** The Brain's latest reported hub cwd (ground truth checks). */
    hubCwd = '';

    async connect(): Promise<void> {
        this.ws = new WebSocket('wss://localhost:7001', { rejectUnauthorized: false });
        this.ws.on('message', (data) => {
            const msg = data.toString();
            if (!msg.startsWith('res:')) return; // broadcasts are not responses
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
        // Poll the Brain's state like a face would; a listing whose
        // generation is newer than any seen is a new chip request.
        this.pollTimer = setInterval(() => void this.poll(), 150);
        await this.poll();
    }

    async close(): Promise<void> {
        if (this.pollTimer) clearInterval(this.pollTimer);
        this.ws?.close();
    }

    private send(command: string): Promise<string | null> {
        return new Promise((resolve) => {
            const id = `fake${++this.seq}`;
            this.pending.set(id, resolve);
            this.ws!.send(`cmd:${id}:${command}`);
            setTimeout(() => {
                if (this.pending.delete(id)) resolve(null);
            }, 5_000);
        });
    }

    private async poll(): Promise<void> {
        const res = await this.send('get_state:');
        if (!res) return;
        let state: any;
        try {
            state = JSON.parse(res);
        } catch {
            return;
        }
        const sector = state.sectors?.[state.active_sector_index];
        const hub = sector?.hubs?.[sector.active_hub_index];
        if (!hub) return;
        this.hubCwd = hub.current_directory ?? '';
        const listing = hub.shell_listing;
        if (!listing || typeof listing.generation !== 'number') return;
        if (this.requests.some((r) => r.generation === listing.generation)) return;

        // A new generation = a new chip request. The previous sets
        // are deprecated and lowered in priority; a set that reaches
        // zero falls off the visible chips.
        for (const set of this.requests) {
            set.deprecated = true;
            set.priority = Math.max(0, set.priority - 1);
        }
        this.requests.push({
            generation: listing.generation,
            dir: listing.path,
            chips: (listing.entries ?? []).map((e: any) => ({
                label: e.name,
                kind: e.is_dir ? 'dir' : 'file'
            })),
            priority: ACTIVE_PRIORITY,
            deprecated: false
        });
    }

    /** Chips a user would currently see: sets still above zero priority. */
    visibleChips(): (Chip & { generation: number; deprecated: boolean })[] {
        return this.requests
            .filter((s) => s.priority > 0)
            .flatMap((s) =>
                s.chips.map((c) => ({ ...c, generation: s.generation, deprecated: s.deprecated }))
            );
    }

    /** Type a command into the terminal, as a face does: the text
     *  as a text event, the submit as an Enter key event — the
     *  Brain encodes both. */
    async runCommand(command: string): Promise<void> {
        await this.send(`terminal_text:${JSON.stringify({ text: command })}`);
        await this.send(
            `terminal_key:${JSON.stringify({ key: 'Enter', code: 'Enter' })}`
        );
    }

    /** Send a raw command and return the Brain's response. */
    async command(command: string): Promise<string | null> {
        return this.send(command);
    }

    /** Type text into the terminal WITHOUT submitting it, as a
     *  face does while a command is being composed. */
    async typeText(text: string): Promise<void> {
        await this.send(`terminal_text:${JSON.stringify({ text })}`);
    }

    /** The hub's dynamic-chip state (Phase 1.7): the type-ahead
     *  filter and the dotfiles flag, straight from get_state. */
    async chipState(): Promise<{ filter: string | null; showHidden: boolean; entryNames: string[] }> {
        const res = await this.send('get_state:');
        if (!res) return { filter: null, showHidden: false, entryNames: [] };
        const state = JSON.parse(res);
        const sector = state.sectors?.[state.active_sector_index];
        const hub = sector?.hubs?.[sector.active_hub_index];
        return {
            filter: hub?.chip_filter ?? null,
            showHidden: hub?.chips_show_hidden ?? false,
            entryNames: (hub?.shell_listing?.entries ?? []).map((e: any) => e.name as string)
        };
    }

    /** Select a directory chip from the current (newest) set: the
     *  chip system sends `cd <dir>` to the terminal, exactly as the
     *  real AcceleratorChips face does. */
    async selectDirectoryChip(label: string): Promise<void> {
        const current = this.requests[this.requests.length - 1];
        const chip = current?.chips.find((c) => c.kind === 'dir' && c.label === label);
        if (!chip) throw new Error(`no directory chip '${label}' in the current set`);
        const quoted = `'${label.replace(/'/g, `'\\''`)}'`;
        await this.runCommand(`cd ${quoted}`);
    }
}

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

test.describe('Phase 1.6 — chip lifecycle via a fake headless chip system', () => {
    test.describe.configure({ mode: 'serial' });
    test.setTimeout(120_000);

    let brain: ChildProcess | undefined;
    let stateDir: string | undefined;
    let fixtureDir: string | undefined;
    const chips = new FakeChipSystem();

    test.beforeAll(async () => {
        const bin = brainBinary();
        if (!fs.existsSync(bin)) {
            throw new Error(
                `tos-brain binary not found at ${bin}. Build it first: ` +
                    '`cargo build -p tos-brain` (or set TOS_BRAIN_BIN).'
            );
        }
        stateDir = fs.mkdtempSync(path.join(os.tmpdir(), 'tos-chips-e2e-'));
        // Fixture home: alpha-dir (with inner.txt inside) + notes.txt.
        fixtureDir = fs.mkdtempSync(path.join(os.tmpdir(), 'tos-chips-fixture-'));
        fs.mkdirSync(path.join(fixtureDir, 'alpha-dir'));
        fs.writeFileSync(path.join(fixtureDir, 'alpha-dir', 'inner.txt'), 'inner\n');
        fs.writeFileSync(path.join(fixtureDir, 'notes.txt'), 'notes\n');
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
        await chips.connect();
    });

    test.afterAll(async () => {
        await chips.close();
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

    async function waitForRequests(count: number): Promise<void> {
        await expect
            .poll(() => chips.requests.length, { timeout: 20_000 })
            .toBeGreaterThanOrEqual(count);
    }

    test('the full chip-request sequence', async () => {
        // At rest: the startup listing is the first chip request —
        // the fixture's chips at full priority, nothing deprecated.
        await waitForRequests(1);
        const first = chips.requests[0];
        expect(first.dir).toBe(fixtureDir);
        expect(first.chips).toEqual(
            expect.arrayContaining([
                { label: 'alpha-dir', kind: 'dir' },
                { label: 'notes.txt', kind: 'file' }
            ])
        );
        expect(first.priority).toBe(ACTIVE_PRIORITY);
        expect(first.deprecated).toBe(false);

        // 1. Run `ls` -> a new chip request comes through, and the
        //    first set is deprecated with lowered priority.
        await chips.runCommand('ls');
        await waitForRequests(2);
        const second = chips.requests[1];
        expect(second.generation).toBeGreaterThan(first.generation);
        expect(second.priority).toBe(ACTIVE_PRIORITY);
        expect(second.deprecated).toBe(false);
        expect(first.deprecated).toBe(true);
        expect(first.priority).toBe(ACTIVE_PRIORITY - 1);

        // 2. Select the directory chip -> `cd 'alpha-dir'` is sent
        //    to the terminal. A new request follows, describing the
        //    NEW directory (inner.txt, not notes.txt), and the
        //    Brain's own model of the cwd agrees.
        await chips.selectDirectoryChip('alpha-dir');
        await waitForRequests(3);
        const third = chips.requests[2];
        expect(third.dir).toBe(path.join(fixtureDir!, 'alpha-dir'));
        expect(third.chips).toEqual(expect.arrayContaining([{ label: 'inner.txt', kind: 'file' }]));
        expect(third.chips.some((c) => c.label === 'notes.txt')).toBe(false);
        await expect.poll(() => chips.hubCwd, { timeout: 10_000 }).toBe(path.join(fixtureDir!, 'alpha-dir'));

        // 3. `ls` verifies the change -> another new request. The
        //    first set has now been demoted three times and has
        //    fallen off the visible chips entirely.
        await chips.runCommand('ls');
        await waitForRequests(4);
        expect(first.priority).toBe(0);
        expect(chips.visibleChips().some((c) => c.generation === first.generation)).toBe(false);
        //    ...while the current directory's chips remain visible.
        expect(chips.visibleChips().some((c) => c.label === 'inner.txt')).toBe(true);

        // 4. Even `pwd` — which changes nothing — results in a chip
        //    request: a new generation, previous set deprecated.
        const before = chips.requests.length;
        await chips.runCommand('pwd');
        await waitForRequests(before + 1);
        const latest = chips.requests[chips.requests.length - 1];
        const previous = chips.requests[chips.requests.length - 2];
        expect(latest.generation).toBeGreaterThan(previous.generation);
        expect(latest.dir).toBe(previous.dir);
        expect(previous.deprecated).toBe(true);
        expect(latest.deprecated).toBe(false);

        // Generations only ever move forward.
        const gens = chips.requests.map((r) => r.generation);
        expect(gens).toEqual([...gens].sort((a, b) => a - b));
        expect(new Set(gens).size).toBe(gens.length);
    });
});

/**
 * Phase 1.7 — dynamic chips, headless: the Brain tracks the
 * shell's input line and publishes the fragment being typed as
 * the hub's chip_filter; the dotfiles switch flips
 * chips_show_hidden in state. (The face's narrowing render of
 * that state is covered in slice.spec.ts.)
 */
test.describe('Phase 1.7 — dynamic chips via a fake headless chip system', () => {
    test.describe.configure({ mode: 'serial' });
    test.setTimeout(120_000);

    let brain: ChildProcess | undefined;
    let stateDir: string | undefined;
    let fixtureDir: string | undefined;
    const chips = new FakeChipSystem();

    test.beforeAll(async () => {
        const bin = brainBinary();
        if (!fs.existsSync(bin)) {
            throw new Error(
                `tos-brain binary not found at ${bin}. Build it first: ` +
                    '`cargo build -p tos-brain` (or set TOS_BRAIN_BIN).'
            );
        }
        stateDir = fs.mkdtempSync(path.join(os.tmpdir(), 'tos-dyn-e2e-'));
        fixtureDir = fs.mkdtempSync(path.join(os.tmpdir(), 'tos-dyn-fixture-'));
        fs.mkdirSync(path.join(fixtureDir, 'alpha-dir'));
        fs.writeFileSync(path.join(fixtureDir, 'shell-one.txt'), 'one\n');
        fs.writeFileSync(path.join(fixtureDir, 'shell-two.txt'), 'two\n');
        fs.writeFileSync(path.join(fixtureDir, 'other.log'), 'log\n');
        fs.writeFileSync(path.join(fixtureDir, '.hidden-file'), 'hidden\n');
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
        await chips.connect();
    });

    test.afterAll(async () => {
        await chips.close();
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

    async function expectFilter(want: string | null): Promise<void> {
        await expect
            .poll(async () => (await chips.chipState()).filter, { timeout: 15_000 })
            .toBe(want);
    }

    test('typing publishes a filter, editing moves it, desync clears it', async () => {
        // At rest: unfiltered, dotfiles flag off — and the listing
        // itself always carries the dotfile (hiding it is the
        // face's job, driven by the flag).
        const rest = await chips.chipState();
        expect(rest.filter).toBeNull();
        expect(rest.showHidden).toBe(false);
        expect(rest.entryNames).toContain('.hidden-file');
        expect(rest.entryNames).toContain('shell-one.txt');

        // Typing publishes the fragment of the word being typed.
        await chips.typeText('sh');
        await expectFilter('sh');
        await chips.typeText('ell');
        await expectFilter('shell');

        // Backspace moves the fragment back.
        await chips.typeText('\x7f');
        await expectFilter('shel');

        // Ctrl+U clears the line; a fresh word filters from scratch.
        await chips.typeText('\x15');
        await expectFilter(null);
        await chips.typeText('ell');
        await expectFilter('ell');

        // An escape sequence (up-arrow: history recall) desyncs
        // the model — the filter clears and typing does not revive
        // it until a resync point.
        await chips.typeText('\x1b[A');
        await expectFilter(null);
        await chips.typeText('x');
        await expectFilter(null);

        // Ctrl+C resyncs to a known-empty line.
        await chips.typeText('\x03');
        await chips.typeText('pw');
        await expectFilter('pw');

        // Submitting clears the filter with the line.
        await chips.runCommand('');
        await expectFilter(null);
    });

    test('the dotfiles switch flips the flag in Brain state', async () => {
        expect(await chips.command('chips_set_show_hidden:true')).toBe('OK');
        await expect
            .poll(async () => (await chips.chipState()).showHidden, { timeout: 15_000 })
            .toBe(true);
        expect(await chips.command('chips_set_show_hidden:false')).toBe('OK');
        await expect
            .poll(async () => (await chips.chipState()).showHidden, { timeout: 15_000 })
            .toBe(false);
    });
});
