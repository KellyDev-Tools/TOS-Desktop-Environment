<script lang="ts">
	import { getTosState, sendCommand, setSetting } from '$lib/stores/ipc.svelte';
	import {
		isSettingsOpen, closeSettings,
		getSettingsTab, setSettingsTab,
		type SettingsTab
	} from '$lib/stores/ui.svelte';

	const open = $derived(isSettingsOpen());
	const activeTab = $derived(getSettingsTab());
	const state = $derived(getTosState());

	const tabs: { id: SettingsTab; label: string }[] = [
		{ id: 'global', label: 'GLOBAL' },
		{ id: 'sectors', label: 'SECTOR' },
		{ id: 'interface', label: 'INTERFACE' },
		{ id: 'marketplace', label: 'MARKETPLACE' },
	];

	// Local settings state for edits
	let themeSelection = $state('dark');
	let uiFeedback = $state('50');
	let sandboxingEnabled = $state(true);

	function handleOverlayClick(e: MouseEvent) {
		if ((e.target as HTMLElement).classList.contains('modal-overlay')) {
			closeSettings();
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') closeSettings();
	}

	async function handleSave() {
		await setSetting('tos.interface.theme', themeSelection);
		await setSetting('tos.interface.ui_feedback', uiFeedback);
		closeSettings();
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="modal-overlay" onclick={handleOverlayClick}>
		<div class="modal-container glass-panel">
			<!-- Header -->
			<div class="modal-header">
				<div class="modal-elbow"></div>
				<div class="modal-title">SYSTEM SETTINGS // CONFIGURATION</div>
				<button class="bezel-btn modal-close" onclick={() => closeSettings()}>✕</button>
			</div>

			<!-- Content -->
			<div class="modal-content">
				<!-- Tab Sidebar -->
				<div class="settings-sidebar">
					{#each tabs as tab}
						<button
							class="settings-nav-btn"
							class:active={activeTab === tab.id}
							onclick={() => setSettingsTab(tab.id)}
						>
							{tab.label}
						</button>
					{/each}
				</div>

				<!-- Tab Content -->
				<div class="settings-pane">
					{#if activeTab === 'global'}
						<div class="settings-group">
							<div class="settings-group-title">SYSTEM</div>
							<div class="settings-row">
								<span class="settings-label">System Prefix</span>
								<span class="settings-value text-mono">{state.sys_prefix || 'ALPHA-2.2'}</span>
							</div>
							<div class="settings-row">
								<span class="settings-label">Active Theme</span>
								<select class="settings-select" bind:value={themeSelection}>
									<option value="dark">LCARS Dark</option>
									<option value="light">LCARS Light</option>
									<option value="classic">Classic Terminal</option>
								</select>
							</div>
							<div class="settings-row">
								<span class="settings-label">Brain Time</span>
								<span class="settings-value text-mono">{state.brain_time || '--:--:--'}</span>
							</div>
						</div>

						<div class="settings-group">
							<div class="settings-group-title">SECURITY</div>
							<div class="settings-row">
								<span class="settings-label">Module Sandboxing</span>
								<label class="toggle">
									<input type="checkbox" bind:checked={sandboxingEnabled} />
									<span class="toggle-slider"></span>
								</label>
							</div>
						</div>

					{:else if activeTab === 'sectors'}
						<div class="settings-group">
							<div class="settings-group-title">ACTIVE SECTORS</div>
							{#each state.sectors as sector, i}
								<div class="settings-row">
									<span class="settings-label">S0{i}: {sector.name.toUpperCase()}</span>
									<span class="settings-value" class:text-success={!sector.frozen} class:text-warning={sector.frozen}>
										{sector.frozen ? 'FROZEN' : 'ACTIVE'}
									</span>
								</div>
							{/each}
						</div>

					{:else if activeTab === 'interface'}
						<div class="settings-group">
							<div class="settings-group-title">BEZEL</div>
							<div class="settings-row">
								<span class="settings-label">UI Feedback Level</span>
								<div class="settings-range-container">
									<input type="range" class="settings-range" min="0" max="100" bind:value={uiFeedback} />
									<span class="settings-range-value text-mono">{uiFeedback}%</span>
								</div>
							</div>
						</div>

						<div class="settings-group">
							<div class="settings-group-title">TERMINAL</div>
							<div class="settings-row">
								<span class="settings-label">Active Module</span>
								<span class="settings-value text-mono">{state.active_terminal_module || 'default'}</span>
							</div>
						</div>

					{:else if activeTab === 'marketplace'}
						<div class="settings-group">
							<div class="settings-group-title">INSTALLED MODULES</div>
							{#if state.available_modules && state.available_modules.length > 0}
								{#each state.available_modules as mod}
									<div class="settings-row">
										<span class="settings-label">{mod.name}</span>
										<span class="settings-value text-dim">{mod.layout}</span>
									</div>
								{/each}
							{:else}
								<div class="settings-empty">No modules installed.</div>
							{/if}
						</div>
					{/if}
				</div>
			</div>

			<!-- Footer -->
			<div class="modal-footer">
				<div class="modal-elbow-bottom"></div>
				<button class="lcars-btn warning" onclick={handleSave}>COMMIT CHANGES</button>
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

	.modal-container {
		width: 48rem;
		max-width: 90vw;
		max-height: 80vh;
		display: flex;
		flex-direction: column;
		animation: scaleIn 0.3s cubic-bezier(0.16, 1, 0.3, 1);
		overflow: hidden;
	}

	/* Header */
	.modal-header {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: var(--space-sm) var(--space-md);
		background: var(--color-surface-raised);
		border-bottom: 1px solid var(--color-border);
	}

	.modal-elbow {
		width: 2rem;
		height: 1.5rem;
		background: var(--color-primary);
		border-bottom-right-radius: var(--radius-elbow);
		flex-shrink: 0;
	}

	.modal-title {
		flex: 1;
		font-family: var(--font-display);
		font-size: 0.8rem;
		font-weight: 700;
		letter-spacing: 0.1em;
		color: var(--color-text);
	}

	.modal-close {
		font-size: 1rem;
	}

	/* Content */
	.modal-content {
		display: flex;
		flex: 1;
		min-height: 0;
		overflow: hidden;
	}

	.settings-sidebar {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: var(--space-sm);
		border-right: 1px solid var(--color-border);
		background: var(--color-surface);
		min-width: 8rem;
	}

	.settings-nav-btn {
		font-family: var(--font-display);
		font-size: 0.7rem;
		font-weight: 700;
		letter-spacing: 0.08em;
		background: transparent;
		color: var(--color-text-dim);
		border: none;
		padding: 0.5rem 0.75rem;
		text-align: left;
		cursor: pointer;
		border-radius: var(--radius-sm);
		transition: all var(--transition-fast);
	}

	.settings-nav-btn:hover {
		background: rgba(255, 255, 255, 0.04);
		color: var(--color-text);
	}

	.settings-nav-btn.active {
		background: var(--color-primary);
		color: #000;
	}

	.settings-pane {
		flex: 1;
		padding: var(--space-md);
		overflow-y: auto;
	}

	/* Settings Groups */
	.settings-group {
		margin-bottom: var(--space-lg);
	}

	.settings-group-title {
		font-family: var(--font-display);
		font-size: 0.65rem;
		font-weight: 700;
		letter-spacing: 0.15em;
		color: var(--color-secondary);
		margin-bottom: var(--space-sm);
		padding-bottom: var(--space-xs);
		border-bottom: 1px solid var(--color-border);
	}

	.settings-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0.4rem 0;
		gap: var(--space-md);
	}

	.settings-label {
		font-size: 0.8rem;
		color: var(--color-text);
	}

	.settings-value {
		font-size: 0.8rem;
		color: var(--color-text-dim);
	}

	.settings-empty {
		font-size: 0.8rem;
		color: var(--color-text-muted);
		font-style: italic;
		padding: var(--space-md) 0;
	}

	/* Select */
	.settings-select {
		background: var(--color-surface-raised);
		color: var(--color-text);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-sm);
		padding: 0.3rem 0.6rem;
		font-family: var(--font-mono);
		font-size: 0.75rem;
		cursor: pointer;
		outline: none;
	}

	.settings-select:focus {
		border-color: var(--color-primary);
	}

	/* Toggle Switch */
	.toggle {
		position: relative;
		display: inline-block;
		width: 2.5rem;
		height: 1.25rem;
	}

	.toggle input {
		opacity: 0;
		width: 0;
		height: 0;
	}

	.toggle-slider {
		position: absolute;
		cursor: pointer;
		inset: 0;
		background: var(--color-surface-raised);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-pill);
		transition: all var(--transition-fast);
	}

	.toggle-slider::before {
		content: '';
		position: absolute;
		height: 0.85rem;
		width: 0.85rem;
		left: 0.15rem;
		bottom: 0.125rem;
		background: var(--color-text-dim);
		border-radius: 50%;
		transition: all var(--transition-fast);
	}

	.toggle input:checked + .toggle-slider {
		background: rgba(102, 204, 102, 0.15);
		border-color: rgba(102, 204, 102, 0.4);
	}

	.toggle input:checked + .toggle-slider::before {
		background: var(--color-success);
		transform: translateX(1.2rem);
	}

	/* Range Slider */
	.settings-range-container {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
	}

	.settings-range {
		-webkit-appearance: none;
		appearance: none;
		width: 8rem;
		height: 4px;
		background: var(--color-surface-raised);
		border-radius: 2px;
		outline: none;
	}

	.settings-range::-webkit-slider-thumb {
		-webkit-appearance: none;
		appearance: none;
		width: 14px;
		height: 14px;
		border-radius: 50%;
		background: var(--color-primary);
		cursor: pointer;
		border: 2px solid var(--color-bg);
	}

	.settings-range-value {
		font-size: 0.7rem;
		color: var(--color-text-dim);
		min-width: 2.5rem;
		text-align: right;
	}

	/* Footer */
	.modal-footer {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: var(--space-sm);
		padding: var(--space-sm) var(--space-md);
		border-top: 1px solid var(--color-border);
		background: var(--color-surface-raised);
	}

	.modal-elbow-bottom {
		flex: 1;
	}
</style>
