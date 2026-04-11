<script lang="ts">
	import { onMount } from 'svelte';
	import type { EditorPaneState, Hub } from '$lib/stores/tos-state.svelte';
	import { submitCommand } from '$lib/stores/ipc.svelte';
	import Prism from 'prismjs';
	import AiContextPanel from './AiContextPanel.svelte';
	import 'prismjs/themes/prism-tomorrow.css'; // VSCode-like dark theme
	// Import common languages to ensure they exist
	import 'prismjs/components/prism-javascript';
	import 'prismjs/components/prism-typescript';
	import 'prismjs/components/prism-rust';
	import 'prismjs/components/prism-python';
	import 'prismjs/components/prism-bash';
	import 'prismjs/components/prism-json';
	import 'prismjs/components/prism-css';
	import 'prismjs/components/prism-markup'; // HTML
	import 'prismjs/components/prism-markup'; // HTML

	let { editorState, activeHub, paneId, paneCwd }: { editorState: EditorPaneState; activeHub: Hub | null; paneId: string, paneCwd?: string } = $props();

	// Local state for debounced typed content
	let localContent = $state('');
	let textareaEl: HTMLTextAreaElement | null = $state(null);
	let editorContentEl: HTMLDivElement | null = $state(null);
	let syncTimeout: any;

	let showAiContext = $state(false);
	
	// Trust State
	let pendingTrustTarget: string | null = $state(null);
	let pendingTrustAction: (() => void) | null = null;
	let prevAnnotationsLength = $state(0);

	const annotationsByLine = $derived.by(() => {
		const map = new Map<number, typeof editorState.annotations>();
		for (const ann of editorState.annotations || []) {
			if (!map.has(ann.line)) map.set(ann.line, []);
			map.get(ann.line)!.push(ann);
		}
		return map;
	});

	$effect(() => {
		// Sync initial or brain-updated state
		if (!editorState.dirty && editorState.content !== localContent) {
			localContent = editorState.content;
		}
	});

	$effect(() => {
		// Semantic Scroll Sync for new annotations
		const currentAnnotations = editorState.annotations || [];
		if (currentAnnotations.length > prevAnnotationsLength) {
			const newest = currentAnnotations[currentAnnotations.length - 1];
			setTimeout(() => {
				const lineEl = document.getElementById(`editor-${paneId}-line-${newest.line}`);
				if (lineEl) {
					lineEl.scrollIntoView({ behavior: 'smooth', block: 'center' });
					lineEl.classList.add('pulse-amber');
					setTimeout(() => lineEl.classList.remove('pulse-amber'), 2000);
				}
			}, 100);
		}
		prevAnnotationsLength = currentAnnotations.length;
	});

	$effect(() => {
		// Restore scroll_offset on mount or when switching files
		if (editorContentEl && editorState.scroll_offset > 0 && !textareaEl?.matches(':focus')) {
			const lineEl = document.getElementById(`editor-${paneId}-line-${editorState.scroll_offset}`);
			if (lineEl) {
				editorContentEl.scrollTop = lineEl.offsetTop;
			}
		}
	});

	function syncContext(line: number, col: number, scrollOffset?: number) {
		clearTimeout(syncTimeout);
		syncTimeout = setTimeout(() => {
			const offset = scrollOffset ?? (editorContentEl ? Math.floor(editorContentEl.scrollTop / 24) : 0);
			const payload = {
				content: localContent,
				cursor_line: line,
				cursor_col: col,
				scroll_offset: offset
			};
			submitCommand(`!ipc editor_context_update:${paneId};${JSON.stringify(payload)}`);
		}, 300); // 300ms debounce
	}

	function handleScroll(e: Event) {
		const target = e.target as HTMLDivElement;
		const line = Math.floor(target.scrollTop / 24); // 24px is line height
		syncContext(editorState.cursor_line, editorState.cursor_col, line);
	}

	function handleLineClick(lineIndex: number) {
		// Send the tapped line to the AI
		const lineContent = localContent.split('\n')[lineIndex];
		const msg = `Provide Context & Analysis for this code block:\nLine ${lineIndex + 1}:\n\`\`\`${editorState.language || 'text'}\n${lineContent}\n\`\`\``;
		submitCommand(`!ipc ai_submit:${msg}`);
	}

	function handleInput(e: Event) {
		const target = e.target as HTMLTextAreaElement;
		localContent = target.value;
		
		// Calculate cursor line and col
		const textBeforeCursor = localContent.substring(0, target.selectionStart);
		const linesBefore = textBeforeCursor.split('\n');
		const line = linesBefore.length - 1;
		const col = linesBefore[linesBefore.length - 1].length;

		syncContext(line, col);
	}

	function checkAndSave(targetPath: string, saveAction: () => void) {
		// Flush context
		if (textareaEl) {
			const textBefore = localContent.substring(0, textareaEl.selectionStart);
			const l = textBefore.split('\n');
			const currentLine = l.length - 1;
			const currentScroll = editorContentEl ? Math.floor(editorContentEl.scrollTop / 24) : 0;
			submitCommand(`!ipc editor_context_update:${paneId};${JSON.stringify({ 
				content: localContent, 
				cursor_line: currentLine, 
				cursor_col: l[l.length - 1].length,
				scroll_offset: currentScroll 
			})}`);
		}
		
		if (paneCwd && !targetPath.startsWith(paneCwd)) {
			// Requires Trust Confirmation
			pendingTrustTarget = targetPath;
			pendingTrustAction = saveAction;
		} else {
			saveAction();
		}
	}

	function approveTrust() {
		if (pendingTrustAction) pendingTrustAction();
		pendingTrustTarget = null;
		pendingTrustAction = null;
	}

	function handleKeydown(e: KeyboardEvent) {
		// Save hotkey Ctrl+S
		if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 's') {
			e.preventDefault();
			if (e.shiftKey) {
				const newPath = prompt("Enter new file path:", editorState.file_path);
				if (newPath && newPath.trim() !== '') {
					checkAndSave(newPath.trim(), () => submitCommand(`!ipc editor_save_as:${paneId};${newPath.trim()}`));
				}
			} else {
				checkAndSave(editorState.file_path, () => submitCommand(`!ipc editor_save:${paneId}`));
			}
			return;
		}

		if (e.key === 'Tab') {
			e.preventDefault();
			const target = e.target as HTMLTextAreaElement;
			const start = target.selectionStart;
			const end = target.selectionEnd;
			const spaces = "    "; // 4 spaces for tab
			
			localContent = localContent.substring(0, start) + spaces + localContent.substring(end);
			// setTimeout trick to update caret position after svelte updates value
			setTimeout(() => {
				if (textareaEl) {
					textareaEl.selectionStart = textareaEl.selectionEnd = start + spaces.length;
					handleInput({ target: textareaEl } as any);
				}
			}, 0);
		}
	}

	// Function to highlight code using Prism
	// We map the backend language string to a Prism language grammar
	const highlightedLines = $derived.by(() => {
		const langStr = editorState.language?.toLowerCase() || 'text';
		// Fallback to plain text if the language isn't loaded
		const grammar = Prism.languages[langStr] || Prism.languages.text || {};
		
		// If grammar exists, highlight, otherwise just escape the text
		let highlighted = '';
		if (Prism.languages[langStr]) {
			try {
				highlighted = Prism.highlight(localContent, grammar, langStr);
			} catch (e) {
				console.warn('Prism highlight error:', e);
				// Manual escape fallback
				highlighted = localContent
					.replace(/&/g, "&amp;")
					.replace(/</g, "&lt;")
					.replace(/>/g, "&gt;")
					.replace(/"/g, "&quot;")
					.replace(/'/g, "&#039;");
			}
		} else {
			// Manual escape fallback
			highlighted = localContent
				.replace(/&/g, "&amp;")
				.replace(/</g, "&lt;")
				.replace(/>/g, "&gt;")
				.replace(/"/g, "&quot;")
				.replace(/'/g, "&#039;");
		}
		
		return highlighted.split('\n');
	});
</script>

<div class="editor-container glass-panel">
	<div class="editor-header">
		<span class="file-path">
			{#if editorState.dirty}<span class="dirty-marker">•</span>{/if}
			{editorState.file_path.split('/').pop() || 'Untitled'}
		</span>
		<span class="header-tools">
			{#if pendingTrustTarget}
				<div class="trust-chip flex items-center">
					<span class="error-text">⚠ OUTSIDE CWD</span>
					<button class="pill-btn danger-chip" onclick={approveTrust}>[ALLOW PENDING WRITE]</button>
					<button class="pill-btn" onclick={() => pendingTrustTarget = null}>[CANCEL]</button>
				</div>
			{/if}
			<span class="pill-badge">{editorState.language || 'text'}</span>
			<span class="pill-badge">{highlightedLines.length} lines</span>
			<span class="pill-badge">{editorState.mode}</span>
			<button class="pill-btn" onclick={() => showAiContext = !showAiContext}>[AI]</button>
			<button class="pill-btn" title="Promote to Pane">[⊞]</button>
		</span>
	</div>
	
	<div class="editor-body-wrapper">
		<div bind:this={editorContentEl} onscroll={handleScroll} class="editor-content" class:line-numbers={editorState.mode !== 'Diff'}>
		{#if editorState.mode === 'Diff'}
			<div class="diff-container">
				{#if !editorState.diff_hunks || editorState.diff_hunks.length === 0}
					<div class="diff-empty">
						No pending proposed edits.
					</div>
				{:else}
					{#each editorState.diff_hunks as hunk, i}
						<div class="diff-hunk-card">
							<div class="diff-hunk-header">
								<span class="hunk-title">PROPOSED EDIT (Lines {hunk.old_start}-{hunk.old_start + hunk.old_count})</span>
								<div class="hunk-actions">
									<button onclick={() => submitCommand(`!ipc editor_edit_apply:${paneId};${i}`)}>[Apply]</button>
									<button onclick={() => submitCommand(`!ipc editor_edit_reject:${paneId};${i}`)}>[✕]</button>
								</div>
							</div>
							<div class="diff-hunk-content">
								{#each hunk.content.split('\n') as diffLine}
									<div class="diff-line" 
										class:diff-add={diffLine.startsWith('+')} 
										class:diff-sub={diffLine.startsWith('-')}>
										{diffLine}
									</div>
								{/each}
							</div>
						</div>
					{/each}
				{/if}
			</div>
		{:else}
			{#if editorState.mode === 'Editor'}
				<textarea 
					bind:this={textareaEl}
					class="editor-textarea"
					value={localContent}
					oninput={handleInput}
					onkeydown={handleKeydown}
					onclick={handleInput}
					onkeyup={handleInput}
					spellcheck="false"
				></textarea>
			{/if}
			<div class="code-layer">
				{#each highlightedLines as line, i}
					<div id="editor-{paneId}-line-{i}" class="editor-line" class:active-line={i === editorState.cursor_line}>
						<!-- svelte-ignore a11y_click_events_have_key_events -->
						<!-- svelte-ignore a11y_no_static_element_interactions -->
						<span class="line-number" onclick={() => handleLineClick(i)} title="Send line to AI">{i + 1}</span>
						<span class="line-text">{@html line || ' '}</span>
						{#if annotationsByLine.has(i)}
							<div class="inline-annotations">
								{#each annotationsByLine.get(i)! as ann}
									<div class="margin-chip" class:error={ann.severity === 'error'}>
										{ann.severity === 'error' ? '⚠' : '💡'} {ann.message}
									</div>
								{/each}
							</div>
						{/if}
					</div>
				{/each}
			</div>
		{/if}
		</div> <!-- Ends editor-content -->

		{#if showAiContext}
			<AiContextPanel {editorState} {paneId} onClose={() => showAiContext = false} />
		{/if}
	</div> <!-- Ends editor-body-wrapper -->
</div>

<style>
	.editor-container {
		display: flex;
		flex-direction: column;
		height: 100%;
		width: 100%;
		background: var(--color-bg-deep);
		font-family: var(--font-mono);
		font-size: 0.85rem;
		overflow: hidden;
	}

	.editor-body-wrapper {
		display: flex;
		flex-direction: row;
		flex: 1;
		height: 100%;
		overflow: hidden;
	}

	.editor-header {
		display: flex;
		justify-content: space-between;
		padding: var(--space-xs) var(--space-sm);
		background: rgba(0, 0, 0, 0.4);
		border-bottom: 1px solid var(--color-border);
		color: var(--color-text-dim);
		font-size: 0.7rem;
		flex-shrink: 0;
	}
	
	.line-number {
		display: inline-block;
		width: 3rem;
		padding-right: 1rem;
		text-align: right;
		color: var(--color-text-muted);
		user-select: none;
		border-right: 1px solid rgba(255, 255, 255, 0.1);
		margin-right: 1rem;
		cursor: pointer;
		transition: color var(--transition-fast), background var(--transition-fast);
	}

	.line-number:hover {
		color: var(--color-primary);
		background: rgba(255, 255, 255, 0.1);
	}

	.file-path {
		font-weight: bold;
		color: var(--color-text);
	}
	
	.dirty-marker {
		color: var(--color-warning);
		margin-right: 4px;
	}

	.pill-badge {
		background: rgba(255, 255, 255, 0.1);
		padding: 2px 6px;
		border-radius: 4px;
		margin-left: 6px;
		font-size: 0.65rem;
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.trust-chip {
		display: inline-flex;
		align-items: center;
		background: rgba(255, 50, 50, 0.1);
		border: 1px solid var(--color-warning);
		border-radius: 4px;
		padding: 1px 4px;
		margin-right: 8px;
	}

	.error-text {
		color: var(--color-warning);
		font-weight: bold;
		margin-right: 6px;
	}

	.danger-chip {
		background: rgba(255, 0, 0, 0.3) !important;
		color: white !important;
	}

	.editor-content {
		flex: 1;
		position: relative;
		overflow-y: auto;
		padding: var(--space-xs) 0;
		/* basic VSCode-like colors */
		color: #d4d4d4; 
		background-color: #1e1e1e;
	}

	.editor-textarea {
		position: absolute;
		top: var(--space-xs);
		bottom: 0;
		left: 4rem; /* match padding for line numbers */
		right: 0;
		min-height: 100%;
		width: calc(100% - 4rem);
		padding: 0;
		margin: 0;
		border: none;
		outline: none;
		background: transparent;
		color: transparent; /* Text is invisible, caret is visible */
		caret-color: #fff;
		font-family: inherit;
		font-size: inherit;
		line-height: inherit;
		resize: none;
		white-space: pre;
		overflow: hidden; /* Hide textarea scrollbars, let container scroll */
		z-index: 10;
		padding-left: 0.5rem;
	}
	
	.code-layer {
		position: relative;
		pointer-events: none; /* Let textarea receive clicks */
		z-index: 1;
		min-height: 100%;
	}

	.editor-line {
		display: flex;
		padding: 0 var(--space-md);
		border-left: 2px solid transparent;
		min-height: 1.5em;
		align-items: flex-start;
		position: relative;
		white-space: pre;
		line-height: inherit;
	}
	
	.editor-line:hover {
		background: rgba(255, 255, 255, 0.05);
	}

	.editor-line.active-line {
		background: rgba(255, 255, 255, 0.03);
		border-left-color: var(--color-primary);
	}

	.inline-annotations {
		position: absolute;
		right: 20px;
		display: flex;
		gap: 8px;
		z-index: 10;
		pointer-events: none; /* Let text selection fall through if desired, or auto for interactions */
	}

	.margin-chip {
		background: rgba(10, 10, 20, 0.85);
		border: 1px solid var(--color-primary);
		color: var(--color-primary);
		padding: 2px 8px;
		border-radius: var(--radius-sm);
		font-family: var(--font-display);
		font-size: 0.65rem;
		white-space: nowrap;
		box-shadow: 0 4px 10px rgba(0, 0, 0, 0.3);
		pointer-events: auto;
		cursor: default;
	}

	.margin-chip.error {
		border-color: var(--color-warning);
		color: var(--color-warning);
	}

	.active-line {
		background-color: rgba(255, 255, 255, 0.05); /* VSCode active line highlight */
		box-shadow: inset 2px 0 0 rgba(255, 255, 255, 0.4);
	}

	:global(.pulse-amber) {
		animation: amberPulse 2s ease-out;
	}

	@keyframes amberPulse {
		0% { background-color: rgba(255, 170, 0, 0.3); box-shadow: inset 2px 0 0 var(--color-warning); }
		100% { background-color: transparent; }
	}

	/* Diff UI */
	.diff-container {
		padding: var(--space-md);
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
		width: 100%;
	}

	.diff-empty {
		color: var(--color-text-dim);
		text-align: center;
		padding: var(--space-xl);
		font-style: italic;
	}

	.diff-hunk-card {
		background: rgba(0, 0, 0, 0.5);
		border: 1px solid var(--color-border);
		border-radius: 4px;
		overflow: hidden;
	}

	.diff-hunk-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: var(--space-xs) var(--space-sm);
		background: rgba(255, 255, 255, 0.05);
		border-bottom: 1px solid var(--color-border);
	}

	.hunk-title {
		font-weight: bold;
		color: var(--color-text-bright);
	}

	.hunk-actions button {
		background: none;
		border: none;
		color: var(--color-primary);
		cursor: pointer;
		font-family: var(--font-mono);
		font-weight: bold;
	}

	.hunk-actions button:hover {
		color: var(--color-text-bright);
		text-decoration: underline;
	}

	.diff-hunk-content {
		padding: var(--space-xs) 0;
	}

	.diff-line {
		white-space: pre;
		padding: 0 var(--space-sm);
		line-height: 1.4;
	}

	.diff-add {
		background: rgba(40, 167, 69, 0.2);
		color: #85e89d;
	}

	.diff-sub {
		background: rgba(203, 36, 49, 0.2);
		color: #f97583;
	}

	.line-text {
		padding-left: 0.5rem;
		flex: 1;
	}
</style>
