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

let prediction = $state<string>('');
let activeWsUrl = $state<string | null>(null);
const DEFAULT_WS_HOST = typeof window !== 'undefined' ? window.location.hostname : '127.0.0.1';
const DEFAULT_WS_URL = `ws://${DEFAULT_WS_HOST}:7001`;

export function getPrediction(): string {
    return prediction;
}

export function clearPrediction() {
    prediction = '';
}

if (typeof window !== 'undefined') {
    const saved = localStorage.getItem('tos_remote_host');
    const windowHost = window.location.hostname;
    const isLocalhost = windowHost === 'localhost' || windowHost === '127.0.0.1';
    
    if (saved) {
        // Migration: If saved is localhost but we are accessing remotely, override to current host
        try {
            const savedUrl = new URL(saved);
            const savedHost = savedUrl.hostname;
            const savedIsLocal = savedHost === 'localhost' || savedHost === '127.0.0.1';
            
            if (savedIsLocal && !isLocalhost) {
                console.info(`[IPC] Remote access detected (${windowHost}). Overriding local saved host (${savedHost}).`);
                activeWsUrl = `ws://${windowHost}:7001`;
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

const SYNC_INTERVAL_MS = 1000;
const RECONNECT_DELAY_MS = 2000;

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
    if (customUrl) {
        setActiveWsUrl(customUrl);
    }

    if (ws && ws.readyState === WebSocket.OPEN) return;
    if (connectionState === 'connecting') return;

    const targetUrl = activeWsUrl || DEFAULT_WS_URL;

    connectionState = 'connecting';
    console.log('[IPC] Connecting to Brain...', targetUrl);

    try {
        ws = new WebSocket(targetUrl);
    } catch {
        connectionState = 'disconnected';
        scheduleReconnect();
        return;
    }

    ws.onopen = () => {
        connectionState = 'connected';
        console.log('[IPC] ✅ Connected to Brain');

        // Immediate first sync to get full state, which also starts the heartbeat
        sendCommand('get_state:').then(res => {
            if (res && typeof res === 'string') {
                handleStateDelta(res);
            }
        });

        // Setup global passive listener for pushed state_delta
        ws!.onmessage = (event) => {
            if (typeof event.data === 'string') {
                if (event.data.startsWith('state_delta:')) {
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
    reconnectTimer = setTimeout(() => {
        reconnectTimer = null;
        connect();
    }, RECONNECT_DELAY_MS);
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

/**
 * Generic IPC sender used by some legacy or high-level components.
 * Formats as "method:payload" and dispatches via sendCommand.
 */
export async function sendIpc(method: string, payload: any): Promise<string | null> {
    const data = typeof payload === 'string' ? payload : JSON.stringify(payload);
    return sendCommand(`${method}:${data}`);
}
