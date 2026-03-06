<script lang="ts">
	import { getTosState } from '$lib/stores/ipc.svelte';
	import { isTerminalToFront, getCurrentMode } from '$lib/stores/ui.svelte';


	const state = $derived(getTosState());
	const toFront = $derived(isTerminalToFront());
	const mode = $derived(getCurrentMode());
	const logs = $derived(state.system_log || []);

	// Visible as background layer on Level 1 (Global Overview),
	// or when explicitly brought to front via the 👁 toggle.
	const visible = $derived(mode === 'global' || toFront);

	function formatTimestamp(ts: string): string {
		if (!ts) return '--:--:--';
		const parts = ts.split('T');
		if (parts.length < 2) return ts;
		return parts[1]?.split('.')[0] || ts;
	}

	let container: HTMLDivElement;

	$effect(() => {
		// Depend on logs length or just access container and logs
		if (logs.length || !logs.length) {
			if (container) {
				container.scrollTop = container.scrollHeight;
			}
		}
	});
</script>

{#if visible}
	<div bind:this={container} class="system-output" class:to-front={toFront} class:bg-layer={mode === 'global' && !toFront}>
		{#each logs as log}
			<div class="log-entry" class:p1={log.priority === 1} class:p2={log.priority === 2} class:p3={log.priority >= 3}>
				<span class="log-timestamp">[{formatTimestamp(log.timestamp)}]</span>
				<span class="log-prefix">SYS_BRAIN //</span>
				<span class="log-text">{log.text.toUpperCase()}</span>
			</div>
		{/each}
	</div>
{/if}

<style>
	.system-output {
		position: absolute;
		inset: 0;
		padding: var(--space-md);
		font-family: var(--font-mono);
		font-size: 0.75rem;
		line-height: 1.6;
		opacity: 0;
		pointer-events: none;
		z-index: 1;
		display: flex;
		flex-direction: column;
		justify-content: flex-end;
		overflow-y: auto;
		transition: opacity var(--transition-normal);
		-ms-overflow-style: none;  /* IE and Edge */
		scrollbar-width: none;  /* Firefox */
	}
	.system-output::-webkit-scrollbar {
		display: none;
	}

	/* Level 1 background layer — faded behind sector tiles */
	.system-output.bg-layer {
		opacity: 0.35;
		pointer-events: none;
	}

	/* Explicitly brought to front (any level) */
	.system-output.to-front {
		opacity: 1;
		pointer-events: auto;
		z-index: 100;
		background: rgba(10, 10, 20, 0.9);
		backdrop-filter: blur(10px);
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

