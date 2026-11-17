<script lang="ts">
	// Phase 1.6 — the prompt surface's input. It acts independently:
	// it is not docked to the terminal view and does not depend on
	// it. Submitting types the command into the terminal (the same
	// bytes as typing it there), so the terminal stays a normal
	// terminal. SEARCH / AI modes are shown but inert — their
	// information providers are frozen by the scope split.
	import { sendTerminalCommand } from '$lib/stores/ipc.svelte';

	let value = $state('');
	let inputEl: HTMLInputElement | null = $state(null);

	$effect(() => {
		inputEl?.focus();
	});

	function onKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			const cmd = value.trim();
			if (cmd) {
				sendTerminalCommand(cmd);
				value = '';
			}
		} else if (e.key === 'Escape') {
			value = '';
			inputEl?.blur();
		}
	}
</script>

<div class="prompt-input-bar glass-panel" data-testid="prompt-input-bar">
	<div class="prompt-modes" role="tablist" aria-label="Prompt mode">
		<button class="mode-pill active" role="tab" aria-selected="true">CMD</button>
		<button
			class="mode-pill"
			role="tab"
			aria-selected="false"
			disabled
			title="Search provider is frozen (scope split)">SEARCH</button
		>
		<button
			class="mode-pill"
			role="tab"
			aria-selected="false"
			disabled
			title="AI provider is frozen (scope split)">AI</button
		>
	</div>
	<input
		bind:this={inputEl}
		bind:value
		class="prompt-input"
		data-testid="prompt-input"
		type="text"
		placeholder="Type a command — Enter runs it in the terminal"
		autocomplete="off"
		spellcheck="false"
		onkeydown={onKeydown}
	/>
</div>

<style>
	.prompt-input-bar {
		display: flex;
		align-items: center;
		gap: 12px;
		margin: 12px;
		padding: 10px 14px;
		border-radius: 14px;
	}
	.prompt-modes {
		display: flex;
		gap: 6px;
		flex: none;
	}
	.mode-pill {
		padding: 6px 12px;
		border-radius: 999px;
		border: 1px solid rgba(0, 229, 255, 0.35);
		background: transparent;
		color: var(--color-text, #e0f7fa);
		font-size: 11px;
		letter-spacing: 0.1em;
		cursor: pointer;
	}
	.mode-pill.active {
		background: var(--color-primary, #00e5ff);
		color: #06222b;
		font-weight: 700;
	}
	.mode-pill:disabled {
		opacity: 0.35;
		cursor: not-allowed;
	}
	.prompt-input {
		flex: 1;
		min-width: 0;
		background: rgba(0, 0, 0, 0.25);
		border: 1px solid rgba(0, 229, 255, 0.25);
		border-radius: 10px;
		padding: 10px 12px;
		color: var(--color-text, #e0f7fa);
		font-size: 15px;
		outline: none;
	}
	.prompt-input:focus {
		border-color: var(--color-primary, #00e5ff);
	}
</style>
