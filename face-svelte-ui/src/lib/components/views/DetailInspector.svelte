<script lang="ts">
	import { onMount } from 'svelte';
	import { fade, slide, fly } from 'svelte/transition';
	import { getTosState, sendCommand, processInspect, getBuffer } from '$lib/stores/ipc.svelte';

	const tosState = $derived(getTosState());
	
	onMount(() => {
		console.log('[DetailInspector] Component mounted');
	});

	type InspectorMode = 'detail' | 'buffer' | 'reset';
	let mode = $state<InspectorMode>('detail');
	
	let selectedPid = $state<number | null>(null);
	let inspectionData = $state<any>(null);
	let bufferData = $state<string>('');
	let loading = $state(false);

	let killSwitchConfirm = $state(false);

	// Historical arrays for btop-style sparkline charts
	let cpuHistory = $state<number[]>(Array(24).fill(0));
	let memHistory = $state<number[]>(Array(24).fill(0));
	let pollInterval: any = null;

	// Reset histories and set up real-time polling on selected PID
	$effect(() => {
		if (selectedPid) {
			cpuHistory = Array(24).fill(0);
			memHistory = Array(24).fill(0);
			loadData();

			if (pollInterval) clearInterval(pollInterval);
			pollInterval = setInterval(loadData, 2000);
		} else {
			if (pollInterval) clearInterval(pollInterval);
		}

		return () => {
			if (pollInterval) clearInterval(pollInterval);
		};
	});

	async function loadData() {
		if (selectedPid === null) return;
		const pidStr = selectedPid.toString();
		
		if (mode === 'detail') {
			// Avoid full-screen scan overlays on periodic updates to keep UI smooth
			if (!inspectionData || inspectionData.pid !== selectedPid) {
				loading = true;
			}
			
			const resp = await processInspect(pidStr);
			if (resp === null) {
				inspectionData = { error: 'Failed to communicate with Brain' };
			} else {
				try {
					const data = typeof resp === 'string' ? JSON.parse(resp) : resp;
					inspectionData = data;

					// Push historical CPU/Memory values for btop-style graphs
					const cpuVal = parseFloat(data.cpu_percent) || 0;
					const rssKb = parseFloat(data.mem_rss) || 0;

					cpuHistory = [...cpuHistory.slice(1), cpuVal];
					memHistory = [...memHistory.slice(1), rssKb];
				} catch (e) {
					inspectionData = { error: 'Failed to parse metadata' };
				}
			}
			loading = false;
		} else if (mode === 'buffer') {
			loading = true;
			const resp = await getBuffer(pidStr);
			bufferData = resp || 'ERROR: Could not retrieve buffer';
			loading = false;
		}
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

	function selectProcess(pid: number) {
		selectedPid = pid;
	}

	// Derived metrics for resource gauges
	const cpuPercent = $derived(
		inspectionData ? (parseFloat(inspectionData.cpu_percent) || 0) : 0
	);
	const memRss = $derived(
		inspectionData ? (parseFloat(inspectionData.mem_rss) || 0) : 0
	);
	const memPercent = $derived(
		Math.min(100, (memRss / 131072) * 100) // Visually scaled to 128MB max
	);

	function formatMem(kb: number): string {
		if (kb >= 1024 * 1024) return (kb / (1024 * 1024)).toFixed(2) + ' GB';
		if (kb >= 1024) return (kb / 1024).toFixed(1) + ' MB';
		return kb + ' KB';
	}
</script>

<div class="detail-inspector {mode}-mode">
	{#snippet blockBar(percent: number, maxBlocks: number = 20)}
		{@const activeBlocks = Math.round((Math.max(0, Math.min(100, percent)) / 100) * maxBlocks)}
		<div class="btop-bar">
			{#each Array(maxBlocks) as _, idx}
				{@const isActive = idx < activeBlocks}
				{@const blockRatio = idx / maxBlocks}
				{@const blockColor = blockRatio > 0.85 ? 'var(--color-danger)' : blockRatio > 0.6 ? 'var(--color-warning)' : 'var(--color-success)'}
				<span 
					class="bar-block" 
					class:active={isActive}
					style="--block-color: {blockColor};"
				>
					{isActive ? '█' : '░'}
				</span>
			{/each}
		</div>
	{/snippet}

	{#snippet sparkline(history: number[], maxVal: number, color: string)}
		{@const width = 240}
		{@const height = 45}
		{@const max = Math.max(maxVal, ...history, 1)}
		{@const points = history.map((val, idx) => {
			const x = (idx / (history.length - 1)) * width;
			const y = height - (val / max) * height * 0.8 - 4;
			return `${x},${y}`;
		}).join(' ')}
		<div class="btop-spark-wrapper">
			<svg width="100%" height="100%" viewBox="0 0 {width} {height}" preserveAspectRatio="none" style="overflow: visible;">
				{#if points}
					<polygon 
						points="0,{height} {points} {width},{height}" 
						fill="url(#grad-{color})" 
						opacity="0.15"
					/>
					<polyline 
						fill="none" 
						stroke="var(--color-{color})" 
						stroke-width="1.5" 
						points={points} 
					/>
				{/if}
				<defs>
					<linearGradient id="grad-{color}" x1="0" y1="0" x2="0" y2="1">
						<stop offset="0%" stop-color="var(--color-{color})" stop-opacity="0.6"/>
						<stop offset="100%" stop-color="var(--color-{color})" stop-opacity="0.0"/>
					</linearGradient>
				</defs>
			</svg>
		</div>
	{/snippet}

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
		<!-- Sidebar for Process Selection (btop-style) -->
		<aside class="process-sidebar glass-panel">
			<div class="sidebar-label">┌─ [PROCESS LIST] ──────────┐</div>
			<div class="process-header-row">
				<span class="hdr-pid">PID</span>
				<span class="hdr-name">NAME</span>
				<span class="hdr-cpu">CPU%</span>
			</div>
			<div class="process-list">
				{#each tosState.sectors as sector}
					{#each sector.hubs as hub}
						{#if hub.activity_listing?.processes}
							{#each hub.activity_listing.processes as proc}
								<button 
									class="proc-item" 
									class:selected={selectedPid === proc.pid}
									onclick={() => selectProcess(proc.pid)}
								>
									<span class="proc-pid">{proc.pid}</span>
									<span class="proc-name">{proc.name}</span>
									<span class="proc-cpu">{proc.cpu_usage.toFixed(1)}%</span>
								</button>
							{/each}
						{/if}
					{/each}
				{/each}
			</div>
			<div class="sidebar-footer">└───────────────────────────┘</div>
		</aside>

		<!-- Main Content Area -->
		<main class="inspector-content glass-panel">
			{#if loading}
				<div class="loading-overlay">SCANNING...</div>
			{/if}

			{#if mode === 'reset'}
				<div class="wireframe-grid" in:fade>
					{#each tosState.sectors as sector, i}
						<div class="wireframe-sector" class:frozen={sector.frozen}>
							<div class="wf-title">S0{i} // {sector.name}</div>
							<div class="wf-metrics">
								<div>HUBS: {sector.hubs.length}</div>
								<div>STATE: {sector.frozen ? 'FROZEN' : 'NOMINAL'}</div>
							</div>
						</div>
					{/each}
				</div>
			{:else if mode === 'detail'}
				{#if inspectionData}
					{#if inspectionData.error}
						<div class="empty-state error-state" in:fade>{inspectionData.error}</div>
					{:else}
						<div class="btop-layout" in:fly={{ y: 15, duration: 300 }}>
							<!-- Left Column: Resource Gauges & Sparklines -->
							<div class="btop-left">
								<!-- CPU Usage Box -->
								<div class="btop-panel cpu-panel">
									<div class="panel-header">
										<span class="panel-title">┌─ [CPU OVERVIEW // GRAPH] ──</span>
										<span class="panel-line"></span>
										<span class="panel-title-right">──┐</span>
									</div>
									<div class="panel-content">
										<div class="gauge-row">
											<div class="large-val-box">
												<span class="btop-label">USAGE</span>
												<span class="btop-val cpu-val" style="color: {cpuPercent > 80 ? 'var(--color-danger)' : cpuPercent > 50 ? 'var(--color-warning)' : 'var(--color-success)'}">
													{cpuPercent.toFixed(1)}%
												</span>
											</div>
											<div class="bar-container">
												{@render blockBar(cpuPercent, 20)}
											</div>
										</div>
										<div class="graph-row">
											{@render sparkline(cpuHistory, 100, 'success')}
										</div>
									</div>
									<div class="panel-footer">└─────────────────────────────────┘</div>
								</div>

								<!-- Memory Usage Box -->
								<div class="btop-panel mem-panel">
									<div class="panel-header">
										<span class="panel-title">┌─ [MEMORY OVERVIEW // GRAPH] ─</span>
										<span class="panel-line"></span>
										<span class="panel-title-right">──┐</span>
									</div>
									<div class="panel-content">
										<div class="gauge-row">
											<div class="large-val-box">
												<span class="btop-label">RSS</span>
												<span class="btop-val mem-val" style="color: var(--color-primary)">
													{formatMem(memRss)}
												</span>
											</div>
											<div class="bar-container">
												{@render blockBar(memPercent, 20)}
											</div>
										</div>
										<div class="graph-row">
											{@render sparkline(memHistory, 262144, 'primary')}
										</div>
									</div>
									<div class="panel-footer">└─────────────────────────────────┘</div>
								</div>
							</div>

							<!-- Right Column: Process Details, Capabilities & History -->
							<div class="btop-right">
								<!-- Stats Panel -->
								<div class="btop-panel stats-panel">
									<div class="panel-header">
										<span class="panel-title">┌─ [PROCESS DETAILS] ─────────</span>
										<span class="panel-line"></span>
										<span class="panel-title-right">──┐</span>
									</div>
									<div class="panel-content details-grid">
										<div class="detail-row"><span class="lbl">PID:</span> <span class="val highlight">{inspectionData.pid}</span></div>
										<div class="detail-row"><span class="lbl">COMMAND:</span> <span class="val highlight cmd-val">{inspectionData.command}</span></div>
										<div class="detail-row">
											<span class="lbl">STATUS:</span> 
											<span class="status-badge" class:running={inspectionData.status?.toUpperCase() === 'S' || inspectionData.status?.toUpperCase() === 'R'}>
												{inspectionData.status?.toUpperCase() === 'S' ? 'SLEEPING' : inspectionData.status?.toUpperCase() === 'R' ? 'RUNNING' : inspectionData.status || '--'}
											</span>
										</div>
										<div class="detail-row"><span class="lbl">USER:</span> <span class="val">{inspectionData.user}</span></div>
										<div class="detail-row"><span class="lbl">UPTIME:</span> <span class="val">{inspectionData.uptime}</span></div>
										<div class="detail-row"><span class="lbl">THREADS:</span> <span class="val">{inspectionData.threads}</span></div>
										<div class="detail-row"><span class="lbl">SANDBOX:</span> <span class="val sandbox-val">{inspectionData.sandbox_tier}</span></div>
									</div>
									<div class="panel-footer">└─────────────────────────────────┘</div>
								</div>

								<!-- Permissions Panel -->
								{#if inspectionData.permissions}
									<div class="btop-panel caps-panel">
										<div class="panel-header">
											<span class="panel-title">┌─ [SECURITY PRIVILEGES] ────</span>
											<span class="panel-line"></span>
											<span class="panel-title-right">──┐</span>
										</div>
										<div class="panel-content caps-list">
											{#each inspectionData.permissions as perm}
												<span class="btop-cap-chip">✦ {perm}</span>
											{/each}
										</div>
										<div class="panel-footer">└─────────────────────────────────┘</div>
									</div>
								{/if}

								<!-- Event Log Panel -->
								{#if inspectionData.event_history}
									<div class="btop-panel log-panel">
										<div class="panel-header">
											<span class="panel-title">┌─ [PROCESS EVENT LOG] ──────</span>
											<span class="panel-line"></span>
											<span class="panel-title-right">──┐</span>
										</div>
										<div class="panel-content history-table">
											{#each inspectionData.event_history as event}
												<div class="history-row">
													<span class="hist-time">[{event.time}]</span>
													<span class="hist-event">{event.event}</span>
												</div>
											{/each}
										</div>
										<div class="panel-footer">└─────────────────────────────────┘</div>
									</div>
								{/if}
							</div>
						</div>
					{/if}
				{:else}
					<div class="empty-state">
						<div class="empty-icon">📊</div>
						<div>SELECT PROCESS TO ENGAGE COGNITIVE PROFILE</div>
						<div class="empty-sub">Inspecting active sector telemetries...</div>
					</div>
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
		background: #080a0f;
		color: var(--color-success);
		font-family: var(--font-mono, 'Courier New', monospace);
		padding: var(--space-md);
		overflow: hidden;
		box-sizing: border-box;
	}

	.inspector-header {
		margin-bottom: var(--space-sm);
		flex-shrink: 0;
	}

	.header-main {
		display: flex;
		justify-content: space-between;
		align-items: center;
		border-bottom: 2px solid var(--color-success);
		padding-bottom: 6px;
	}

	.inspector-header h2 {
		margin: 0;
		font-size: 1.1rem;
		font-weight: bold;
		letter-spacing: 0.1em;
		color: var(--color-success);
		text-shadow: 0 0 10px rgba(102, 204, 102, 0.4);
	}

	.mode-tabs {
		display: flex;
		gap: 6px;
	}

	.mode-tabs button {
		background: rgba(102, 204, 102, 0.05);
		border: 1px solid rgba(102, 204, 102, 0.4);
		color: rgba(102, 204, 102, 0.7);
		padding: 4px 12px;
		font-family: var(--font-mono, monospace);
		font-size: 0.75rem;
		cursor: pointer;
		border-radius: 3px;
		transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
	}

	.mode-tabs button:hover {
		background: rgba(102, 204, 102, 0.15);
		color: var(--color-success);
		border-color: var(--color-success);
		box-shadow: 0 0 8px rgba(102, 204, 102, 0.2);
	}

	.mode-tabs button.active {
		background: var(--color-success);
		color: #05070a;
		border-color: var(--color-success);
		font-weight: 700;
		box-shadow: 0 0 12px rgba(102, 204, 102, 0.4);
	}

	.warning-banner {
		background: rgba(247, 168, 51, 0.1);
		border: 1px solid var(--color-warning);
		color: var(--color-warning);
		padding: 4px 12px;
		font-size: 0.75rem;
		font-weight: 700;
		margin-top: 6px;
		text-align: center;
		border-radius: 3px;
		text-shadow: 0 0 5px rgba(247, 168, 51, 0.3);
		animation: pulse-warn 2s infinite ease-in-out;
	}

	@keyframes pulse-warn {
		0%, 100% { opacity: 0.9; }
		50% { opacity: 0.6; }
	}

	.buffer-warning {
		background: rgba(0, 150, 255, 0.1);
		border-color: var(--color-primary);
		color: var(--color-primary);
		text-shadow: 0 0 5px rgba(0, 150, 255, 0.3);
	}

	.inspector-body {
		flex: 1;
		display: flex;
		gap: var(--space-md);
		min-height: 0;
	}

	/* Sidebar styling */
	.process-sidebar {
		width: 280px;
		display: flex;
		flex-direction: column;
		background: rgba(10, 14, 23, 0.7);
		border: 1px solid rgba(102, 204, 102, 0.25);
		box-shadow: inset 0 0 15px rgba(102, 204, 102, 0.05);
		border-radius: 4px;
		padding: 6px;
		box-sizing: border-box;
		flex-shrink: 0;
	}

	.sidebar-label {
		font-size: 0.7rem;
		padding: 4px 6px;
		color: var(--color-success);
		opacity: 0.85;
		white-space: nowrap;
		letter-spacing: 0.05em;
	}

	.process-header-row {
		display: flex;
		font-size: 0.7rem;
		font-weight: bold;
		padding: 6px;
		border-bottom: 1px solid rgba(102, 204, 102, 0.3);
		color: var(--color-success);
		opacity: 0.9;
		letter-spacing: 0.05em;
	}

	.hdr-pid { width: 60px; }
	.hdr-name { flex: 1; }
	.hdr-cpu { width: 50px; text-align: right; }

	.process-list {
		flex: 1;
		overflow-y: auto;
		margin: 4px 0;
		padding-right: 2px;
	}

	/* Scrollbars */
	.process-list::-webkit-scrollbar,
	.inspector-content::-webkit-scrollbar,
	.history-table::-webkit-scrollbar {
		width: 4px;
	}
	.process-list::-webkit-scrollbar-track,
	.inspector-content::-webkit-scrollbar-track,
	.history-table::-webkit-scrollbar-track {
		background: rgba(0, 0, 0, 0.2);
	}
	.process-list::-webkit-scrollbar-thumb,
	.inspector-content::-webkit-scrollbar-thumb,
	.history-table::-webkit-scrollbar-thumb {
		background: rgba(102, 204, 102, 0.3);
		border-radius: 2px;
	}
	.process-list::-webkit-scrollbar-thumb:hover,
	.inspector-content::-webkit-scrollbar-thumb:hover,
	.history-table::-webkit-scrollbar-thumb:hover {
		background: var(--color-success);
	}

	.proc-item {
		width: 100%;
		display: flex;
		align-items: center;
		padding: 8px 6px;
		background: transparent;
		border: none;
		border-bottom: 1px solid rgba(102, 204, 102, 0.06);
		color: rgba(102, 204, 102, 0.85);
		font-family: var(--font-mono, monospace);
		font-size: 0.75rem;
		cursor: pointer;
		text-align: left;
		transition: all 0.15s ease;
		border-radius: 2px;
	}

	.proc-item:hover {
		background: rgba(102, 204, 102, 0.08);
		color: var(--color-success);
		padding-left: 10px;
	}

	.proc-item.selected {
		background: rgba(102, 204, 102, 0.18);
		color: #fff;
		font-weight: bold;
		border-left: 3px solid var(--color-success);
		text-shadow: 0 0 6px rgba(102, 204, 102, 0.6);
	}

	.proc-pid {
		width: 60px;
		opacity: 0.7;
	}

	.proc-name {
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.proc-cpu {
		width: 50px;
		text-align: right;
		color: var(--color-primary);
		font-weight: bold;
	}

	.sidebar-footer {
		font-size: 0.7rem;
		padding: 4px 6px;
		color: var(--color-success);
		opacity: 0.7;
		white-space: nowrap;
	}

	/* Main viewport content */
	.inspector-content {
		flex: 1;
		background: rgba(7, 10, 17, 0.85);
		border: 1px solid rgba(102, 204, 102, 0.25);
		border-radius: 4px;
		position: relative;
		overflow-y: auto;
		padding: var(--space-md);
		box-sizing: border-box;
		display: flex;
		flex-direction: column;
	}

	.loading-overlay {
		position: absolute;
		inset: 0;
		background: rgba(5, 7, 11, 0.9);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 10;
		font-size: 1.2rem;
		color: var(--color-success);
		letter-spacing: 0.4em;
		font-weight: bold;
		text-shadow: 0 0 10px var(--color-success);
		animation: blink 1.2s infinite ease-in-out;
	}

	/* btop Layout Grid */
	.btop-layout {
		display: grid;
		grid-template-columns: 1.1fr 1fr;
		gap: var(--space-md);
		height: 100%;
		min-height: 0;
		align-items: start;
	}

	.btop-left, .btop-right {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
		min-height: 0;
	}

	/* btop Panel Cards */
	.btop-panel {
		background: rgba(12, 17, 28, 0.9);
		border-radius: 4px;
		display: flex;
		flex-direction: column;
		box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
		overflow: hidden;
	}

	.panel-header {
		display: flex;
		align-items: center;
		padding: 2px 8px;
		font-size: 0.7rem;
		font-weight: bold;
		color: var(--color-success);
		white-space: nowrap;
		opacity: 0.9;
	}

	.panel-title {
		flex-shrink: 0;
	}

	.panel-line {
		flex-grow: 1;
		border-bottom: 1px dashed rgba(102, 204, 102, 0.2);
		margin: 0 6px;
	}

	.panel-title-right {
		flex-shrink: 0;
	}

	.panel-content {
		padding: 10px 14px;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.panel-footer {
		font-size: 0.7rem;
		padding: 2px 8px;
		color: var(--color-success);
		opacity: 0.7;
		margin-top: -2px;
	}

	/* Gauge & Progress block bars */
	.gauge-row {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 12px;
	}

	.large-val-box {
		display: flex;
		flex-direction: column;
	}

	.btop-label {
		font-size: 0.65rem;
		opacity: 0.6;
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.btop-val {
		font-size: 1.3rem;
		font-weight: 900;
		font-family: var(--font-mono, monospace);
		text-shadow: 0 0 8px rgba(255, 255, 255, 0.1);
	}

	.bar-container {
		flex: 1;
		display: flex;
		align-items: center;
		justify-content: flex-end;
	}

	.btop-bar {
		display: flex;
		gap: 2px;
		background: rgba(0, 0, 0, 0.3);
		padding: 3px 6px;
		border-radius: 3px;
		border: 1px solid rgba(102, 204, 102, 0.15);
	}

	.bar-block {
		font-size: 0.95rem;
		line-height: 1;
		color: rgba(102, 204, 102, 0.15);
		transition: all 0.3s ease;
	}

	.bar-block.active {
		color: var(--block-color);
		text-shadow: 0 0 5px var(--block-color);
	}

	/* Sparklines Graph Row */
	.graph-row {
		height: 60px;
		background: rgba(0, 0, 0, 0.4);
		border-radius: 4px;
		border: 1px solid rgba(102, 204, 102, 0.1);
		position: relative;
		overflow: hidden;
		padding: 4px;
		box-sizing: border-box;
	}

	.btop-spark-wrapper {
		width: 100%;
		height: 100%;
	}

	/* Process detail fields */
	.details-grid {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 8px var(--space-md);
	}

	.detail-row {
		display: flex;
		justify-content: space-between;
		font-size: 0.75rem;
		border-bottom: 1px dotted rgba(102, 204, 102, 0.1);
		padding-bottom: 4px;
		align-items: center;
	}

	.detail-row .lbl {
		opacity: 0.6;
		font-size: 0.7rem;
	}

	.detail-row .val {
		color: #fff;
		font-weight: 500;
	}

	.detail-row .val.highlight {
		color: var(--color-success);
		text-shadow: 0 0 4px rgba(102, 204, 102, 0.4);
	}

	.detail-row .val.cmd-val {
		font-weight: bold;
		color: var(--color-primary);
		max-width: 120px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.detail-row .val.sandbox-val {
		color: var(--color-warning);
	}

	.status-badge {
		background: rgba(255, 51, 51, 0.1);
		border: 1px solid rgba(255, 51, 51, 0.4);
		color: #ff5555;
		font-size: 0.65rem;
		padding: 2px 6px;
		border-radius: 2px;
		text-shadow: 0 0 3px rgba(255, 51, 51, 0.3);
		text-transform: uppercase;
		font-weight: bold;
	}

	.status-badge.running {
		background: rgba(102, 204, 102, 0.1);
		border: 1px solid rgba(102, 204, 102, 0.4);
		color: var(--color-success);
		text-shadow: 0 0 3px rgba(102, 204, 102, 0.3);
	}

	/* Security privileges */
	.caps-list {
		flex-direction: row;
		flex-wrap: wrap;
		gap: 6px;
	}

	.btop-cap-chip {
		background: rgba(0, 150, 255, 0.08);
		border: 1px solid rgba(0, 150, 255, 0.3);
		color: var(--color-primary);
		padding: 2px 6px;
		font-size: 0.65rem;
		border-radius: 2px;
		letter-spacing: 0.05em;
		transition: all 0.2s ease;
	}

	.btop-cap-chip:hover {
		background: rgba(0, 150, 255, 0.18);
		color: #fff;
		box-shadow: 0 0 6px rgba(0, 150, 255, 0.3);
	}

	/* Event logs table */
	.history-table {
		display: flex;
		flex-direction: column;
		gap: 4px;
		max-height: 120px;
		overflow-y: auto;
		padding-right: 4px;
	}

	.history-row {
		font-size: 0.7rem;
		display: flex;
		gap: 10px;
		align-items: flex-start;
		border-bottom: 1px solid rgba(102, 204, 102, 0.05);
		padding-bottom: 2px;
	}

	.hist-time {
		color: var(--color-warning);
		opacity: 0.8;
		flex-shrink: 0;
	}

	.hist-event {
		color: rgba(255, 255, 255, 0.85);
		word-break: break-all;
	}

	/* Buffer View */
	.buffer-view {
		font-size: 0.75rem;
		line-height: 1.3;
		color: var(--color-success);
		background: rgba(0, 0, 0, 0.5);
		border-radius: 4px;
		padding: 12px;
		border: 1px dashed rgba(102, 204, 102, 0.2);
		overflow: auto;
		max-height: 100%;
	}

	.buffer-view pre {
		margin: 0;
		font-family: var(--font-mono, monospace);
	}

	/* Reset Mode (Wireframe) */
	.wireframe-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
		gap: var(--space-md);
	}

	.wireframe-sector {
		border: 1px solid rgba(102, 204, 102, 0.3);
		padding: var(--space-md);
		background: rgba(10, 14, 23, 0.8);
		box-shadow: inset 0 0 15px rgba(102, 204, 102, 0.1);
		border-radius: 4px;
		transition: all 0.2s ease;
	}

	.wireframe-sector:hover {
		border-color: var(--color-success);
		box-shadow: inset 0 0 20px rgba(102, 204, 102, 0.2);
	}

	.wireframe-sector.frozen {
		border-color: rgba(255, 51, 51, 0.4);
		color: #ff5555;
		box-shadow: inset 0 0 15px rgba(255, 51, 51, 0.1);
	}

	.wireframe-sector.frozen:hover {
		border-color: #ff3333;
		box-shadow: inset 0 0 20px rgba(255, 51, 51, 0.2);
	}

	.wf-title {
		font-weight: bold;
		border-bottom: 1px solid currentColor;
		padding-bottom: 4px;
		margin-bottom: 8px;
		font-size: 0.8rem;
		letter-spacing: 0.05em;
	}

	.wf-metrics {
		font-size: 0.7rem;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	/* Empty state view */
	.empty-state {
		height: 100%;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: var(--space-md);
		color: var(--color-success);
		opacity: 0.5;
		text-align: center;
		margin: auto;
	}

	.empty-icon {
		font-size: 2.2rem;
		animation: pulse-icon 2s infinite ease-in-out;
	}

	@keyframes pulse-icon {
		0%, 100% { transform: scale(1); opacity: 0.4; }
		50% { transform: scale(1.1); opacity: 0.8; }
	}

	.empty-sub {
		font-size: 0.7rem;
		opacity: 0.6;
	}

	.inspector-footer {
		margin-top: var(--space-sm);
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-sm);
		flex-shrink: 0;
	}

	.interlock-status {
		color: var(--color-warning);
		font-size: 0.7rem;
		letter-spacing: 0.1em;
		text-shadow: 0 0 4px rgba(247, 168, 51, 0.4);
		animation: blink 2.5s infinite ease-in-out;
	}

	.kill-switch {
		background: rgba(255, 51, 51, 0.05);
		border: 1px solid #ff3333;
		color: #ff3333;
		padding: 8px 24px;
		font-family: var(--font-mono, monospace);
		font-weight: bold;
		font-size: 0.85rem;
		cursor: pointer;
		transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
		border-radius: 4px;
		letter-spacing: 0.05em;
	}

	.kill-switch:hover {
		background: rgba(255, 51, 51, 0.15);
		box-shadow: 0 0 12px rgba(255, 51, 51, 0.3);
	}

	.kill-switch.confirming {
		background: #ff3333;
		color: #000;
		animation: shake 0.2s infinite;
		box-shadow: 0 0 20px rgba(255, 51, 51, 0.5);
	}

	@keyframes blink {
		0%, 100% { opacity: 1; }
		50% { opacity: 0.45; }
	}

	@keyframes shake {
		0%, 100% { transform: translateX(0); }
		25% { transform: translateX(-4px); }
		75% { transform: translateX(4px); }
	}
</style>
