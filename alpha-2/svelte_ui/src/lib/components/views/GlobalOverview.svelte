<script lang="ts">
	import { getTosState } from '$lib/stores/ipc.svelte';
	import { setCurrentMode, type ViewMode } from '$lib/stores/ui.svelte';
	import * as ipc from '$lib/stores/ipc.svelte';

	const state = $derived(getTosState());
	const sectors = $derived(state.sectors || []);
	const activeIndex = $derived(state.active_sector_index);

	let dragHoverSector = $state<number | null>(null);

	async function handleSectorClick(index: number) {
		await ipc.switchSector(index);
		setCurrentMode('hubs');
	}

	function handleDragOver(e: DragEvent, index: number) {
		e.preventDefault();
		dragHoverSector = index;
	}

	function handleDragLeave(e: DragEvent) {
		dragHoverSector = null;
	}

	async function handleDrop(e: DragEvent, index: number) {
		e.preventDefault();
		dragHoverSector = null;
		
		const dt = e.dataTransfer;
		if (dt && dt.files.length > 0) {
			const file = dt.files[0];
			if (file.name.endsWith('.tos-session')) {
				const text = await file.text();
				const baseName = file.name.replace('.tos-session', '');
				// Switch to sector then import session
				await ipc.switchSector(index);
				await ipc.sendCommand(`session_import:${baseName};${text}`);
			}
		}
	}
</script>

<div class="global-overview">
	<div class="sector-grid">
		{#each sectors as sector, i}
			<button
				class="sector-tile"
				class:active={i === activeIndex}
				class:drag-hover={dragHoverSector === i}
				onclick={() => handleSectorClick(i)}
				ondragover={(e) => handleDragOver(e, i)}
				ondragleave={handleDragLeave}
				ondrop={(e) => handleDrop(e, i)}
			>
				<div class="sector-thumbnail">
					{#if sector.snapshot}
						<img src={sector.snapshot} alt="Sector {i} Preview" />
					{:else}
						<div class="no-feed">NO ACTIVE FEED</div>
					{/if}
				</div>
				<div class="sector-id">S0{i}</div>
				<div class="sector-name">{sector.name.toUpperCase()}</div>
				<div class="sector-meta">TYPE: {(sector.type || 'STANDARD').toUpperCase()}</div>
				<div class="sector-meta">STATUS: {(sector.status || 'ACTIVE').toUpperCase()}</div>
			</button>
		{/each}
	</div>
</div>

<style>
	.global-overview {
		width: 100%;
		height: 100%;
		padding: var(--space-md);
		animation: scaleIn 0.4s cubic-bezier(0.16, 1, 0.3, 1);
	}

	.sector-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(11rem, 1fr));
		gap: var(--space-md);
	}

	.sector-tile {
		display: flex;
		flex-direction: column;
		background: var(--glass-bg);
		border: 1px solid var(--glass-border);
		backdrop-filter: blur(var(--glass-blur));
		border-radius: var(--radius-md);
		padding: var(--space-md);
		cursor: pointer;
		transition: all var(--transition-normal);
		text-align: left;
		color: var(--color-text);
		font-family: var(--font-body);
		position: relative;
		overflow: hidden;
	}

	.sector-tile::before {
		content: '';
		position: absolute;
		inset: 0;
		background: linear-gradient(135deg, rgba(247, 168, 51, 0.03), transparent 40%);
		opacity: 0;
		transition: opacity var(--transition-normal);
	}

	.sector-tile:hover {
		border-color: var(--color-border-active);
		transform: translateY(-2px);
		box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
	}

	.sector-tile:hover::before {
		opacity: 1;
	}

	.sector-tile.active {
		border-color: var(--color-primary);
		box-shadow: 0 0 12px rgba(247, 168, 51, 0.15);
	}

	.sector-tile.active::before {
		opacity: 1;
	}

	.sector-tile.drag-hover {
		border-color: var(--color-success) !important;
		box-shadow: 0 0 16px rgba(102, 204, 102, 0.4) !important;
		transform: scale(1.02);
	}

	.sector-thumbnail {
		width: 100%;
		aspect-ratio: 16 / 9;
		background: repeating-linear-gradient(
			45deg,
			rgba(255, 255, 255, 0.02),
			rgba(255, 255, 255, 0.02) 4px,
			transparent 4px,
			transparent 8px
		);
		border-radius: var(--radius-sm);
		overflow: hidden;
		margin-bottom: var(--space-sm);
		display: flex;
		align-items: center;
		justify-content: center;
		border: 1px solid var(--color-border);
	}

	.sector-thumbnail img {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}

	.no-feed {
		font-size: 0.6rem;
		font-family: var(--font-display);
		letter-spacing: 0.1em;
		color: var(--color-text-muted);
	}

	.sector-id {
		font-size: 0.65rem;
		font-family: var(--font-mono);
		color: var(--color-text-muted);
	}

	.sector-name {
		font-family: var(--font-display);
		font-weight: 700;
		font-size: 0.95rem;
		color: var(--color-primary);
		margin: 0.15rem 0;
	}

	.sector-meta {
		font-size: 0.75rem;
		color: var(--color-text-dim);
		line-height: 1.4;
	}
</style>
