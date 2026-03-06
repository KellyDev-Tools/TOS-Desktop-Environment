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

	function getBorderClass(sector: any): string {
		const hub = sector.hubs?.[sector.active_hub_index];
		if (!hub) return '';
		if (hub.is_running) return 'border-running';
		if (hub.last_exit_status !== null && hub.last_exit_status !== undefined) {
			if (hub.last_exit_status === 0) return 'border-success';
			return 'border-error';
		}
		return '';
	}
</script>

<div class="global-overview">
	<div class="sector-grid">
		{#each sectors as sector, i}
			<button
				class="sector-tile {getBorderClass(sector)}"
				class:active={i === activeIndex}
				class:drag-hover={dragHoverSector === i}
				onclick={() => handleSectorClick(i)}
				ondragover={(e) => handleDragOver(e, i)}
				ondragleave={handleDragLeave}
				ondrop={(e) => handleDrop(e, i)}
			>
				<div class="sector-thumbnail">
					{#if sector.active_apps && sector.active_apps.length > 0}
						<div class="app-matrix">
							{#each sector.active_apps.slice(0, 4) as app}
								<div class="matrix-item app-item">
									<div class="matrix-icon">⊞</div>
									<div class="matrix-label">{app.title || app.model_id}</div>
								</div>
							{/each}
						</div>
					{:else if sector.hubs && sector.hubs.length > 0}
						<div class="app-matrix">
							{#each sector.hubs.slice(0, 4) as hub}
								<div class="matrix-item hub-item">
									<div class="matrix-icon">_</div>
									<div class="matrix-label">{hub.mode || 'CMD'}</div>
								</div>
							{/each}
						</div>
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

	/* Kinetic Borders */
	.sector-tile.border-running {
		border: 2px solid transparent;
		background-clip: padding-box;
		position: relative;
	}

	.sector-tile.border-running::after {
		content: '';
		position: absolute;
		inset: -2px;
		z-index: -1;
		border-radius: inherit;
		background: linear-gradient(90deg, var(--color-primary), var(--color-accent), var(--color-primary));
		background-size: 200% 100%;
		animation: slide-gradient 1.5s linear infinite;
	}

	.sector-tile.border-success {
		border-color: var(--color-success);
	}

	.sector-tile.border-error {
		border-color: var(--color-warning);
	}

	@keyframes slide-gradient {
		0% { background-position: 100% 0; }
		100% { background-position: -100% 0; }
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

	.app-matrix {
		display: grid;
		grid-template-columns: 1fr 1fr;
		grid-template-rows: 1fr 1fr;
		gap: 4px;
		width: 100%;
		height: 100%;
		padding: 4px;
	}

	.matrix-item {
		background: rgba(0, 0, 0, 0.4);
		border: 1px solid rgba(255, 255, 255, 0.1);
		border-radius: 2px;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		overflow: hidden;
	}

	.matrix-item.hub-item {
		border-color: rgba(247, 168, 51, 0.2);
	}

	.matrix-item.app-item {
		border-color: rgba(102, 204, 102, 0.2);
	}

	.matrix-icon {
		font-size: 1.2rem;
		margin-bottom: 2px;
		color: var(--color-text-dim);
	}

	.hub-item .matrix-icon {
		color: var(--color-primary);
		font-family: var(--font-mono);
		animation: blink 1s infinite;
	}

	.app-item .matrix-icon {
		color: var(--color-success);
	}

	.matrix-label {
		font-size: 0.45rem;
		font-family: var(--font-mono);
		opacity: 0.8;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		max-width: 90%;
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
