<script lang="ts">
	import { onMount } from 'svelte';
	import { slide, scale, fade } from 'svelte/transition';
	import {
		connect, disconnect, getTosState, getConnectionState,
		submitCommand, sendCommand,
		splitCreate, splitClose, splitFocusDirection, splitEqualize
	} from '$lib/stores/ipc.svelte';
	import {
		getCurrentMode, setCurrentMode,
		isSidebarLeftExpanded, isSidebarRightExpanded,
		toggleSidebarLeft, toggleSidebarRight,
		isSettingsOpen, isPortalModalOpen,
		openSettings, setSettingsTab, toggleTerminalToFront,
		getPromptMode, setPromptMode,
		type ViewMode, type PromptMode
	} from '$lib/stores/ui.svelte';

	// View Components
	import GlobalOverview from '$lib/components/views/GlobalOverview.svelte';
	import CommandHub from '$lib/components/views/CommandHub.svelte';
	import ApplicationFocus from '$lib/components/views/ApplicationFocus.svelte';
	import Marketplace from '$lib/components/views/Marketplace.svelte';
	import DetailInspector from '$lib/components/views/DetailInspector.svelte';

	// Module Components
	import BrainStatus from '$lib/components/modules/BrainStatus.svelte';
	import Telemetry from '$lib/components/modules/Telemetry.svelte';
	import Minimap from '$lib/components/modules/Minimap.svelte';
	import PriorityStack from '$lib/components/modules/PriorityStack.svelte';
	import MiniLog from '$lib/components/modules/MiniLog.svelte';
	import StatusBadges from '$lib/components/modules/StatusBadges.svelte';

	// Overlays
	import SystemOutput from '$lib/components/SystemOutput.svelte';
	import DisconnectOverlay from '$lib/components/DisconnectOverlay.svelte';
	import SettingsModal from '$lib/components/SettingsModal.svelte';
	import PortalModal from '$lib/components/PortalModal.svelte';

	const state = $derived(getTosState());
	const connState = $derived(getConnectionState());
	const mode = $derived(getCurrentMode());
	const promptMode = $derived(getPromptMode());
	const sidebarLeft = $derived(isSidebarLeftExpanded());
	const sidebarRight = $derived(isSidebarRightExpanded());
	const activeSector = $derived(state.sectors?.[state.active_sector_index]);

	// Bezel States based on Face Specification §3.1
	const bottomBezelState = $derived(
		mode === 'global' ? 'collapsed-locked' :
		mode === 'hubs' ? 'expanded' :
		mode === 'sectors' ? 'collapsed-expandable' :
		'collapsed-locked'
	);

	// Hierarchy level buttons
	const levels: { label: string; mode: ViewMode; key: string }[] = [
		{ label: 'GlobalOverview', mode: 'global', key: '1' },
		{ label: 'CommandHub', mode: 'hubs', key: '2' },
		{ label: 'ApplicationFocus', mode: 'sectors', key: '3' },
		{ label: 'DetailView', mode: 'detail', key: '4' },
	];

	// Prompt mode config
	const promptModes: { id: PromptMode; label: string }[] = [
		{ id: 'cmd', label: 'CMD' },
		{ id: 'search', label: 'SEARCH' },
		{ id: 'ai', label: 'AI' },
	];

	const promptPrefix = $derived(
		promptMode === 'cmd' ? 'CMD ▸' :
		promptMode === 'search' ? 'SRCH ▸' :
		'AI ▸'
	);

	const promptPlaceholder = $derived(
		promptMode === 'cmd' ? 'ENTER COMMAND...' :
		promptMode === 'search' ? 'SEARCH FILES & CONTENT...' :
		'ASK AI COPILOT...'
	);

	let cmdInput = $state('');

	import OnboardingOverlay from '$lib/components/OnboardingOverlay.svelte';
	import ExpandedBezel from '$lib/components/ExpandedBezel.svelte';
	import { bezelExpand } from '$lib/stores/ipc.svelte';

	let cinematicActive = $state(false);
	let cinematicStage = $state<'none' | 'sweep' | 'logs' | 'zoom'>('none');
	let sessionPopoverOpen = $state(false);
	let heuristicSuggestions = $state<{ text: string, score: f32, source: String }[]>([]);

	onMount(() => {
		connect();
		
		// Check for first run to trigger cinematic
		const isFirstRun = getTosState().settings.global['tos.onboarding.first_run_complete'] !== 'true';
		if (isFirstRun) {
			startCinematic();
		}

		return () => disconnect();
	});

	function startCinematic() {
		cinematicActive = true;
		cinematicStage = 'sweep';
		
		setTimeout(() => { if (cinematicActive) cinematicStage = 'logs'; }, 4000);
		setTimeout(() => { if (cinematicActive) cinematicStage = 'zoom'; }, 8000);
		setTimeout(() => { if (cinematicActive) skipCinematic(); }, 12000);
	}

	function skipCinematic() {
		cinematicActive = false;
		cinematicStage = 'none';
	}

	async function handleSubmit(e: Event) {
		e.preventDefault();
		if (!cmdInput.trim()) return;
		const input = cmdInput;
		cmdInput = '';

		if (promptMode === 'cmd') {
			await submitCommand(input);
		} else if (promptMode === 'search') {
			await sendCommand(`search:${input}`);
		} else if (promptMode === 'ai') {
			await sendCommand(`ai_submit:${input}`);
		}
	}

	async function handleInput(e: Event) {
		const target = e.target as HTMLInputElement;
		const val = target.value;
		
		if (promptMode === 'cmd' && val.length > 1) {
			const resp = await sendCommand(`heuristic_query:${val}`);
			const cleanJson = resp.split(' (').next() || '[]';
			try {
				heuristicSuggestions = JSON.parse(cleanJson);
			} catch (e) {
				heuristicSuggestions = [];
			}
		} else {
			heuristicSuggestions = [];
		}
	}

	function applySuggestion(suggestion: string) {
		// Simple replacement for now: find last whitespace or beginning
		const parts = cmdInput.split(' ');
		parts[parts.length - 1] = suggestion;
		cmdInput = parts.join(' ');
		heuristicSuggestions = [];
		// Re-focus
		document.getElementById('cmd-input')?.focus();
	}

	function handleLevelClick(m: ViewMode) {
		setCurrentMode(m);
		sendCommand(`set_mode:${m}`);
	}

	// View title derived from mode
	const viewTitle = $derived(
		mode === 'global' ? 'GLOBAL OVERVIEW' :
		mode === 'hubs' ? `HUB VIEW // COMMAND` :
		mode === 'sectors' ? `APPLICATION FOCUS // ${activeSector?.name?.toUpperCase() || 'UNKNOWN'}` :
		mode === 'detail' ? 'DETAIL INSPECTOR // LEVEL 4' :
		mode === 'buffer' ? 'RAW DATA BUFFER // LEVEL 5' :
		mode === 'spatial' ? 'SPATIAL TOPOLOGY // 3D SHELL' :
		'TOS'
	);

	function handleGlobalKeydown(e: KeyboardEvent) {
		if (cinematicActive) {
			e.preventDefault();
			skipCinematic();
			return;
		}

		// Don't intercept if typing in an input
		const tag = (e.target as HTMLElement)?.tagName;
		const isInput = tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT';

		// Escape closes modals (handled by modal components themselves)

		if (e.ctrlKey || e.metaKey) {
			// Ctrl+1-4: Switch hierarchy levels
			const levelMap: Record<string, ViewMode> = { '1': 'global', '2': 'hubs', '3': 'sectors', '4': 'detail', 'm': 'marketplace' };
			if (levelMap[e.key]) {
				e.preventDefault();
				handleLevelClick(levelMap[e.key]);
				return;
			}

			// Ctrl+, : Open settings
			if (e.key === ',') {
				e.preventDefault();
				openSettings();
				return;
			}

			// Ctrl+/ : Focus command input
			if (e.key === '/') {
				e.preventDefault();
				document.getElementById('cmd-input')?.focus();
				return;
			}

			// Ctrl+T : Toggle terminal overlay
			if (e.key === 't' && !isInput) {
				e.preventDefault();
				toggleTerminalToFront();
				return;
			}

			// Split Pane Controls (Only when not in input)
			if (!isInput) {
				if (e.key === '\\' || e.key === '-') {
					e.preventDefault();
					splitCreate();
					return;
				}

				if (e.key === 'ArrowUp') { e.preventDefault(); splitFocusDirection('Up'); return; }
				if (e.key === 'ArrowDown') { e.preventDefault(); splitFocusDirection('Down'); return; }
				if (e.key === 'ArrowLeft') { e.preventDefault(); splitFocusDirection('Left'); return; }
				if (e.key === 'ArrowRight') { e.preventDefault(); splitFocusDirection('Right'); return; }

				if (e.key === 'W' && e.shiftKey) { // Ctrl+Shift+W to avoid browser close
					e.preventDefault();
					splitClose();
					return;
				}

				if (e.key === '0') {
					e.preventDefault();
					splitEqualize();
					return;
				}
			}
		}
	}
</script>

<svelte:window onkeydown={handleGlobalKeydown} />

<div class="lcars-container">
	<!-- Cinematic Background -->
	<div class="tri-module tri-top-left"></div>
	<div class="tri-module tri-bottom-right"></div>

	<!-- ═══════════ TOP BEZEL ═══════════ -->
	<header class="lcars-header">
		<div class="lcars-bar lcars-bar-top">
			<div class="lcars-elbow top-left"></div>

			<!-- Left Section: Title -->
			<div class="header-section header-left">
				<button class="bezel-btn" title="Toggle Left Sidebar" onclick={() => toggleSidebarLeft()}>◀</button>
				<div class="lcars-title-area">
					<span class="lcars-prefix">{state.sys_prefix || 'ALPHA-2.2 // INTEL-DRIVEN'}</span>
				</div>
				
				<!-- Sector Chip with Popover -->
				{#if state.sectors[state.active_sector_index]}
					{@const activeSec = state.sectors[state.active_sector_index]}
					<div class="sector-chip-wrapper">
						<button class="sector-name-chip" onclick={() => sessionPopoverOpen = !sessionPopoverOpen}>
							<span class="live-pulse"></span>
							{activeSec.name.toUpperCase()}
						</button>
						
						{#if sessionPopoverOpen}
							<div class="session-popover glass-panel" transition:fade={{duration: 200}}>
								<div class="popover-header">SESSION_MANAGER</div>
								<div class="popover-actions">
									<button class="popover-btn" onclick={() => { /* sessionSave(); */ sessionPopoverOpen = false; }}>SAVE_SESSION</button>
									<button class="popover-btn" onclick={() => { /* sessionExport(); */ sessionPopoverOpen = false; }}>EXPORT_JSON</button>
									<div class="divider"></div>
									<div class="popover-label">NAMED_SESSIONS</div>
									<div class="session-list">
										<div class="session-item empty">NO_NAMED_SESSIONS</div>
									</div>
								</div>
							</div>
						{/if}
					</div>
				{/if}
			</div>

			<!-- Center Section: Brain Status + Telemetry -->
			<div class="header-section header-center">
				<BrainStatus />
				<Telemetry />
			</div>

			<!-- Right Section: Status Badges -->
			<div class="header-section header-right">
				<StatusBadges />
				<button 
					class="bezel-btn" 
					title="Open Portal (Long-press for Marketplace)" 
					onclick={() => import('$lib/stores/ui.svelte').then(m => m.openPortalModal())}
					onmousedown={(e) => {
						const timer = setTimeout(() => {
							setCurrentMode('marketplace');
							sendCommand('set_mode:marketplace');
						}, 800);
						const up = () => { clearTimeout(timer); window.removeEventListener('mouseup', up); };
						window.addEventListener('mouseup', up);
					}}
				>
					⊕ PORTAL
				</button>
				<button class="bezel-btn settings-btn" title="System Settings (Ctrl+,)" onclick={() => openSettings()}>⚙ SYS</button>
				<button class="bezel-btn help-btn" title="Help & Onboarding" onclick={() => { setSettingsTab('global'); openSettings(); }}>?</button>
				<button class="bezel-btn" title="Toggle Right Sidebar" onclick={() => toggleSidebarRight()}>▶</button>
			</div>

			<div class="lcars-elbow top-right"></div>
		</div>
	</header>

	<!-- ═══════════ MAIN AREA ═══════════ -->
	<main class="lcars-main">
		<!-- Left Sidebar -->
		<nav class="lcars-sidebar lcars-sidebar-left" class:expanded={sidebarLeft}>
			<div class="sidebar-top">
				{#each levels as level}
					<button
						class="lcars-btn level-btn"
						class:active={mode === level.mode}
						onclick={() => handleLevelClick(level.mode)}
					>
						{level.key}
					</button>
				{/each}
				<div class="sidebar-divider"></div>
				<button
					class="lcars-btn market-btn"
					class:active={mode === 'marketplace'}
					onclick={() => { setCurrentMode('marketplace'); sendCommand('set_mode:marketplace'); }}
					title="Marketplace (⊞)"
				>
					⊞
				</button>
			</div>
			<div class="sidebar-modules">
				<Minimap />
			</div>
			<div class="sidebar-spacer"></div>
		</nav>

		<!-- Viewport -->
		<section class="lcars-content-area">
			<div class="lcars-content-bezel">
				<div class="glass-panel viewport" id="main-viewport">
					<!-- Viewport Header -->
					<div class="viewport-header">
						<div class="viewport-title">{viewTitle}</div>
						<div class="viewport-controls">
							<button class="bezel-btn" title="Add Sector">+</button>
							<button class="bezel-btn" title="Close Sector">−</button>
						</div>
					</div>

					<!-- Viewport Content -->
					<div class="viewport-content" class:bezel-zoomed={state.bezel_expanded}>
						<DisconnectOverlay />
						<OnboardingOverlay />
						<ExpandedBezel />

						{#if cinematicActive}
							<!-- svelte-ignore a11y_click_events_have_key_events -->
							<!-- svelte-ignore a11y_no_static_element_interactions -->
							<div 
								class="cinematic-overlay {cinematicStage}" 
								transition:fade={{ duration: 1000 }}
								onclick={skipCinematic}
							>
								{#if cinematicStage === 'sweep'}
									<div class="sweep-grid"></div>
									<div class="intro-title" in:scale={{ duration: 2000 }}>TOS // TACTICAL_OPERATING_SYSTEM</div>
								{:else if cinematicStage === 'logs'}
									<div class="boot-logs">
										{#each state.system_log.slice(-30) as log}
											<div class="boot-line">{log.text}</div>
										{/each}
									</div>
								{:else if cinematicStage === 'zoom'}
									<div class="zoom-effect"></div>
								{/if}
								
								<div class="skip-hint">Press any key to skip</div>
							</div>
						{/if}

						{#if connState === 'connected' && !cinematicActive}
							<!-- Background / Level 1 Layer -->
							<div class="spatial-layer level-1" class:zoomed={mode !== 'global'}>
								<SystemOutput />
								{#if mode === 'global' || mode === 'hubs' || mode === 'sectors'}
									<!-- Keep Global visible underneath for blur effect when zoomed -->
									<div transition:fade={{duration: 200}} style="width:100%; height:100%;">
										<GlobalOverview />
									</div>
								{/if}
							</div>

							<!-- Higher Levels Layer -->
							{#if mode !== 'global'}
								<div class="spatial-layer level-higher" in:scale={{start: 0.95, duration: 400, opacity: 0}} out:fade={{duration: 200}}>
									{#if mode === 'hubs'}
										<CommandHub />
									{:else if mode === 'marketplace'}
										<Marketplace />
									{:else if mode === 'sectors'}
										<ApplicationFocus />
									{:else if mode === 'detail'}
										<DetailInspector />
									{:else if mode === 'buffer'}
										<div class="placeholder-view">
											<div class="placeholder-title">RAW DATA BUFFER</div>
											<div class="placeholder-sub">Level 5 — Hex Stream View</div>
										</div>
									{:else if mode === 'spatial'}
										<div class="placeholder-view">
											<div class="placeholder-title">SPATIAL TOPOLOGY</div>
											<div class="placeholder-sub">3D Sector Shell</div>
										</div>
									{/if}
								</div>
							{/if}
						{/if}
					</div>
				</div>
			</div>

			<!-- ═══════════ BOTTOM BEZEL ═══════════ -->
			<!-- svelte-ignore a11y_click_events_have_key_events -->
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<footer 
				class="lcars-footer {bottomBezelState}"
				onclick={(e) => {
					// Only expand if clicking the bezel area, not the inputs/buttons
					if ((e.target as HTMLElement).classList.contains('lcars-bar') && mode !== 'detail') {
						bezelExpand();
					}
				}}
			>
				<div class="lcars-bar lcars-bar-bottom">
					<div class="lcars-elbow bottom-left"></div>
					<div class="lcars-input-area">
						<div class="mode-toggle-pill">
							{#each promptModes as pm}
								<button
									class="pill-btn"
									class:active={promptMode === pm.id}
									onclick={() => setPromptMode(pm.id)}
								>
									{pm.label}
								</button>
							{/each}
						</div>
						<form class="prompt-form" onsubmit={handleSubmit}>
							<span class="prompt-prefix">{promptPrefix}</span>
							<input
								type="text"
								class="cmd-input"
								id="cmd-input"
								placeholder={promptPlaceholder}
								autocomplete="off"
								bind:value={cmdInput}
								oninput={handleInput}
								disabled={mode === 'detail'}
							/>
						</form>
						
						{#if heuristicSuggestions.length > 0}
							<div class="heuristic-chips" transition:slide={{ axis: 'y', duration: 200 }}>
								{#each heuristicSuggestions as sug}
									<button class="heuristic-chip" onclick={() => applySuggestion(sug.text)}>
										<span class="chip-source">{sug.source}</span>
										{sug.text}
									</button>
								{/each}
							</div>
						{/if}
					</div>
					<div class="lcars-elbow bottom-right"></div>
				</div>
			</footer>
		</section>

		<!-- Right Sidebar -->
		<aside class="lcars-sidebar lcars-sidebar-right" class:expanded={sidebarRight}>
			<div class="sidebar-modules">
				<PriorityStack />
				<MiniLog />
			</div>
			<div class="sidebar-spacer"></div>
		</aside>
	</main>
</div>

<SettingsModal />
<PortalModal />

<style>
	/* Heuristic Chips */
	.heuristic-chips {
		position: absolute;
		bottom: 100%;
		left: var(--space-md);
		right: var(--space-md);
		display: flex;
		gap: var(--space-xs);
		padding: var(--space-xs) 0;
		overflow-x: auto;
		z-index: 1000;
	}

	.heuristic-chip {
		background: rgba(247, 168, 51, 0.1);
		border: 1px solid rgba(247, 168, 51, 0.3);
		color: var(--color-primary);
		font-family: var(--font-mono);
		font-size: 0.7rem;
		padding: 4px 10px;
		border-radius: var(--radius-pill);
		cursor: pointer;
		white-space: nowrap;
		display: flex;
		align-items: center;
		gap: 6px;
		transition: all 0.2s;
	}

	.heuristic-chip:hover {
		background: var(--color-primary);
		color: #000;
	}

	.chip-source {
		font-size: 0.5rem;
		opacity: 0.6;
		font-weight: 700;
		text-transform: uppercase;
		border-right: 1px solid currentColor;
		padding-right: 6px;
	}

	/* ── Container ── */
	.lcars-container {
		width: 100vw;
		height: 100vh;
		display: flex;
		flex-direction: column;
		position: relative;
		overflow: hidden;
	}

	/* ── Triangular Background Modules ── */
	.tri-module {
		position: absolute;
		width: 40vw;
		height: 40vh;
		opacity: 0.015;
		pointer-events: none;
		z-index: 0;
	}

	.tri-top-left {
		top: -10vh;
		left: -10vw;
		background: linear-gradient(135deg, var(--color-primary), transparent);
		clip-path: polygon(0 0, 100% 0, 0 100%);
	}

	.tri-bottom-right {
		bottom: -10vh;
		right: -10vw;
		background: linear-gradient(315deg, var(--color-secondary), transparent);
		clip-path: polygon(100% 0, 100% 100%, 0 100%);
	}

	/* ── Top Header/Bezel ── */
	.lcars-header {
		flex-shrink: 0;
		z-index: var(--z-bezel);
	}

	.lcars-bar-top {
		display: flex;
		align-items: stretch;
		height: 2.5rem;
		background: var(--color-surface);
		border-bottom: 1px solid var(--color-border);
	}

	.header-section {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		padding: 0 var(--space-sm);
	}

	.header-left {
		flex: 1;
	}

	.header-center {
		flex: 2;
		justify-content: center;
		gap: var(--space-xl);
	}

	.header-right {
		flex: 1;
		justify-content: flex-end;
	}

	.lcars-title-area {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
	}

	.lcars-prefix {
		font-family: var(--font-display);
		font-size: 0.7rem;
		font-weight: 500;
		color: var(--color-text-dim);
		letter-spacing: 0.08em;
	}

	/* ── Main Layout ── */
	.lcars-main {
		flex: 1;
		display: flex;
		min-height: 0;
		z-index: var(--z-base);
	}

	/* ── Sidebars ── */
	.lcars-sidebar {
		display: flex;
		flex-direction: column;
		width: 2.8rem;
		background: var(--color-surface);
		border-right: 1px solid var(--color-border);
		transition: width var(--transition-slow);
		overflow: hidden;
		flex-shrink: 0;
		z-index: var(--z-sidebar);
	}

	.lcars-sidebar-right {
		border-right: none;
		border-left: 1px solid var(--color-border);
	}

	.lcars-sidebar.expanded {
		width: 12rem;
	}

	.sidebar-top {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: var(--space-xs);
	}

	.sidebar-modules {
		flex: 1;
		overflow-y: auto;
		overflow-x: hidden;
	}

	.sidebar-spacer {
		flex-shrink: 0;
		height: 3rem;
		background: linear-gradient(to top, var(--color-primary), var(--color-secondary));
		opacity: 0.15;
		margin: var(--space-xs);
		border-radius: var(--radius-md);
	}

	/* Hierarchy level buttons */
	.level-btn {
		width: 100%;
		padding: 0.5rem;
		font-size: 0.75rem;
		background: var(--color-surface-raised);
		color: var(--color-text-dim);
		border-radius: var(--radius-sm);
		transition: all var(--transition-fast);
	}

	.level-btn.active {
		background: var(--color-primary);
		color: #000;
		font-weight: 800;
	}

	.level-btn:hover:not(.active) {
		background: rgba(247, 168, 51, 0.12);
		color: var(--color-primary);
	}

    .sidebar-divider {
        height: 1px;
        background: rgba(255, 255, 255, 0.1);
        margin: 10px 4px;
    }

    .market-btn {
        background: rgba(92, 136, 218, 0.1) !important;
        border: 1px solid rgba(92, 136, 218, 0.2) !important;
        color: var(--color-secondary) !important;
    }

    .market-btn.active {
        background: var(--color-secondary) !important;
        color: #000 !important;
        box-shadow: 0 0 15px var(--color-secondary);
    }

	/* ── Content Area ── */
	.lcars-content-area {
		flex: 1;
		display: flex;
		flex-direction: column;
		min-width: 0;
	}
	/* ── Sector Chip & Session Popover ── */
	.sector-chip-wrapper {
		position: relative;
		margin-left: 1rem;
	}

	.sector-name-chip {
		background: rgba(255, 255, 255, 0.05);
		border: 1px solid var(--color-primary);
		color: var(--color-primary);
		font-family: var(--font-display);
		font-weight: 700;
		font-size: 0.75rem;
		padding: 4px 12px;
		border-radius: var(--radius-pill);
		display: flex;
		align-items: center;
		gap: 8px;
		cursor: pointer;
		transition: all 0.2s;
	}

	.sector-name-chip:hover {
		background: var(--color-primary);
		color: #000;
	}

	.live-pulse {
		width: 8px;
		height: 8px;
		background: var(--color-primary);
		border-radius: 50%;
		box-shadow: 0 0 10px var(--color-primary);
		animation: blink 2s infinite;
	}

	@keyframes blink {
		0%, 100% { opacity: 1; transform: scale(1); }
		50% { opacity: 0.3; transform: scale(0.8); }
	}

	.session-popover {
		position: absolute;
		top: 130%;
		left: 0;
		width: 200px;
		background: rgba(10, 10, 20, 0.95);
		border: 1px solid var(--color-border);
		border-radius: 4px;
		box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
		z-index: 1100;
		display: flex;
		flex-direction: column;
		overflow: hidden;
		backdrop-filter: blur(20px);
	}

	.popover-header {
		background: rgba(255, 255, 255, 0.05);
		padding: 8px 12px;
		font-family: var(--font-mono);
		font-size: 0.6rem;
		font-weight: 700;
		color: var(--color-text-dim);
		border-bottom: 1px solid rgba(255, 255, 255, 0.1);
	}

	.popover-actions {
		padding: 10px;
		display: flex;
		flex-direction: column;
		gap: 5px;
	}

	.popover-btn {
		background: transparent;
		border: none;
		border-left: 3px solid var(--color-secondary);
		color: var(--color-text);
		text-align: left;
		padding: 6px 12px;
		font-family: var(--font-display);
		font-weight: 600;
		font-size: 0.7rem;
		cursor: pointer;
		transition: all 0.2s;
	}

	.popover-btn:hover {
		background: rgba(255, 255, 255, 0.1);
		border-left-width: 8px;
	}

	.divider {
		height: 1px;
		background: rgba(255, 255, 255, 0.1);
		margin: 5px 0;
	}

	.popover-label {
		font-size: 0.6rem;
		font-family: var(--font-mono);
		color: var(--color-text-muted);
		margin-bottom: 5px;
	}

	.session-list {
		max-height: 150px;
		overflow-y: auto;
	}

	.session-item.empty {
		font-size: 0.65rem;
		color: var(--color-text-dim);
		opacity: 0.5;
		font-style: italic;
		text-align: center;
		padding: 10px;
	}

	.lcars-content-bezel {
		flex: 1;
		min-height: 0;
		padding: var(--space-xs);
	}

	.viewport {
		height: 100%;
		display: flex;
		flex-direction: column;
		overflow: hidden;
		position: relative;
	}

	.viewport-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: var(--space-sm) var(--space-md);
		border-bottom: 1px solid var(--color-border);
		flex-shrink: 0;
	}

	.viewport-title {
		font-family: var(--font-display);
		font-size: 0.95rem;
		font-weight: 700;
		letter-spacing: 0.08em;
		color: var(--color-text);
	}

	.viewport-controls {
		display: flex;
		gap: var(--space-xs);
	}

	.viewport-content {
		flex: 1;
		position: relative;
		overflow: hidden;
	}

	/* ── Bottom Footer/Bezel ── */
	.lcars-footer {
		flex-shrink: 0;
	}

	.lcars-bar-bottom {
		display: flex;
		align-items: stretch;
		height: 2.5rem;
		background: var(--color-surface);
		border-top: 1px solid var(--color-border);
		transition: height var(--transition-normal) cubic-bezier(0.2, 0.8, 0.2, 1);
		overflow: hidden;
	}

	/* Bottom Bezel States */
	.lcars-footer.collapsed-locked .lcars-bar-bottom {
		height: 0.5rem;
		pointer-events: none;
	}

	.lcars-footer.collapsed-expandable .lcars-bar-bottom {
		height: 0.5rem;
		cursor: row-resize;
	}

	.lcars-footer.collapsed-expandable:hover .lcars-bar-bottom,
	.lcars-footer.collapsed-expandable:focus-within .lcars-bar-bottom {
		height: 2.5rem;
	}

	.lcars-input-area {
		flex: 1;
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		transition: opacity var(--transition-fast);
	}

	.lcars-footer.collapsed-locked .lcars-input-area,
	.lcars-footer.collapsed-expandable:not(:hover):not(:focus-within) .lcars-input-area {
		opacity: 0;
		pointer-events: none;
	}

	/* Mode Toggle Pill */
	.mode-toggle-pill {
		display: flex;
		border-radius: var(--radius-pill);
		overflow: hidden;
		border: 1px solid var(--color-border);
		margin-left: var(--space-sm);
		flex-shrink: 0;
	}

	.pill-btn {
		font-family: var(--font-display);
		font-size: 0.6rem;
		font-weight: 700;
		letter-spacing: 0.08em;
		padding: 0.25rem 0.6rem;
		background: transparent;
		color: var(--color-text-dim);
		border: none;
		cursor: pointer;
		transition: all var(--transition-fast);
	}

	.pill-btn:not(:last-child) {
		border-right: 1px solid var(--color-border);
	}

	.pill-btn:hover:not(.active) {
		background: rgba(255, 255, 255, 0.04);
		color: var(--color-text);
	}

	.pill-btn.active {
		background: var(--color-primary);
		color: #000;
	}

	.prompt-form {
		display: flex;
		align-items: center;
		width: 100%;
		gap: var(--space-sm);
		padding: 0 var(--space-md);
	}

	.prompt-prefix {
		font-family: var(--font-display);
		font-size: 0.7rem;
		font-weight: 700;
		color: var(--color-primary);
		letter-spacing: 0.08em;
		flex-shrink: 0;
	}

	.cmd-input {
		flex: 1;
		background: transparent;
		border: none;
		color: var(--color-text);
		font-family: var(--font-mono);
		font-size: 0.85rem;
		outline: none;
		caret-color: var(--color-primary);
	}

	.cmd-input::placeholder {
		color: var(--color-text-muted);
		font-family: var(--font-display);
		letter-spacing: 0.08em;
		font-size: 0.75rem;
	}

	.prefix { color: var(--color-primary); }
	.input-placeholder { color: var(--color-text-dim); opacity: 0.5; font-style: italic; }

	/* ── Spatial Zoom Animations ── */
	.viewport-content {
		position: relative;
		transform-style: preserve-3d;
		perspective: 1200px;
		transition: transform 0.5s cubic-bezier(0.16, 1, 0.3, 1);
	}

	.viewport-content.bezel-zoomed {
		transform: scale(0.9) translateY(-5%);
	}

	.spatial-layer {
		position: absolute;
		inset: 0;
		transition: all 0.5s cubic-bezier(0.16, 1, 0.3, 1);
	}

	.spatial-layer.level-1 {
		z-index: 10;
	}

	.spatial-layer.level-1.zoomed {
		transform: translateZ(-200px) scale(0.95);
		opacity: 0.4;
		filter: blur(4px);
		pointer-events: none;
	}

	.spatial-layer.level-higher {
		z-index: 20;
		background: rgba(10, 10, 20, 0.7);
		backdrop-filter: blur(10px);
	}

	/* ── Placeholder Views (Detail/Buffer/Spatial) ── */
	.placeholder-view {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		height: 100%;
		gap: var(--space-sm);
		animation: fadeIn 0.5s ease;
	}

	.placeholder-title {
		font-family: var(--font-display);
		font-size: 1.4rem;
		font-weight: 700;
		letter-spacing: 0.12em;
		color: var(--color-primary);
	}

	.placeholder-sub {
		font-size: 0.85rem;
		color: var(--color-text-dim);
	}

	/* ── Elbows ── */
	.lcars-elbow {
		width: 3rem;
		height: 100%;
		flex-shrink: 0;
	}

	.lcars-elbow.top-left {
		background: var(--color-primary);
		border-bottom-right-radius: var(--radius-elbow);
	}

	.lcars-elbow.top-right {
		background: var(--color-secondary);
		border-bottom-left-radius: var(--radius-elbow);
	}

	.lcars-elbow.bottom-left {
		background: var(--color-primary);
		border-top-right-radius: var(--radius-elbow);
	}

	.lcars-elbow.bottom-right {
		background: var(--color-secondary);
		border-top-left-radius: var(--radius-elbow);
	}
	/* ── Cinematic Intro Styles ── */
	.cinematic-overlay {
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		z-index: 1000;
		background: #000;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		overflow: hidden;
		cursor: pointer;
	}

	.sweep-grid {
		position: absolute;
		width: 200%;
		height: 200%;
		background: 
			linear-gradient(90deg, var(--color-primary-dim) 1px, transparent 1px),
			linear-gradient(var(--color-primary-dim) 1px, transparent 1px);
		background-size: 50px 50px;
		animation: sweepMove 20s linear infinite;
		mask-image: radial-gradient(circle at center, black 0%, transparent 70%);
	}

	@keyframes sweepMove {
		from { transform: rotate(15deg) translateY(0); }
		to { transform: rotate(15deg) translateY(-200px); }
	}

	.intro-title {
		font-family: var(--font-display);
		font-weight: 700;
		font-size: 2.2rem;
		letter-spacing: 0.15em;
		color: var(--color-primary);
		text-shadow: 0 0 20px var(--color-primary);
		z-index: 2;
	}

	.boot-logs {
		width: 80%;
		height: 70%;
		font-family: var(--font-mono);
		font-size: 0.75rem;
		color: var(--color-success);
		padding: 40px;
		display: flex;
		flex-direction: column;
		justify-content: flex-end;
		gap: 2px;
		overflow: hidden;
		mask-image: linear-gradient(to top, black 80%, transparent 100%);
	}

	.boot-line {
		animation: slideUpLog 0.1s ease-out;
		opacity: 0.7;
	}

	@keyframes slideUpLog {
		from { transform: translateY(10px); opacity: 0; }
		to { transform: translateY(0); opacity: 0.7; }
	}

	.zoom-effect {
		position: absolute;
		width: 100%;
		height: 100%;
		border: 2px solid var(--color-primary);
		animation: kineticZoom 4s ease-in forwards;
	}

	@keyframes kineticZoom {
		from { transform: scale(0.5); opacity: 0; }
		to { transform: scale(1.5); opacity: 1; border-width: 50px; }
	}

	.skip-hint {
		position: absolute;
		bottom: 40px;
		font-family: var(--font-display);
		font-size: 0.7rem;
		letter-spacing: 0.2em;
		color: var(--color-text-dim);
		animation: pulse 2s infinite;
	}

	@keyframes pulse {
		0%, 100% { opacity: 0.3; }
		50% { opacity: 0.7; }
	}
</style>

