<script lang="ts">
	import { getConnectionState } from '$lib/stores/ipc.svelte';

	const connState = $derived(getConnectionState());

	const messages: Record<string, { title: string; sub: string; color: string }> = {
		disconnected: {
			title: 'BRAIN LINK OFFLINE',
			sub: 'Attempting to establish connection to Brain Core...',
			color: 'var(--color-warning)'
		},
		connecting: {
			title: 'SYNCHRONIZING',
			sub: 'Establishing WebSocket bridge to Brain Core...',
			color: 'var(--color-accent)'
		}
	};

	const msg = $derived(messages[connState] || messages.disconnected);
</script>

{#if connState !== 'connected'}
	<div class="disconnect-overlay">
		<div class="disconnect-card glass-panel">
			<div class="pulse-ring" style="--ring-color: {msg.color}"></div>
			<div class="disconnect-icon" style="color: {msg.color}">⬡</div>
			<div class="disconnect-title" style="color: {msg.color}">{msg.title}</div>
			<div class="disconnect-sub">{msg.sub}</div>
			<div class="disconnect-dots">
				<span class="dot-anim"></span>
				<span class="dot-anim"></span>
				<span class="dot-anim"></span>
			</div>
		</div>
	</div>
{/if}

<style>
	.disconnect-overlay {
		position: absolute;
		inset: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		background: rgba(10, 10, 20, 0.7);
		backdrop-filter: blur(6px);
		z-index: var(--z-overlay);
		animation: fadeIn 0.5s ease;
	}

	.disconnect-card {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-md);
		padding: var(--space-2xl) var(--space-xl);
		text-align: center;
		position: relative;
		max-width: 22rem;
	}

	.pulse-ring {
		position: absolute;
		width: 6rem;
		height: 6rem;
		top: 1.5rem;
		border-radius: 50%;
		border: 2px solid var(--ring-color);
		opacity: 0.2;
		animation: pulse 2s ease-in-out infinite;
	}

	.disconnect-icon {
		font-size: 2.5rem;
		animation: float 3s ease-in-out infinite alternate;
	}

	.disconnect-title {
		font-family: var(--font-display);
		font-size: 1.1rem;
		font-weight: 700;
		letter-spacing: 0.15em;
	}

	.disconnect-sub {
		font-size: 0.8rem;
		color: var(--color-text-dim);
		line-height: 1.5;
	}

	.disconnect-dots {
		display: flex;
		gap: 6px;
	}

	.dot-anim {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--color-text-muted);
		animation: pulse 1.5s ease-in-out infinite;
	}

	.dot-anim:nth-child(2) {
		animation-delay: 0.3s;
	}

	.dot-anim:nth-child(3) {
		animation-delay: 0.6s;
	}
</style>
