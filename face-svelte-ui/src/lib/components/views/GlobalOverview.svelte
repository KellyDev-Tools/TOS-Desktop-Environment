<script lang="ts">
	import { fade, scale } from 'svelte/transition';
	import { getTosState } from '$lib/stores/ipc.svelte';
	import { setCurrentMode, type ViewMode } from '$lib/stores/ui.svelte';
	import * as ipc from '$lib/stores/ipc.svelte';
	import { longpress } from '$lib/actions/longpress';
	import SectorContextMenu from '../SectorContextMenu.svelte';

	const tosState = $derived(getTosState());
	const sectors = $derived(tosState.sectors || []);
	const activeIndex = $derived(tosState.active_sector_index);

	let showResetModal = $state(false);
	let resetConfirmText = $state('');
	const RESET_KEYWORD = 'RED ALERT';

	let dragHoverSector = $state<number | null>(null);

	let cmState = $state<{
		open: boolean;
		x: number;
		y: number;
		sectorIndex: number;
		sectorName: string;
	}>({ open: false, x: 0, y: 0, sectorIndex: 0, sectorName: '' });

	function openContextMenu(e: MouseEvent | CustomEvent, index: number, name: string) {
		e.preventDefault();
		const ev = e instanceof CustomEvent ? e.detail : e;
		cmState = {
			open: true,
			x: ev.clientX,
			y: ev.clientY,
			sectorIndex: index,
			sectorName: name
		};
	}

	async function handleSectorClick(index: number) {
		console.log(`[GlobalOverview] Sector ${index} clicked`);
		setCurrentMode('hubs');
		await ipc.switchSector(index);
	}

	async function handleSystemReset() {
		if (resetConfirmText === RESET_KEYWORD) {
			await ipc.systemReset();
			showResetModal = false;
			resetConfirmText = '';
		}
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
		const classes = [];
		const hub = sector.hubs?.[sector.active_hub_index];
		if (hub) {
			if (hub.is_running) classes.push('border-running');
			if (hub.last_exit_status !== null && hub.last_exit_status !== undefined) {
				if (hub.last_exit_status === 0) classes.push('border-success');
				else classes.push('border-error');
			}
		}
		if (sector.priority >= 3) classes.push(`priority-${sector.priority}`);
		return classes.join(' ');
	}
</script>

<div class="global-overview">
	<div class="sector-grid">
		{#each sectors as sector, i}
			<button
				use:longpress={{ onLongPress: (e) => openContextMenu(e as CustomEvent, i, sector.name) }}
				oncontextmenu={(e: any) => openContextMenu(e, i, sector.name)}
				class="sector-tile {getBorderClass(sector)}"
				class:active={i === activeIndex}
				class:drag-hover={dragHoverSector === i}
				onclick={() => handleSectorClick(i)}
				ondragover={(e: any) => handleDragOver(e, i)}
				ondragleave={handleDragLeave}
				ondrop={(e: any) => handleDrop(e, i)}
			>
				{#if sector.priority >= 4}
					<div aria-roledescription="chip" class="priority-chip">PRIORITY {sector.priority}</div>
				{/if}
				<div class="sector-thumbnail">
					{#if sector.hubs && sector.hubs[0]?.activity_listing?.processes?.length}
						<div class="app-matrix">
							{#each sector.hubs[0].activity_listing.processes.slice(0, 4) as app}
								<div class="matrix-item app-item">
									<div class="matrix-icon">⊞</div>
									<div class="matrix-label">{app.name}</div>
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
						<div class="stat-group-container">
							<div class="stat-group" in:fade={{ delay: 200 + i * 50 }}>
								<span class="stat-label">TYPE</span>
								<span class="stat-value">{sector.name === 'CORE' ? 'SYSTEM' : 'ISOLATED'}</span>
							</div>
							<div class="stat-group" in:fade={{ delay: 300 + i * 50 }}>
								<span class="stat-label">STATUS</span>
								<span class="stat-value status-online">ACTIVE</span>
							</div>
						</div>
					{/if}
				</div>
				<div class="sector-id">S0{i}</div>
				<div class="sector-name">{sector.name.toUpperCase()}</div>
				<div class="sector-meta">TYPE: {(sector.name === 'CORE' ? 'SYSTEM' : 'ISOLATED')}</div>
				<div class="sector-meta">STATUS: ACTIVE</div>
			</button>
		{/each}
	</div>

	<div class="overview-actions">
		<button class="danger-zone-btn" onclick={() => showResetModal = true}>
			<span class="btn-icon">⚠</span> SYSTEM_RESET
		</button>
	</div>
	
	{#if cmState.open}
		<SectorContextMenu
			x={cmState.x}
			y={cmState.y}
			sectorIndex={cmState.sectorIndex}
			sectorName={cmState.sectorName}
			onClose={() => cmState.open = false}
		/>
	{/if}

	{#if showResetModal}
		<div class="modal-overlay" transition:fade>
			<div class="reset-modal glass-panel" in:scale={{ start: 0.9 }}>
				<header class="reset-header">
					<div class="alert-icon">⚠</div>
					<h2>CRITICAL_SYSTEM_RESET</h2>
				</header>
				
				<div class="reset-body">
					<p>Warning: This operation will terminate all active sectors, purge transient memory, and restore the system to factory defaults.</p>
					<p class="danger-text">ALL UNSAVED DATA WILL BE LOST.</p>
					
					<div class="confirm-box">
						<label for="reset-input">TYPE "{RESET_KEYWORD}" TO CONFIRM:</label>
						<input 
							id="reset-input"
							type="text" 
							bind:value={resetConfirmText} 
							placeholder="..."
							class="glass-input"
						/>
					</div>
				</div>

				<div class="modal-actions">
					<button class="lcars-btn secondary" onclick={() => { showResetModal = false; resetConfirmText = ''; }}>ABORT_OPERATION</button>
					<button 
						class="lcars-btn danger" 
						disabled={resetConfirmText !== RESET_KEYWORD}
						onclick={handleSystemReset}
					>
						EXECUTE_RESET
					</button>
				</div>
			</div>
		</div>
	{/if}
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

	/* Priority Glows */
	.sector-tile.priority-3 {
		box-shadow: 0 0 10px rgba(102, 153, 255, 0.2);
	}

	.sector-tile.priority-4 {
		border-color: var(--color-accent);
		box-shadow: 0 0 15px rgba(247, 168, 51, 0.3);
	}

	.sector-tile.priority-5 {
		border-color: var(--color-danger);
		box-shadow: 0 0 20px rgba(255, 51, 51, 0.4);
		animation: redAlertPulse 2s infinite;
	}

	.priority-chip {
		position: absolute;
		top: 0;
		right: 0;
		background: var(--color-accent);
		color: black;
		font-size: 0.55rem;
		font-weight: 900;
		padding: 2px 6px;
		border-radius: 0 0 0 4px;
		z-index: 2;
	}

	.priority-5 .priority-chip {
		background: var(--color-danger);
		color: white;
	}

	@keyframes redAlertPulse {
		0% { box-shadow: 0 0 10px rgba(255, 51, 51, 0.4); }
		50% { box-shadow: 0 0 25px rgba(255, 51, 51, 0.7); }
		100% { box-shadow: 0 0 10px rgba(255, 51, 51, 0.4); }
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

	.stat-group-container {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.stat-group {
		display: flex;
		flex-direction: column;
		align-items: center;
	}

	.stat-label {
		font-size: 0.6rem;
		color: var(--color-text-muted);
		font-family: var(--font-mono);
	}

	.stat-value {
		font-size: 0.8rem;
		color: var(--color-text);
		font-family: var(--font-display);
		font-weight: 700;
	}

	.status-online {
		color: var(--color-success);
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

	.overview-actions {
		margin-top: 40px;
		display: flex;
		justify-content: flex-end;
		padding-top: 20px;
		border-top: 1px solid rgba(255, 255, 255, 0.05);
	}

	.danger-zone-btn {
		background: rgba(255, 51, 51, 0.1);
		border: 1px solid rgba(255, 51, 51, 0.3);
		color: #ff3333;
		padding: 8px 20px;
		border-radius: 4px;
		font-family: var(--font-display);
		font-weight: 700;
		font-size: 0.75rem;
		cursor: pointer;
		transition: all 0.2s;
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.danger-zone-btn:hover {
		background: rgba(255, 51, 51, 0.2);
		border-color: #ff3333;
		box-shadow: 0 0 15px rgba(255, 51, 51, 0.3);
	}

	/* Reset Modal */
	.modal-overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.9);
		backdrop-filter: blur(10px);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}

	.reset-modal {
		width: 500px;
		padding: 40px;
		border: 1px solid var(--color-danger);
		display: flex;
		flex-direction: column;
		gap: 30px;
		border-radius: 20px;
		box-shadow: 0 0 50px rgba(255, 51, 51, 0.2);
	}

	.reset-header {
		display: flex;
		align-items: center;
		gap: 20px;
		color: var(--color-danger);
	}

	.reset-header h2 {
		font-family: var(--font-display);
		font-size: 1.5rem;
		letter-spacing: 0.1em;
		margin: 0;
	}

	.alert-icon {
		font-size: 2rem;
		animation: blink 1s infinite;
	}

	.reset-body p {
		line-height: 1.5;
		margin-bottom: 15px;
		color: var(--color-text-dim);
	}

	.danger-text {
		color: var(--color-danger);
		font-weight: 800;
		letter-spacing: 0.05em;
	}

	.confirm-box {
		margin-top: 30px;
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.confirm-box label {
		font-family: var(--font-mono);
		font-size: 0.7rem;
		opacity: 0.7;
	}

	.glass-input {
		background: rgba(255, 255, 255, 0.05);
		border: 1px solid rgba(255, 255, 255, 0.1);
		padding: 12px;
		color: white;
		font-family: var(--font-mono);
		border-radius: 4px;
		outline: none;
		text-align: center;
		letter-spacing: 0.2em;
	}

	.glass-input:focus {
		border-color: var(--color-danger);
		background: rgba(255, 255, 255, 0.1);
	}

	.modal-actions {
		display: flex;
		gap: 20px;
	}

	.modal-actions .lcars-btn {
		flex: 1;
		font-size: 0.85rem;
	}

	.lcars-btn.danger {
		background: var(--color-danger);
		color: white !important;
	}

	.lcars-btn.danger:disabled {
		opacity: 0.3;
		cursor: not-allowed;
	}

	.lcars-btn.secondary {
		background: rgba(255, 255, 255, 0.1);
		color: white !important;
	}

	.lcars-btn:hover:not(:disabled) {
		filter: brightness(1.2);
		box-shadow: 0 0 15px currentColor;
	}

	@keyframes blink {
		0%, 100% { opacity: 1; }
		50% { opacity: 0.3; }
	}

	@keyframes scaleIn {
		from { opacity: 0; transform: scale(0.95); }
		to { opacity: 1; transform: scale(1); }
	}
</style>
