<script lang="ts">
	import { fade, slide, fly } from 'svelte/transition';
	import { getTosState, sendCommand, processInspect, getBuffer } from '$lib/stores/ipc.svelte';

	const tosState = $derived(getTosState());
	
	type InspectorMode = 'detail' | 'buffer' | 'reset';
	let mode = $state<InspectorMode>('detail');
	
	let selectedPid = $state<string | null>(null);
	let inspectionData = $state<any>(null);
	let bufferData = $state<string>('');
	let loading = $state(false);

	let killSwitchConfirm = $state(false);

	$effect(() => {
		if (selectedPid) {
			loadData();
		}
	});

	async function loadData() {
		if (!selectedPid) return;
		loading = true;
		if (mode === 'detail') {
			const resp = await processInspect(selectedPid);
			try {
				inspectionData = JSON.parse(resp);
			} catch (e) {
				inspectionData = { error: 'Failed to parse metadata' };
			}
		} else if (mode === 'buffer') {
			bufferData = await getBuffer(selectedPid);
		}
		loading = false;
	}

	async function activateKillSwitch() {
		if (killSwitchConfirm) {
			await sendCommand('tactical_kill_switch:');
			killSwitchConfirm = false;
		} else {
			killSwitchConfirm = true;
			setTimeout(() => { killSwitchConfirm = false; }, 3000);
		}
	}

	function selectProcess(pid: string) {
		selectedPid = pid;
	}
</script>

<div class="detail-inspector {mode}-mode">
	<header class="inspector-header">
		<div class="header-main">
			<h2>LEVEL 4 // DEEP_INSPECTION</h2>
			<div class="mode-tabs">
				<button class:active={mode === 'detail'} onclick={() => mode = 'detail'}>DETAIL_VIEW</button>
				<button class:active={mode === 'buffer'} onclick={() => mode = 'buffer'}>BUFFER_VIEW</button>
				<button class:active={mode === 'reset'} onclick={() => mode = 'reset'}>TACTICAL_RESET</button>
			</div>
		</div>
		{#if mode === 'reset'}
			<div class="warning-banner" transition:slide>SYSTEM DEADLOCK OR HIGH LATENCY DETECTED. DIAGNOSTIC RENDERER ACTIVE.</div>
		{:else if mode === 'buffer'}
			<div class="warning-banner buffer-warning" transition:slide>⚠ WARNING: RAW MEMORY ACCESS IS READ-ONLY. PRIVILEGE ELEVATION ACTIVE.</div>
		{/if}
	</header>

	<div class="inspector-body">
		<!-- Sidebar for Process Selection -->
		<aside class="process-sidebar glass-panel">
			<div class="sidebar-label">ACTIVE_PROCESSES</div>
			<div class="process-list">
				{#each tosState.sectors as sector}
					{#each sector.hubs as hub}
						{#each hub.activity_listing.processes as proc}
							<button 
								class="proc-item" 
								class:selected={selectedPid === proc.pid}
								onclick={() => selectProcess(proc.pid)}
							>
								<span class="proc-pid">{proc.pid}</span>
								<span class="proc-name">{proc.name}</span>
								<span class="proc-cpu">{proc.cpu_usage}%</span>
							</button>
						{/each}
					{/each}
				{/each}
			</div>
		</aside>

		<!-- Main Content Area -->
		<main class="inspector-content glass-panel">
			{#if loading}
				<div class="loading-overlay">SCANNING...</div>
			{/if}

			{#if mode === 'reset'}
				<div class="wireframe-grid" in:fade>
					{#each tosState.sectors as sector, i}
						<div class="wireframe-sector" class:frozen={sector.frozen} class:deadlocked={sector.status === 'Deadlocked'}>
							<div class="wf-title">S0{i} // {sector.name}</div>
							<div class="wf-metrics">
								<div>HUBS: {sector.hubs.length}</div>
								<div>STATE: {sector.frozen ? 'FROZEN' : (sector.status === 'Deadlocked' ? 'DEADLOCKED' : 'NOMINAL')}</div>
							</div>
						</div>
					{/each}
				</div>
			{:else if mode === 'detail'}
				{#if inspectionData}
					<div class="metadata-view" in:fly={{ y: 20 }}>
						<div class="meta-section">
							<h3>STRUCTURED_METADATA // PID {inspectionData.pid}</h3>
							<div class="meta-grid">
								<div class="meta-item"><span class="label">COMMAND:</span> <span class="val">{inspectionData.command}</span></div>
								<div class="meta-item"><span class="label">USER:</span> <span class="val">{inspectionData.user}</span></div>
								<div class="meta-item"><span class="label">STATUS:</span> <span class="val status-{inspectionData.status.toLowerCase()}">{inspectionData.status}</span></div>
								<div class="meta-item"><span class="label">UPTIME:</span> <span class="val">{inspectionData.uptime}</span></div>
								<div class="meta-item"><span class="label">CPU:</span> <span class="val">{inspectionData.cpu_percent}%</span></div>
								<div class="meta-item"><span class="label">MEM_RSS:</span> <span class="val">{inspectionData.mem_rss} KB</span></div>
								<div class="meta-item"><span class="label">THREADS:</span> <span class="val">{inspectionData.threads}</span></div>
								<div class="meta-item"><span class="label">SANDBOX:</span> <span class="val">{inspectionData.sandbox_tier}</span></div>
							</div>
						</div>
						<div class="meta-section">
							<h3>SECURITY_CAPABILITIES</h3>
							<div class="caps-list">
								{#each inspectionData.permissions as perm}
									<span aria-roledescription="chip" class="cap-chip">{perm}</span>
								{/each}
							</div>
						</div>
						<div class="meta-section">
							<h3>EVENT_HISTORY</h3>
							<div class="history-table">
								{#each inspectionData.event_history as event}
									<div class="history-row">
										<span class="hist-time">[{event.time}]</span>
										<span class="hist-event">{event.event}</span>
									</div>
								{/each}
							</div>
						</div>
					</div>
				{:else}
					<div class="empty-state">SELECT PROCESS TO INSPECT METADATA</div>
				{/if}
			{:else if mode === 'buffer'}
				{#if bufferData}
					<div class="buffer-view" in:fade>
						<pre>{bufferData}</pre>
					</div>
				{:else}
					<div class="empty-state">SELECT PROCESS TO READ MEMORY BUFFER</div>
				{/if}
			{/if}
		</main>
	</div>

	<footer class="inspector-footer">
		{#if mode === 'reset'}
			<button class="kill-switch" class:confirming={killSwitchConfirm} onclick={activateKillSwitch}>
				{killSwitchConfirm ? 'SYSTEM RE-AUTH REQUIRED — CLICK AGAIN TO EXECUTE' : '[[ GLOBAL PROCESS KILL-SWITCH ]]'}
			</button>
		{:else}
			<div class="interlock-status">PROMPT INTERLOCK: ENGAGED [Expanded Bezel Disabled]</div>
		{/if}
	</footer>
</div>

<style>
	.detail-inspector {
		height: 100%;
		display: flex;
		flex-direction: column;
		background: #000;
		color: var(--color-success);
		font-family: var(--font-mono);
		padding: var(--space-lg);
		background-image: 
			linear-gradient(rgba(102, 204, 102, 0.05) 1px, transparent 1px),
			linear-gradient(90deg, rgba(102, 204, 102, 0.05) 1px, transparent 1px);
		background-size: 40px 40px;
	}

	.inspector-header {
		margin-bottom: var(--space-md);
	}

	.header-main {
		display: flex;
		justify-content: space-between;
		align-items: center;
		border-bottom: 1px solid var(--color-success);
		padding-bottom: var(--space-sm);
	}

	.inspector-header h2 {
		margin: 0;
		font-size: 1.2rem;
		letter-spacing: 0.1em;
		color: var(--color-success);
	}

	.mode-tabs {
		display: flex;
		gap: 2px;
	}

	.mode-tabs button {
		background: rgba(102, 204, 102, 0.1);
		border: 1px solid var(--color-success);
		color: var(--color-success);
		padding: 4px 12px;
		font-family: var(--font-mono);
		font-size: 0.7rem;
		cursor: pointer;
		transition: all 0.2s;
	}

	.mode-tabs button.active {
		background: var(--color-success);
		color: #000;
		font-weight: 700;
	}

	.warning-banner {
		background: var(--color-warning);
		color: #000;
		padding: 4px 12px;
		font-size: 0.7rem;
		font-weight: 700;
		margin-top: var(--space-xs);
		text-align: center;
	}

	.buffer-warning {
		background: var(--color-primary);
	}

	.inspector-body {
		flex: 1;
		display: flex;
		gap: var(--space-md);
		min-height: 0;
	}

	.process-sidebar {
		width: 250px;
		display: flex;
		flex-direction: column;
		background: rgba(0, 10, 0, 0.6);
	}

	.sidebar-label {
		font-size: 0.6rem;
		padding: 8px;
		border-bottom: 1px solid rgba(102, 204, 102, 0.3);
		color: var(--color-success);
		opacity: 0.7;
	}

	.process-list {
		flex: 1;
		overflow-y: auto;
	}

	.proc-item {
		width: 100%;
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 8px;
		background: transparent;
		border: none;
		border-bottom: 1px solid rgba(102, 204, 102, 0.1);
		color: var(--color-success);
		font-family: var(--font-mono);
		font-size: 0.7rem;
		cursor: pointer;
		text-align: left;
	}

	.proc-item:hover {
		background: rgba(102, 204, 102, 0.1);
	}

	.proc-item.selected {
		background: rgba(102, 204, 102, 0.2);
		border-left: 3px solid var(--color-success);
	}

	.proc-pid {
		width: 40px;
		opacity: 0.6;
	}

	.proc-name {
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.proc-cpu {
		color: var(--color-primary);
	}

	.inspector-content {
		flex: 1;
		background: rgba(0, 5, 0, 0.8);
		position: relative;
		overflow-y: auto;
		padding: var(--space-lg);
	}

	.loading-overlay {
		position: absolute;
		inset: 0;
		background: rgba(0, 0, 0, 0.8);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 10;
		font-size: 1.5rem;
		letter-spacing: 0.5em;
		animation: blink 1s infinite;
	}

	/* Metadata View */
	.meta-section {
		margin-bottom: var(--space-xl);
	}

	.meta-section h3 {
		font-size: 0.8rem;
		color: var(--color-primary);
		border-bottom: 1px solid rgba(247, 168, 51, 0.3);
		padding-bottom: 4px;
		margin-bottom: var(--space-md);
	}

	.meta-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: var(--space-md);
	}

	.meta-item {
		font-size: 0.8rem;
		display: flex;
		gap: 10px;
	}

	.meta-item .label {
		opacity: 0.6;
		width: 100px;
	}

	.status-running { color: var(--color-success); }

	.caps-list {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}

	.cap-chip {
		background: rgba(102, 204, 102, 0.1);
		border: 1px solid var(--color-success);
		padding: 2px 8px;
		font-size: 0.65rem;
		border-radius: 2px;
	}

	.history-table {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.history-row {
		font-size: 0.75rem;
		display: flex;
		gap: 15px;
	}

	.hist-time { opacity: 0.5; }

	/* Buffer View */
	.buffer-view {
		font-size: 0.75rem;
		line-height: 1.2;
		color: var(--color-success);
		opacity: 0.9;
	}

	.buffer-view pre {
		margin: 0;
	}

	/* Reset Mode (Wireframe) */
	.wireframe-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
		gap: var(--space-lg);
	}

	.wireframe-sector {
		border: 1px solid var(--color-success);
		padding: var(--space-md);
		background: rgba(0, 0, 0, 0.8);
		box-shadow: inset 0 0 15px rgba(102, 204, 102, 0.2);
	}

	.wireframe-sector.frozen { border-color: var(--color-primary); color: var(--color-primary); }
	.wireframe-sector.deadlocked {
		border-color: #ff3333;
		color: #ff3333;
		animation: blink 0.5s infinite;
	}

	.wf-title {
		font-weight: 700;
		border-bottom: 1px solid currentColor;
		padding-bottom: 4px;
		margin-bottom: 8px;
		font-size: 0.8rem;
	}

	.wf-metrics { font-size: 0.7rem; }

	.empty-state {
		height: 100%;
		display: flex;
		align-items: center;
		justify-content: center;
		opacity: 0.3;
		letter-spacing: 0.2em;
		font-size: 1.2rem;
		text-align: center;
	}

	.inspector-footer {
		margin-top: var(--space-md);
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-md);
	}

	.interlock-status {
		color: var(--color-warning);
		font-size: 0.7rem;
		letter-spacing: 0.1em;
	}

	.kill-switch {
		background: transparent;
		border: 2px solid #ff3333;
		color: #ff3333;
		padding: 10px 30px;
		font-family: var(--font-display);
		font-weight: 900;
		font-size: 1rem;
		cursor: pointer;
		transition: all 0.2s;
	}

	.kill-switch.confirming {
		background: #ff3333;
		color: #000;
		animation: shake 0.2s infinite;
	}

	@keyframes blink {
		0%, 100% { opacity: 1; }
		50% { opacity: 0.3; }
	}

	@keyframes shake {
		0%, 100% { transform: translateX(0); }
		25% { transform: translateX(-5px); }
		75% { transform: translateX(5px); }
	}
</style>
