<script lang="ts">
	import type { EditorPaneState } from '$lib/stores/tos-state.svelte';
	import { submitCommand } from '$lib/stores/ipc.svelte';

	let { editorState, paneId, onClose }: { editorState: EditorPaneState; paneId: string, onClose: () => void } = $props();

	let contextScope = $state('visible'); // default
</script>

<div class="ai-context-panel">
	<div class="panel-header">
		<span class="panel-title">AI CONTEXT</span>
		<div class="panel-actions">
			<button class="icon-btn" title="Refresh Context" aria-label="Refresh Context">⟳</button>
			<button class="icon-btn" onclick={onClose} title="Close Panel" aria-label="Close Panel">✕</button>
		</div>
	</div>

	<div class="panel-body">
		<div class="context-summary section">
			<div class="file-focus">📄 {editorState.file_path.split('/').pop()} : Ln {editorState.cursor_line + 1}</div>
			<div class="error-summary">⚠ 0 errors in context</div>
		</div>

		<div class="context-scope section">
			<div class="section-title">CONTEXT SCOPE</div>
			<label class="scope-radio">
				<input type="radio" value="visible" bind:group={contextScope} />
				<span>Visible range (default)</span>
			</label>
			<label class="scope-radio">
				<input type="radio" value="selection" bind:group={contextScope} />
				<span>Selection only</span>
			</label>
			<label class="scope-radio">
				<input type="radio" value="full" bind:group={contextScope} />
				<span>Full file</span>
			</label>
			<label class="scope-radio">
				<input type="radio" value="imports" bind:group={contextScope} />
				<span>Full file + imports</span>
			</label>
		</div>

		<div class="active-annotations section">
			<div class="section-title">ACTIVE ANNOTATIONS</div>
			{#if (!editorState.annotations || editorState.annotations.length === 0)}
				<div class="empty-state">No active annotations in scope.</div>
			{:else}
				{#each editorState.annotations as annotation, i}
					<div class="annotation-item" class:error={annotation.severity === 'error'}>
						<div class="annotation-text">› Line {annotation.line + 1} — {annotation.message}</div>
						<div class="annotation-actions">
							<button onclick={() => submitCommand(`!ipc ai_submit:Explain line ${annotation.line + 1}`)}>[Ask AI]</button>
							<button onclick={() => submitCommand(`!ipc ai_submit:Fix line ${annotation.line + 1} error`)}>[Fix]</button>
							<!-- We map the clear command eventually -->
						</div>
					</div>
				{/each}
			{/if}
		</div>

		<div class="recent-edits section">
			<div class="section-title">RECENT AI EDITS</div>
			<div class="recent-edit-item">
				<div class="edit-name">› stage_context_bindings</div>
				<div class="edit-meta">Just now · <button class="link-btn">[Undo]</button></div>
			</div>
		</div>
	</div>
</div>

<style>
	.ai-context-panel {
		width: 300px;
		height: 100%;
		display: flex;
		flex-direction: column;
		background: rgba(10, 10, 15, 0.95);
		border-left: 1px solid var(--color-border);
		font-family: var(--font-mono);
		font-size: 0.8rem;
		color: var(--color-text);
		overflow: hidden;
	}

	.panel-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: var(--space-sm) var(--space-md);
		border-bottom: 1px solid var(--color-border);
		background: rgba(255, 255, 255, 0.02);
	}

	.panel-title {
		font-family: var(--font-display);
		font-weight: 700;
		color: var(--color-primary);
		letter-spacing: 0.05em;
	}

	.panel-actions {
		display: flex;
		gap: 8px;
	}

	.icon-btn {
		background: none;
		border: none;
		color: var(--color-text-dim);
		cursor: pointer;
		font-size: 1rem;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 2px;
	}

	.icon-btn:hover {
		color: var(--color-text-bright);
	}

	.panel-body {
		flex: 1;
		overflow-y: auto;
		padding: var(--space-md);
		display: flex;
		flex-direction: column;
		gap: 20px;
	}

	.section {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.file-focus {
		color: var(--color-text-bright);
		font-weight: bold;
	}

	.error-summary {
		color: var(--color-warning);
	}

	.section-title {
		font-family: var(--font-display);
		font-size: 0.65rem;
		font-weight: 700;
		color: var(--color-secondary);
		border-bottom: 1px solid rgba(255, 255, 255, 0.1);
		padding-bottom: 4px;
		margin-bottom: 4px;
		letter-spacing: 0.05em;
	}

	.scope-radio {
		display: flex;
		align-items: center;
		gap: 8px;
		cursor: pointer;
		color: var(--color-text-dim);
	}

	.scope-radio:hover {
		color: var(--color-text-bright);
	}

	.empty-state {
		color: var(--color-text-dim);
		font-style: italic;
		font-size: 0.75rem;
	}

	.annotation-item, .recent-edit-item {
		background: rgba(255, 255, 255, 0.03);
		padding: 8px;
		border-left: 2px solid var(--color-primary);
		font-size: 0.75rem;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.annotation-item.error {
		border-left-color: var(--color-warning);
	}

	.annotation-text, .edit-name {
		color: var(--color-text-bright);
	}

	.annotation-actions button, .link-btn {
		background: none;
		border: none;
		color: var(--color-primary);
		cursor: pointer;
		font-family: var(--font-mono);
		font-size: 0.75rem;
		padding: 0;
	}

	.annotation-actions button:hover, .link-btn:hover {
		text-decoration: underline;
		color: var(--color-primary-light);
	}

	.edit-meta {
		color: var(--color-text-dim);
	}
</style>
