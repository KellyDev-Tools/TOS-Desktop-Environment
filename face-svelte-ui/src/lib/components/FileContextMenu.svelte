<script lang="ts">
	import { fade } from 'svelte/transition';
	import { sendCommand } from '$lib/stores/ipc.svelte';
	import { focusTrap } from '$lib/actions/focusTrap';
	
	let { x = 0, y = 0, name = '', path = '', isDir = false, onClose, onStage }: {
		x: number;
		y: number;
		name: string;
		path: string;
		isDir: boolean;
		onClose: () => void;
		onStage: (path: string) => void;
	} = $props();

	async function performAction(action: string) {
		if (action === 'stage') {
			onStage(path);
		} else if (action === 'inspect') {
			await sendCommand(`l4_inspect_path:${path}`);
		} else if (action === 'open_with') {
			await sendCommand(`open_with_dialog:${path}`);
		} else if (action === 'trust') {
			await sendCommand(`elevate_trust_tier:${path}`);
		} else if (action === 'purge') {
			if (confirm(`Are you sure you want to purge (delete) ${name}?`)) {
				await sendCommand(`purge_path:${path}`);
			}
		}
		onClose();
	}
</script>

<svelte:window onclick={onClose} oncontextmenu={(e) => { e.preventDefault(); onClose(); }} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div 
	class="file-context-menu glass-panel" 
	style="left: {x}px; top: {y}px;" 
	transition:fade={{ duration: 150 }} 
	role="button" tabindex="0" onclick={(e) => e.stopPropagation()} 
	oncontextmenu={(e) => e.stopPropagation()}
	use:focusTrap
>
	<div class="menu-header">
		{isDir ? 'DIRECTORY' : 'FILE'} // {name.toUpperCase()}
	</div>
	<div class="menu-body">
		<button class="menu-btn" onclick={() => performAction('stage')}>[STAGE ACTION] Copy to Prompt</button>
		<button class="menu-btn" onclick={() => performAction('inspect')}>[INSPECT PATH] Lvl 4 Diagnostics</button>
		<button class="menu-btn" onclick={() => performAction('open_with')}>[OPEN WITH...] App Picker</button>
		<button class="menu-btn" onclick={() => performAction('trust')}>[TRUST TIER...] Elevate Trust</button>
		<div class="menu-divider"></div>
		<button class="menu-btn critical" onclick={() => performAction('purge')}>[PURGE NODES] Delete Path</button>
	</div>
</div>

<style>
	.file-context-menu {
		position: fixed;
		z-index: 10000;
		width: 250px;
		background: rgba(10, 10, 15, 0.95);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-sm);
		backdrop-filter: blur(var(--glass-blur));
		box-shadow: 0 10px 30px rgba(0, 0, 0, 0.8), 0 0 0 1px rgba(255, 255, 255, 0.05);
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.menu-header {
		background: rgba(255, 255, 255, 0.05);
		color: var(--color-primary);
		font-family: var(--font-mono);
		font-size: 0.65rem;
		font-weight: 700;
		padding: 8px 12px;
		border-bottom: 1px solid rgba(255, 255, 255, 0.1);
		word-break: break-all;
	}

	.menu-body {
		display: flex;
		flex-direction: column;
		padding: 4px;
	}

	.menu-divider {
		height: 1px;
		background: rgba(255, 255, 255, 0.05);
		margin: 4px 0;
	}

	.menu-btn {
		background: transparent;
		border: none;
		color: var(--color-text);
		text-align: left;
		padding: 8px 12px;
		font-family: var(--font-display);
		font-size: 0.75rem;
		cursor: pointer;
		border-radius: 2px;
		transition: all 0.2s;
	}

	.menu-btn:hover {
		background: rgba(255, 255, 255, 0.05);
		color: white;
		padding-left: 16px;
	}

	.menu-btn.critical {
		color: #ff3333;
	}

	.menu-btn.critical:hover {
		background: rgba(255, 51, 51, 0.1);
		color: #ff3333;
	}
</style>
