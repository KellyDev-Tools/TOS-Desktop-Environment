<script lang="ts">
	import { fade, scale } from 'svelte/transition';
	import { getTosState, confirmationAccept, confirmationReject } from '$lib/stores/ipc.svelte';
	import { focusTrap } from '$lib/actions/focusTrap';

	const tosState = $derived(getTosState());
	const req = $derived(tosState.pending_confirmation);

	async function handleAccept() {
		if (req) {
			await confirmationAccept(req.id);
		}
	}

	async function handleReject() {
		if (req) {
			await confirmationReject(req.id);
		}
	}
</script>

{#if req}
	<div class="modal-overlay trust-gate confirmation-overlay" data-testid="trust-confirmation-overlay">
		<div 
			class="confirmation-card glass-panel" 
			role="dialog" 
			aria-modal="true"
			use:focusTrap
			in:scale={{ start: 0.9, duration: 400 }}
		>
			<header class="confirmation-header">
				<div class="alert-icon">⚠</div>
				<div class="header-text">
					<h2>TRUST_CONFIRMATION_REQUIRED</h2>
					<p>Classified Tactical Operation Detected</p>
				</div>
			</header>

			<main class="confirmation-body">
				<div class="message-box">
					<div class="message-label">REASON:</div>
					<div class="message-text">{req.message}</div>
				</div>

				{#if req.progress > 0}
					<div class="progress-container">
						<div class="progress-bar">
							<div class="progress-fill" style="width: {req.progress * 100}%"></div>
						</div>
						<div class="progress-text">{(req.progress * 100).toFixed(0)}% COMPLETE</div>
					</div>
				{/if}

				<div class="risk-warning">
					WARNING: Proceeding may alter system stability or security posture.
					Ensure current request matches intended operation.
				</div>
			</main>

			<footer class="confirmation-footer">
				<button class="lcars-btn secondary" onclick={handleReject}>
					REJECT_OPERATION
				</button>
				<button class="lcars-btn danger" onclick={handleAccept}>
					EXECUTE_AUTHORIZATION
				</button>
			</footer>
		</div>
	</div>
{/if}

<style>
	.modal-overlay {
		position: fixed;
		inset: 0;
		background: rgba(10, 10, 20, 0.9);
		backdrop-filter: blur(12px);
		z-index: var(--z-modal);
		display: flex;
		align-items: center;
		justify-content: center;
		padding: var(--space-xl);
	}

	.confirmation-card {
		width: 100%;
		max-width: 32rem;
		border: 1px solid var(--color-warning);
		display: flex;
		flex-direction: column;
		overflow: hidden;
		box-shadow: 0 0 50px rgba(255, 153, 51, 0.15);
	}

	.confirmation-header {
		background: var(--color-warning);
		color: black;
		padding: var(--space-md) var(--space-xl);
		display: flex;
		align-items: center;
		gap: var(--space-md);
	}

	.alert-icon {
		font-size: 2.5rem;
		font-weight: bold;
		animation: blink 1s infinite;
	}

	.header-text h2 {
		margin: 0;
		font-family: var(--font-display);
		font-size: 1.1rem;
		letter-spacing: 0.1em;
	}

	.header-text p {
		margin: 0;
		font-size: 0.65rem;
		font-family: var(--font-mono);
		opacity: 0.8;
		text-transform: uppercase;
	}

	.confirmation-body {
		padding: var(--space-2xl);
		display: flex;
		flex-direction: column;
		gap: var(--space-xl);
	}

	.message-box {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.message-label {
		font-family: var(--font-mono);
		font-size: 0.65rem;
		color: var(--color-warning);
		opacity: 0.7;
		letter-spacing: 0.1em;
	}

	.message-text {
		font-family: var(--font-display);
		font-size: 1.2rem;
		font-weight: 700;
		color: white;
		line-height: 1.3;
	}

	.progress-container {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.progress-bar {
		height: 8px;
		background: rgba(255, 255, 255, 0.05);
		border-radius: 4px;
		overflow: hidden;
	}

	.progress-fill {
		height: 100%;
		background: var(--color-warning);
		box-shadow: 0 0 10px var(--color-warning);
		transition: width 0.3s ease;
	}

	.progress-text {
		font-family: var(--font-mono);
		font-size: 0.6rem;
		text-align: right;
		color: var(--color-warning);
	}

	.risk-warning {
		font-size: 0.75rem;
		color: var(--color-text-dim);
		line-height: 1.5;
		padding: var(--space-md);
		background: rgba(255, 51, 51, 0.05);
		border-left: 3px solid var(--color-danger);
	}

	.confirmation-footer {
		padding: var(--space-xl);
		background: rgba(255, 255, 255, 0.02);
		border-top: 1px solid rgba(255, 255, 255, 0.05);
		display: flex;
		gap: var(--space-md);
	}

	.lcars-btn {
		flex: 1;
	}

	.lcars-btn.danger {
		background: var(--color-danger);
		color: white !important;
	}

	.lcars-btn.secondary {
		background: rgba(255, 255, 255, 0.1);
		color: white !important;
	}

	@keyframes blink {
		0%, 100% { opacity: 1; }
		50% { opacity: 0.3; }
	}
</style>
