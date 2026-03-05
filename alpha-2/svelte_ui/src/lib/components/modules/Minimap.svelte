<script lang="ts">
	import { getTosState } from '$lib/stores/ipc.svelte';
	import { getCurrentMode, setCurrentMode, type ViewMode } from '$lib/stores/ui.svelte';

	const state = $derived(getTosState());
	const mode = $derived(getCurrentMode());

	// Determine which sectors to show and active marker
	const sectors = $derived(state.sectors || []);
	const activeIndex = $derived(state.active_sector_index);
</script>

<div class="minimap">
	<div class="minimap-header">TACTICAL MAP</div>
	<div class="minimap-grid">
		{#each sectors as sector, i}
			<div class="minimap-sector" class:active={i === activeIndex}>
				<div class="sector-label">S{i}</div>
				<div class="sector-hubs">
					{#each sector.hubs as _, j}
						<div class="hub-dot" class:active={i === activeIndex && j === sector.active_hub_index}></div>
					{/each}
				</div>
				{#if sector.frozen}
					<div class="frozen-indicator">❄</div>
				{/if}
			</div>
		{/each}
	</div>
</div>

<style>
	.minimap {
		padding: var(--space-sm);
		animation: fadeIn 0.3s ease;
	}

	.minimap-header {
		font-family: var(--font-display);
		font-size: 0.55rem;
		font-weight: 700;
		letter-spacing: 0.15em;
		color: var(--color-text-muted);
		margin-bottom: var(--space-xs);
	}

	.minimap-grid {
		display: flex;
		flex-direction: column;
		gap: 3px;
	}

	.minimap-sector {
		display: flex;
		align-items: center;
		gap: var(--space-xs);
		padding: 3px 6px;
		border-radius: var(--radius-sm);
		border: 1px solid transparent;
		transition: all var(--transition-fast);
		cursor: pointer;
		font-size: 0.65rem;
		font-family: var(--font-mono);
		color: var(--color-text-dim);
	}

	.minimap-sector:hover {
		background: rgba(255, 255, 255, 0.04);
		border-color: var(--color-border);
	}

	.minimap-sector.active {
		background: rgba(247, 168, 51, 0.08);
		border-color: var(--color-border-active);
		color: var(--color-primary);
	}

	.sector-label {
		font-weight: 600;
		min-width: 1.2rem;
	}

	.sector-hubs {
		display: flex;
		gap: 3px;
	}

	.hub-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--color-surface-raised);
		border: 1px solid var(--color-border);
		transition: all var(--transition-fast);
	}

	.hub-dot.active {
		background: var(--color-primary);
		border-color: var(--color-primary);
		box-shadow: 0 0 6px rgba(247, 168, 51, 0.4);
	}

	.frozen-indicator {
		font-size: 0.5rem;
		margin-left: auto;
	}
</style>
