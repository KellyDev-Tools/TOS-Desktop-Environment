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
			<div 
				aria-roledescription="chip" 
				class="warning-chip amberPulse"
				data-color={tosState.settings?.global?.['tos.ai.chip_color'] || 'secondary'}
			>
				<span aria-roledescription="chip" class="chip-icon">⚠</span>
				<span aria-roledescription="chip" class="chip-text">{warning.text.replace('[TRUST] ', '')}</span>
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
		background: var(--color-warning); /* Default */
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

	.warning-chip[data-color="secondary"] { 
		background: var(--color-secondary); 
		box-shadow: 0 4px 12px rgba(102, 204, 204, 0.4);
	}
	.warning-chip[data-color="primary"] { 
		background: var(--color-primary); 
		box-shadow: 0 4px 12px rgba(247, 168, 51, 0.4);
	}
	.warning-chip[data-color="warning"] { 
		background: var(--color-warning); 
		box-shadow: 0 4px 12px rgba(255, 153, 0, 0.4);
	}
</style>
