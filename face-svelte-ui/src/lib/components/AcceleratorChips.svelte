<script lang="ts">
	// Phase 1.6 — accelerator chips: a touch-friendly overlay whose
	// whole job is access + reduced typing. The terminal stays a
	// completely normal terminal; these chips are generated from
	// context the Brain already holds (the shell hub's directory
	// listing, refreshed by the Brain on cwd change and on command
	// completion) and tapping one types into the terminal for you:
	// a directory chip runs `cd`, a file chip inserts the file's
	// name, a breadcrumb chip cds to that ancestor.
	//
	// Phase 1.7 — dynamic chips: the panel reacts while you type.
	// The Brain tracks the shell's input line and publishes the
	// fragment being typed (hub.chip_filter); the strip narrows
	// to matching entries until the line is submitted or cleared.
	// Dotfiles are hidden by default (like `ls`); the strip's own
	// `.*` switch flips hub.chips_show_hidden in the Brain, so
	// every face agrees.
	import {
		getTosState,
		getConnectionState,
		getConnectionScope,
		sendCommand,
		sendTerminalCommand,
		sendTerminalText,
		shellQuote
	} from '$lib/stores/ipc.svelte';

	const tosState = $derived(getTosState());
	const conn = $derived(getConnectionState());
	const scope = $derived(getConnectionScope());
	const activeSector = $derived(tosState.sectors[tosState.active_sector_index]);
	const activeHub = $derived(activeSector?.hubs[activeSector.active_hub_index]);
	const listing = $derived(activeHub?.shell_listing ?? null);
	const cwd = $derived(listing?.path ?? activeHub?.current_directory ?? '');
	const chipFilter = $derived(activeHub?.chip_filter ?? null);
	const showHidden = $derived(activeHub?.chips_show_hidden ?? false);

	// The entries actually shown: dotfiles out unless the panel's
	// switch says otherwise; while a fragment is being typed,
	// matches only — prefix matches first, then substring
	// matches. The listing arrives dirs-first and filtering
	// preserves that order inside each class.
	const visibleEntries = $derived.by(() => {
		if (!listing) return [];
		let entries = listing.entries;
		if (!showHidden) entries = entries.filter((e) => !e.name.startsWith('.'));
		const frag = chipFilter?.toLowerCase() ?? '';
		if (frag) {
			const prefix = entries.filter((e) => e.name.toLowerCase().startsWith(frag));
			const within = entries.filter(
				(e) => !e.name.toLowerCase().startsWith(frag) && e.name.toLowerCase().includes(frag)
			);
			entries = [...prefix, ...within];
		}
		return entries;
	});

	function toggleHidden() {
		sendCommand(`chips_set_show_hidden:${!showHidden}`);
	}

	// Breadcrumb segments for the current directory.
	const crumbs = $derived.by(() => {
		if (!cwd) return [] as { label: string; path: string }[];
		const out: { label: string; path: string }[] = [{ label: '/', path: '/' }];
		let acc = '';
		for (const part of cwd.split('/').filter(Boolean)) {
			acc += '/' + part;
			out.push({ label: part, path: acc });
		}
		return out;
	});

	function tapCrumb(path: string) {
		sendTerminalCommand(`cd ${shellQuote(path)}`);
	}
	function tapDir(name: string) {
		sendTerminalCommand(`cd ${shellQuote(name)}`);
	}
	function tapFile(name: string) {
		sendTerminalText(`${shellQuote(name)} `);
	}

	function sendFaceCommand(cmd: 'toggle-prompt' | 'show-prompt' | 'hide-prompt') {
		if (typeof window === 'undefined') return;
		const wk = (window as unknown as {
			webkit?: { messageHandlers?: { tosFace?: { postMessage: (m: string) => void } } };
		}).webkit;
		wk?.messageHandlers?.tosFace?.postMessage(cmd);
		window.dispatchEvent(new CustomEvent('tos:face-command', { detail: cmd }));
	}

	const touchKeys: { id: string; label: string; title: string; bytes: string }[] = [
		{ id: 'tab', label: 'Tab', title: 'Send Tab completion', bytes: '\t' },
		{ id: 'up', label: '↑', title: 'Previous command', bytes: '\x1b[A' },
		{ id: 'down', label: '↓', title: 'Next command', bytes: '\x1b[B' },
		{ id: 'ctrlc', label: '^C', title: 'Interrupt (Ctrl+C)', bytes: '\x03' },
		{ id: 'enter', label: '↵', title: 'Send Enter', bytes: '\n' }
	];
</script>

<div class="accel-strip" data-testid="accel-chips">
	<div class="accel-status" title="Brain connection">
		<span class="status-dot" class:online={conn === 'connected'}></span>
		<span class="status-label">{activeSector?.name ?? 'TOS'}</span>
		{#if conn === 'connected'}
			<span class="status-scope" data-testid="accel-scope">{scope === 'remote' ? 'REMOTE' : 'LOCAL'}</span>
		{/if}
	</div>

	<button
		type="button"
		class="accel-chip prompt-toggle"
		data-testid="accel-prompt-toggle"
		aria-label="Toggle command prompt"
		title="Summon or dismiss prompt"
		onclick={() => sendFaceCommand('toggle-prompt')}>▸ CMD</button
	>

	{#if conn !== 'connected'}
		<span class="accel-note">{conn === 'connecting' ? 'CONNECTING…' : 'DISCONNECTED'}</span>
	{:else if listing}
		<div class="accel-keys" data-testid="accel-touch-keys" role="group" aria-label="Terminal keys">
			{#each touchKeys as key (key.id)}
				<button
					type="button"
					class="accel-chip key"
					data-testid={`accel-key-${key.id}`}
					title={key.title}
					onclick={() => sendTerminalText(key.bytes)}>{key.label}</button
				>
			{/each}
		</div>
		<nav class="accel-crumbs" aria-label="Current directory">
			{#each crumbs as crumb (crumb.path)}
				<button
					type="button"
					class="accel-chip crumb"
					data-testid="accel-crumb"
					onclick={() => tapCrumb(crumb.path)}>{crumb.label}</button
				>
			{/each}
		</nav>
		<button
			type="button"
			class="accel-chip toggle"
			class:on={showHidden}
			data-testid="accel-toggle-hidden"
			title="Show or hide dotfiles"
			onclick={toggleHidden}>.*</button
		>
		{#if chipFilter}
			<span class="accel-filter" data-testid="accel-filter">⌕ {chipFilter}</span>
		{/if}
		<div class="accel-entries">
			{#each visibleEntries as entry (entry.name)}
				{#if entry.is_dir}
					<button
						type="button"
						class="accel-chip dir"
						data-testid="accel-chip-dir"
						onclick={() => tapDir(entry.name)}>📁 {entry.name}</button
					>
				{:else}
					<button
						type="button"
						class="accel-chip file"
						data-testid="accel-chip-file"
						onclick={() => tapFile(entry.name)}>📄 {entry.name}</button
					>
				{/if}
			{/each}
			{#if chipFilter && visibleEntries.length === 0}
				<span class="accel-note" data-testid="accel-nomatch">no matches</span>
			{/if}
		</div>
	{:else}
		<span class="accel-note">{cwd}</span>
	{/if}
</div>

<style>
	.accel-strip {
		display: flex;
		align-items: center;
		gap: 10px;
		height: 100%;
		padding: 0 10px;
		overflow-x: auto;
		overflow-y: hidden;
		white-space: nowrap;
		scrollbar-width: none;
		touch-action: pan-x;
		user-select: none;
		-webkit-user-select: none;
	}
	.accel-strip::-webkit-scrollbar {
		display: none;
	}
	.accel-status {
		display: flex;
		align-items: center;
		gap: 6px;
		flex: none;
		font-size: 11px;
		letter-spacing: 0.08em;
		color: var(--color-text, #e0f7fa);
	}
	.status-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: #ff5252;
	}
	.status-scope {
		font-size: 10px;
		letter-spacing: 0.08em;
		opacity: 0.75;
	}
	.status-dot.online {
		background: #69f0ae;
	}
	.accel-keys,
	.accel-crumbs,
	.accel-entries {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.accel-chip {
		flex: none;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		min-height: 44px;
		min-width: 44px;
		padding: 8px 14px;
		border-radius: 999px;
		border: 1px solid rgba(0, 229, 255, 0.35);
		background: rgba(10, 25, 35, 0.72);
		color: var(--color-text, #e0f7fa);
		font-size: 13px;
		cursor: pointer;
		backdrop-filter: blur(6px);
		touch-action: manipulation;
		-webkit-tap-highlight-color: transparent;
		transition: transform 80ms ease, border-color 120ms ease, background-color 120ms ease;
	}
	.accel-chip:hover {
		border-color: var(--color-primary, #00e5ff);
	}
	.accel-chip:active {
		transform: scale(0.96);
		background: rgba(0, 229, 255, 0.22);
		border-color: var(--color-primary, #00e5ff);
	}
	.accel-chip.prompt-toggle {
		border-color: var(--color-primary, #00e5ff);
		color: var(--color-primary, #00e5ff);
		font-weight: 700;
		letter-spacing: 0.06em;
	}
	.accel-chip.key {
		font-family: var(--font-mono, monospace);
		padding: 8px 12px;
	}
	.accel-chip.crumb {
		border-style: dashed;
		opacity: 0.85;
	}
	.accel-chip.toggle {
		font-family: var(--font-mono, monospace);
		padding: 8px 12px;
		opacity: 0.75;
	}
	.accel-chip.toggle.on {
		opacity: 1;
		border-color: var(--color-primary, #00e5ff);
		color: var(--color-primary, #00e5ff);
	}
	.accel-filter {
		flex: none;
		font-size: 12px;
		letter-spacing: 0.04em;
		color: var(--color-primary, #00e5ff);
		opacity: 0.9;
	}
	.accel-chip.dir {
		color: var(--color-primary, #00e5ff);
	}
	.accel-note {
		font-size: 12px;
		letter-spacing: 0.08em;
		opacity: 0.7;
		color: var(--color-text, #e0f7fa);
	}
</style>
