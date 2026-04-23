<script lang="ts">
	import { getTosState } from '$lib/stores/ipc.svelte';
	import { slide, fade } from 'svelte/transition';

	const tosState = $derived(getTosState());
	const activeSector = $derived(tosState.sectors[tosState.active_sector_index]);
	const activeHub = $derived(
		activeSector && activeSector.hubs[activeSector.active_hub_index]
			? activeSector.hubs[activeSector.active_hub_index]
			: null
	);
	const thoughts = $derived(activeHub?.active_thoughts || []);

	function getStatusLabel(status: string) {
		switch (status) {
			case 'Thinking': return '✦ THINKING';
			case 'Decided': return '✦ DECIDED';
			case 'Actioned': return '✦ EXECUTED';
			case 'Failed': return '⚠ FAILED';
			default: return status.toUpperCase();
		}
	}
</script>

<div class="thoughts-container">
	{#each thoughts as thought (thought.id)}
		<div aria-roledescription="chip" class="thought-chip {thought.status.toLowerCase()}" transition:slide>
			<div aria-roledescription="chip" class="chip-inner glass-panel">
				<div class="status-bar">
					<span class="status-marker"></span>
					<span class="status-label">{getStatusLabel(thought.status)}</span>
					<span class="thought-time">{new Date(thought.timestamp).toLocaleTimeString([], {hour: '2-digit', minute:'2-digit', second:'2-digit'})}</span>
				</div>
				<div class="thought-body">
					<div class="thought-title">{thought.title}</div>
					<div class="thought-content">{thought.content}</div>
				</div>
				{#if thought.status === 'Thinking'}
					<div class="scan-line"></div>
				{/if}
			</div>
		</div>
	{/each}
</div>

<style>
	.thoughts-container {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
		margin-bottom: var(--space-md);
	}

	.thought-chip {
		position: relative;
	}

	.chip-inner {
		padding: var(--space-sm) var(--space-md);
		border-left: 3px solid var(--color-primary);
		background: rgba(0, 255, 255, 0.05);
		overflow: hidden;
	}

	.status-bar {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		margin-bottom: var(--space-xs);
	}

	.status-marker {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--color-primary);
		box-shadow: 0 0 8px var(--color-primary);
	}

	.thinking .status-marker {
		animation: blink 1s infinite;
	}

	.decided .status-marker {
		background: var(--color-success);
		box-shadow: 0 0 8px var(--color-success);
	}

	.failed .status-marker {
		background: var(--color-error);
		box-shadow: 0 0 8px var(--color-error);
	}

	.status-label {
		font-family: var(--font-mono);
		font-size: 0.55rem;
		font-weight: 700;
		color: var(--color-primary);
		letter-spacing: 0.1em;
	}

	.thought-time {
		margin-left: auto;
		font-family: var(--font-mono);
		font-size: 0.5rem;
		opacity: 0.4;
	}

	.thought-body {
		display: flex;
		flex-direction: column;
	}

	.thought-title {
		font-family: var(--font-display);
		font-weight: 700;
		font-size: 0.75rem;
		margin-bottom: 2px;
	}

	.thought-content {
		font-size: 0.75rem;
		line-height: 1.4;
		color: var(--color-text-dim);
	}

	.scan-line {
		position: absolute;
		top: 0;
		left: 0;
		width: 100%;
		height: 2px;
		background: linear-gradient(to right, transparent, var(--color-primary), transparent);
		animation: scanDown 2s linear infinite;
		opacity: 0.3;
	}

	@keyframes scanDown {
		0% { top: 0; }
		100% { top: 100%; }
	}

	@keyframes blink {
		0%, 100% { opacity: 1; }
		50% { opacity: 0.3; }
	}

	.decided .chip-inner {
		border-left-color: var(--color-success);
		background: rgba(0, 255, 100, 0.05);
	}

	.failed .chip-inner {
		border-left-color: var(--color-error);
		background: rgba(255, 0, 0, 0.05);
	}
</style>
