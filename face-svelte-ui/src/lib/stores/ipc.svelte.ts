/**
 * TOS IPC Bridge — WebSocket connection to the Brain.
 * Manages connection lifecycle, state synchronization, and command dispatch.
 * Uses Svelte 5 runes for reactive connection state.
 */

import { getDefaultState, normalizeHierarchyLevel, type TosState } from './tos-state.svelte';

export type ConnectionState = 'disconnected' | 'connecting' | 'connected';

// --- Reactive Singleton State ---
let connectionState = $state<ConnectionState>('disconnected');
let tosState = $state<TosState>(getDefaultState());
let lastSyncTime = $state<number>(0);
let syncLatency = $state<string>('--');

let ws: WebSocket | null = null;
let syncInterval: ReturnType<typeof setInterval> | null = null;
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;

let prediction = $state<string>('');
let activeWsUrl = $state<string | null>(null);
const DEFAULT_WS_HOST = '127.0.0.1';
const DEFAULT_WS_URL = `wss://${DEFAULT_WS_HOST}:7001`;

type IpcRawListener = (message: string) => void;
const rawListeners = new Set<IpcRawListener>();

export function addRawListener(listener: IpcRawListener): void {
    rawListeners.add(listener);
}

export function removeRawListener(listener: IpcRawListener): void {
    rawListeners.delete(listener);
}

export function getPrediction(): string {
    return prediction;
}

export function clearPrediction() {
    prediction = '';
}

if (typeof window !== 'undefined') {
    const isElectron = 'tosElectron' in window;
    const queryBrain = new URLSearchParams(window.location.search).get('brain')?.trim() || null;
    let saved = localStorage.getItem('tos_remote_host');
    if (saved && saved.startsWith('ws://')) {
        saved = saved.replace('ws://', 'wss://');
        localStorage.setItem('tos_remote_host', saved);
    }
    const windowHost = window.location.hostname;
    const isLocalhost = windowHost === 'localhost' || windowHost === '127.0.0.1';
    
    if (queryBrain) {
        activeWsUrl = queryBrain.startsWith('ws://') ? queryBrain.replace('ws://', 'wss://') : queryBrain;
        console.log(`[IPC] Initialized with ?brain= URL: ${activeWsUrl}`);
    } else if (isElectron) {
        // Under Electron custom protocols (like tos-app://), windowHost is 'renderer'.
        // Bypass remote migration overrides completely and prioritize saved url, or fallback to wss.
        activeWsUrl = saved || 'wss://127.0.0.1:7001';
        console.log(`[IPC] Electron detected. Initialized activeWsUrl: ${activeWsUrl}`);
    } else {
        if (saved) {
            // Migration: If saved is localhost but we are accessing remotely, override to current host
            try {
                const savedUrl = new URL(saved);
                const savedHost = savedUrl.hostname;
                const savedIsLocal = savedHost === 'localhost' || savedHost === '127.0.0.1';
                
                if (savedIsLocal && !isLocalhost) {
                    console.info(`[IPC] Remote access detected (${windowHost}). Overriding local saved host (${savedHost}).`);
                    activeWsUrl = `wss://${windowHost}:7001`;
                } else {
                    activeWsUrl = saved;
                }
            } catch {
                activeWsUrl = DEFAULT_WS_URL;
            }
        } else {
            activeWsUrl = DEFAULT_WS_URL;
        }
        console.log(`[IPC] Initialized with Brain URL: ${activeWsUrl}`);
    }
}

const SYNC_INTERVAL_MS = 1000;
const RECONNECT_BASE_MS = 1000;
const RECONNECT_MAX_MS = 30_000;
let reconnectAttempts = 0;
let hasConnectedBefore = false;

export function getActiveWsUrl(): string | null {
    return activeWsUrl;
}

export function setActiveWsUrl(url: string) {
    activeWsUrl = url;
    if (typeof window !== 'undefined') {
        localStorage.setItem('tos_remote_host', url);
    }
    // Force reconnect if we're changing URLs
    if (connectionState !== 'disconnected') {
        disconnect();
        scheduleReconnect();
    }
}

// --- Public Reactive Getters ---
export type ConnectionScope = 'local' | 'remote' | 'disconnected';

export function getConnectionScope(): ConnectionScope {
    if (connectionState !== 'connected') return 'disconnected';
    try {
        const host = new URL(activeWsUrl ?? DEFAULT_WS_URL).hostname;
        return host === 'localhost' || host === '127.0.0.1' || host === '::1'
            ? 'local'
            : 'remote';
    } catch {
        return 'local';
    }
}

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

// --- Internal Command Queue ---
let commandQueue: Promise<any> = Promise.resolve();

/**
 * Send a command to the Brain and return the response.
 * Returns null if not connected.
 * 
 * NOTE: This is wrapped in a sequential queue to prevent overlapping 
 * commands from stealing each other's responses (since the Brain
 * currently sends a single line in response to a single-line request).
 */
export function sendCommand(cmd: string): Promise<string | null> {
    if (!ws || ws.readyState !== WebSocket.OPEN) {
        console.warn('[IPC] Cannot send — not connected.');
        return Promise.resolve(null);
    }

    const activeWs = ws;

    const runCommand = (): Promise<string | null> => {
        return new Promise((resolve) => {
            const id = Math.random().toString(36).substring(7);
            const handler = (event: MessageEvent) => {
                console.log(`[IPC] Received: ${event.data.substring(0, 50)}...`);
                if (typeof event.data === 'string' && event.data.startsWith(`res:${id}:`)) {
                    activeWs.removeEventListener('message', handler);
                    const response = event.data.substring(id.length + 5);
                    
                    if (response === 'CONFIRMATION_REQUIRED') {
                        sendCommand('get_state_delta:' + tosState.version).then(res => {
                            if (res && res !== 'NO_CHANGE') {
                                handleStateDelta(res);
                            }
                        });
                    }
                    resolve(response);            
                }
            };
            activeWs.addEventListener('message', handler);
            activeWs.send(`cmd:${id}:${cmd}`);

            // Timeout after 5s
            setTimeout(() => {
                activeWs.removeEventListener('message', handler);
                resolve(null);
            }, 5000);
        });
    };

    const next = commandQueue.then(() => runCommand());
    commandQueue = next.then(() => { }).catch(() => { });

    return next.then(res => {
        if (!res) return null;
        return res;
    }).catch(() => null);
}

// --- State Synchronization & Pushed Heartbeats (1Hz) ---
let heartbeatTimer: ReturnType<typeof setTimeout> | null = null;
const HEARTBEAT_TIMEOUT_MS = 5000;

function handleStateDelta(payload: string) {
    try {
        const parsed = JSON.parse(payload) as TosState;
        parsed.current_level = normalizeHierarchyLevel(parsed.current_level);
        Object.assign(tosState, parsed);
        lastSyncTime = Date.now();
        resetHeartbeat();
    } catch (e) {
        console.error('[IPC] Failed to parse state:', e);
    }
}

function resetHeartbeat() {
    if (heartbeatTimer) clearTimeout(heartbeatTimer);
    heartbeatTimer = setTimeout(() => {
        console.warn('[IPC] Heartbeat lost (5s without state_delta). Disconnecting...');
        disconnect();
    }, HEARTBEAT_TIMEOUT_MS);
}

// --- WebSocket Lifecycle ---

export function connect(customUrl?: string): void {
    if (ws && ws.readyState === WebSocket.OPEN) return;
    if (connectionState === 'connecting') return;

    if (!customUrl && typeof window !== 'undefined' && 'tosElectron' in window) {
        const electron = (window as any).tosElectron;
        connectionState = 'connecting';
        electron.getBrainUrl().then((url: string) => {
            if (url) {
                console.info(`[IPC] Connecting to Electron-provided Brain URL: ${url}`);
                activeWsUrl = url;
                proceedConnect(url);
            } else {
                proceedConnect(activeWsUrl || DEFAULT_WS_URL);
            }
        }).catch((err: any) => {
            console.error('[IPC] Failed to get Brain URL from Electron:', err);
            proceedConnect(activeWsUrl || DEFAULT_WS_URL);
        });
        return;
    }

    if (customUrl) {
        setActiveWsUrl(customUrl);
    }
    proceedConnect(activeWsUrl || DEFAULT_WS_URL);
}

function proceedConnect(targetUrl: string): void {
    if (ws && ws.readyState === WebSocket.OPEN) return;

    if (ws) {
        try {
            ws.close();
        } catch {}
        ws = null;
    }

    connectionState = 'connecting';
    console.log('[IPC] Connecting to Brain...', targetUrl);

    try {
        ws = new WebSocket(targetUrl);
    } catch (err) {
        console.error('[IPC] Failed to create WebSocket:', err);
        connectionState = 'disconnected';
        scheduleReconnect();
        return;
    }

    ws.onopen = () => {
        connectionState = 'connected';
        reconnectAttempts = 0;
        console.log('[IPC] ✅ Connected to Brain');

        if (hasConnectedBefore) {
            // A reconnect, not the first connect. The Brain replays
            // its scrollback to every new connection (Phase 1.5);
            // tell raw listeners to reset first so the replay lands
            // as the whole truth instead of appending to the frozen
            // pre-drop content. Dispatched synchronously here, before
            // any replay message can arrive.
            rawListeners.forEach((listener) => {
                try {
                    listener('tos:reconnected');
                } catch (e) {
                    console.error('[IPC] Raw listener error:', e);
                }
            });
        }
        hasConnectedBefore = true;

        // Immediate first sync to get full state, which also starts the heartbeat
        sendCommand('get_state:').then(res => {
            if (res && typeof res === 'string') {
                handleStateDelta(res);
            }
        });

        // Setup global passive listener for pushed state_delta
        ws!.onmessage = (event) => {
            if (typeof event.data === 'string') {
                // Dispatch to raw listeners first
                rawListeners.forEach(listener => {
                    try {
                        listener(event.data);
                    } catch (e) {
                        console.error('[IPC] Raw listener error:', e);
                    }
                });

                if (event.data === 'heartbeat') {
                    lastSyncTime = Date.now();
                    resetHeartbeat();
                } else if (event.data.startsWith('state_delta:')) {
                    handleStateDelta(event.data.substring(12));
                } else if (event.data.startsWith('ai_prediction_received:')) {
                    prediction = event.data.substring(23);
                }
            }
        };
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
    if (heartbeatTimer) {
        clearTimeout(heartbeatTimer);
        heartbeatTimer = null;
    }
}

function scheduleReconnect(): void {
    if (reconnectTimer) return;
    // Exponential backoff with jitter: 1s, 2s, 4s, … capped at 30s.
    // A fixed 2s retry hammers a down Brain and drains a laptop on
    // a dropped network; the cap keeps recovery prompt after sleep.
    const backoff = Math.min(RECONNECT_BASE_MS * 2 ** reconnectAttempts, RECONNECT_MAX_MS);
    const delay = backoff * (0.8 + Math.random() * 0.4);
    reconnectAttempts += 1;
    reconnectTimer = setTimeout(() => {
        reconnectTimer = null;
        connect();
    }, delay);
}

// Wake triggers: sleep/wake and network returns often leave a
// half-open socket and a pending backoff timer. On 'online' or on
// the face becoming visible again, retry immediately instead of
// waiting out the backoff.
if (typeof window !== 'undefined') {
    const wakeReconnect = () => {
        if (connectionState !== 'disconnected') return;
        if (reconnectTimer) {
            clearTimeout(reconnectTimer);
            reconnectTimer = null;
        }
        reconnectAttempts = 0;
        connect();
    };
    window.addEventListener('online', wakeReconnect);
    document.addEventListener('visibilitychange', () => {
        if (document.visibilityState === 'visible') wakeReconnect();
    });
}

// --- Convenience Helpers ---

export async function submitCommand(cmd: string): Promise<string | null> {
    return sendCommand(`prompt_submit:${cmd}`);
}

export const systemReset = () => sendCommand('system_reset');

export const processInspect = (pid: string) => sendCommand(`process_inspect:${pid}`);
export const getBuffer = (pid: string) => sendCommand(`get_buffer:${pid}`);

export async function predictCommand(partial: string): Promise<void> {
    await sendCommand(`ai_predict_command:${partial}`);
}

export async function setMode(mode: string): Promise<void> {
    await sendCommand(`set_mode:${mode}`);
}

export async function switchSector(index: number): Promise<void> {
    await sendCommand(`set_active_sector:${index}`);
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

export async function splitResize(paneId: string, weight: number): Promise<void> {
    await sendCommand(`split_resize:${paneId};${weight}`);
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

export async function splitSwap(paneA?: string, paneB?: string): Promise<void> {
    const payload = (paneA && paneB) ? `${paneA};${paneB}` : '';
    await sendCommand(`split_swap:${payload}`);
}

export async function splitDetachContext(): Promise<void> {
    await sendCommand('split_detach:context');
}

export async function splitDetachFresh(): Promise<void> {
    await sendCommand('split_detach:fresh');
}

// --- Onboarding Helpers ---

export async function onboardingSkipCinematic(): Promise<void> {
    await sendCommand('onboarding_skip_cinematic');
}

export async function onboardingSkipTour(): Promise<void> {
    await sendCommand('onboarding_skip_tour');
}

export async function onboardingAdvanceStep(step?: number): Promise<void> {
    const payload = step !== undefined ? `${step}` : '';
    await sendCommand(`onboarding_advance_step:${payload}`);
}

export async function onboardingHintDismiss(id: string): Promise<void> {
    await sendCommand(`onboarding_hint_dismiss:${id}`);
}

export async function onboardingHintsSuppress(): Promise<void> {
    await sendCommand('onboarding_hints_suppress');
}

export async function onboardingReplayTour(): Promise<void> {
    await sendCommand('onboarding_replay_tour');
}

export async function onboardingResetHints(): Promise<void> {
    await sendCommand('onboarding_reset_hints');
}

// --- Bezel Helpers ---

export async function bezelExpand(): Promise<void> {
    await sendCommand('bezel_expand');
}

export async function bezelCollapse(): Promise<void> {
    await sendCommand('bezel_collapse');
}

export async function bezelOutputAction(action: string): Promise<void> {
    await sendCommand(`bezel_output_action:${action}`);
}

export async function bezelPanePromote(): Promise<void> {
    await sendCommand('bezel_pane_promote');
}

export async function bezelSwipe(dir: 'Left' | 'Right'): Promise<void> {
    await sendCommand(`bezel_swipe:${dir}`);
}

// --- Marketplace Helpers ---

export async function marketplaceGetHome(): Promise<any> {
    const raw = await sendCommand('marketplace_home:');
    return raw ? JSON.parse(raw) : null;
}

export async function marketplaceGetCategory(id: string): Promise<any> {
    const raw = await sendCommand(`marketplace_category:${id}`);
    return raw ? JSON.parse(raw) : null;
}

export async function marketplaceGetDetail(id: string): Promise<any> {
    const raw = await sendCommand(`marketplace_detail:${id}`);
    return raw ? JSON.parse(raw) : null;
}

export async function marketplaceInstall(id: string): Promise<string | null> {
    return sendCommand(`marketplace_install:${id}`);
}

export async function marketplaceGetStatus(id: string): Promise<any> {
    const raw = await sendCommand(`marketplace_status:${id}`);
    return raw ? JSON.parse(raw) : null;
}

export async function marketplaceSearchAi(query: string): Promise<any> {
    const raw = await sendCommand(`marketplace_search_ai:${query}`);
    return raw ? JSON.parse(raw) : null;
}

export async function marketplaceInstallCancel(id: string): Promise<void> {
    await sendCommand(`marketplace_install_cancel:${id}`);
}

// --- Trust Confirmation Helpers ---

export async function confirmationAccept(id: string): Promise<void> {
    await sendCommand(`confirmation_accept:${id}`);
}

export async function confirmationReject(id: string): Promise<void> {
    await sendCommand(`confirmation_reject:${id}`);
}

// --- Directory Helpers ---

export async function dirPickFile(index: number): Promise<void> {
    await sendCommand(`dir_pick_file:${index}`);
}

export async function dirPickDir(index: number): Promise<void> {
    await sendCommand(`dir_pick_dir:${index}`);
}

export async function dirNavigate(path: string): Promise<void> {
    await sendCommand(`dir_navigate:${path}`);
}

// --- Terminal Typing Helpers (Phase 1.6) ---
//
// The accelerator chips and the prompt surface's input act on the
// terminal by *typing into it* — the terminal stays a completely
// normal terminal, and a chip tap is indistinguishable from the
// user typing the same text.

/** Type text into the terminal exactly as if the user typed it. */
export async function sendTerminalText(text: string): Promise<void> {
    const hex = Array.from(new TextEncoder().encode(text))
        .map((b) => b.toString(16).padStart(2, '0'))
        .join('');
    await sendCommand(`terminal_input_hex:${hex}`);
}

/** Type a command into the terminal and run it (text + Enter). */
export async function sendTerminalCommand(command: string): Promise<void> {
    await sendTerminalText(command + '\n');
}

/** Single-quote a string for safe typing into a POSIX shell. */
export function shellQuote(text: string): string {
    return `'${text.replace(/'/g, `'\\''`)}'`;
}

/**
 * Generic IPC sender used by some legacy or high-level components.
 * Formats as "method:payload" and dispatches via sendCommand.
 */
export async function sendIpc(method: string, payload: any): Promise<string | null> {
    const data = typeof payload === 'string' ? payload : JSON.stringify(payload);
    return sendCommand(`${method}:${data}`);
}
