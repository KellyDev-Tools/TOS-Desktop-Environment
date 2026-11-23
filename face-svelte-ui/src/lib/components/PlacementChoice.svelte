<script lang="ts">
	// Placement, prominence `explicit` (Architecture §26.4): a GUI
	// launch from the prompt waits in Brain state until the user
	// says where it runs. Terminal submissions never wait — this
	// card only ever appears for a catalogue app launch.
	import { getTosState, placementChoose } from '$lib/stores/ipc.svelte';

	const tosState = $derived(getTosState());
	const pending = $derived(tosState.placement_pending ?? null);

	async function choose(side: 'host' | 'server') {
		if (pending) await placementChoose(pending.id, side);
	}
</script>

{#if pending}
	<div class="placement-choice" data-testid="placement-choice" role="dialog" aria-modal="false">
		<div class="placement-title">Run {pending.app_name} where?</div>
		<div class="placement-buttons">
			<button
				type="button"
				class="placement-btn"
				data-testid="placement-choose-host"
				disabled={!pending.host_exec}
				title={pending.host_exec ? 'Run it on this machine' : 'Not installed on this machine'}
				onclick={() => choose('host')}
			>
				Here
			</button>
			<button
				type="button"
				class="placement-btn"
				data-testid="placement-choose-server"
				disabled={!pending.server_exec}
				title={pending.server_exec ? 'Run it on the server, shown here' : 'Not installed on the server'}
				onclick={() => choose('server')}
			>
				On server
			</button>
		</div>
	</div>
{/if}

<style>
	.placement-choice {
		position: fixed;
		left: 50%;
		bottom: 12vh;
		transform: translateX(-50%);
		z-index: 60;
		background: var(--hatch-widget-surface, #1c2333);
		color: var(--hatch-widget-text, #eef1f6);
		border: 1px solid var(--hatch-widget-border, #3a4358);
		border-radius: 14px;
		padding: 14px 16px;
		box-shadow: 0 10px 32px rgba(0, 0, 0, 0.45);
		min-width: 280px;
	}
	.placement-title {
		font-size: 15px;
		margin-bottom: 10px;
		text-align: center;
	}
	.placement-buttons {
		display: flex;
		gap: 10px;
	}
	.placement-btn {
		flex: 1;
		min-height: 48px;
		border-radius: 10px;
		border: 1px solid var(--hatch-widget-border, #3a4358);
		background: var(--hatch-widget-surface-muted, #252d42);
		color: inherit;
		font-size: 15px;
		cursor: pointer;
	}
	.placement-btn:disabled {
		opacity: 0.4;
		cursor: default;
	}
</style>
