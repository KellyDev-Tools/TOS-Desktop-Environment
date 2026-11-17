<script lang="ts">
	// Phase 1.6 — accelerator chips: a touch-friendly overlay whose
	// whole job is access + reduced typing. The terminal stays a
	// completely normal terminal; these chips are generated from
	// context the Brain already holds (the shell hub's directory
	// listing, refreshed by the Brain on cwd change and on command
	// completion) and tapping one types into the terminal for you:
	// a directory chip runs `cd`, a file chip inserts the file's
	// name, a breadcrumb chip cds to that ancestor.
	import {
		getTosState,
		getConnectionState,
		getConnectionScope,
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
</script>

<div class="accel-strip" data-testid="accel-chips">
	<div class="accel-status" title="Brain connection">
		<span class="status-dot" class:online={conn === 'connected'}></span>
		<span class="status-label">{activeSector?.name ?? 'TOS'}</span>
		{#if conn === 'connected'}
			<span class="status-scope" data-testid="accel-scope">{scope === 'remote' ? 'REMOTE' : 'LOCAL'}</span>
		{/if}
	</div>

	{#if conn !== 'connected'}
		<span class="accel-note">{conn === 'connecting' ? 'CONNECTING…' : 'DISCONNECTED'}</span>
	{:else if listing}
		<nav class="accel-crumbs" aria-label="Current directory">
			{#each crumbs as crumb (crumb.path)}
				<button
					class="accel-chip crumb"
					data-testid="accel-crumb"
					onclick={() => tapCrumb(crumb.path)}>{crumb.label}</button
				>
			{/each}
		</nav>
		<div class="accel-entries">
			{#each listing.entries as entry (entry.name)}
				{#if entry.is_dir}
					<button
						class="accel-chip dir"
						data-testid="accel-chip-dir"
						onclick={() => tapDir(entry.name)}>📁 {entry.name}</button
					>
				{:else}
					<button
						class="accel-chip file"
						data-testid="accel-chip-file"
						onclick={() => tapFile(entry.name)}>📄 {entry.name}</button
					>
				{/if}
			{/each}
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
	.accel-crumbs,
	.accel-entries {
		display: flex;
		align-items: center;
		gap: 6px;
	}
	.accel-chip {
		flex: none;
		padding: 7px 12px;
		border-radius: 999px;
		border: 1px solid rgba(0, 229, 255, 0.35);
		background: rgba(10, 25, 35, 0.72);
		color: var(--color-text, #e0f7fa);
		font-size: 12px;
		cursor: pointer;
		backdrop-filter: blur(6px);
	}
	.accel-chip:hover {
		border-color: var(--color-primary, #00e5ff);
	}
	.accel-chip.crumb {
		border-style: dashed;
		opacity: 0.85;
	}
	.accel-chip.dir {
		color: var(--color-primary, #00e5ff);
	}
	.accel-note {
		font-size: 11px;
		letter-spacing: 0.08em;
		opacity: 0.7;
		color: var(--color-text, #e0f7fa);
	}
</style>
