/**
 * UI State — Local view state that doesn't come from the Brain.
 * Hierarchy level, sidebar visibility, component slot configuration, etc.
 */

export type ViewMode = 'global' | 'hubs' | 'sectors' | 'app' | 'marketplace' | 'detail' | 'buffer' | 'spatial' | 'logs';
export type SettingsTab = 'global' | 'interface' | 'security' | 'ai' | 'sessions' | 'marketplace' | 'sectors';
export type PromptMode = 'cmd' | 'search' | 'ai';

// --- Reactive state ---
let currentMode = $state<ViewMode>('global');
let sidebarLeftExpanded = $state(false);
let sidebarRightExpanded = $state(false);
let terminalToFront = $state(false);
let settingsOpen = $state(false);
let settingsTab = $state<SettingsTab>('global');
let portalModalOpen = $state(false);
let promptMode = $state<PromptMode>('cmd');
let followingId = $state<string | null>(null);

// --- Getters ---
export function getCurrentMode(): ViewMode { return currentMode; }
export function isSidebarLeftExpanded(): boolean { return sidebarLeftExpanded; }
export function isSidebarRightExpanded(): boolean { return sidebarRightExpanded; }
export function isTerminalToFront(): boolean { return terminalToFront; }
export function isSettingsOpen(): boolean { return settingsOpen; }
export function getSettingsTab(): SettingsTab { return settingsTab; }
export function isPortalModalOpen(): boolean { return portalModalOpen; }
export function getPromptMode(): PromptMode { return promptMode; }
export function getFollowingId(): string | null { return followingId; }

// --- Actions ---
export function setCurrentMode(mode: ViewMode): void {
    currentMode = mode;
}

export function toggleSidebarLeft(): void {
    sidebarLeftExpanded = !sidebarLeftExpanded;
}

export function toggleSidebarRight(): void {
    sidebarRightExpanded = !sidebarRightExpanded;
}

export function toggleTerminalToFront(): void {
    terminalToFront = !terminalToFront;
}

export function openSettings(tab?: SettingsTab): void {
    settingsOpen = true;
    if (tab) settingsTab = tab;
}

export function closeSettings(): void {
    settingsOpen = false;
}

export function setSettingsTab(tab: SettingsTab): void {
    settingsTab = tab;
}

export function openPortalModal(): void {
    portalModalOpen = true;
}

export function closePortalModal(): void {
    portalModalOpen = false;
}

export function setPromptMode(mode: PromptMode): void {
    promptMode = mode;
}

export function toggleFollow(id: string): void {
    followingId = followingId === id ? null : id;
}
