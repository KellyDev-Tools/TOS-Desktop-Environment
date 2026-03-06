/**
 * TOS IPC Bridge — WebSocket connection to the Brain.
 * Manages connection lifecycle, state synchronization, and command dispatch.
 * Uses Svelte 5 runes for reactive connection state.
 */

import { getDefaultState, type TosState } from './tos-state.svelte';

export type ConnectionState = 'disconnected' | 'connecting' | 'connected';

// --- Reactive Singleton State ---
let connectionState = $state<ConnectionState>('disconnected');
let tosState = $state<TosState>(getDefaultState());
let lastSyncTime = $state<number>(0);
let syncLatency = $state<string>('--');

let ws: WebSocket | null = null;
let syncInterval: ReturnType<typeof setInterval> | null = null;
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;

const WS_URL = 'ws://127.0.0.1:7001';
const SYNC_INTERVAL_MS = 1000;
const RECONNECT_DELAY_MS = 3000;

// --- Public Reactive Getters ---
export function getConnectionState(): ConnectionState {
    return connectionState;
}

export function getTosState(): TosState {
    return tosState;
}

export function getSyncLatency(): string {
    return syncLatency;
}

// --- IPC Command Dispatch ---

/**
 * Send a command to the Brain and return the response.
 * Returns null if not connected.
 */
export function sendCommand(cmd: string): Promise<string | null> {
    if (!ws || ws.readyState !== WebSocket.OPEN) {
        console.warn('[IPC] Cannot send — not connected.');
        return Promise.resolve(null);
    }

    const activeWs = ws;
    if (!activeWs) return Promise.resolve(null);

    return new Promise((resolve) => {
        const handler = (event: MessageEvent) => {
            activeWs.removeEventListener('message', handler);
            resolve(event.data);
        };
        activeWs.addEventListener('message', handler);
        activeWs.send(cmd);

        // Timeout after 5s
        setTimeout(() => {
            activeWs.removeEventListener('message', handler);
            resolve(null);
        }, 5000);
    });
}

// --- State Synchronization ---

async function syncState(): Promise<void> {
    if (!ws || ws.readyState !== WebSocket.OPEN) return;

    const start = performance.now();
    try {
        const response = await sendCommand('get_state:');
        if (!response) return;

        let rawState = response;
        // Strip the Rust diagnostic duration suffix e.g. "JSON (123µs)"
        if (rawState.includes(' (')) {
            rawState = rawState.substring(0, rawState.lastIndexOf(' ('));
        }

        const parsed = JSON.parse(rawState) as TosState;
        tosState = parsed;
        lastSyncTime = Date.now();
        syncLatency = `${(performance.now() - start).toFixed(0)}ms`;
    } catch (e) {
        console.error('[IPC] Sync failure:', e);
    }
}

// --- WebSocket Lifecycle ---

export function connect(): void {
    if (ws && ws.readyState === WebSocket.OPEN) return;
    if (connectionState === 'connecting') return;

    connectionState = 'connecting';
    console.log('[IPC] Connecting to Brain...', WS_URL);

    try {
        ws = new WebSocket(WS_URL);
    } catch {
        connectionState = 'disconnected';
        scheduleReconnect();
        return;
    }

    ws.onopen = () => {
        connectionState = 'connected';
        console.log('[IPC] ✅ Connected to Brain');

        // Start sync loop
        if (syncInterval) clearInterval(syncInterval);
        syncInterval = setInterval(syncState, SYNC_INTERVAL_MS);

        // Immediate first sync
        syncState();
    };

    ws.onclose = () => {
        console.warn('[IPC] Connection closed');
        cleanup();
        scheduleReconnect();
    };

    ws.onerror = (err) => {
        console.error('[IPC] WebSocket error:', err);
        // onclose will fire after onerror, which handles cleanup
    };
}

export function disconnect(): void {
    if (reconnectTimer) {
        clearTimeout(reconnectTimer);
        reconnectTimer = null;
    }
    cleanup();
    ws?.close();
    ws = null;
}

function cleanup(): void {
    connectionState = 'disconnected';
    if (syncInterval) {
        clearInterval(syncInterval);
        syncInterval = null;
    }
}

function scheduleReconnect(): void {
    if (reconnectTimer) return;
    reconnectTimer = setTimeout(() => {
        reconnectTimer = null;
        connect();
    }, RECONNECT_DELAY_MS);
}

// --- Convenience Helpers ---

export async function submitCommand(cmd: string): Promise<string | null> {
    return sendCommand(`prompt_submit:${cmd}`);
}

export async function setMode(mode: string): Promise<void> {
    await sendCommand(`set_mode:${mode}`);
}

export async function switchSector(index: number): Promise<void> {
    await sendCommand(`switch_sector:${index}`);
}

export async function setSetting(key: string, value: string): Promise<void> {
    await sendCommand(`set_setting:${key};${value}`);
}

export async function createPortal(): Promise<string | null> {
    return sendCommand('portal_create');
}

export async function revokePortal(token: string): Promise<void> {
    await sendCommand(`portal_revoke:${token}`);
}

// --- Split Layout Helpers ---

export async function splitCreate(w?: number, h?: number): Promise<void> {
    const payload = (w && h) ? `${w};${h}` : '';
    await sendCommand(`split_create:${payload}`);
}

export async function splitClose(paneId?: string): Promise<void> {
    const payload = paneId || '';
    await sendCommand(`split_close:${payload}`);
}

export async function splitFocus(paneId: string): Promise<void> {
    await sendCommand(`split_focus:${paneId}`);
}

export async function splitFocusDirection(dir: 'Up' | 'Down' | 'Left' | 'Right'): Promise<void> {
    await sendCommand(`split_focus_direction:${dir}`);
}

export async function splitEqualize(): Promise<void> {
    await sendCommand('split_equalize');
}

export async function splitFullscreen(paneId?: string): Promise<void> {
    const payload = paneId || '';
    await sendCommand(`split_fullscreen:${payload}`);
}

export async function splitFullscreenExit(): Promise<void> {
    await sendCommand('split_fullscreen_exit');
}

