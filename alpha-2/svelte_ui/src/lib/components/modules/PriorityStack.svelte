<script lang="ts">
	import { getTosState } from '$lib/stores/ipc.svelte';

	const state = $derived(getTosState());
	const recentLogs = $derived((state.system_log || []).slice(-3));
</script>

<div class="priority-stack">
	<div class="priority-header">PRIORITY</div>
	{#each recentLogs as log}
		<div class="priority-item" class:high={log.priority >= 3} class:medium={log.priority === 2}>
			<div class="priority-border"></div>
			<span class="priority-text">{log.text.toUpperCase()}</span>
		</div>
	{/each}
	{#if recentLogs.length === 0}
		<div class="priority-empty">NO ALERTS</div>
	{/if}
</div>

<style>
	.priority-stack {
		padding: var(--space-sm);
		animation: fadeIn 0.3s ease;
	}

	.priority-header {
		font-family: var(--font-display);
		font-size: 0.55rem;
		font-weight: 700;
		letter-spacing: 0.15em;
		color: var(--color-text-muted);
		margin-bottom: var(--space-xs);
	}

	.priority-item {
		display: flex;
		align-items: flex-start;
		gap: var(--space-xs);
		padding: 3px 0;
		font-size: 0.6rem;
		font-family: var(--font-mono);
		color: var(--color-text-dim);
		line-height: 1.3;
	}

	.priority-border {
		width: 2px;
		min-height: 0.8rem;
		flex-shrink: 0;
		background: var(--color-warning);
		border-radius: 1px;
	}

	.priority-item.high .priority-border {
		background: var(--color-danger);
	}

	.priority-item.medium .priority-border {
		background: var(--color-primary);
	}

	.priority-text {
		word-break: break-word;
	}

	.priority-empty {
		font-size: 0.6rem;
		opacity: 0.3;
		font-style: italic;
	}
</style>
