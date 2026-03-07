<script lang="ts">
	import { fade, fly } from 'svelte/transition';
	import { getTosState, bezelCollapse, bezelSwipe, bezelPanePromote } from '$lib/stores/ipc.svelte';

	const state = $derived(getTosState());
	const activeSector = $derived(state.sectors[state.active_sector_index]);
	const activeHub = $derived(activeSector.hubs[activeSector.active_hub_index]);
	
	let swipeDir: 'Left' | 'Right' = 'Right';

	function handleSwipe(dir: 'Left' | 'Right') {
		swipeDir = dir;
		bezelSwipe(dir);
	}
</script>

{#if state.bezel_expanded}
	<div 
		class="expanded-bezel-overlay glass-panel"
		transition:fly={{ y: 300, duration: 500 }}
	>
		<div class="bezel-header">
			<div class="lcars-pill primary">ACTIVE_HUB // {activeHub.mode.toUpperCase()}</div>
			<div class="bezel-controls">
				<button class="lcars-btn-sm" onclick={() => bezelCollapse()}>✕ CLOSE</button>
			</div>
		</div>

		<div class="bezel-main">
			<div class="bezel-left">
				<div class="context-info">
					<div class="label">CWD</div>
					<div class="value">{activeHub.current_directory}</div>
				</div>
				<div class="context-info">
					<div class="label">SECTOR</div>
					<div class="value">{activeSector.name}</div>
				</div>

				<div class="action-chips">
					<div class="chip-group">
						<div class="group-label">SHELL_ACTIONS</div>
						<button class="lcars-btn-sm" onclick={() => bezelPanePromote()}>⊞ PROMOTE_TO_PANE</button>
						<button class="lcars-btn-sm">⏹ STOP (Ctrl+C)</button>
						<button class="lcars-btn-sm">⧉ NEW_TERMINAL</button>
					</div>
				</div>
			</div>

			<div class="bezel-center">
				<div class="output-panel glass-panel">
					<div class="output-header">TERMINAL_OUTPUT // TAIL</div>
					<div class="output-scroll">
						{#each activeHub.terminal_output.slice(-20) as line}
							<div class="term-line">{line.text}</div>
						{/each}
					</div>
				</div>
			</div>

			<div class="bezel-right">
				{#if activeSector.active_apps.length > 0}
					<div class="app-navigator">
						<div class="navigator-label">NAVIGATE_LEVEL_3</div>
						<div class="app-swipe-controls">
							<button class="swipe-btn" onclick={() => handleSwipe('Left')}>◀</button>
							<div class="active-app-badge">
								{activeSector.active_apps[activeSector.active_app_index]?.title || 'NO_APP'}
							</div>
							<button class="swipe-btn" onclick={() => handleSwipe('Right')}>▶</button>
						</div>
						<div class="app-count">{activeSector.active_app_index + 1} / {activeSector.active_apps.length}</div>
					</div>
				{/if}
			</div>
		</div>

		<div class="bezel-prompt-area">
			<div class="prompt-preview">
				<span class="prefix">CMD ></span>
				<span class="input-placeholder">COMMAND_STAGED_OR_IDLE...</span>
			</div>
		</div>
	</div>
{/if}

<style>
	.expanded-bezel-overlay {
		position: absolute;
		left: 0;
		right: 0;
		bottom: 0px;
		height: 40%;
		z-index: 500;
		background: rgba(10, 10, 20, 0.95);
		border-top: 2px solid var(--color-primary);
		display: flex;
		flex-direction: column;
		backdrop-filter: blur(20px);
		box-shadow: 0 -20px 50px rgba(0, 0, 0, 0.5);
	}

	.bezel-header {
		padding: 10px 20px;
		display: flex;
		justify-content: space-between;
		align-items: center;
		background: rgba(255, 255, 255, 0.05);
		border-bottom: 1px solid rgba(255, 255, 255, 0.1);
	}

	.lcars-pill {
		padding: 2px 15px;
		border-radius: var(--radius-pill);
		font-family: var(--font-display);
		font-weight: 700;
		font-size: 0.7rem;
		background: var(--color-primary);
		color: #000;
	}

	.bezel-main {
		flex: 1;
		display: grid;
		grid-template-columns: 300px 1fr 300px;
		padding: 20px;
		gap: 20px;
		overflow: hidden;
	}

	.bezel-left, .bezel-right {
		display: flex;
		flex-direction: column;
		gap: 20px;
	}

	.context-info {
		display: flex;
		flex-direction: column;
	}

	.context-info .label {
		font-family: var(--font-mono);
		font-size: 0.6rem;
		color: var(--color-primary);
		opacity: 0.7;
		letter-spacing: 0.1em;
	}

	.context-info .value {
		font-family: var(--font-mono);
		font-size: 0.8rem;
		color: var(--color-text);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.action-chips {
		margin-top: 10px;
	}

	.chip-group {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.group-label {
		font-family: var(--font-display);
		font-size: 0.65rem;
		font-weight: 700;
		color: var(--color-secondary);
		border-bottom: 1px solid var(--color-secondary);
		margin-bottom: 5px;
	}

	.lcars-btn-sm {
		background: rgba(255, 255, 255, 0.05);
		border: none;
		border-left: 5px solid var(--color-primary);
		color: var(--color-text-dim);
		padding: 5px 10px;
		font-family: var(--font-display);
		font-size: 0.65rem;
		text-align: left;
		cursor: pointer;
		transition: all 0.2s;
	}

	.lcars-btn-sm:hover {
		background: rgba(255, 255, 255, 0.1);
		color: white;
		border-left-width: 10px;
	}

	.output-panel {
		height: 100%;
		display: flex;
		flex-direction: column;
		background: rgba(0, 0, 0, 0.3);
		border: 1px solid rgba(255, 255, 255, 0.05);
	}

	.output-header {
		padding: 5px 10px;
		font-family: var(--font-mono);
		font-size: 0.6rem;
		color: var(--color-success);
		border-bottom: 1px solid rgba(0, 0, 0, 0.5);
	}

	.output-scroll {
		flex: 1;
		padding: 10px;
		font-family: var(--font-mono);
		font-size: 0.75rem;
		overflow-y: auto;
		color: var(--color-success);
	}

	.term-line { opacity: 0.8; }

	.app-navigator {
		background: rgba(255, 255, 255, 0.03);
		padding: 15px;
		border: 1px solid rgba(255, 255, 255, 0.05);
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 15px;
	}

	.navigator-label {
		font-family: var(--font-display);
		font-size: 0.65rem;
		color: var(--color-primary);
		letter-spacing: 0.1em;
	}

	.app-swipe-controls {
		display: flex;
		align-items: center;
		gap: 15px;
		width: 100%;
	}

	.swipe-btn {
		background: transparent;
		border: 1px solid var(--color-primary);
		color: var(--color-primary);
		width: 30px;
		height: 30px;
		cursor: pointer;
		transition: all 0.2s;
	}

	.swipe-btn:hover {
		background: var(--color-primary);
		color: black;
	}

	.active-app-badge {
		flex: 1;
		text-align: center;
		font-family: var(--font-display);
		font-weight: 700;
		font-size: 0.8rem;
		color: white;
	}

	.app-count {
		font-family: var(--font-mono);
		font-size: 0.6rem;
		opacity: 0.5;
	}

	.bezel-prompt-area {
		padding: 15px 40px;
		background: rgba(255, 255, 255, 0.02);
		border-top: 1px solid rgba(255, 255, 255, 0.05);
	}

	.prompt-preview {
		display: flex;
		gap: 15px;
		font-family: var(--font-mono);
		font-size: 1.1rem;
	}

	.prefix { color: var(--color-primary); }
	.input-placeholder { color: var(--color-text-dim); opacity: 0.5; font-style: italic; }
</style>
