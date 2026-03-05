<script lang="ts">
	import { getTosState } from '$lib/stores/ipc.svelte';
	import { isTerminalToFront } from '$lib/stores/ui.svelte';

	const state = $derived(getTosState());
	const toFront = $derived(isTerminalToFront());
	const logs = $derived(state.system_log || []);

	function formatTimestamp(ts: string): string {
		if (!ts) return '--:--:--';
		const parts = ts.split('T');
		if (parts.length < 2) return ts;
		return parts[1]?.split('.')[0] || ts;
	}
</script>

<div class="system-output" class:to-front={toFront}>
	{#each logs as log}
		<div class="log-entry" class:p1={log.priority === 1} class:p2={log.priority === 2} class:p3={log.priority >= 3}>
			<span class="log-timestamp">[{formatTimestamp(log.timestamp)}]</span>
			<span class="log-prefix">SYS_BRAIN //</span>
			<span class="log-text">{log.text.toUpperCase()}</span>
		</div>
	{/each}
</div>

<style>
	.system-output {
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		padding: var(--space-sm) var(--space-md);
		font-family: var(--font-mono);
		font-size: 0.75rem;
		line-height: 1.6;
		opacity: 0.5;
		pointer-events: none;
		z-index: 1;
		max-height: 30%;
		overflow-y: auto;
		transition: opacity var(--transition-normal), z-index 0ms;
	}

	.system-output.to-front {
		opacity: 1;
		pointer-events: auto;
		z-index: 10;
		background: var(--color-surface-overlay);
		max-height: 100%;
	}

	.log-entry {
		display: flex;
		gap: var(--space-sm);
		animation: slideDown 0.2s ease;
	}

	.log-timestamp {
		color: var(--color-text-muted);
		flex-shrink: 0;
	}

	.log-prefix {
		color: var(--color-accent);
		font-weight: 600;
		flex-shrink: 0;
	}

	.log-text {
		color: var(--color-text-dim);
	}

	.log-entry.p2 .log-text {
		color: var(--color-primary);
	}

	.log-entry.p3 .log-text {
		color: var(--color-warning);
	}
</style>
