<script lang="ts">
	import { getTosState } from '$lib/stores/ipc.svelte';

	import SplitLayout from './SplitLayout.svelte';
	import AiChat from './AiChat.svelte';
	import TacticalContextMenu from '../TacticalContextMenu.svelte';
	import { getPromptMode } from '$lib/stores/ui.svelte';
	import { longpress } from '$lib/actions/longpress';

	const state = $derived(getTosState());
	const activeSector = $derived(state.sectors[state.active_sector_index]);
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
			: (state.terminal_output || [])
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
</script>

<div class="command-hub">
	{#if splitLayout}
		<SplitLayout node={splitLayout} {activeHub} />
	{:else}
		<!-- Classic Dual-Column View (Fallback) -->
		<!-- Left Column: Context Chips -->
		<div class="left-column">
			{#if getPromptMode() === 'ai'}
				<AiChat />
			{:else if activeHub?.json_context}
				{@const ctx = activeHub.json_context}
				<div class="context-chip glass-panel">
					<div class="chip-title">JSON CONTEXT // {ctx.type || 'DATA'}</div>
					<div class="chip-row"><strong>NAME:</strong> {ctx.name || '--'}</div>
					{#if ctx.state}
						<div class="chip-row"><strong>STATE:</strong> <span class="ctx-state">{ctx.state}</span></div>
					{/if}
					{#if ctx.active_file}
						<div class="chip-row"><strong>FILE:</strong> {ctx.active_file}</div>
					{/if}
					{#if ctx.metadata}
						<div class="chip-metadata">
							{#each Object.entries(ctx.metadata) as [k, v]}
								<div><strong>{k.toUpperCase()}:</strong> {v}</div>
							{/each}
						</div>
					{/if}
				</div>
			{:else if activeHub?.shell_listing}
				{@const dir = activeHub.shell_listing}
				<div class="context-chip glass-panel">
					<div class="chip-title" style="color: var(--color-primary)">DIR PREVIEW // {dir.path}</div>
					<div class="directory-list">
						{#each dir.entries as entry}
							<div class="dir-entry">
								<span class="dir-type" class:is-dir={entry.is_dir}>{entry.is_dir ? '[DIR]' : ''}</span>
								<span class="dir-name" class:is-dir={entry.is_dir}>{entry.name}</span>
								{#if !entry.is_dir}
									<span class="dir-size">{entry.size} B</span>
								{/if}
							</div>
						{/each}
					</div>
				</div>
			{:else if activeHub?.activity_listing}
				{@const act = activeHub.activity_listing}
				<div class="context-chip glass-panel">
					<div class="chip-title" style="color: var(--color-warning)">SYSTEM ACTIVITY // RECENT</div>
					<div class="activity-list">
						{#each act.processes.slice(0, 10) as proc}
							<!-- svelte-ignore a11y_no_static_element_interactions -->
							<div 
								class="activity-item" 
								class:stopped={proc.status === 'stopped' || proc.status === 'sleeping'} 
								use:longpress
								onlongpress={(e) => handleContextMenu(e, proc)}
								oncontextmenu={(e) => handleContextMenu(e, proc)}
							>
								{#if proc.snapshot}
									<img class="proc-thumb snapshot" src="data:image/jpeg;base64,{proc.snapshot}" alt="Process Thumbnail" />
								{:else}
									<div class="proc-thumb icon">⊞</div>
								{/if}
								<div class="proc-info">
									<div class="proc-meta">
										<span class="proc-pid">PID {proc.pid}:</span>
										<span class="proc-name">{proc.name.toUpperCase()}</span>
									</div>
									<div class="proc-stats">
										CPU: {proc.cpu_usage.toFixed(1)}% | MEM: {(proc.mem_usage / 1024 / 1024).toFixed(1)} MB
									</div>
								</div>
							</div>
						{/each}
					</div>
				</div>
			{:else}
				<div class="context-chip glass-panel empty-chip">
					<div class="empty-text">AWAITING CONTEXT EXPORT...</div>
				</div>
			{/if}
		</div>

		<!-- Right Column: Terminal Output -->
		<div class="right-column">
			<div class="terminal-container">
				{#each termOutput as line}
					<div class="term-line" style="color: {priorityColor(line.priority)}">{line.text || ''}</div>
				{/each}
				<div class="cursor-blink">_</div>
			</div>
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
	}

	.dir-entry {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
		font-family: var(--font-mono);
		font-size: 0.8rem;
		padding: 0.1rem 0;
	}

	.dir-type {
		min-width: 2.5rem;
		color: var(--color-text-muted);
	}

	.dir-type.is-dir {
		color: var(--color-primary);
	}

	.dir-name.is-dir {
		font-weight: 700;
	}

	.dir-size {
		margin-left: auto;
		opacity: 0.5;
	}

	/* Activity Listing */
	.activity-list {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
		margin-top: var(--space-sm);
	}

	.activity-item {
		padding: var(--space-sm);
		background: rgba(0, 0, 0, 0.2);
		border-radius: var(--radius-sm);
		border: 1px solid var(--color-border);
		transition: opacity var(--transition-fast);
		display: flex;
		gap: var(--space-sm);
		align-items: center;
	}

	.activity-item.stopped {
		opacity: 0.4;
	}

	.proc-thumb {
		width: 32px;
		height: 32px;
		flex-shrink: 0;
		border-radius: 4px;
		border: 1px solid rgba(255, 255, 255, 0.1);
	}

	.proc-thumb.snapshot {
		object-fit: cover;
	}

	.proc-thumb.icon {
		display: flex;
		align-items: center;
		justify-content: center;
		background: rgba(255, 255, 255, 0.05);
		font-size: 1.2rem;
		color: var(--color-text-dim);
	}

	.proc-info {
		flex: 1;
		display: flex;
		flex-direction: column;
		min-width: 0;
	}

	.proc-meta {
		font-size: 0.8rem;
	}

	.proc-pid {
		color: var(--color-accent);
		font-weight: 600;
	}

	.proc-stats {
		font-family: var(--font-mono);
		font-size: 0.7rem;
		opacity: 0.6;
		margin-top: 0.15rem;
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
</style>
