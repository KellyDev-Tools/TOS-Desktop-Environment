<script lang="ts">
	import { getTosState } from '$lib/stores/ipc.svelte';
	import { slide } from 'svelte/transition';

	import ActiveThoughts from './ActiveThoughts.svelte';

	const tosState = $derived(getTosState());
	const activeSector = $derived(tosState.sectors[tosState.active_sector_index]);
	const activeHub = $derived(
		activeSector && activeSector.hubs[activeSector.active_hub_index]
			? activeSector.hubs[activeSector.active_hub_index]
			: null
	);
	const history = $derived((activeHub as any)?.ai_history || []);
</script>

<div class="ai-chat-container glass-panel">
	<div class="chat-header">
		<span class="chat-title">✦ AI_CONVERSATION_HISTORY</span>
		<span class="chat-meta">{history.length} MESSAGES</span>
	</div>
	
	<div class="chat-scrollarea">
		<ActiveThoughts />
		{#if history.length === 0 && (activeHub?.active_thoughts?.length || 0) === 0}
			<div class="chat-empty">
				<div class="empty-icon">✧</div>
				<p>AWAITING INTENT...</p>
				<p class="sub text-mono">Switch to [AI] mode and type to begin.</p>
			</div>
		{:else}
			{#each history as msg}
				<div class="chat-message {msg.role}" transition:slide>
					<div class="msg-header">
						<span class="role-badge">{msg.role.toUpperCase()}</span>
						<span class="timestamp">{new Date(msg.timestamp).toLocaleTimeString()}</span>
					</div>
					<div class="msg-content">
						{msg.content}
					</div>
				</div>
			{/each}
		{/if}
	</div>
</div>

<style>
	.ai-chat-container {
		display: flex;
		flex-direction: column;
		height: 100%;
		overflow: hidden;
		animation: fadeIn 0.3s ease-out;
	}

	.chat-header {
		padding: var(--space-xs) var(--space-md);
		background: rgba(255, 255, 255, 0.05);
		border-bottom: 1px solid var(--color-border);
		display: flex;
		justify-content: space-between;
		align-items: center;
		flex-shrink: 0;
	}

	.chat-title {
		font-family: var(--font-display);
		font-weight: 700;
		font-size: 0.7rem;
		color: var(--color-primary);
		letter-spacing: 0.05em;
	}

	.chat-meta {
		font-family: var(--font-mono);
		font-size: 0.55rem;
		opacity: 0.5;
	}

	.chat-scrollarea {
		flex: 1;
		overflow-y: auto;
		padding: var(--space-md);
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
	}

	.chat-empty {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		opacity: 0.3;
		text-align: center;
	}

	.empty-icon {
		font-size: 2rem;
		margin-bottom: var(--space-sm);
	}

	.chat-message {
		padding: var(--space-sm) var(--space-md);
		background: rgba(255, 255, 255, 0.02);
		border-radius: var(--radius-sm);
		border-left: 2px solid transparent;
		position: relative;
	}

	.chat-message.user {
		border-left-color: var(--color-secondary);
		background: rgba(247, 168, 51, 0.05); /* Secondary color tint */
	}

	.chat-message.assistant {
		border-left-color: var(--color-primary);
		background: rgba(0, 255, 255, 0.02);
	}

	.role-badge {
		font-family: var(--font-mono);
		font-size: 0.55rem;
		font-weight: 700;
		margin-right: var(--space-sm);
	}

	.user .role-badge { color: var(--color-secondary); }
	.assistant .role-badge { color: var(--color-primary); }

	.timestamp {
		font-family: var(--font-mono);
		font-size: 0.5rem;
		opacity: 0.4;
	}

	.msg-content {
		font-size: 0.8rem;
		line-height: 1.5;
		white-space: pre-wrap;
		margin-top: 4px;
		color: var(--color-text);
	}

	@keyframes fadeIn {
		from { opacity: 0; transform: translateY(10px); }
		to { opacity: 1; transform: translateY(0); }
	}
</style>
