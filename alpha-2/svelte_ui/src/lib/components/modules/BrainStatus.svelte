<script lang="ts">
	import { getTosState } from '$lib/stores/ipc.svelte';
	import { getCurrentMode } from '$lib/stores/ui.svelte';

	const tosState = $derived(getTosState());
	const mode = $derived(getCurrentMode());
	const activeSector = $derived(tosState.sectors[tosState.active_sector_index]);
</script>

<div class="brain-status">
	<span class="time-label">BRAIN TIME</span>
	<span class="time-value">{tosState.brain_time || '--:--:--'}</span>
	<span class="status-badge active">{(tosState.sys_status || 'DISCONNECTED').toUpperCase()}</span>
</div>

<style>
	.brain-status {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		animation: fadeIn 0.3s ease;
	}

	.time-label {
		font-size: 0.6rem;
		opacity: 0.5;
		font-family: var(--font-display);
		letter-spacing: 0.08em;
	}

	.time-value {
		font-family: var(--font-mono);
		font-size: 0.85rem;
		color: var(--color-text);
	}

	.status-badge {
		display: inline-flex;
		align-items: center;
		padding: 0.125rem 0.625rem;
		border-radius: var(--radius-pill);
		font-family: var(--font-display);
		font-size: 0.7rem;
		font-weight: 600;
		letter-spacing: 0.06em;
		background: rgba(0, 0, 0, 0.6);
		color: var(--color-success);
		border: 1px solid rgba(102, 204, 102, 0.25);
	}
</style>
