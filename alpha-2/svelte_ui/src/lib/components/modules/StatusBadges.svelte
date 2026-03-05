<script lang="ts">
	import { openSettings, openPortalModal, toggleTerminalToFront } from '$lib/stores/ui.svelte';
</script>

<div class="status-badges">
	<button class="bezel-btn" title="Toggle Terminal" onclick={() => toggleTerminalToFront()}>👁</button>
	<button class="bezel-btn" title="Web Portal" onclick={() => openPortalModal()}>📡</button>
	<button class="bezel-btn" title="Settings" onclick={() => openSettings()}>⚙</button>
</div>

<style>
	.status-badges {
		display: flex;
		gap: var(--space-sm);
		align-items: center;
		animation: fadeIn 0.3s ease;
	}
</style>
