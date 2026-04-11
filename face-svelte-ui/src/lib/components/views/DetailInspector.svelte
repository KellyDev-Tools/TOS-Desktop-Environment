<script lang="ts">
	import { getTosState, sendCommand } from '$lib/stores/ipc.svelte';

	const tosState = $derived(getTosState());
	
	let killSwitchConfirm = $state(false);

	async function activateKillSwitch() {
		if (killSwitchConfirm) {
			await sendCommand('tactical_kill_switch:');
			killSwitchConfirm = false;
		} else {
			killSwitchConfirm = true;
			setTimeout(() => { killSwitchConfirm = false; }, 3000);
		}
	}
</script>

<div class="detail-inspector wireframe-mode">
	<header class="inspector-header">
		<h2>TACTICAL RESET // LEVEL 4</h2>
		<div class="warning-banner">SYSTEM DEADLOCK OR HIGH LATENCY DETECTED. DIAGNOSTIC RENDERER ACTIVE.</div>
	</header>

	<div class="wireframe-grid">
		{#each tosState.sectors as sector, i}
			<div class="wireframe-sector" class:frozen={sector.frozen} class:deadlocked={sector.status === 'Deadlocked'}>
				<div class="wf-title">S0{i} // {sector.name}</div>
				
				<div class="wf-metrics">
					<div>HUBS: {sector.hubs.length}</div>
					<div>APPS: {sector.hubs[0]?.activity_listing?.processes?.length || 0}</div>
					<div>STATE: {sector.frozen ? 'FROZEN' : (sector.status === 'Deadlocked' ? 'DEADLOCKED' : 'NOMINAL')}</div>
				</div>
			</div>
		{/each}
	</div>

	<footer class="inspector-footer">
		<div class="interlock-status">PROMPT INTERLOCK: ENGAGED [Expanded Bezel Disabled]</div>
		<button class="kill-switch" class:confirming={killSwitchConfirm} onclick={activateKillSwitch}>
			{killSwitchConfirm ? 'SYSTEM RE-AUTH REQUIRED — CLICK AGAIN TO EXECUTE' : '[[ GLOBAL PROCESS KILL-SWITCH ]]'}
		</button>
	</footer>
</div>

<style>
	.detail-inspector.wireframe-mode {
		height: 100%;
		display: flex;
		flex-direction: column;
		background: #000;
		color: var(--color-success);
		font-family: var(--font-mono);
		padding: var(--space-xl);
		background-image: 
			linear-gradient(rgba(102, 204, 102, 0.1) 1px, transparent 1px),
			linear-gradient(90deg, rgba(102, 204, 102, 0.1) 1px, transparent 1px);
		background-size: 50px 50px;
		animation: pulse 4s infinite;
	}

	.inspector-header h2 {
		margin: 0;
		color: var(--color-warning);
		font-size: 1.5rem;
		text-shadow: 0 0 10px var(--color-warning);
	}

	.warning-banner {
		background: var(--color-warning);
		color: #000;
		padding: var(--space-sm) var(--space-md);
		font-weight: 700;
		margin-top: var(--space-md);
		border-radius: 2px;
	}

	.wireframe-grid {
		flex: 1;
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
		gap: var(--space-lg);
		margin-top: var(--space-xl);
		align-content: flex-start;
	}

	.wireframe-sector {
		border: 1px solid var(--color-success);
		padding: var(--space-md);
		background: rgba(0, 0, 0, 0.8);
		box-shadow: inset 0 0 15px rgba(102, 204, 102, 0.2);
	}

	.wireframe-sector.frozen {
		border-color: var(--color-primary);
		color: var(--color-primary);
	}

	.wireframe-sector.deadlocked {
		border-color: #ff3333;
		color: #ff3333;
		box-shadow: inset 0 0 20px rgba(255, 51, 51, 0.5);
		animation: blink 0.5s infinite;
	}

	.wf-title {
		font-weight: 700;
		border-bottom: 1px solid currentColor;
		padding-bottom: var(--space-xs);
		margin-bottom: var(--space-sm);
	}

	.inspector-footer {
		margin-top: auto;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-md);
	}

	.interlock-status {
		color: var(--color-warning);
		font-size: 0.85rem;
		letter-spacing: 0.1em;
	}

	.kill-switch {
		background: transparent;
		border: 2px solid #ff3333;
		color: #ff3333;
		padding: var(--space-md) var(--space-xl);
		font-family: var(--font-display);
		font-weight: 900;
		font-size: 1.2rem;
		cursor: pointer;
		border-radius: var(--radius-sm);
		transition: all 0.2s;
		box-shadow: 0 0 20px rgba(255, 51, 51, 0.2);
	}

	.kill-switch:hover {
		background: rgba(255, 51, 51, 0.1);
		box-shadow: 0 0 30px rgba(255, 51, 51, 0.4);
	}

	.kill-switch.confirming {
		background: #ff3333;
		color: #000;
		animation: shake 0.2s infinite;
	}

	@keyframes shake {
		0%, 100% { transform: translateX(0); }
		25% { transform: translateX(-5px); }
		75% { transform: translateX(5px); }
	}

	@keyframes pulse {
		0%, 100% { opacity: 1; }
		50% { opacity: 0.8; }
	}
</style>
