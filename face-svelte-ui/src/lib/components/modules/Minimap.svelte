<script lang="ts">
	import { fade, slide, fly } from 'svelte/transition';
	import { getTosState, sendCommand } from '$lib/stores/ipc.svelte';
	import { getCurrentMode, setCurrentMode, type ViewMode } from '$lib/stores/ui.svelte';

	const tosState = $derived(getTosState());
	const mode = $derived(getCurrentMode());

	const activeSector = $derived(tosState.sectors[tosState.active_sector_index]);
	const activeHub = $derived(activeSector?.hubs[activeSector?.active_hub_index]);

	let isProjected = $state(false);

	function toggleProjected() {
		isProjected = !isProjected;
	}

	function handleSectorClick(idx: number) {
		sendCommand(`set_active_sector:${idx}`);
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div 
	class="minimap-container" 
	class:projected={isProjected} 
	onclick={toggleProjected}
	onkeydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggleProjected()}
	role="button"
	tabindex="0"
	aria-label="Tactical Minimap"
>
	<div class="minimap-header">
		<span>TACTICAL_MAP</span>
		<span class="depth-indicator">LVL_{mode === 'global' ? 1 : (mode === 'hubs' ? 2 : (mode === 'sectors' ? 3 : 4))}</span>
	</div>

	<div class="minimap-content">
		{#if mode === 'global'}
			<div class="level-1-view" in:fade>
				{#each tosState.sectors as sector, i}
					<button 
						class="sector-tile" 
						class:active={i === tosState.active_sector_index}
						onclick={(e) => { e.stopPropagation(); handleSectorClick(i); }}
					>
						<div class="tile-header">S{i}</div>
						<div class="tile-body">
							{#each sector.hubs as hub, j}
								<div class="hub-marker" class:active={j === sector.active_hub_index}></div>
							{/each}
						</div>
					</button>
				{/each}
			</div>
		{:else if mode === 'hubs' || mode === 'sectors'}
			<div class="level-2-view" in:fade>
				<div class="sector-focus-label">{activeSector?.name.toUpperCase()}</div>
				<div class="hub-hierarchy">
					{#each activeSector.hubs as hub, i}
						<div class="hub-node" class:active={i === activeSector.active_hub_index}>
							<div class="hub-icon">⬢</div>
							<div class="hub-info">
								<div class="hub-name">HUB_{i}</div>
								<div class="hub-apps">{hub.activity_listing.processes.length} APPS</div>
							</div>
						</div>
					{/each}
				</div>
			</div>
		{:else if mode === 'detail' || mode === 'buffer'}
			<div class="level-4-view" in:fade>
				<div class="surface-focus-label">INSPECTION_TARGET</div>
				<div class="surface-map">
					<div class="surface-ring central"></div>
					<div class="surface-ring orbit-1"></div>
					<div class="target-marker"></div>
				</div>
			</div>
		{/if}
	</div>

	{#if isProjected}
		<div class="projection-overlay glass-panel" transition:fade={{ duration: 200 }}>
			<div class="projection-header">SYSTEM_TOPOLOGY_EXPANDED</div>
			<div class="projection-grid">
				{#each tosState.sectors as sector, i}
					<div class="projection-sector" class:active={i === tosState.active_sector_index}>
						<div class="proj-sector-name">{sector.name}</div>
						<div class="proj-hub-list">
							{#each sector.hubs as hub}
								<div class="proj-hub-item">
									<span class="dot"></span>
									{hub.current_directory}
								</div>
							{/each}
						</div>
					</div>
				{/each}
			</div>
			<div class="close-hint">CLICK ANYWHERE TO COLLAPSE</div>
		</div>
	{/if}
</div>

<style>
	.minimap-container {
		padding: var(--space-sm);
		cursor: pointer;
		position: relative;
		transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
		border-radius: var(--radius-md);
		background: rgba(0, 0, 0, 0.2);
	}

	.minimap-container:hover {
		background: rgba(255, 255, 255, 0.05);
		box-shadow: 0 0 15px rgba(247, 168, 51, 0.1);
	}

	.minimap-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		font-family: var(--font-display);
		font-size: 0.55rem;
		font-weight: 700;
		letter-spacing: 0.15em;
		color: var(--color-text-dim);
		margin-bottom: var(--space-sm);
		border-bottom: 1px solid rgba(255, 255, 255, 0.1);
		padding-bottom: 4px;
	}

	.depth-indicator {
		color: var(--color-primary);
		opacity: 0.8;
	}

	.minimap-content {
		min-height: 80px;
	}

	/* Level 1 */
	.level-1-view {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 6px;
	}

	.sector-tile {
		background: rgba(255, 255, 255, 0.03);
		border: 1px solid rgba(255, 255, 255, 0.1);
		border-radius: 2px;
		padding: 4px;
		cursor: pointer;
		transition: all 0.2s;
	}

	.sector-tile.active {
		border-color: var(--color-primary);
		background: rgba(247, 168, 51, 0.1);
	}

	.tile-header {
		font-size: 0.5rem;
		font-family: var(--font-mono);
		opacity: 0.5;
		margin-bottom: 2px;
	}

	.tile-body {
		display: flex;
		flex-wrap: wrap;
		gap: 2px;
	}

	.hub-marker {
		width: 4px;
		height: 4px;
		background: rgba(255, 255, 255, 0.2);
		border-radius: 50%;
	}

	.hub-marker.active {
		background: var(--color-primary);
		box-shadow: 0 0 4px var(--color-primary);
	}

	/* Level 2 */
	.sector-focus-label {
		font-size: 0.6rem;
		color: var(--color-primary);
		margin-bottom: 6px;
		border-left: 2px solid var(--color-primary);
		padding-left: 5px;
	}

	.hub-hierarchy {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.hub-node {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 4px;
		background: rgba(255, 255, 255, 0.02);
		border-radius: 2px;
		opacity: 0.6;
	}

	.hub-node.active {
		opacity: 1;
		background: rgba(247, 168, 51, 0.05);
		border-right: 2px solid var(--color-primary);
	}

	.hub-icon {
		font-size: 0.8rem;
		color: var(--color-primary);
	}

	.hub-name {
		font-size: 0.55rem;
		font-weight: 700;
	}

	.hub-apps {
		font-size: 0.45rem;
		opacity: 0.5;
	}

	/* Level 4 */
	.surface-focus-label {
		font-size: 0.6rem;
		color: var(--color-warning);
		text-align: center;
		margin-bottom: 10px;
	}

	.surface-map {
		position: relative;
		width: 60px;
		height: 60px;
		margin: 0 auto;
	}

	.surface-ring {
		position: absolute;
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);
		border: 1px dashed rgba(255, 255, 255, 0.1);
		border-radius: 50%;
	}

	.surface-ring.central { width: 20px; height: 20px; }
	.surface-ring.orbit-1 { width: 50px; height: 50px; animation: spin 10s linear infinite; }

	.target-marker {
		position: absolute;
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);
		width: 8px;
		height: 8px;
		background: var(--color-warning);
		clip-path: polygon(50% 0%, 0% 100%, 100% 100%);
		box-shadow: 0 0 10px var(--color-warning);
	}

	@keyframes spin {
		from { transform: translate(-50%, -50%) rotate(0deg); }
		to { transform: translate(-50%, -50%) rotate(360deg); }
	}

	/* Projection Overlay */
	.projection-overlay {
		position: fixed;
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);
		width: 60vw;
		max-height: 70vh;
		background: rgba(10, 10, 20, 0.95);
		border: 1px solid var(--color-primary);
		z-index: 2000;
		padding: var(--space-xl);
		box-shadow: 0 0 100px rgba(0, 0, 0, 0.8);
		pointer-events: all;
	}

	.projection-header {
		font-family: var(--font-display);
		font-size: 1.2rem;
		color: var(--color-primary);
		border-bottom: 1px solid var(--color-primary);
		padding-bottom: 10px;
		margin-bottom: 20px;
		letter-spacing: 0.2em;
	}

	.projection-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
		gap: 20px;
		overflow-y: auto;
		max-height: 50vh;
	}

	.projection-sector {
		background: rgba(255, 255, 255, 0.03);
		border: 1px solid rgba(255, 255, 255, 0.1);
		padding: 15px;
		border-radius: 4px;
	}

	.projection-sector.active {
		border-color: var(--color-primary);
		box-shadow: inset 0 0 20px rgba(247, 168, 51, 0.1);
	}

	.proj-sector-name {
		font-weight: 700;
		color: var(--color-primary);
		margin-bottom: 10px;
		font-size: 0.9rem;
	}

	.proj-hub-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.proj-hub-item {
		font-size: 0.7rem;
		opacity: 0.8;
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.proj-hub-item .dot {
		width: 6px;
		height: 6px;
		background: var(--color-primary);
		border-radius: 50%;
	}

	.close-hint {
		margin-top: 30px;
		text-align: center;
		font-size: 0.7rem;
		opacity: 0.4;
		letter-spacing: 0.1em;
	}
</style>
