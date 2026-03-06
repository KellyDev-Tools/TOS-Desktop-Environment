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

	const tabs: { id: SettingsTab; label: string; icon: string }[] = [
		{ id: 'global',      label: 'SYSTEM',      icon: '⬡' },
		{ id: 'interface',   label: 'INTERFACE',   icon: '◈' },
		{ id: 'ai',          label: 'AI',           icon: '✦' },
		{ id: 'marketplace', label: 'MARKETPLACE',  icon: '⊞' },
		{ id: 'sectors',     label: 'SECTORS',      icon: '⬡' },
	];

	// Local settings state
	let themeSelection = $state('dark');
	let uiFeedback = $state('50');
	let sandboxingEnabled = $state(true);
	let newBackendId = $state('');

	// AI key status
	const openaiKeySet = $derived(
		!!state.settings?.global?.['tos.ai.openai_api_key'] ||
		!!state.settings?.global?.['tos.ai.openai_key_configured']
	);
	const anthropicKeySet = $derived(
		!!state.settings?.global?.['tos.ai.anthropic_api_key'] ||
		!!state.settings?.global?.['tos.ai.anthropic_key_configured']
	);

	function handleOverlayClick(e: MouseEvent) {
		if ((e.target as HTMLElement).classList.contains('modal-overlay')) closeSettings();
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Escape') closeSettings();
	}

	async function handleSave() {
		await setSetting('tos.interface.theme', themeSelection);
		await setSetting('tos.interface.ui_feedback', uiFeedback);
		closeSettings();
	}

	async function toggleBehavior(id: string, currentlyEnabled: boolean) {
		const cmd = currentlyEnabled ? `ai_behavior_disable:${id}` : `ai_behavior_enable:${id}`;
		await sendCommand(cmd);
	}

	async function setDefaultBackend() {
		if (!newBackendId.trim()) return;
		await sendCommand(`ai_backend_set_default:${newBackendId.trim()}`);
		newBackendId = '';
	}

	async function setBehaviorBackend(behaviorId: string, backendId: string) {
		if (backendId) {
			await sendCommand(`ai_backend_set_behavior:${behaviorId};${backendId}`);
		} else {
			await sendCommand(`ai_backend_clear_behavior:${behaviorId}`);
		}
	}

	async function activateAiModule(id: string) {
		await sendCommand(`ai_backend_set_default:${id}`);
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
							<span class="nav-icon">{tab.icon}</span>
							{tab.label}
						</button>
					{/each}
				</div>

				<!-- Tab Content -->
				<div class="settings-pane">

					<!-- ═══ SYSTEM TAB ═══ -->
					{#if activeTab === 'global'}
						<div class="settings-group">
							<div class="settings-group-title">SYSTEM IDENTITY</div>
							<div class="settings-row">
								<span class="settings-label">System Prefix</span>
								<span class="settings-value text-mono">{state.sys_prefix || 'ALPHA-2.2'}</span>
							</div>
							<div class="settings-row">
								<span class="settings-label">Brain Time</span>
								<span class="settings-value text-mono">{state.brain_time || '--:--:--'}</span>
							</div>
							<div class="settings-row">
								<span class="settings-label">Active Theme</span>
								<select class="settings-select" bind:value={themeSelection}>
									<option value="dark">LCARS Dark</option>
									<option value="light">LCARS Light</option>
									<option value="classic">Classic Terminal</option>
								</select>
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

					<!-- ═══ INTERFACE TAB ═══ -->
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

					<!-- ═══ AI TAB ═══ -->
					{:else if activeTab === 'ai'}
						<div class="settings-group">
							<div class="settings-group-title">BACKEND</div>
							<div class="settings-row">
								<span class="settings-label">Active Default</span>
								<span class="settings-value text-mono ai-backend-id">
									{state.ai_default_backend || state.active_ai_module || 'none'}
								</span>
							</div>
							<div class="settings-row">
								<span class="settings-label">Set Default Backend</span>
								<div class="settings-input-row">
									<input
										class="settings-input text-mono"
										placeholder="module-id or url…"
										bind:value={newBackendId}
										onkeydown={(e) => e.key === 'Enter' && setDefaultBackend()}
									/>
									<button class="lcars-btn-sm primary" onclick={setDefaultBackend}>SET</button>
								</div>
							</div>
						</div>

						<div class="settings-group">
							<div class="settings-group-title">API KEY STATUS</div>
							<div class="settings-row">
								<span class="settings-label">OpenAI</span>
								<span class="key-badge" class:configured={openaiKeySet}>
									{openaiKeySet ? '✓ CONFIGURED' : '✗ NOT SET'}
								</span>
							</div>
							<div class="settings-row">
								<span class="settings-label">Anthropic</span>
								<span class="key-badge" class:configured={anthropicKeySet}>
									{anthropicKeySet ? '✓ CONFIGURED' : '✗ NOT SET'}
								</span>
							</div>
							<div class="settings-hint">
								Set keys via environment: <code>OPENAI_API_KEY</code>, <code>ANTHROPIC_API_KEY</code>
							</div>
						</div>

						<div class="settings-group">
							<div class="settings-group-title">INSTALLED AI MODULES</div>
							{#if state.available_ai_modules?.length}
								{#each state.available_ai_modules as mod}
									<div class="settings-row module-row">
										<div class="module-info">
											<span class="settings-label">{mod.name}</span>
											{#if mod.provider}
												<span class="module-provider">{mod.provider.toUpperCase()}</span>
											{/if}
										</div>
										<button
											class="lcars-btn-sm"
											class:primary={state.active_ai_module === mod.id || state.ai_default_backend === mod.id}
											onclick={() => activateAiModule(mod.id)}
										>
											{state.active_ai_module === mod.id || state.ai_default_backend === mod.id ? 'ACTIVE ✓' : 'ACTIVATE'}
										</button>
									</div>
								{/each}
							{:else}
								<div class="settings-empty">No AI modules installed. Visit Marketplace →</div>
							{/if}
						</div>

						<div class="settings-group">
							<div class="settings-group-title">BEHAVIORS</div>
							{#if state.ai_behaviors?.length}
								{#each state.ai_behaviors as behavior}
									<div class="behavior-card glass-panel">
										<div class="behavior-header">
											<span class="behavior-name">{behavior.name}</span>
											<label class="toggle">
												<input
													type="checkbox"
													checked={behavior.enabled}
													onchange={() => toggleBehavior(behavior.id, behavior.enabled)}
												/>
												<span class="toggle-slider"></span>
											</label>
										</div>
										<div class="behavior-meta">
											<span class="behavior-id text-mono">{behavior.id}</span>
											{#if behavior.backend_override}
												<span class="behavior-backend-badge">→ {behavior.backend_override}</span>
											{/if}
										</div>
										<div class="behavior-backend-row">
											<span class="settings-label small">Backend override</span>
											<select
												class="settings-select small"
												value={behavior.backend_override ?? ''}
												onchange={(e) => setBehaviorBackend(behavior.id, (e.target as HTMLSelectElement).value)}
											>
												<option value="">(use default)</option>
												{#each (state.available_ai_modules || []) as mod}
													<option value={mod.id}>{mod.name}</option>
												{/each}
											</select>
										</div>
									</div>
								{/each}
							{:else}
								<div class="settings-empty">No behaviors registered.</div>
							{/if}
						</div>

					<!-- ═══ MARKETPLACE TAB ═══ -->
					{:else if activeTab === 'marketplace'}
						<div class="settings-group">
							<div class="settings-group-title">TERMINAL MODULES</div>
							{#if state.available_modules?.length}
								{#each state.available_modules as mod}
									<div class="settings-row">
										<span class="settings-label">{mod.name}</span>
										<span class="settings-value text-dim">{mod.layout}</span>
									</div>
								{/each}
							{:else}
								<div class="settings-empty">No terminal modules installed.</div>
							{/if}
						</div>

						<div class="settings-group">
							<div class="settings-group-title">AI MODULES</div>
							{#if state.available_ai_modules?.length}
								{#each state.available_ai_modules as mod}
									<div class="settings-row">
										<div class="module-info">
											<span class="settings-label">{mod.name}</span>
											{#if mod.provider}
												<span class="module-provider">{mod.provider.toUpperCase()}</span>
											{/if}
										</div>
										<span class="settings-value text-dim text-mono">{mod.id}</span>
									</div>
								{/each}
							{:else}
								<div class="settings-empty">No AI modules installed.</div>
							{/if}
						</div>

					<!-- ═══ SECTORS TAB ═══ -->
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
					{/if}
				</div>
			</div>

			<!-- Footer -->
			<div class="modal-footer">
				<div class="modal-elbow-bottom"></div>
				<button class="bezel-btn" onclick={() => setSettingsTab('ai')}>✦ AI CONFIG</button>
				<button class="bezel-btn" onclick={() => setSettingsTab('marketplace')}>⊞ MARKETPLACE</button>
				<button class="lcars-btn warning" onclick={handleSave}>COMMIT CHANGES</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.modal-overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.65);
		backdrop-filter: blur(6px);
		z-index: var(--z-modal);
		display: flex;
		align-items: center;
		justify-content: center;
		animation: fadeIn 0.2s ease;
	}

	.modal-container {
		width: 54rem;
		max-width: 94vw;
		max-height: 85vh;
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
		flex-shrink: 0;
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

	.modal-close { font-size: 1rem; }

	/* Content */
	.modal-content {
		display: flex;
		flex: 1;
		min-height: 0;
		overflow: hidden;
	}

	/* Sidebar */
	.settings-sidebar {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: var(--space-sm);
		border-right: 1px solid var(--color-border);
		background: var(--color-surface);
		min-width: 9rem;
		flex-shrink: 0;
	}

	.settings-nav-btn {
		font-family: var(--font-display);
		font-size: 0.68rem;
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
		display: flex;
		align-items: center;
		gap: var(--space-xs);
	}

	.nav-icon {
		font-size: 0.85rem;
		opacity: 0.7;
	}

	.settings-nav-btn:hover { background: rgba(255,255,255,0.04); color: var(--color-text); }
	.settings-nav-btn.active { background: var(--color-primary); color: #000; }
	.settings-nav-btn.active .nav-icon { opacity: 1; }

	/* Pane */
	.settings-pane {
		flex: 1;
		padding: var(--space-md);
		overflow-y: auto;
	}

	.settings-group { margin-bottom: var(--space-lg); }

	.settings-group-title {
		font-family: var(--font-display);
		font-size: 0.62rem;
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

	.settings-label { font-size: 0.8rem; color: var(--color-text); }
	.settings-label.small { font-size: 0.72rem; }
	.settings-value { font-size: 0.8rem; color: var(--color-text-dim); }
	.settings-empty {
		font-size: 0.8rem;
		color: var(--color-text-muted);
		font-style: italic;
		padding: var(--space-md) 0;
	}

	/* Inputs */
	.settings-input-row {
		display: flex;
		gap: var(--space-xs);
		align-items: center;
	}

	.settings-input {
		background: var(--color-surface-raised);
		color: var(--color-text);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-sm);
		padding: 0.3rem 0.6rem;
		font-family: var(--font-mono);
		font-size: 0.75rem;
		outline: none;
		width: 12rem;
	}
	.settings-input:focus { border-color: var(--color-primary); }

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
	.settings-select.small { font-size: 0.68rem; padding: 0.2rem 0.4rem; }
	.settings-select:focus { border-color: var(--color-primary); }

	/* AI-specific */
	.ai-backend-id {
		color: var(--color-primary);
		font-weight: 700;
	}

	.key-badge {
		font-family: var(--font-mono);
		font-size: 0.72rem;
		padding: 0.2rem 0.6rem;
		border-radius: var(--radius-pill);
		background: rgba(238, 68, 68, 0.12);
		border: 1px solid rgba(238, 68, 68, 0.3);
		color: var(--color-danger);
		font-weight: 700;
		letter-spacing: 0.05em;
	}
	.key-badge.configured {
		background: rgba(102, 204, 102, 0.12);
		border-color: rgba(102, 204, 102, 0.3);
		color: var(--color-success);
	}

	.settings-hint {
		font-size: 0.72rem;
		color: var(--color-text-muted);
		padding: var(--space-xs) 0;
	}
	.settings-hint code {
		font-family: var(--font-mono);
		background: rgba(255,255,255,0.06);
		padding: 0.1em 0.35em;
		border-radius: 3px;
		font-size: 0.68rem;
	}

	.behavior-card {
		padding: var(--space-sm) var(--space-md);
		margin-bottom: var(--space-sm);
		border-radius: var(--radius-md);
	}

	.behavior-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: var(--space-xs);
	}

	.behavior-name {
		font-family: var(--font-display);
		font-size: 0.78rem;
		font-weight: 700;
		color: var(--color-text);
		letter-spacing: 0.04em;
	}

	.behavior-meta {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		margin-bottom: var(--space-xs);
	}

	.behavior-id {
		font-size: 0.68rem;
		color: var(--color-text-muted);
	}

	.behavior-backend-badge {
		font-size: 0.68rem;
		color: var(--color-accent);
		background: rgba(92, 136, 218, 0.12);
		border: 1px solid rgba(92, 136, 218, 0.2);
		padding: 0.1rem 0.4rem;
		border-radius: var(--radius-pill);
	}

	.behavior-backend-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-sm);
	}

	.module-row { align-items: center; }

	.module-info {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
	}

	.module-provider {
		font-family: var(--font-mono);
		font-size: 0.62rem;
		padding: 0.1rem 0.4rem;
		border-radius: var(--radius-pill);
		background: rgba(92, 136, 218, 0.12);
		border: 1px solid rgba(92, 136, 218, 0.2);
		color: var(--color-accent);
		font-weight: 700;
	}

	/* Small button variant */
	.lcars-btn-sm {
		font-family: var(--font-display);
		font-size: 0.62rem;
		font-weight: 700;
		letter-spacing: 0.08em;
		padding: 0.25rem 0.6rem;
		border: 1px solid var(--color-border);
		background: transparent;
		color: var(--color-text-dim);
		border-radius: var(--radius-sm);
		cursor: pointer;
		transition: all var(--transition-fast);
		white-space: nowrap;
	}
	.lcars-btn-sm:hover { background: rgba(255,255,255,0.06); color: var(--color-text); }
	.lcars-btn-sm.primary {
		background: rgba(247, 168, 51, 0.12);
		border-color: rgba(247, 168, 51, 0.35);
		color: var(--color-primary);
	}

	/* Toggle */
	.toggle {
		position: relative;
		display: inline-block;
		width: 2.5rem;
		height: 1.25rem;
		flex-shrink: 0;
	}
	.toggle input { opacity: 0; width: 0; height: 0; }
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
	.toggle input:checked + .toggle-slider { background: rgba(102,204,102,0.15); border-color: rgba(102,204,102,0.4); }
	.toggle input:checked + .toggle-slider::before { background: var(--color-success); transform: translateX(1.2rem); }

	/* Range */
	.settings-range-container { display: flex; align-items: center; gap: var(--space-sm); }
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
		width: 14px; height: 14px;
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
		flex-shrink: 0;
	}
	.modal-elbow-bottom { flex: 1; }
</style>
