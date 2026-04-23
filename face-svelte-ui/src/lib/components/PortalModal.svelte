<script lang="ts">
	import { isPortalModalOpen, closePortalModal } from '$lib/stores/ui.svelte';
	import { createPortal, revokePortal } from '$lib/stores/ipc.svelte';
	import { focusTrap } from '$lib/actions/focusTrap';

	const open = $derived(isPortalModalOpen());

	let portalUrl = $state('https://tos.live/portal/...');
	let portalToken = $state('');
	let generating = $state(false);
	let copied = $state(false);

	$effect(() => {
		if (open) {
			generateLink();
		}
	});

	async function generateLink() {
		generating = true;
		copied = false;
		const response = await createPortal();
		if (response) {
			try {
				const data = JSON.parse(response);
				portalUrl = data.url || `https://tos.live/portal/${data.token || 'demo'}`;
				portalToken = data.token || '';
			} catch {
				// Fallback for non-JSON response
				portalToken = response.trim();
				portalUrl = `https://tos.live/portal/${portalToken}`;
			}
		} else {
			portalUrl = `https://tos.live/portal/${crypto.randomUUID().substring(0, 12)}`;
			portalToken = portalUrl.split('/').pop() || '';
		}
		generating = false;
	}

	async function handleCopy() {
		try {
			await navigator.clipboard.writeText(portalUrl);
			copied = true;
			setTimeout(() => { copied = false; }, 2000);
		} catch {
			// Fallback: select the input
			const input = document.getElementById('portal-link-input') as HTMLInputElement;
			input?.select();
		}
	}

	async function handleRevoke() {
		if (portalToken) {
			await revokePortal(portalToken);
		}
		closePortalModal();
	}

	function handleOverlayClick(e: MouseEvent) {
		if ((e.target as HTMLElement).classList.contains('modal-overlay')) {
			closePortalModal();
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape' && open) closePortalModal();
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="modal-overlay" onclick={handleOverlayClick} role="button" tabindex="0">
		<div 
			class="portal-container glass-panel" 
			role="dialog" 
			aria-modal="true" 
			tabindex="-1" 
			onclick={(e) => e.stopPropagation()}
			use:focusTrap
		>
			<!-- Header -->
			<div class="portal-header">
				<div class="portal-elbow"></div>
				<div class="portal-title">WEB PORTAL // SECURE LINK</div>
				<button class="bezel-btn" onclick={() => closePortalModal()}>✕</button>
			</div>

			<!-- Content -->
			<div class="portal-content">
				{#if generating}
					<div class="portal-generating">
						<div class="pulse-dot"></div>
						<span>GENERATING SECURE ACCESS TOKEN...</span>
					</div>
				{:else}
					<div class="portal-status">PORTAL LINK READY</div>
					<div class="portal-link-box">
						<input
							type="text"
							id="portal-link-input"
							class="portal-input"
							readonly
							value={portalUrl}
						/>
					</div>
					<div class="portal-expiry">
						THIS LINK EXPIRES IN 15 MINUTES OR AFTER ONE SUCCESSFUL HANDSHAKE.
					</div>
					<div class="portal-actions">
						<button class="lcars-btn" onclick={handleCopy}>
							{copied ? '✓ COPIED' : 'COPY LINK'}
						</button>
						<button class="lcars-btn warning" onclick={handleRevoke}>
							REVOKE
						</button>
					</div>
				{/if}
			</div>

			<!-- Footer -->
			<div class="portal-footer">
				<div class="portal-elbow-bottom"></div>
			</div>
		</div>
	</div>
{/if}

<style>
	.modal-overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.6);
		backdrop-filter: blur(4px);
		z-index: var(--z-modal);
		display: flex;
		align-items: center;
		justify-content: center;
		animation: fadeIn 0.2s ease;
	}

	.portal-container {
		width: 32rem;
		max-width: 90vw;
		display: flex;
		flex-direction: column;
		animation: scaleIn 0.3s cubic-bezier(0.16, 1, 0.3, 1);
		overflow: hidden;
	}

	.portal-header {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-sm) var(--space-md);
		background: var(--color-accent);
	}

	.portal-elbow {
		width: 2rem;
		height: 1.5rem;
		background: var(--color-accent);
		border-bottom-right-radius: var(--radius-elbow);
		flex-shrink: 0;
		opacity: 0.7;
	}

	.portal-title {
		flex: 1;
		font-family: var(--font-display);
		font-size: 0.8rem;
		font-weight: 700;
		letter-spacing: 0.1em;
		color: #fff;
	}

	.portal-header .bezel-btn {
		color: #fff;
		border-color: rgba(255, 255, 255, 0.3);
	}

	.portal-content {
		padding: var(--space-xl);
		display: flex;
		flex-direction: column;
		align-items: center;
		text-align: center;
		gap: var(--space-md);
	}

	.portal-generating {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		font-family: var(--font-display);
		font-size: 0.8rem;
		font-weight: 700;
		color: var(--color-primary);
		letter-spacing: 0.08em;
	}

	.pulse-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--color-primary);
		animation: pulse 1.5s ease-in-out infinite;
	}

	.portal-status {
		font-family: var(--font-display);
		font-size: 0.75rem;
		font-weight: 700;
		letter-spacing: 0.12em;
		color: var(--color-success);
	}

	.portal-link-box {
		width: 100%;
	}

	.portal-input {
		width: 100%;
		text-align: center;
		font-family: var(--font-mono);
		font-size: 1rem;
		padding: 0.75rem;
		background: var(--color-surface-raised);
		border: 1px solid var(--color-accent);
		border-radius: var(--radius-md);
		color: var(--color-accent);
		outline: none;
		cursor: text;
	}

	.portal-expiry {
		font-size: 0.65rem;
		color: var(--color-text-muted);
		line-height: 1.4;
	}

	.portal-actions {
		display: flex;
		gap: var(--space-md);
		margin-top: var(--space-sm);
	}

	.portal-footer {
		padding: var(--space-sm) var(--space-md);
		border-top: 1px solid var(--color-border);
		background: var(--color-surface-raised);
	}

	.portal-elbow-bottom {
		width: 2rem;
		height: 1rem;
		background: var(--color-secondary);
		border-top-left-radius: var(--radius-elbow);
		margin-left: auto;
	}
</style>
