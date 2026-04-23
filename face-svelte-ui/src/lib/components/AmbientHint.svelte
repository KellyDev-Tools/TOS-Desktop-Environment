<script lang="ts">
	import { fade, fly } from 'svelte/transition';
	import { onboardingHintDismiss, getTosState } from '$lib/stores/ipc.svelte';

	let { 
		id, 
		text, 
		targetSelector = '', 
		onDismiss = () => {} 
	}: { 
		id: string; 
		text: string; 
		targetSelector?: string;
		onDismiss?: () => void;
	} = $props();

	const tosState = $derived(getTosState());
	const isDismissed = $derived(tosState.settings.global[`tos.hint.dismissed.${id}`] === 'true');
	const isSuppressed = $derived(tosState.settings.global['tos.hint.suppressed'] === 'true');

	let visible = $state(false);
	let targetRect = $state<DOMRect | null>(null);

	$effect(() => {
		if (!isDismissed && !isSuppressed) {
			// Small delay before showing
			const timer = setTimeout(() => {
				visible = true;
				if (targetSelector) {
					const el = document.querySelector(targetSelector);
					if (el) targetRect = el.getBoundingClientRect();
				}
			}, 1000);
			return () => clearTimeout(timer);
		}
	});

	function dismiss() {
		visible = false;
		onboardingHintDismiss(id);
		onDismiss();
	}

	function getPosition() {
		if (!targetRect) return 'bottom: 100px; right: 40px;';
		return `top: ${targetRect.bottom + 10}px; left: ${targetRect.left}px;`;
	}
</script>

{#if visible && !isDismissed && !isSuppressed}
	<div 
		class="ambient-hint glass-panel"
		style={getPosition()}
		transition:fade={{ duration: 300 }}
	>
		<div class="hint-icon">💡</div>
		<div class="hint-content">
			<div class="hint-label">SYSTEM_HINT</div>
			<div class="hint-text">{text}</div>
		</div>
		<button class="hint-dismiss" onclick={dismiss}>✕</button>
	</div>
{/if}

<style>
	.ambient-hint {
		position: fixed;
		z-index: 1000;
		padding: 10px 15px;
		display: flex;
		align-items: center;
		gap: 12px;
		border-left: 4px solid var(--color-primary);
		max-width: 300px;
		box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
		pointer-events: all;
		background: rgba(10, 10, 20, 0.9);
		backdrop-filter: blur(10px);
	}

	.hint-icon {
		font-size: 1.2rem;
		filter: drop-shadow(0 0 5px var(--color-primary));
	}

	.hint-content {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.hint-label {
		font-family: var(--font-mono);
		font-size: 0.6rem;
		color: var(--color-primary);
		letter-spacing: 0.1em;
		opacity: 0.8;
	}

	.hint-text {
		font-size: 0.75rem;
		line-height: 1.4;
		color: var(--color-text);
	}

	.hint-dismiss {
		background: transparent;
		border: none;
		color: var(--color-text-dim);
		cursor: pointer;
		padding: 5px;
		font-size: 0.8rem;
		transition: color 0.2s;
	}

	.hint-dismiss:hover {
		color: white;
	}
</style>
