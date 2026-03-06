<script lang="ts">
	import { getTosState, splitFocus } from '$lib/stores/ipc.svelte';
	import type { SplitPane, Hub } from '$lib/stores/tos-state.svelte';

	let { pane, activeHub }: { pane: SplitPane; activeHub: Hub | null } = $props();

	const state = $derived(getTosState());
	const isFocused = $derived(activeHub?.focused_pane_id === pane.id);

	// Terminal output — prefer hub-level, fall back to global
	const termOutput = $derived(
		activeHub?.terminal_output?.length
			? activeHub.terminal_output
			: (state.terminal_output || [])
	);

	function priorityColor(p: number): string {
		if (p >= 3) return 'var(--color-warning)';
		if (p === 2) return 'var(--color-primary)';
		if (p === 1) return 'var(--color-success)';
		return 'inherit';
	}

	function handleFocus() {
		if (!isFocused) {
			splitFocus(pane.id);
		}
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div 
	class="split-pane-leaf glass-panel" 
	class:focused={isFocused}
	onclick={handleFocus}
>
	<div class="pane-header">
		<div class="pane-id text-mono">PANE // {pane.id.slice(0, 8)}</div>
		<div class="pane-cwd text-mono">{pane.cwd}</div>
	</div>

	<div class="pane-content">
		{#if pane.content === 'Terminal'}
			<div class="terminal-container">
				{#each termOutput as line}
					<div class="term-line" style="color: {priorityColor(line.priority)}">{line.text || ''}</div>
				{/each}
				{#if isFocused}
					<div class="cursor-blink">_</div>
				{/if}
			</div>
		{:else if typeof pane.content === 'object' && 'Application' in pane.content}
			<div class="app-placeholder">
				<div class="app-icon">⊞</div>
				<div class="app-name">{pane.content.Application.toUpperCase()}</div>
				<div class="app-status">RE-ROUTING RENDERER...</div>
			</div>
		{:else}
			<div class="pane-unknown">UNKNOWN CONTENT TYPE</div>
		{/if}
	</div>
</div>

<style>
	.split-pane-leaf {
		display: flex;
		flex-direction: column;
		height: 100%;
		width: 100%;
		overflow: hidden;
		border: 1px solid var(--color-border);
		transition: border-color var(--transition-fast);
		background: rgba(0, 0, 0, 0.2);
	}

	.split-pane-leaf.focused {
		border-color: var(--color-primary);
		box-shadow: inset 0 0 10px rgba(247, 168, 51, 0.1);
	}

	.pane-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: var(--space-xs) var(--space-sm);
		background: rgba(255, 255, 255, 0.03);
		border-bottom: 1px solid var(--color-border);
		font-size: 0.65rem;
		color: var(--color-text-dim);
		flex-shrink: 0;
	}

	.pane-cwd {
		opacity: 0.6;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		max-width: 60%;
	}

	.pane-content {
		flex: 1;
		min-height: 0;
		position: relative;
	}

	.terminal-container {
		font-family: var(--font-mono);
		font-size: 0.8rem;
		line-height: 1.4;
		height: 100%;
		overflow-y: auto;
		padding: var(--space-sm);
	}

	.term-line {
		white-space: pre-wrap;
		word-break: break-all;
	}

	.cursor-blink {
		display: inline-block;
		width: 8px;
		height: 15px;
		background: var(--color-primary);
		animation: blink 1s step-end infinite;
		vertical-align: middle;
	}

	@keyframes blink {
		from, to { opacity: 1; }
		50% { opacity: 0; }
	}

	.app-placeholder {
		height: 100%;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: var(--space-md);
		opacity: 0.5;
	}

	.app-icon { font-size: 2rem; color: var(--color-primary); }
	.app-name { font-family: var(--font-display); font-weight: 700; letter-spacing: 0.1em; }
	.app-status { font-size: 0.7rem; font-style: italic; }
</style>
