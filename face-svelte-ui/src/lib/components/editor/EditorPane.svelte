<script lang="ts">
	import type { EditorPaneState, Hub } from '$lib/stores/tos-state.svelte';
	import Prism from 'prismjs';
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

	let { editorState, activeHub }: { editorState: EditorPaneState; activeHub: Hub | null } = $props();

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
				highlighted = Prism.highlight(editorState.content, grammar, langStr);
			} catch (e) {
				console.warn('Prism highlight error:', e);
				// Manual escape fallback
				highlighted = editorState.content
					.replace(/&/g, "&amp;")
					.replace(/</g, "&lt;")
					.replace(/>/g, "&gt;")
					.replace(/"/g, "&quot;")
					.replace(/'/g, "&#039;");
			}
		} else {
			// Manual escape fallback
			highlighted = editorState.content
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
		<span class="file-path">{editorState.file_path}</span>
		<span class="file-stats">
			{#if editorState.dirty}
				<span class="dirty-marker">●</span>
			{/if}
			<span class="pill-badge">{editorState.language || 'text'}</span>
			<span class="pill-badge">{highlightedLines.length} lines</span>
			<span class="pill-badge">{editorState.mode}</span>
		</span>
	</div>
	
	<div class="editor-content line-numbers">
		{#each highlightedLines as line, i}
			<div class="editor-line" class:active-line={i === editorState.cursor_line}>
				<span class="line-number">{i + 1}</span>
				<span class="line-text">{@html line || ' '}</span>
			</div>
		{/each}
	</div>
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

	.editor-content {
		flex: 1;
		overflow-y: auto;
		padding: var(--space-xs) 0;
		/* basic VSCode-like colors */
		color: #d4d4d4; 
		background-color: #1e1e1e;
	}

	.editor-line {
		display: flex;
		white-space: pre;
	}
	
	.editor-line:hover {
		background: rgba(255, 255, 255, 0.05); /* VSCode line highlight on hover */
	}

	.active-line {
		background: rgba(255, 255, 255, 0.1); 
		border-left: 2px solid var(--color-primary); /* Emulate active cursor line */
	}

	.line-number {
		color: #858585; /* VSCode line number color */
		text-align: right;
		min-width: 3rem;
		padding-right: 1rem;
		user-select: none;
		flex-shrink: 0;
	}
	
	.line-text {
		padding-left: 0.5rem;
		flex: 1;
	}
</style>
