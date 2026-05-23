<!--
    TOS Cinematic Intro Archive
    Originally used for dramatic LCARS-style boot sequence on first launch.
    Archived to docs/archive/cinematic_intro.svelte on 2026-05-23.
-->

<script lang="ts">
	import { getTosState } from '$lib/stores/ipc.svelte';
	import { scale, fade } from 'svelte/transition';

	const tosState = getTosState();
	let cinematicActive = $state(true);
	let cinematicStage = $state<'sweep' | 'logs' | 'zoom'>('sweep');

	function startCinematic() {
		cinematicActive = true;
		cinematicStage = 'sweep';
		setTimeout(() => { if (cinematicActive) cinematicStage = 'logs'; }, 4000);
		setTimeout(() => { if (cinematicActive) cinematicStage = 'zoom'; }, 8000);
		setTimeout(() => { if (cinematicActive) skipCinematic(); }, 12000);
	}

	function skipCinematic() {
		cinematicActive = false;
	}
</script>

{#if cinematicActive}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div 
		class="cinematic-overlay {cinematicStage}" 
		transition:fade={{ duration: 1000 }}
		role="button" tabindex="0" onclick={skipCinematic}
	>
		{#if cinematicStage === 'sweep'}
			<div class="sweep-grid"></div>
			<div class="intro-title" in:scale={{ duration: 2000 }}>TOS // TACTICAL_OPERATING_SYSTEM</div>
		{:else if cinematicStage === 'logs'}
			<div class="boot-logs">
				{#each tosState.system_log.slice(-30) as log}
					<div class="boot-line">{log.text}</div>
				{/each}
			</div>
		{:else if cinematicStage === 'zoom'}
			<div class="zoom-effect"></div>
		{/if}
		
		<div class="skip-hint">Press any key to skip</div>
	</div>
{/if}

<style>
	/* Cinematic Overlay & Boot Stages */
	.cinematic-overlay {
		position: absolute;
		top: 0;
		left: 0;
		width: 100%;
		height: 100%;
		z-index: 9999;
		background: #000;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		font-family: var(--font-mono);
		overflow: hidden;
	}

	.intro-title {
		font-size: 2.5rem;
		font-family: var(--font-display);
		font-weight: 900;
		color: var(--color-primary);
		text-shadow: 0 0 20px var(--color-primary);
		letter-spacing: 0.15em;
		text-align: center;
	}

	.sweep-grid {
		position: absolute;
		width: 200%;
		height: 200%;
		background: linear-gradient(rgba(0, 229, 255, 0.03) 1px, transparent 1px),
		            linear-gradient(90deg, rgba(0, 229, 255, 0.03) 1px, transparent 1px);
		background-size: 40px 40px;
		transform: rotateX(60deg) translateY(-50%) rotate(0deg);
		animation: gridMove 20s linear infinite;
	}

	@keyframes gridMove {
		from { transform: rotateX(60deg) translateY(-50%) rotate(0deg); }
		to { transform: rotateX(60deg) translateY(50%) rotate(360deg); }
	}

	.boot-logs {
		width: 80%;
		max-width: 800px;
		height: 70%;
		display: flex;
		flex-direction: column;
		gap: 4px;
		color: var(--color-success);
		font-size: 0.75rem;
		overflow-y: auto;
	}

	.boot-line {
		border-left: 2px solid var(--color-success);
		padding-left: 8px;
		animation: typewriter 0.2s steps(40, end);
	}

	.zoom-effect {
		position: absolute;
		width: 100%;
		height: 100%;
		background: radial-gradient(circle, transparent 20%, #000 80%),
		            repeating-conic-gradient(from 0deg, rgba(0,229,255,0.05) 0deg 30deg, transparent 30deg 60deg);
		animation: spin 5s linear infinite;
	}

	@keyframes spin {
		100% { transform: rotate(360deg); }
	}

	.skip-hint {
		position: absolute;
		bottom: 30px;
		font-size: 0.75rem;
		opacity: 0.5;
		color: #fff;
		animation: pulse 1s infinite alternate;
	}
</style>
