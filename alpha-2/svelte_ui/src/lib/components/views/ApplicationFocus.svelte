<script lang="ts">
	import { getTosState } from '$lib/stores/ipc.svelte';

	const state = $derived(getTosState());
	const currentSector = $derived(state.sectors[state.active_sector_index]);
</script>

<div class="app-focus">
	<div class="app-container glass-panel">
		<div class="app-chrome">
			<div class="app-title">MAIN_SURFACE // BINARY_X86_64</div>
			<div class="app-controls">
				<span class="dot"></span>
				<span class="dot"></span>
			</div>
		</div>
		<div class="app-content">
			<div class="terminal-output">
				<span class="text-success">[APPLICATION SURFACE ACTIVE]</span>
				<br /><br />
				<span>$ ./run_sector_agent --mode=autonomous</span>
				<br />
				<span class="text-success">INITIALIZING NEURAL NET... OK</span>
				<br />
				<span class="text-success">MAPPING TOPOLOGICAL SPACE... OK</span>
				<br />
				<span class="cursor-blink">AWAITING OPERATOR INPUT_</span>
			</div>
		</div>
	</div>
</div>

<style>
	.app-focus {
		width: 100%;
		height: 100%;
		padding: var(--space-md);
		animation: scaleIn 0.4s cubic-bezier(0.16, 1, 0.3, 1);
	}

	.app-container {
		height: 100%;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.app-chrome {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: var(--space-sm) var(--space-md);
		background: var(--color-surface-raised);
		border-bottom: 1px solid var(--color-border);
	}

	.app-title {
		font-family: var(--font-display);
		font-size: 0.7rem;
		font-weight: 700;
		letter-spacing: 0.08em;
		color: var(--color-text-dim);
	}

	.app-controls {
		display: flex;
		gap: 6px;
	}

	.dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--color-primary);
		opacity: 0.5;
	}

	.app-content {
		flex: 1;
		padding: var(--space-xl);
		overflow-y: auto;
	}

	.terminal-output {
		font-family: var(--font-mono);
		font-size: 1rem;
		line-height: 1.8;
		color: var(--color-success);
	}

	.cursor-blink {
		animation: blink 1s infinite;
	}
</style>
