/**
 * §14.2: Configurable Keyboard Shortcut Store
 *
 * Face-side keybinding resolver. Loads the user's keybinding map from the Brain
 * on connect and resolves KeyboardEvent → action string at runtime.
 *
 * Usage in +page.svelte:
 *   import { resolveKeyboardAction, loadKeybindings } from '$lib/stores/keybindings.svelte';
 *   // On connect: await loadKeybindings();
 *   // In handleGlobalKeydown: const action = resolveKeyboardAction(e);
 */

import { sendCommand } from './ipc.svelte';

// ── Types ──────────────────────────────────────────────────────────────

export interface KeyCombo {
	ctrl: boolean;
	shift: boolean;
	alt: boolean;
	meta: boolean;
	key: string;
}

export interface Keybinding {
	combo: KeyCombo;
	action: string;
	description: string;
}

export interface KeybindingMap {
	bindings: Keybinding[];
}

// ── State ──────────────────────────────────────────────────────────────

let keybindingMap = $state<KeybindingMap>({ bindings: [] });
let loaded = $state(false);

// ── Public API ─────────────────────────────────────────────────────────

/**
 * Load keybindings from the Brain. Call once after connection.
 */
export async function loadKeybindings(): Promise<void> {
	try {
		const response = await sendCommand('keybindings_get');
		if (response && !response.startsWith('ERROR')) {
			const parsed = JSON.parse(response) as KeybindingMap;
			if (parsed.bindings && Array.isArray(parsed.bindings)) {
				keybindingMap = parsed;
				loaded = true;
			}
		}
	} catch (e) {
		console.warn('[KEYBINDINGS] Failed to load from Brain, using empty map:', e);
	}
}

/**
 * Resolve a KeyboardEvent to an action string using the loaded keybinding map.
 * Returns null if no binding matches.
 */
export function resolveKeyboardAction(e: KeyboardEvent): string | null {
	if (!loaded || keybindingMap.bindings.length === 0) return null;

	for (const binding of keybindingMap.bindings) {
		if (
			binding.combo.ctrl === (e.ctrlKey || e.metaKey) &&
			binding.combo.shift === e.shiftKey &&
			binding.combo.alt === e.altKey &&
			matchKey(binding.combo.key, e.key)
		) {
			return binding.action;
		}
	}
	return null;
}

/**
 * Get the current keybinding map (reactive).
 */
export function getKeybindingMap(): KeybindingMap {
	return keybindingMap;
}

/**
 * Get whether keybindings have been loaded.
 */
export function isKeybindingsLoaded(): boolean {
	return loaded;
}

/**
 * Remap a keybinding via Brain IPC.
 */
export async function remapKeybinding(
	comboStr: string,
	action: string,
	description: string
): Promise<string> {
	const response = await sendCommand(`keybindings_set:${comboStr};${action};${description}`);
	// Refresh local map
	await loadKeybindings();
	return response ?? 'ERROR: No response';
}

/**
 * Reset all keybindings to spec defaults via Brain IPC.
 */
export async function resetKeybindings(): Promise<string> {
	const response = await sendCommand('keybindings_reset');
	await loadKeybindings();
	return response ?? 'ERROR: No response';
}

/**
 * Get the display string for an action's current keybinding.
 * Returns undefined if the action has no binding.
 */
export function getKeybindingDisplay(action: string): string | undefined {
	const binding = keybindingMap.bindings.find((b) => b.action === action);
	if (!binding) return undefined;
	return formatCombo(binding.combo);
}

// ── Internal Helpers ───────────────────────────────────────────────────

function matchKey(bindingKey: string, eventKey: string): boolean {
	// Normalize common mismatches
	const normalize = (k: string): string => {
		if (k === 'Space') return ' ';
		return k;
	};
	return normalize(bindingKey) === normalize(eventKey);
}

function formatCombo(combo: KeyCombo): string {
	const parts: string[] = [];
	if (combo.ctrl) parts.push('Ctrl');
	if (combo.alt) parts.push('Alt');
	if (combo.shift) parts.push('Shift');
	if (combo.meta) parts.push('Meta');
	parts.push(combo.key);
	return parts.join('+');
}
