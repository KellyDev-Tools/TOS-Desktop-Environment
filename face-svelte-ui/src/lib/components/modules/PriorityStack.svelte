<script lang="ts">
	import { getTosState } from '$lib/stores/ipc.svelte';
	import { fade, slide } from 'svelte/transition';

	const tosState = $derived(getTosState());
	
	// §31.5: Priority-gated notification center (Level 2+)
	const prioritizedLogs = $derived((tosState.system_log || [])
		.filter(log => log.priority >= 2)
		.slice(-8)
		.reverse());

	function getLevelLabel(p: number): string {
		if (p >= 3) return 'CRITICAL';
		if (p === 2) return 'TACTICAL';
		return 'INFO';
	}
</script>

<div class="priority-stack">
	<div class="priority-header">
		<span>NOTIFICATIONS // CENTER</span>
		<span class="active-count">{prioritizedLogs.length}</span>
	</div>
	
	<div class="notification-list">
		{#each prioritizedLogs as log (log.timestamp + log.text)}
			<div 
				class="priority-item" 
				class:high={log.priority >= 3} 
				class:medium={log.priority === 2}
				transition:slide={{ duration: 300 }}
			>
				<div class="priority-indicator">
					<div class="indicator-bar"></div>
					<span class="level-tag">{getLevelLabel(log.priority)}</span>
				</div>
				<div class="priority-body">
					<span class="priority-text">{log.text.toUpperCase()}</span>
				</div>
			</div>
		{/each}
	</div>

	{#if prioritizedLogs.length === 0}
		<div class="priority-empty" in:fade>
			<div class="pulse-ring"></div>
			<span>SCANNING_FOR_ALERTS...</span>
		</div>
	{/if}
</div>

<style>
	.priority-stack {
		padding: var(--space-sm);
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
		height: 100%;
	}

	.priority-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		font-family: var(--font-display);
		font-size: 0.55rem;
		font-weight: 700;
		letter-spacing: 0.1em;
		color: var(--color-text-muted);
		padding-bottom: 5px;
		border-bottom: 1px solid rgba(255, 255, 255, 0.1);
	}

	.active-count {
		background: rgba(255, 255, 255, 0.1);
		padding: 1px 6px;
		border-radius: 3px;
		font-family: var(--font-mono);
	}

	.notification-list {
		display: flex;
		flex-direction: column;
		gap: 6px;
		overflow-y: auto;
		scrollbar-width: none;
	}
	.notification-list::-webkit-scrollbar { display: none; }

	.priority-item {
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding: 8px;
		background: rgba(255, 255, 255, 0.03);
		border-radius: var(--radius-sm);
		border-left: 3px solid var(--color-primary);
		transition: transform 0.2s;
	}

	.priority-item.high {
		border-left-color: var(--color-warning);
		background: rgba(247, 168, 51, 0.05);
	}

	.priority-item.medium {
		border-left-color: var(--color-secondary);
	}

	.priority-indicator {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.level-tag {
		font-family: var(--font-display);
		font-size: 0.5rem;
		font-weight: 800;
		letter-spacing: 0.05em;
		opacity: 0.7;
	}

	.priority-body {
		font-family: var(--font-mono);
		font-size: 0.65rem;
		line-height: 1.3;
		color: var(--color-text-dim);
	}

	.high .priority-body { color: var(--color-warning); }

	.priority-text {
		word-break: break-word;
	}

	.priority-empty {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 10px;
		font-size: 0.6rem;
		opacity: 0.3;
		font-family: var(--font-display);
		font-weight: 700;
	}

	.pulse-ring {
		width: 20px;
		height: 20px;
		border: 1px solid currentColor;
		border-radius: 50%;
		animation: pulse 2s infinite;
	}

	@keyframes pulse {
		0% { transform: scale(0.8); opacity: 0.8; }
		100% { transform: scale(2); opacity: 0; }
	}
</style>
