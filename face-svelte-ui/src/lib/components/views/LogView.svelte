<script lang="ts">
	import { fade, slide } from 'svelte/transition';
	import { getTosState, sendCommand } from '$lib/stores/ipc.svelte';

	const tosState = $derived(getTosState());
	
	let filterText = $state('');
	let selectedCategory = $state('ALL');
	
	const categories = ['ALL', 'SYSTEM', 'AI', 'TRUST', 'NETWORK', 'USER'];
	
	const filteredLogs = $derived(
		tosState.system_log.filter(log => {
			const matchesText = log.text.toLowerCase().includes(filterText.toLowerCase());
			const matchesCategory = selectedCategory === 'ALL' || 
				(selectedCategory === 'SYSTEM' && !log.text.startsWith('[')) ||
				(selectedCategory === 'AI' && log.text.includes('[AI')) ||
				(selectedCategory === 'TRUST' && log.text.includes('[TRUST')) ||
				(selectedCategory === 'NETWORK' && log.text.includes('[NET')) ||
				(selectedCategory === 'USER' && log.text.includes('[USER'));
			return matchesText && matchesCategory;
		}).reverse()
	);

	function getLogClass(text: string) {
		if (text.includes('ERR') || text.includes('FAIL')) return 'log-error';
		if (text.includes('WARN')) return 'log-warn';
		if (text.includes('[AI')) return 'log-ai';
		if (text.includes('[TRUST')) return 'log-trust';
		return '';
	}

	function formatTime(dateStr: string | Date) {
		const d = new Date(dateStr);
		return d.toLocaleTimeString('en-GB', { hour12: false, fractionalSecondDigits: 3 });
	}

	async function clearLogs() {
		await sendCommand('clear_system_log');
	}

	async function exportLogs() {
		// Mock export
		const blob = new Blob([JSON.stringify(tosState.system_log, null, 2)], { type: 'application/json' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = `tos_logs_${new Date().toISOString()}.json`;
		a.click();
	}
</script>

<div class="log-view-container">
	<header class="log-header">
		<div class="header-left">
			<h2>GLOBAL_TOS_LOG_SECTOR</h2>
			<div class="log-stats">
				<span>TOTAL: {tosState.system_log.length}</span>
				<span>VISIBLE: {filteredLogs.length}</span>
			</div>
		</div>
		<div class="log-controls">
			<input 
				type="text" 
				placeholder="FILTER LOGS..." 
				bind:value={filterText}
				class="filter-input"
			/>
			<select bind:value={selectedCategory} class="category-select">
				{#each categories as cat}
					<option value={cat}>{cat}</option>
				{/each}
			</select>
			<button class="log-btn" onclick={exportLogs}>EXPORT</button>
			<button class="log-btn danger" onclick={clearLogs}>CLEAR</button>
		</div>
	</header>

	<main class="log-scroll-area">
		<div class="log-list">
			{#each filteredLogs as log (log.timestamp + log.text)}
				<div class="log-row {getLogClass(log.text)}" in:slide={{ duration: 200 }}>
					<span class="log-time">{formatTime(log.timestamp)}</span>
					<span class="log-priority">P{log.priority}</span>
					<span class="log-content">{log.text}</span>
				</div>
			{/each}
		</div>
	</main>
</div>

<style>
	.log-view-container {
		height: 100%;
		display: flex;
		flex-direction: column;
		background: rgba(10, 10, 15, 0.95);
		color: var(--color-text);
		font-family: var(--font-mono);
		border: 1px solid var(--color-border);
	}

	.log-header {
		padding: var(--space-md) var(--space-lg);
		border-bottom: 2px solid var(--color-primary);
		display: flex;
		justify-content: space-between;
		align-items: center;
		background: rgba(247, 168, 51, 0.05);
	}

	.header-left h2 {
		margin: 0;
		font-family: var(--font-display);
		font-size: 1.1rem;
		color: var(--color-primary);
		letter-spacing: 0.1em;
	}

	.log-stats {
		font-size: 0.6rem;
		display: flex;
		gap: 15px;
		opacity: 0.6;
		margin-top: 4px;
	}

	.log-controls {
		display: flex;
		gap: var(--space-sm);
	}

	.filter-input {
		background: rgba(0, 0, 0, 0.5);
		border: 1px solid var(--color-border);
		color: var(--color-text);
		padding: 4px 10px;
		font-family: var(--font-mono);
		font-size: 0.75rem;
		width: 200px;
	}

	.category-select {
		background: rgba(0, 0, 0, 0.5);
		border: 1px solid var(--color-border);
		color: var(--color-primary);
		padding: 4px;
		font-family: var(--font-mono);
		font-size: 0.75rem;
	}

	.log-btn {
		background: rgba(247, 168, 51, 0.1);
		border: 1px solid var(--color-primary);
		color: var(--color-primary);
		padding: 4px 12px;
		font-family: var(--font-mono);
		font-size: 0.7rem;
		cursor: pointer;
		transition: all 0.2s;
	}

	.log-btn:hover {
		background: var(--color-primary);
		color: #000;
	}

	.log-btn.danger {
		border-color: #ff3333;
		color: #ff3333;
	}

	.log-btn.danger:hover {
		background: #ff3333;
		color: #000;
	}

	.log-scroll-area {
		flex: 1;
		overflow-y: auto;
		padding: var(--space-md);
		background: 
			linear-gradient(rgba(247, 168, 51, 0.02) 1px, transparent 1px),
			linear-gradient(90deg, rgba(247, 168, 51, 0.02) 1px, transparent 1px);
		background-size: 50px 50px;
	}

	.log-list {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.log-row {
		display: flex;
		gap: 15px;
		padding: 4px 8px;
		font-size: 0.75rem;
		border-left: 2px solid transparent;
		white-space: pre-wrap;
		word-break: break-all;
	}

	.log-row:hover {
		background: rgba(255, 255, 255, 0.05);
	}

	.log-time {
		color: var(--color-secondary);
		opacity: 0.7;
		flex-shrink: 0;
		width: 100px;
	}

	.log-priority {
		color: var(--color-text-dim);
		opacity: 0.5;
		flex-shrink: 0;
		width: 30px;
		text-align: center;
	}

	.log-content {
		flex: 1;
	}

	.log-error { border-left-color: #ff3333; color: #ff9999; background: rgba(255, 0, 0, 0.05); }
	.log-warn { border-left-color: var(--color-warning); color: var(--color-warning); }
	.log-ai { border-left-color: var(--color-secondary); color: var(--color-secondary); }
	.log-trust { border-left-color: var(--color-success); color: var(--color-success); }
</style>
