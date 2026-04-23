<script lang="ts">
	import { getTosState } from '$lib/stores/ipc.svelte';

	const tosState = $derived(getTosState());
	
	// Filter for trust warnings in the system log
	const trustWarnings = $derived(
		(tosState.system_log || [])
			.filter(line => line.text.startsWith('[TRUST]'))
			.slice(-3) // Show last 3 warnings
	);
</script>

{#if trustWarnings.length > 0}
	<div class="warning-container">
		{#each trustWarnings as warning}
			<div class="warning-chip amberPulse">
				<span class="chip-icon">⚠</span>
				<span class="chip-text">{warning.text.replace('[TRUST] ', '')}</span>
			</div>
		{/each}
	</div>
{/if}

<style>
	.warning-container {
		position: absolute;
		top: var(--space-md);
		right: var(--space-md);
		display: flex;
		flex-direction: column;
		gap: var(--space-xs);
		z-index: 100;
		pointer-events: none;
	}

	.warning-chip {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-xs) var(--space-md);
		background: var(--color-warning);
		color: black;
		font-weight: 800;
		font-family: var(--font-mono);
		font-size: 0.75rem;
		border-radius: var(--radius-sm);
		box-shadow: 0 4px 12px rgba(255, 153, 0, 0.4);
		border: 1px solid rgba(0, 0, 0, 0.2);
		text-transform: uppercase;
		pointer-events: auto;
	}

	.chip-icon {
		font-size: 1rem;
	}

	@keyframes amberPulse {
		0% { box-shadow: 0 0 0 0 rgba(255, 153, 0, 0.7); }
		70% { box-shadow: 0 0 0 10px rgba(255, 153, 0, 0); }
		100% { box-shadow: 0 0 0 0 rgba(255, 153, 0, 0); }
	}

	.amberPulse {
		animation: amberPulse 2s infinite;
	}
</style>
