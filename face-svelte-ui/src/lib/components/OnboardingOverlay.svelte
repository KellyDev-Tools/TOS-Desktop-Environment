<script lang="ts">
	import { fade, fly, scale } from 'svelte/transition';
	import { 
		getTosState, setSetting, 
		onboardingSkipTour, onboardingAdvanceStep 
	} from '$lib/stores/ipc.svelte';

	const tosState = $derived(getTosState());
	const isFirstRun = $derived(tosState.settings.global['tos.onboarding.first_run_complete'] !== 'true');

	let currentStep = $state(0);
	let visible = $state(true);

	const steps = [
		{
			title: 'TRUST CONFIGURATION',
			desc: 'Alpha-2.2 introduces non-blocking Command Trust. Choose your default system security posture before we begin.',
			action: 'Step 0 of 8'
		},
		{
			title: 'WELCOME TO TOS',
			desc: 'You are now operating within the Tactical Operating System. This is a level-based workspace designed for extreme multitasking and AI collaboration.',
			action: 'Level 1: Global Overview'
		},
		{
			title: 'HIERARCHY NAVIGATION',
			desc: 'Use the sidebar or Ctrl+1-4 to jump between levels. Level 1 is your cockpit; Level 2 is your engine room; Level 3 is your focal point.',
			action: 'Level 2: Hub Logic'
		},
		{
			title: 'MULTIPLE SECTORS',
			desc: 'TOS can run dozens of isolated sectors simultaneously. Each sector has its own terminal history and application tosState.',
			action: 'Sector Management'
		},
		{
			title: 'AI CO-PILOT',
			desc: 'The AI mode (Ctrl+5) provides real-time analysis. Built-in behaviors like Passive Observer watch for errors and stage fixes automatically.',
			action: 'Intelligence Layer'
		},
		{
			title: 'SPLIT VIEWPORTS',
			desc: 'Tiling is native. Use Ctrl+\\ to split any hub into recursive panes for terminals or applications.',
			action: 'Spatial Layout'
		},
		{
			title: 'THE MARKETPLACE',
			desc: 'Expand your system via the Marketplace button. Install new Terminal output modules, AI behaviors, and application interfaces.',
			action: 'Ecosystem Access'
		},
		{
			title: 'SYSTEM COMPLETE',
			desc: 'Interface calibrated. You are now cleared for standard operations. Access help [?] in the top bezel at any time.',
			action: 'Operational Ready'
		}
	];

	function nextStep() {
		if (currentStep < steps.length - 1) {
			currentStep++;
			onboardingAdvanceStep(currentStep);
		} else {
			finish();
		}
	}

	function prevStep() {
		if (currentStep > 0) {
			currentStep--;
			onboardingAdvanceStep(currentStep);
		}
	}

	function finish() {
		visible = false;
		setSetting('tos.onboarding.first_run_complete', 'true');
		setSetting('tos.onboarding.wizard_complete', 'true');
		onboardingSkipTour();
	}

	function setTrust(key: string, value: string) {
		setSetting(key, value);
	}
</script>

{#if visible && isFirstRun}
	<div class="onboarding-modal-overlay" transition:fade={{ duration: 400 }}>
		<div class="onboarding-card glass-panel" in:scale={{ duration: 600, start: 0.9 }}>
			<div class="card-bezel top">
				<div class="lcars-pill-info">{steps[currentStep].action}</div>
				<div class="lcars-title">TOS // ONBOARDING_SEQUENCER</div>
			</div>

			<div class="card-content">
				{#key currentStep}
					<div class="step-animation-wrap" in:fly={{ x: 20, duration: 400 }} out:fade={{ duration: 100 }}>
						<h1 class="step-title">{steps[currentStep].title}</h1>
						<p class="step-desc">{steps[currentStep].desc}</p>

						{#if currentStep === 0}
							<!-- Trust Config Step -->
							<div class="trust-options">
								<div class="trust-group">
									<div class="trust-label">PRIVILEGE ESCALATION (sudo/su)</div>
									<div class="trust-btns">
										<button 
											class="lcars-btn-sm" 
											class:active={tosState.settings.global['tos.trust.privilege_escalation'] === 'warn'}
											onclick={() => setTrust('tos.trust.privilege_escalation', 'warn')}
										>WARN</button>
										<button 
											class="lcars-btn-sm" 
											class:active={tosState.settings.global['tos.trust.privilege_escalation'] === 'allow'}
											onclick={() => setTrust('tos.trust.privilege_escalation', 'allow')}
										>ALLOW</button>
										<button 
											class="lcars-btn-sm" 
											class:active={tosState.settings.global['tos.trust.privilege_escalation'] === 'block'}
											onclick={() => setTrust('tos.trust.privilege_escalation', 'block')}
										>BLOCK</button>
									</div>
								</div>
								<div class="trust-group">
									<div class="trust-label">RECURSIVE DESTRUCTIVE (rm -rf)</div>
									<div class="trust-btns">
										<button 
											class="lcars-btn-sm" 
											class:active={tosState.settings.global['tos.trust.recursive_bulk'] === 'warn'}
											onclick={() => setTrust('tos.trust.recursive_bulk', 'warn')}
										>WARN</button>
										<button 
											class="lcars-btn-sm" 
											class:active={tosState.settings.global['tos.trust.recursive_bulk'] === 'allow'}
											onclick={() => setTrust('tos.trust.recursive_bulk', 'allow')}
										>ALLOW</button>
										<button 
											class="lcars-btn-sm" 
											class:active={tosState.settings.global['tos.trust.recursive_bulk'] === 'block'}
											onclick={() => setTrust('tos.trust.recursive_bulk', 'block')}
										>BLOCK</button>
									</div>
								</div>
							</div>
						{/if}

						{#if currentStep === 1}
							<div class="onboarding-visualization level-1-viz">
								<div class="viz-sector-grid">
									{#each Array(4) as _}
										<div class="viz-tile"></div>
									{/each}
								</div>
							</div>
						{/if}
						
						{#if currentStep === 5}
							<div class="onboarding-visualization split-viz">
								<div class="viz-split">
									<div class="viz-pane"></div>
									<div class="viz-pane sub-split">
										<div></div>
										<div></div>
									</div>
								</div>
							</div>
						{/if}
					</div>
				{/key}
			</div>

			<div class="card-footer card-bezel bottom">
				<div class="footer-left">
					<button class="lcars-btn-sm subtle" onclick={finish}>SKIP TOUR</button>
				</div>
				<div class="footer-right">
					{#if currentStep > 0}
						<button class="lcars-btn-sm" onclick={prevStep}>BACK</button>
					{/if}
					
					{#if currentStep < steps.length - 1}
						<button class="lcars-btn-sm primary active" onclick={nextStep}>
							{currentStep === 0 ? 'START TOUR' : 'NEXT'}
						</button>
					{:else}
						<button class="lcars-btn-sm primary active" onclick={finish}>FINISH</button>
					{/if}
				</div>
			</div>
		</div>
	</div>
{/if}

<style>
	.onboarding-modal-overlay {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		z-index: 9999;
		background: radial-gradient(circle at center, rgba(0, 0, 0, 0.4) 0%, rgba(0, 0, 0, 0.8) 100%);
		display: flex;
		align-items: center;
		justify-content: center;
		backdrop-filter: blur(10px);
	}

	.onboarding-card {
		width: 600px;
		min-height: 400px;
		max-height: 90vh;
		background: rgba(10, 10, 15, 0.8);
		border-radius: 4px;
		border: 1px solid var(--color-border);
		display: flex;
		flex-direction: column;
		overflow: hidden;
		box-shadow: 0 0 50px rgba(0, 0, 0, 0.5), inset 0 0 20px rgba(247, 168, 51, 0.05);
	}

	.card-bezel {
		padding: 10px 20px;
		display: flex;
		align-items: center;
		gap: 20px;
		background: rgba(255, 255, 255, 0.03);
		flex-shrink: 0;
	}

	.card-bezel.top {
		border-bottom: 2px solid var(--color-primary);
	}

	.card-bezel.bottom {
		border-top: 2px solid var(--color-primary);
		justify-content: space-between;
		padding: 15px 20px;
	}

	.lcars-pill-info {
		background: var(--color-primary);
		color: black;
		padding: 2px 12px;
		border-radius: 10px 0 0 10px;
		font-family: var(--font-display);
		font-weight: 700;
		font-size: 0.7rem;
	}

	.lcars-title {
		font-family: var(--font-display);
		font-weight: 700;
		letter-spacing: 0.1em;
		font-size: 0.8rem;
		opacity: 0.7;
	}

	.card-content {
		flex: 1;
		padding: 40px;
		position: relative;
		min-height: 0;
		overflow-y: auto;
	}

	.step-title {
		font-family: var(--font-display);
		font-weight: 700;
		font-size: 1.8rem;
		margin: 0 0 20px 0;
		background: linear-gradient(135deg, #fff 0%, var(--color-primary) 100%);
		-webkit-background-clip: text;
		background-clip: text;
		-webkit-text-fill-color: transparent;
		letter-spacing: 0.05em;
	}

	.step-desc {
		font-size: 1rem;
		line-height: 1.6;
		color: var(--color-text-dim);
		margin-bottom: 30px;
	}

	.trust-options {
		display: flex;
		flex-direction: column;
		gap: 20px;
		margin-top: 20px;
	}

	.trust-group {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.trust-label {
		font-family: var(--font-mono);
		font-size: 0.7rem;
		color: var(--color-primary);
		letter-spacing: 0.05em;
	}

	.trust-btns {
		display: flex;
		gap: 10px;
	}

	.lcars-btn-sm {
		flex: 1;
		background: rgba(255, 255, 255, 0.05);
		border: none;
		color: var(--color-text-dim);
		font-family: var(--font-display);
		font-weight: 600;
		padding: 8px;
		cursor: pointer;
		transition: all 0.2s;
	}

	.lcars-btn-sm:hover { background: rgba(255, 255, 255, 0.1); color: white; }
	.lcars-btn-sm.active { background: var(--color-primary); color: black; }
	.lcars-btn-sm.subtle { flex: 0; background: transparent; color: var(--color-text); opacity: 0.7; font-size: 0.7rem; }
	.lcars-btn-sm.primary { border-left: 10px solid var(--color-primary); }

	.footer-right { display: flex; gap: 10px; align-items: center; }

	.onboarding-visualization {
		margin-top: 30px;
		height: 80px;
		border-radius: 4px;
		background: rgba(0, 0, 0, 0.3);
		border: 1px dashed rgba(255, 255, 255, 0.1);
	}

	.viz-sector-grid {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 10px;
		padding: 15px;
		height: 100%;
	}

	.viz-tile {
		background: rgba(76, 175, 80, 0.1);
		border: 1px solid rgba(76, 175, 80, 0.3);
		border-radius: 2px;
	}

	.viz-split {
		display: flex;
		height: 100%;
		gap: 5px;
		padding: 10px;
		box-sizing: border-box;
	}

	.viz-pane {
		flex: 1;
		background: rgba(247, 168, 51, 0.1);
		border: 1px solid rgba(247, 168, 51, 0.3);
		box-sizing: border-box;
		min-height: 0;
	}

	.sub-split {
		display: flex;
		flex-direction: column;
		border: none;
		background: transparent;
		gap: 5px;
		padding: 0;
		height: 100%;
	}

	.sub-split div {
		flex: 1;
		background: rgba(247, 168, 51, 0.1);
		border: 1px solid rgba(247, 168, 51, 0.3);
		box-sizing: border-box;
		min-height: 0;
	}
</style>
