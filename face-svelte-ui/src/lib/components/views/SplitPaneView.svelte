<script lang="ts">
	import { getTosState, splitFocus, submitCommand } from '$lib/stores/ipc.svelte';
	import type { SplitPane, Hub } from '$lib/stores/tos-state.svelte';
	import EditorPane from '../editor/EditorPane.svelte';

	let { pane, activeHub }: { pane: SplitPane; activeHub: Hub | null } = $props();

	const tosState = $derived(getTosState());
	const isFocused = $derived(activeHub?.focused_pane_id === pane.id);

	// Terminal output — prefer hub-level, fall back to global
	const termOutput = $derived(
		activeHub?.terminal_output?.length
			? activeHub.terminal_output
			: (tosState.terminal_output || [])
	);

	function priorityColor(p: number): string {
		if (p >= 3) return 'var(--color-warning)';
		if (p === 2) return 'var(--color-primary)';
		if (p === 1) return 'var(--color-success)';
		return 'inherit';
	}

	function renderTermLine(text: string): string {
		if (!text) return '';
		
		// Escape HTML first
		let escaped = text
			.replace(/&/g, "&amp;")
			.replace(/</g, "&lt;")
			.replace(/>/g, "&gt;");
			
		// Regex to find things pointing to files like /path/to/file.rs:14 or src/main.js:142:1
		// Pattern: ([a-zA-Z0-9_/\-.]+\.[a-zA-Z0-9]+):(\d+)
		const pathRegex = /([a-zA-Z0-9_/\-.]+\.[a-zA-Z0-9]+):(\d+)/g;
		
		return escaped.replace(pathRegex, (match, path, line) => {
			return `<span class="term-path-link" onclick="window.tos_submitCommand('!ipc editor_open:${path};${line}')" title="Hold explicitly or Click to open in Editor">${match}</span>`;
		});
	}

	// We expose submitCommand globally for innerHTML onclick handlers
	$effect(() => {
		if (typeof window !== 'undefined') {
			(window as any).tos_submitCommand = submitCommand;
		}
	});

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
					<div class="term-line" style="color: {priorityColor(line.priority)}">
						{@html renderTermLine(line.text)}
					</div>
				{/each}
				{#if isFocused}
					<div class="cursor-blink">_</div>
				{/if}
			</div>
		{:else if typeof pane.content === 'object' && 'Editor' in pane.content}
			<EditorPane editorState={pane.content.Editor} activeHub={activeHub} paneId={pane.id} />
		{:else if typeof pane.content === 'object' && 'Application' in pane.content}
			<div class="app-placeholder">
				<div class="app-icon">⊞</div>
				<div class="app-name">{typeof pane.content === 'object' && 'Application' in pane.content ? (pane.content as any).Application : ''.toUpperCase()}</div>
				<div class="app-status">SESSION RESTORED — PENDING LAUNCH</div>
				<button 
					class="lcars-btn-sm primary" 
					style="margin-top: var(--space-md);"
					onclick={(e) => { e.stopPropagation(); submitCommand(typeof pane.content === 'object' && 'Application' in pane.content ? (pane.content as any).Application : 'focus'); }}
				>
					▶ RELAUNCH APP
				</button>
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

	:global(.term-path-link) {
		color: var(--color-warning); /* Highlight in amber */
		cursor: pointer;
		text-decoration: underline;
		text-decoration-color: rgba(255, 193, 7, 0.4);
	}

	:global(.term-path-link:hover) {
		background-color: rgba(255, 193, 7, 0.1);
		text-decoration-color: var(--color-warning);
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
	
	.lcars-btn-sm {
		font-family: var(--font-display);
		font-size: 0.62rem;
		font-weight: 700;
		letter-spacing: 0.08em;
		padding: 0.4rem 0.8rem;
		border: 1px solid rgba(247, 168, 51, 0.35);
		background: rgba(247, 168, 51, 0.12);
		color: var(--color-primary);
		border-radius: var(--radius-sm);
		cursor: pointer;
		transition: all var(--transition-fast);
		white-space: nowrap;
	}
	.lcars-btn-sm:hover {
		background: rgba(247, 168, 51, 0.2);
		color: #fff;
	}
</style>
