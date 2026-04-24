<script lang="ts">
	import { getTosState, getSyncLatency } from '$lib/stores/ipc.svelte';
	import { getCurrentFps } from '$lib/stores/ui.svelte';

	const tosState = $derived(getTosState());
	const latency = $derived(getSyncLatency());
	const fps = $derived(getCurrentFps());

	// TODO: Pull real telemetry from Brain state when available
	const cpu = '12%';
	const mem = '4.2GB';
	const net = '1.2MB/s';
</script>

<div class="telemetry">
	<span class="telemetry-label">SYSTEM TELEMETRY</span>
	<div class="telemetry-metrics">
		<span class="metric" class:warning={fps < 55}>FPS: {fps}</span>
		<span class="sep">|</span>
		<span class="metric">CPU: {cpu}</span>
		<span class="sep">|</span>
		<span class="metric">MEM: {mem}</span>
		<span class="sep">|</span>
		<span class="metric">NET: {net}</span>
	</div>
</div>

<style>
	.telemetry {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.15rem;
		animation: fadeIn 0.3s ease;
	}

	.telemetry-label {
		font-family: var(--font-display);
		font-size: 0.7rem;
		font-weight: 800;
		letter-spacing: 0.1em;
		color: var(--color-primary);
	}

	.telemetry-metrics {
		display: flex;
		align-items: center;
		gap: var(--space-xs);
		font-size: 0.7rem;
		font-family: var(--font-mono);
		color: var(--color-text-dim);
	}

	.sep {
		opacity: 0.3;
	}

	.warning {
		color: var(--color-warning);
	}
</style>
