<script lang="ts">
	import { getTosState } from '$lib/stores/ipc.svelte';

	const state = $derived(getTosState());
	const lastLog = $derived(
		state.system_log && state.system_log.length > 0
			? state.system_log[state.system_log.length - 1]
			: { text: 'SYSTEM READY.', priority: 0 }
	);
	const text = $derived((lastLog.text || '').toUpperCase());
</script>

<div class="mini-log">
	<div class="mini-log-text">{text}</div>
</div>

<style>
	.mini-log {
		padding: 2px var(--space-sm);
		border-left: 2px solid var(--color-accent);
		border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
	}

	.mini-log-text {
		font-size: 0.55rem;
		font-family: var(--font-mono);
		color: var(--color-success);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		max-width: 10rem;
	}
</style>
