<script lang="ts">
	import { getTosState, dirPickFile, dirPickDir, dirNavigate, sendCommand } from '$lib/stores/ipc.svelte';
	import { slide, fade } from 'svelte/transition';

	import SplitLayout from './SplitLayout.svelte';
	import AiChat from './AiChat.svelte';
	import WarningChip from '../WarningChip.svelte';
	import TacticalContextMenu from '../TacticalContextMenu.svelte';
	import { getPromptMode } from '$lib/stores/ui.svelte';
	import { longpress } from '$lib/actions/longpress';

	const tosState = $derived(getTosState());
	const activeSector = $derived(tosState.sectors[tosState.active_sector_index]);
	const activeHub = $derived(
		activeSector && activeSector.hubs[activeSector.active_hub_index]
			? activeSector.hubs[activeSector.active_hub_index]
			: null
	);

	const splitLayout = $derived(activeHub?.split_layout);

	// Terminal output — prefer hub-level, fall back to global
	const termOutput = $derived(
		activeHub?.terminal_output?.length
			? activeHub.terminal_output
			: (tosState.terminal_output || [])
	);

	function priorityColor(p: number): string {
		if (p >= 3) return 'var(--color-warning)';
		if (p === 2) return 'var(--color-primary)';
		if (p === 1) return 'var(--color-success)';
		return 'inherit';
	}

	let cmState = $state<{
		open: boolean;
		x: number;
		y: number;
		processName: string;
		processPid: number;
	}>({
		open: false,
		x: 0,
		y: 0,
		processName: '',
		processPid: 0
	});

	function handleContextMenu(e: MouseEvent | CustomEvent, proc: any) {
		e.preventDefault();
		const ev = e instanceof CustomEvent ? e.detail : e;
		cmState = {
			open: true,
			x: ev.clientX,
			y: ev.clientY,
			processName: proc.name,
			processPid: proc.pid
		};
	}

	function handleEntryClick(index: number, isDir: boolean) {
		if (isDir) {
			dirPickDir(index);
		} else {
			dirPickFile(index);
		}
	}

	function handleSearchResultClick(path: string, isDir: boolean) {
		if (isDir) {
			dirNavigate(path);
		} else {
			const lastSlash = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'));
			if (lastSlash !== -1) {
				const parent = path.substring(0, lastSlash);
				dirNavigate(parent);
			}
			sendCommand(`view_file:${path}`);
		}
	}

	function escapeHtml(text: string): string {
		return text
			.replace(/&/g, '&amp;')
			.replace(/</g, '&lt;')
			.replace(/>/g, '&gt;')
			.replace(/"/g, '&quot;')
			.replace(/'/g, '&#039;');
	}

	function cleanAnsiHtml(text: string): string {
		if (!text) return '';

		// Strip cursor positioning, screen clears, etc., but keep styling 'm' sequences
		let clean = text.replace(/\x1b\[\?[0-9]+[hl]/g, ''); 
		clean = clean.replace(/\x1b\[[0-9]*[A-IK-ORZcf-nqry=><]/g, ''); 

		if (!clean.includes('\x1b')) {
			return escapeHtml(clean);
		}

		const escapeCodes: Record<string, string> = {
			'0': 'reset',
			'1': 'font-weight: bold',
			'4': 'text-decoration: underline',
			'30': 'color: #1e1e1e', 
			'31': 'color: var(--color-danger)', 
			'32': 'color: var(--color-success)', 
			'33': 'color: var(--color-warning)', 
			'34': 'color: var(--color-primary)', 
			'35': 'color: #cc99cc', 
			'36': 'color: #00ffff', 
			'37': 'color: var(--color-text)', 
			'90': 'color: var(--color-text-muted)', 
			'91': 'color: #ff5555', 
			'92': 'color: #55ff55', 
			'93': 'color: #ffff55', 
			'94': 'color: #5555ff', 
			'95': 'color: #ff55ff', 
			'96': 'color: #55ffff', 
			'97': 'color: var(--color-text-bright)', 
		};

		const parts = clean.split(/\x1b\[/);
		let html = escapeHtml(parts[0]);
		let openSpans = 0;

		for (let i = 1; i < parts.length; i++) {
			const part = parts[i];
			const mIdx = part.indexOf('m');
			if (mIdx === -1) {
				html += '[' + escapeHtml(part);
				continue;
			}

			const codeString = part.substring(0, mIdx);
			const content = part.substring(mIdx + 1);
			const codes = codeString.split(';');

			const hasReset = codes.includes('0') || codeString === '';

			if (hasReset) {
				while (openSpans > 0) {
					html += '</span>';
					openSpans--;
				}
			}

			const styles: string[] = [];
			for (const rawCode of codes) {
				if (rawCode === '0' || rawCode === '') continue;
				const normalizedCode = parseInt(rawCode, 10).toString();
				const style = escapeCodes[normalizedCode];
				if (style) {
					styles.push(style);
				}
			}

			if (styles.length > 0) {
				html += `<span style="${styles.join(';')}">`;
				openSpans++;
			}

			html += escapeHtml(content);
		}

		while (openSpans > 0) {
			html += '</span>';
			openSpans--;
		}

		return html;
	}
</script>

<div class="command-hub command-hub-view">
	<WarningChip />

	{#if splitLayout}
		<SplitLayout node={splitLayout} {activeHub} />
	{:else}
		<!-- Classic Dual-Column View (Fallback) -->
		<!-- Left Column: Context Chips -->
		<div class="left-column">
			{#if activeHub?.json_context}
				{@const ctx = activeHub.json_context}
				<div aria-roledescription="chip" class="context-chip glass-panel" transition:slide>
					<div aria-roledescription="chip" class="chip-title">JSON CONTEXT // {ctx.type || 'DATA'}</div>
					<div aria-roledescription="chip" class="chip-row"><strong>NAME:</strong> {ctx.name || '--'}</div>
					{#if ctx.state}
						<div aria-roledescription="chip" class="chip-row"><strong>STATE:</strong> <span class="ctx-state">{ctx.state}</span></div>
					{/if}
					{#if ctx.active_file}
						<div aria-roledescription="chip" class="chip-row"><strong>FILE:</strong> {ctx.active_file}</div>
					{/if}
					{#if ctx.metadata}
						<div aria-roledescription="chip" class="chip-metadata">
							{#each Object.entries(ctx.metadata) as [k, v]}
								<div><strong>{k.toUpperCase()}:</strong> {v}</div>
							{/each}
						</div>
					{/if}
				</div>
			{/if}

			{#if activeHub?.shell_listing}
				{@const dir = activeHub.shell_listing}
				<div aria-roledescription="chip" class="context-chip glass-panel" transition:slide>
					<div aria-roledescription="chip" class="chip-title" style="color: var(--color-primary)">DIR PREVIEW // {dir.path}</div>
					
					{#if activeHub?.staged_command}
						<div class="staging-banner" transition:fade>
							<div class="banner-tag">STAGING</div>
							<div class="staged-content">
								<span class="staged-cmd">{activeHub.staged_command}</span>
								{#if activeHub.ai_explanation}
									<span class="staged-hint">— {activeHub.ai_explanation}</span>
								{/if}
							</div>
						</div>
					{/if}

					<div class="directory-list">
						{#each dir.entries as entry, i}
							<button 
								class="dir-entry interactive" 
								onclick={() => handleEntryClick(i, entry.is_dir)}
							>
								<span class="dir-type" class:is-dir={entry.is_dir}>{entry.is_dir ? '[DIR]' : ''}</span>
								<span class="dir-name" class:is-dir={entry.is_dir}>{entry.name}</span>
								{#if !entry.is_dir}
									<span class="dir-size">{(entry.size / 1024).toFixed(1)} KB</span>
								{/if}
							</button>
						{/each}
					</div>
				</div>
			{/if}

			{#if !activeHub?.json_context && !activeHub?.shell_listing}
				<div aria-roledescription="chip" class="context-chip glass-panel empty-chip">
					<div class="empty-text">AWAITING CONTEXT EXPORT...</div>
				</div>
			{/if}
		</div>

		<!-- Right Column: Dedicated Panel based on promptMode -->
		<div class="right-column">
			{#if getPromptMode() === 'ai'}
				<div class="ai-panel-wrapper" transition:fade={{ duration: 150 }}>
					<AiChat />
				</div>
			{:else if getPromptMode() === 'search'}
				<div class="search-panel-container glass-panel" transition:fade={{ duration: 150 }}>
					<div class="search-panel-header">
						<span class="search-panel-title">✦ SECTOR QUERY ANALYSIS ENGINE // SEARCH_RESULTS</span>
						<span class="search-panel-meta">
							{activeHub?.search_results?.reduce((acc, r) => acc + r.matches.length, 0) || 0} HITS IDENTIFIED
						</span>
					</div>
					
					<div class="search-panel-body">
						{#if !activeHub?.search_results || activeHub.search_results.length === 0}
							<div class="search-empty">
								<div class="search-empty-icon">🔍</div>
								<div class="search-empty-text">AWAITING QUERY EMISSION...</div>
								<div class="search-empty-sub">Type a search pattern in the [SEARCH] prompt.</div>
							</div>
						{:else}
							{#each activeHub.search_results as result}
								<div class="search-group glass-panel">
									<div class="group-header">
										<span class="group-dot"></span>
										<span class="group-title">{result.source_sector.toUpperCase()}</span>
										<span class="group-count">{result.matches.length} MATCHES</span>
									</div>
									<div class="group-list">
										{#each result.matches as m}
											{@const parts = m.match(/(.*)\s+\[(FILE|DIR)\]$/) || [m, m, 'FILE']}
											{@const path = parts[1]}
											{@const type = parts[2]}
											<button 
												class="search-item interactive" 
												onclick={() => handleSearchResultClick(path, type === 'DIR')}
											>
												<span class="item-icon" class:is-dir={type === 'DIR'}>
													{type === 'DIR' ? '📁' : '📄'}
												</span>
												<span class="item-path">{path}</span>
												<span class="item-badge" class:badge-dir={type === 'DIR'} class:badge-file={type === 'FILE'}>
													{type}
												</span>
											</button>
										{/each}
									</div>
								</div>
							{/each}
						{/if}
					</div>
				</div>
			{:else}
				<div class="terminal-container" transition:fade={{ duration: 150 }}>
					{#each termOutput as line}
						<div class="term-line" style="color: {priorityColor(line.priority)}">{@html cleanAnsiHtml(line.text || '')}</div>
					{/each}
					<div class="cursor-blink">_</div>
				</div>
			{/if}
		</div>
	{/if}

	{#if cmState.open}
		<TacticalContextMenu 
			x={cmState.x} 
			y={cmState.y} 
			processName={cmState.processName} 
			processPid={cmState.processPid} 
			onClose={() => cmState.open = false} 
		/>
	{/if}
</div>


<style>
	.command-hub {
		position: relative;
		display: grid;
		grid-template-columns: 1fr 1.5fr;
		gap: var(--space-md);
		height: 100%;
		padding: var(--space-md);
		animation: scaleIn 0.4s cubic-bezier(0.16, 1, 0.3, 1);
	}

	.left-column {
		display: flex;
		flex-direction: column;
		overflow-y: auto;
	}

	.right-column {
		display: flex;
		flex-direction: column;
	}

	.context-chip {
		padding: var(--space-md);
		flex: 1;
	}

	.empty-chip {
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.empty-text {
		opacity: 0.4;
		font-style: italic;
		font-weight: 300;
		font-size: 0.85rem;
	}

	.ctx-state {
		display: inline-block;
		padding: 0.1rem 0.5rem;
		background: rgba(0, 0, 0, 0.4);
		color: var(--color-success);
		border-radius: var(--radius-pill);
		border: 1px solid rgba(255, 255, 255, 0.1);
		font-size: 0.75rem;
	}

	.chip-metadata {
		margin-top: var(--space-sm);
		font-size: 0.8rem;
		color: var(--color-text-dim);
	}

	/* Directory Listing */
	.directory-list {
		margin-top: var(--space-sm);
		display: flex;
		flex-direction: column;
	}

	.dir-entry {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		font-family: var(--font-mono);
		font-size: 0.8rem;
		padding: 0.25rem 0.5rem;
		border-radius: var(--radius-sm);
		width: 100%;
		text-align: left;
		background: transparent;
		border: none;
		color: inherit;
		cursor: default;
	}

	.dir-entry.interactive {
		cursor: pointer;
		transition: background var(--transition-fast), transform var(--transition-fast);
	}

	.dir-entry.interactive:hover {
		background: rgba(255, 255, 255, 0.05);
		transform: translateX(4px);
	}

	.dir-entry.interactive:active {
		background: rgba(var(--color-primary-rgb), 0.2);
		transform: translateX(2px);
	}

	.dir-type {
		min-width: 2.5rem;
		color: var(--color-text-muted);
	}

	.dir-type.is-dir {
		color: var(--color-primary);
	}

	.dir-name {
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.dir-name.is-dir {
		font-weight: 700;
	}

	.dir-size {
		margin-left: auto;
		opacity: 0.5;
	}

	/* Staging Banner */
	.staging-banner {
		margin: var(--space-sm) 0;
		padding: var(--space-sm) var(--space-md);
		background: linear-gradient(90deg, rgba(var(--color-primary-rgb), 0.15), transparent);
		border-left: 3px solid var(--color-primary);
		border-radius: var(--radius-sm);
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.banner-tag {
		font-size: 0.65rem;
		font-weight: 800;
		letter-spacing: 0.1em;
		color: var(--color-primary);
		opacity: 0.8;
	}

	.staged-content {
		display: flex;
		align-items: baseline;
		gap: var(--space-sm);
		overflow: hidden;
	}

	.staged-cmd {
		font-family: var(--font-mono);
		font-size: 0.9rem;
		color: var(--color-text-bright);
		white-space: nowrap;
	}

	.staged-hint {
		font-size: 0.75rem;
		color: var(--color-text-dim);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}



	/* Terminal */
	.terminal-container {
		font-family: var(--font-mono);
		font-size: 0.85rem;
		line-height: 1.6;
		height: 100%;
		overflow-y: auto;
		padding: var(--space-md);
		background: rgba(0, 0, 0, 0.3);
		border-radius: var(--radius-md);
		border: 1px solid var(--color-border);
	}

	.cursor-blink {
		animation: blink 1s infinite;
		color: var(--color-primary);
	}

	/* Premium AI & Search Panel wrappers */
	.ai-panel-wrapper {
		height: 100%;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.search-panel-container {
		height: 100%;
		display: flex;
		flex-direction: column;
		overflow: hidden;
		background: rgba(0, 0, 0, 0.3);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-md);
	}

	.search-panel-header {
		padding: var(--space-sm) var(--space-md);
		background: rgba(255, 255, 255, 0.05);
		border-bottom: 1px solid var(--color-border);
		display: flex;
		justify-content: space-between;
		align-items: center;
		flex-shrink: 0;
	}

	.search-panel-title {
		font-family: var(--font-display);
		font-weight: 700;
		font-size: 0.75rem;
		color: var(--color-accent);
		letter-spacing: 0.05em;
	}

	.search-panel-meta {
		font-family: var(--font-mono);
		font-size: 0.65rem;
		opacity: 0.6;
		color: var(--color-success);
	}

	.search-panel-body {
		flex: 1;
		overflow-y: auto;
		padding: var(--space-md);
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
	}

	.search-empty {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		opacity: 0.4;
		text-align: center;
		gap: var(--space-xs);
	}

	.search-empty-icon {
		font-size: 2.5rem;
		animation: searchPulseAnimation 2s infinite ease-in-out;
	}

	.search-empty-text {
		font-family: var(--font-display);
		font-weight: 700;
		font-size: 0.9rem;
		letter-spacing: 0.05em;
	}

	.search-empty-sub {
		font-family: var(--font-mono);
		font-size: 0.75rem;
	}

	.search-group {
		padding: var(--space-sm) var(--space-md);
		background: rgba(255, 255, 255, 0.02);
		border: 1px solid rgba(255, 255, 255, 0.05);
		border-radius: var(--radius-sm);
	}

	.group-header {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		border-bottom: 1px solid rgba(255, 255, 255, 0.05);
		padding-bottom: var(--space-xs);
		margin-bottom: var(--space-sm);
	}

	.group-dot {
		width: 6px;
		height: 6px;
		background: var(--color-primary);
		border-radius: 50%;
		box-shadow: 0 0 8px var(--color-primary);
	}

	.group-title {
		font-family: var(--font-display);
		font-weight: 700;
		font-size: 0.7rem;
		color: var(--color-text-bright);
		letter-spacing: 0.05em;
	}

	.group-count {
		margin-left: auto;
		font-family: var(--font-mono);
		font-size: 0.6rem;
		opacity: 0.5;
	}

	.group-list {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.search-item {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		font-family: var(--font-mono);
		font-size: 0.8rem;
		padding: 0.35rem 0.5rem;
		border-radius: var(--radius-sm);
		width: 100%;
		text-align: left;
		background: transparent;
		border: none;
		color: inherit;
		cursor: pointer;
		transition: background var(--transition-fast), transform var(--transition-fast), border-color var(--transition-fast);
		border: 1px solid transparent;
	}

	.search-item:hover {
		background: rgba(255, 255, 255, 0.05);
		border-color: rgba(255, 255, 255, 0.1);
		transform: translateX(4px);
	}

	.search-item:active {
		background: rgba(var(--color-primary-rgb), 0.15);
		transform: translateX(2px);
	}

	.item-icon {
		opacity: 0.6;
		font-size: 0.9rem;
	}

	.item-icon.is-dir {
		color: var(--color-primary);
		opacity: 0.9;
	}

	.item-path {
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		flex: 1;
		color: var(--color-text);
	}

	.item-badge {
		font-size: 0.6rem;
		font-weight: 700;
		padding: 0.1rem 0.35rem;
		border-radius: 3px;
		border: 1px solid transparent;
	}

	.badge-file {
		background: rgba(0, 255, 255, 0.1);
		color: var(--color-primary);
		border-color: rgba(0, 255, 255, 0.2);
	}

	.badge-dir {
		background: rgba(247, 168, 51, 0.1);
		color: var(--color-secondary);
		border-color: rgba(247, 168, 51, 0.2);
	}

	@keyframes searchPulseAnimation {
		0%, 100% { transform: scale(1); opacity: 0.4; }
		50% { transform: scale(1.1); opacity: 0.8; }
	}
</style>
