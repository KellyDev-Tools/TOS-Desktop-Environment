/**
 * TOS Electron — Brain Discovery Service
 * ─────────────────────────────────────────────────────────────────────────────
 * Discovers TOS Brain instances on the local network via:
 *   1. Local socket probe ($XDG_RUNTIME_DIR/tos/brain.sock)
 *   2. Environment variable (TOS_BRAIN_WS)
 *   3. mDNS/DNS-SD scan (_tos-brain._tcp)
 *   4. Saved remote hosts (~/.config/tos/remote-hosts.toml)
 *   5. Manual host:port entry (settable via IPC)
 *
 * Priority cascade: env var → local socket → saved hosts → mDNS scan
 */

import { ipcMain } from 'electron';
import * as net from 'net';
import * as dgram from 'dgram';
import * as path from 'path';
import * as fs from 'fs';
import * as os from 'os';

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

export interface BrainInstance {
    /** Display name (hostname or mDNS service name) */
    name: string;
    /** WebSocket URL to connect to */
    wsUrl: string;
    /** How this instance was discovered */
    source: 'local' | 'env' | 'mdns' | 'saved' | 'manual';
    /** TCP host */
    host: string;
    /** WebSocket port */
    wsPort: number;
    /** Anchor (TCP) port, if known */
    anchorPort?: number;
    /** Whether the Brain responded to a probe */
    reachable: boolean;
    /** Last seen timestamp */
    lastSeen: number;
}

export interface DiscoveryState {
    /** Currently connected Brain URL */
    activeUrl: string;
    /** All discovered instances */
    instances: BrainInstance[];
    /** Whether an mDNS scan is in progress */
    scanning: boolean;
}

// ─────────────────────────────────────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────────────────────────────────────

const DEFAULT_WS_PORT = 7001;
const DEFAULT_ANCHOR_PORT = 7000;
const MDNS_MULTICAST_ADDR = '224.0.0.251';
const MDNS_PORT = 5353;
const PROBE_TIMEOUT_MS = 2000;
const SCAN_DURATION_MS = 5000;
const SAVED_HOSTS_FILE = path.join(
    process.env.HOME || os.homedir(),
    '.config', 'tos', 'remote-hosts.toml'
);

// ─────────────────────────────────────────────────────────────────────────────
// State
// ─────────────────────────────────────────────────────────────────────────────

let discoveryState: DiscoveryState = {
    activeUrl: process.env.TOS_BRAIN_WS ?? `ws://127.0.0.1:${DEFAULT_WS_PORT}`,
    instances: [],
    scanning: false,
};

// ─────────────────────────────────────────────────────────────────────────────
// TCP Probe — checks if a host:port is reachable
// ─────────────────────────────────────────────────────────────────────────────

function probeHost(host: string, port: number, timeoutMs: number = PROBE_TIMEOUT_MS): Promise<boolean> {
    return new Promise((resolve) => {
        const socket = new net.Socket();
        socket.setTimeout(timeoutMs);

        socket.on('connect', () => {
            socket.destroy();
            resolve(true);
        });

        socket.on('timeout', () => {
            socket.destroy();
            resolve(false);
        });

        socket.on('error', () => {
            socket.destroy();
            resolve(false);
        });

        socket.connect(port, host);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Local Brain Detection
// ─────────────────────────────────────────────────────────────────────────────

async function discoverLocalBrain(): Promise<BrainInstance | null> {
    const host = '127.0.0.1';
    const reachable = await probeHost(host, DEFAULT_WS_PORT);

    if (reachable) {
        return {
            name: 'Local Brain',
            wsUrl: `ws://${host}:${DEFAULT_WS_PORT}`,
            source: 'local',
            host,
            wsPort: DEFAULT_WS_PORT,
            anchorPort: DEFAULT_ANCHOR_PORT,
            reachable: true,
            lastSeen: Date.now(),
        };
    }

    // Try probing the anchor port to get the port map
    const anchorReachable = await probeHost(host, DEFAULT_ANCHOR_PORT);
    if (anchorReachable) {
        return {
            name: 'Local Brain',
            wsUrl: `ws://${host}:${DEFAULT_WS_PORT}`,
            source: 'local',
            host,
            wsPort: DEFAULT_WS_PORT,
            anchorPort: DEFAULT_ANCHOR_PORT,
            reachable: true,
            lastSeen: Date.now(),
        };
    }

    return null;
}

// ─────────────────────────────────────────────────────────────────────────────
// Saved Hosts — reads ~/.config/tos/remote-hosts.toml
// ─────────────────────────────────────────────────────────────────────────────

interface SavedHost {
    name: string;
    host: string;
    port: number;
}

function loadSavedHosts(): SavedHost[] {
    try {
        if (!fs.existsSync(SAVED_HOSTS_FILE)) return [];
        const content = fs.readFileSync(SAVED_HOSTS_FILE, 'utf-8');

        // Simple TOML parser for [[remote]] entries
        const hosts: SavedHost[] = [];
        let current: Partial<SavedHost> | null = null;

        for (const line of content.split('\n')) {
            const trimmed = line.trim();
            if (trimmed === '[[remote]]') {
                if (current?.host) hosts.push(current as SavedHost);
                current = { name: 'Remote Brain', port: DEFAULT_WS_PORT };
            } else if (current && trimmed.includes('=')) {
                const [key, ...rest] = trimmed.split('=');
                const value = rest.join('=').trim().replace(/^["']|["']$/g, '');
                const k = key.trim();
                if (k === 'name') current.name = value;
                else if (k === 'host') current.host = value;
                else if (k === 'port') current.port = parseInt(value, 10) || DEFAULT_WS_PORT;
            }
        }
        if (current?.host) hosts.push(current as SavedHost);
        return hosts;
    } catch (err) {
        console.warn('[Discovery] Failed to load saved hosts:', err);
        return [];
    }
}

function saveSavedHosts(hosts: SavedHost[]): void {
    try {
        const dir = path.dirname(SAVED_HOSTS_FILE);
        if (!fs.existsSync(dir)) {
            fs.mkdirSync(dir, { recursive: true });
        }

        const lines = hosts.map(h =>
            `[[remote]]\nname = "${h.name}"\nhost = "${h.host}"\nport = ${h.port}\n`
        );
        fs.writeFileSync(SAVED_HOSTS_FILE, lines.join('\n'), 'utf-8');
    } catch (err) {
        console.warn('[Discovery] Failed to save hosts:', err);
    }
}

async function discoverSavedHosts(): Promise<BrainInstance[]> {
    const saved = loadSavedHosts();
    const results: BrainInstance[] = [];

    for (const host of saved) {
        const reachable = await probeHost(host.host, host.port);
        results.push({
            name: host.name,
            wsUrl: `ws://${host.host}:${host.port}`,
            source: 'saved',
            host: host.host,
            wsPort: host.port,
            reachable,
            lastSeen: reachable ? Date.now() : 0,
        });
    }

    return results;
}

// ─────────────────────────────────────────────────────────────────────────────
// mDNS Discovery — scans for _tos-brain._tcp services
// ─────────────────────────────────────────────────────────────────────────────

/**
 * Build a minimal mDNS query packet for _tos-brain._tcp.local
 * DNS wire format: header (12 bytes) + question section
 */
function buildMdnsQuery(): Buffer {
    const header = Buffer.alloc(12);
    // Transaction ID = 0 (standard for mDNS)
    // Flags = 0 (standard query)
    header.writeUInt16BE(1, 4); // QDCOUNT = 1

    // Question: _tos-brain._tcp.local
    const labels = ['_tos-brain', '_tcp', 'local'];
    const parts: Buffer[] = [header];

    for (const label of labels) {
        const len = Buffer.alloc(1);
        len.writeUInt8(label.length);
        parts.push(len, Buffer.from(label, 'utf-8'));
    }
    parts.push(Buffer.alloc(1)); // null terminator
    // QTYPE = PTR (12), QCLASS = IN (1)
    const qtype = Buffer.alloc(4);
    qtype.writeUInt16BE(12, 0);  // PTR
    qtype.writeUInt16BE(1, 2);   // IN
    parts.push(qtype);

    return Buffer.concat(parts);
}

/**
 * Parse a basic mDNS response to extract service instances.
 * This is a simplified parser that looks for TXT records containing
 * brain_ws port information and A/AAAA records for addresses.
 */
function parseMdnsResponse(msg: Buffer, rinfo: dgram.RemoteInfo): BrainInstance | null {
    try {
        // Skip if too small for a valid DNS response
        if (msg.length < 12) return null;

        const flags = msg.readUInt16BE(2);
        const isResponse = (flags & 0x8000) !== 0;
        if (!isResponse) return null;

        // Simple heuristic: if the response came from a host and contains
        // "tos-brain" in the payload, treat it as a Brain instance
        const payload = msg.toString('utf-8', 12);
        if (!payload.includes('tos-brain')) return null;

        // Try to extract port from TXT record (brain_ws=XXXX)
        let wsPort = DEFAULT_WS_PORT;
        const wsMatch = payload.match(/brain_ws=(\d+)/);
        if (wsMatch) {
            wsPort = parseInt(wsMatch[1], 10);
        }

        let anchorPort: number | undefined;
        const tcpMatch = payload.match(/brain_tcp=(\d+)/);
        if (tcpMatch) {
            anchorPort = parseInt(tcpMatch[1], 10);
        }

        return {
            name: `Brain @ ${rinfo.address}`,
            wsUrl: `ws://${rinfo.address}:${wsPort}`,
            source: 'mdns',
            host: rinfo.address,
            wsPort,
            anchorPort,
            reachable: true, // responded to mDNS = reachable
            lastSeen: Date.now(),
        };
    } catch {
        return null;
    }
}

function scanMdns(durationMs: number = SCAN_DURATION_MS): Promise<BrainInstance[]> {
    return new Promise((resolve) => {
        const instances: BrainInstance[] = [];
        const seenHosts = new Set<string>();

        let socket: dgram.Socket;
        try {
            socket = dgram.createSocket({ type: 'udp4', reuseAddr: true });
        } catch {
            console.warn('[Discovery] Failed to create mDNS socket');
            resolve([]);
            return;
        }

        const cleanup = () => {
            try { socket.close(); } catch { }
            resolve(instances);
        };

        socket.on('message', (msg, rinfo) => {
            if (seenHosts.has(rinfo.address)) return;
            const instance = parseMdnsResponse(msg, rinfo);
            if (instance) {
                seenHosts.add(rinfo.address);
                instances.push(instance);
                console.log(`[Discovery] Found Brain via mDNS: ${instance.wsUrl}`);
            }
        });

        socket.on('error', (err) => {
            console.warn('[Discovery] mDNS socket error:', err.message);
            cleanup();
        });

        socket.bind(0, () => {
            try {
                socket.addMembership(MDNS_MULTICAST_ADDR);
                const query = buildMdnsQuery();
                socket.send(query, 0, query.length, MDNS_PORT, MDNS_MULTICAST_ADDR);
                console.log('[Discovery] mDNS query sent for _tos-brain._tcp.local');
            } catch (err) {
                console.warn('[Discovery] Failed to send mDNS query:', err);
            }
        });

        setTimeout(cleanup, durationMs);
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// Full Discovery — runs all discovery methods
// ─────────────────────────────────────────────────────────────────────────────

async function runFullDiscovery(): Promise<BrainInstance[]> {
    console.log('[Discovery] Starting full Brain discovery...');
    discoveryState.scanning = true;

    const allInstances: BrainInstance[] = [];
    const seenUrls = new Set<string>();

    const addUnique = (instance: BrainInstance) => {
        if (!seenUrls.has(instance.wsUrl)) {
            seenUrls.add(instance.wsUrl);
            allInstances.push(instance);
        }
    };

    // 1. If env var set, add it first
    if (process.env.TOS_BRAIN_WS) {
        try {
            const envUrl = new URL(process.env.TOS_BRAIN_WS);
            const host = envUrl.hostname;
            const port = parseInt(envUrl.port, 10) || DEFAULT_WS_PORT;
            const reachable = await probeHost(host, port);
            addUnique({
                name: 'Environment Override',
                wsUrl: process.env.TOS_BRAIN_WS,
                source: 'env',
                host,
                wsPort: port,
                reachable,
                lastSeen: reachable ? Date.now() : 0,
            });
        } catch { }
    }

    // 2. Probe local Brain
    const local = await discoverLocalBrain();
    if (local) addUnique(local);

    // 3. Load saved hosts (probe in parallel)
    const saved = await discoverSavedHosts();
    saved.forEach(addUnique);

    // 4. mDNS scan (runs for SCAN_DURATION_MS)
    try {
        const mdns = await scanMdns();
        mdns.forEach(addUnique);
    } catch (err) {
        console.warn('[Discovery] mDNS scan failed:', err);
    }

    discoveryState.instances = allInstances;
    discoveryState.scanning = false;

    console.log(`[Discovery] Found ${allInstances.length} Brain instances:`);
    allInstances.forEach(i => {
        console.log(`  - ${i.name} (${i.wsUrl}) [${i.source}] ${i.reachable ? '✅' : '❌'}`);
    });

    return allInstances;
}

// ─────────────────────────────────────────────────────────────────────────────
// IPC Handlers
// ─────────────────────────────────────────────────────────────────────────────

export function registerDiscoveryHandlers(): void {
    // Get current discovery state
    ipcMain.handle('tos:discovery-state', () => {
        return discoveryState;
    });

    // Trigger a full discovery scan
    ipcMain.handle('tos:discovery-scan', async () => {
        const instances = await runFullDiscovery();
        return instances;
    });

    // Connect to a specific Brain instance
    ipcMain.handle('tos:discovery-connect', (_event, wsUrl: string) => {
        discoveryState.activeUrl = wsUrl;
        console.log(`[Discovery] Active Brain changed to: ${wsUrl}`);
        return { success: true, wsUrl };
    });

    // Add a manual host entry
    ipcMain.handle('tos:discovery-add-host', async (_event, host: string, port: number, name?: string) => {
        const wsPort = port || DEFAULT_WS_PORT;
        const displayName = name || `Brain @ ${host}`;
        const reachable = await probeHost(host, wsPort);

        const instance: BrainInstance = {
            name: displayName,
            wsUrl: `ws://${host}:${wsPort}`,
            source: 'manual',
            host,
            wsPort,
            reachable,
            lastSeen: reachable ? Date.now() : 0,
        };

        // Add to state
        discoveryState.instances = discoveryState.instances.filter(i => i.wsUrl !== instance.wsUrl);
        discoveryState.instances.push(instance);

        // Save to persistent hosts file
        const saved = loadSavedHosts();
        if (!saved.some(s => s.host === host && s.port === wsPort)) {
            saved.push({ name: displayName, host, port: wsPort });
            saveSavedHosts(saved);
        }

        return instance;
    });

    // Remove a saved host
    ipcMain.handle('tos:discovery-remove-host', (_event, wsUrl: string) => {
        discoveryState.instances = discoveryState.instances.filter(i => i.wsUrl !== wsUrl);

        // Remove from saved file
        try {
            const parsed = new URL(wsUrl);
            const saved = loadSavedHosts().filter(
                s => !(s.host === parsed.hostname && s.port === (parseInt(parsed.port, 10) || DEFAULT_WS_PORT))
            );
            saveSavedHosts(saved);
        } catch { }

        return { success: true };
    });

    // Probe a specific host
    ipcMain.handle('tos:discovery-probe', async (_event, host: string, port: number) => {
        const reachable = await probeHost(host, port || DEFAULT_WS_PORT);
        return { host, port: port || DEFAULT_WS_PORT, reachable };
    });

    console.log('[Discovery] IPC handlers registered');
}

// ─────────────────────────────────────────────────────────────────────────────
// Exports
// ─────────────────────────────────────────────────────────────────────────────

export { runFullDiscovery, discoverLocalBrain, scanMdns, loadSavedHosts };
