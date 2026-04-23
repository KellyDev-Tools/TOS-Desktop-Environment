<script lang="ts">
	import { getConnectionState, connect, getActiveWsUrl } from '$lib/stores/ipc.svelte';
	import { focusTrap } from '$lib/actions/focusTrap';

	const connState = $derived(getConnectionState());

	const messages: Record<string, { title: string; sub: string; color: string }> = {
		disconnected: {
			title: 'BRAIN LINK OFFLINE',
			sub: 'Select a known Brain or enter connection details manually.',
			color: 'var(--color-warning)'
		},
		connecting: {
			title: 'SYNCHRONIZING',
			sub: 'Establishing WebSocket bridge to Brain Core...',
			color: 'var(--color-accent)'
		}
	};

	const msg = $derived(messages[connState] || messages.disconnected);

	// Form state
	let hostInput = $state('127.0.0.1');
	let portInput = $state('7001');

	function handleConnect() {
		if (!hostInput || !portInput) return;
		const uri = `ws://${hostInput}:${portInput}`;
		connect(uri);
	}

	// Load recent from store URL broadly
	$effect(() => {
		if (connState === 'disconnected') {
			const current = getActiveWsUrl();
			if (current) {
				try {
					const url = new URL(current);
					hostInput = url.hostname;
					portInput = url.port;
				} catch {
					// ignore invalid
				}
			}
		}
	});
</script>

{#if connState !== 'connected'}
	<div class="disconnect-overlay">
		<div class="disconnect-card glass-panel" use:focusTrap>
			<div class="pulse-ring" style="--ring-color: {msg.color}"></div>
			<div class="disconnect-icon" style="color: {msg.color}">⬡</div>
			<div class="disconnect-title" style="color: {msg.color}">{msg.title}</div>
			<div class="disconnect-sub">{msg.sub}</div>

			{#if connState === 'connecting'}
				<div class="disconnect-dots">
					<span class="dot-anim"></span>
					<span class="dot-anim"></span>
					<span class="dot-anim"></span>
				</div>
			{:else}
				<!-- Manual Entry Form -->
				<div class="connection-form">
					<div class="input-group">
						<label for="host">Host</label>
						<input
							id="host"
							type="text"
							bind:value={hostInput}
							placeholder="127.0.0.1"
							autocomplete="off"
							spellcheck="false"
						/>
					</div>
					<div class="input-group">
						<label for="port">Port</label>
						<input
							id="port"
							type="text"
							bind:value={portInput}
							placeholder="7001"
							autocomplete="off"
							spellcheck="false"
						/>
					</div>
					<button class="connect-btn" onclick={handleConnect}>CONNECT</button>
					<div class="hint-text">
						<span class="muted">mDNS Scanning</span> restricted in web environment.
					</div>
				</div>
			{/if}
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
		background: rgba(10, 10, 20, 0.85);
		backdrop-filter: blur(8px);
		z-index: var(--z-overlay);
	}

	.disconnect-card {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-md);
		padding: var(--space-2xl) var(--space-xl);
		text-align: center;
		position: relative;
		width: 100%;
		max-width: 24rem;
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

	.connection-form {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
		width: 100%;
		margin-top: var(--space-md);
		text-align: left;
	}

	.input-group {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.input-group label {
		font-size: 0.75rem;
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 1px;
	}

	.input-group input {
		background: rgba(0, 0, 0, 0.4);
		border: 1px solid var(--color-border);
		color: var(--color-text);
		padding: 0.6rem;
		border-radius: var(--radius-sm);
		font-family: var(--font-mono);
		transition: border-color 0.2s ease;
		font-size: 0.9rem;
	}

	.input-group input:focus {
		outline: none;
		border-color: var(--color-accent);
	}

	.connect-btn {
		background: var(--color-accent);
		color: var(--color-bg);
		border: none;
		padding: 0.75rem;
		border-radius: var(--radius-sm);
		font-weight: bold;
		font-family: var(--font-display);
		letter-spacing: 1px;
		cursor: pointer;
		transition: opacity 0.2s ease, transform 0.1s ease;
		margin-top: var(--space-sm);
	}

	.connect-btn:hover {
		opacity: 0.9;
	}

	.connect-btn:active {
		transform: scale(0.98);
	}

	.hint-text {
		font-size: 0.75rem;
		text-align: center;
		color: var(--color-text-dim);
		margin-top: var(--space-sm);
	}

	.hint-text .muted {
		text-decoration: line-through;
		opacity: 0.5;
	}
</style>
