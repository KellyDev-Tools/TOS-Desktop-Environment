<script lang="ts">
	import type { SplitNode, Hub } from '$lib/stores/tos-state.svelte';
	import { splitResize } from '$lib/stores/ipc.svelte';
	import SplitPaneView from './SplitPaneView.svelte';
	import SplitLayout from './SplitLayout.svelte';

	let { node, activeHub }: { node: SplitNode; activeHub: Hub | null } = $props();

	function isLeaf(n: SplitNode): n is { Leaf: any } {
		return n && 'Leaf' in n;
	}

	let draggingIdx = $state<number | null>(null);
	let containerEl = $state<HTMLDivElement | null>(null);

	function startDragging(i: number, e: MouseEvent) {
		e.preventDefault();
		draggingIdx = i;
		window.addEventListener('mousemove', handleDragging);
		window.addEventListener('mouseup', stopDragging);
	}

	function stopDragging() {
		draggingIdx = null;
		window.removeEventListener('mousemove', handleDragging);
		window.removeEventListener('mouseup', stopDragging);
	}

	function handleDragging(e: MouseEvent) {
		if (draggingIdx === null || !containerEl || !('Container' in node)) return;
		
		const rect = containerEl.getBoundingClientRect();
		const container = node.Container;
		const orientation = container.orientation;
		
		let ratio;
		if (orientation === 'Vertical') {
			ratio = (e.clientX - rect.left) / rect.width;
		} else {
			ratio = (e.clientY - rect.top) / rect.height;
		}
		
		// Snap assist (25/50/75)
		if (Math.abs(ratio - 0.25) < 0.03) ratio = 0.25;
		else if (Math.abs(ratio - 0.5) < 0.03) ratio = 0.5;
		else if (Math.abs(ratio - 0.75) < 0.03) ratio = 0.75;
		
		// Clamp
		ratio = Math.max(0.1, Math.min(0.9, ratio));
		
		// Rebalance weights between the two children separated by the divider
		const childA = container.children[draggingIdx];
		const childB = container.children[draggingIdx + 1];
		
		// We need a pane ID to send to split_resize. 
		// If it's a container, we'd ideally resize the whole branch, 
		// but the Brain IPC usually targets a leaf.
		// For now, let's find the first leaf in the branch.
		const findFirstLeafId = (n: any): string | null => {
			if ('Leaf' in n) return n.Leaf.id;
			if ('Container' in n && n.Container.children.length > 0) return findFirstLeafId(n.Container.children[0]);
			return null;
		};

		const paneId = findFirstLeafId(childA);
		if (paneId) {
			// Brain handles weight relative to siblings. 
			// If there are exactly 2 siblings, ratio works well.
			splitResize(paneId, ratio);
		}
	}
</script>

{#if isLeaf(node)}
	<SplitPaneView pane={node.Leaf} {activeHub} />
{:else if node && 'Container' in node}
	{@const container = node.Container}
	<div 
		class="split-container {container.orientation.toLowerCase()}"
		bind:this={containerEl}
	>
		{#each container.children as child, i}
			<div 
				class="split-item" 
				style="flex: {(isLeaf(child) ? child.Leaf.weight : (child as any).Container?.children[0]?.Leaf?.weight || 1) || 1};"
			>
				<SplitLayout node={child} {activeHub} />
				
				{#if i < container.children.length - 1}
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div 
						class="split-divider {container.orientation.toLowerCase()} interactive"
						onmousedown={(e) => startDragging(i, e)}
					>
						<div class="divider-handle"></div>
						<div class="snap-indicators">
							<div class="snap-dot" style="left: 25%"></div>
							<div class="snap-dot" style="left: 50%"></div>
							<div class="snap-dot" style="left: 75%"></div>
						</div>
					</div>
				{/if}
			</div>
		{/each}
	</div>
{/if}

<style>
	.split-container {
		display: flex;
		height: 100%;
		width: 100%;
		gap: 0;
		position: relative;
	}

	.split-container.vertical {
		flex-direction: row;
	}

	.split-container.horizontal {
		flex-direction: column;
	}

	.split-item {
		display: flex;
		position: relative;
		min-width: 0;
		min-height: 0;
		flex-basis: 0;
	}

	.split-divider {
		position: absolute;
		z-index: 100;
		background: var(--color-border);
		transition: background 0.2s;
	}

	.split-divider.interactive {
		pointer-events: all;
	}

	.split-divider.interactive:hover {
		background: var(--color-primary);
		box-shadow: 0 0 10px var(--color-primary);
	}

	.split-divider.vertical {
		top: 0;
		bottom: 0;
		right: -3px;
		width: 6px;
		cursor: col-resize;
	}

	.split-divider.horizontal {
		left: 0;
		right: 0;
		bottom: -3px;
		height: 6px;
		cursor: row-resize;
	}

	.divider-handle {
		position: absolute;
		inset: 0;
		background: transparent;
	}

	.snap-indicators {
		position: absolute;
		inset: 0;
		pointer-events: none;
		opacity: 0;
		transition: opacity 0.3s;
	}

	.split-divider:hover .snap-indicators {
		opacity: 0.3;
	}

	/* Indicators are only shown for the primary axis */
	.vertical > .snap-indicators { width: 100vw; left: -50vw; top: 50%; height: 2px; }
	.horizontal > .snap-indicators { height: 100vh; top: -50vh; left: 50%; width: 2px; }

	.snap-dot {
		position: absolute;
		width: 4px;
		height: 4px;
		background: var(--color-primary);
		border-radius: 50%;
		transform: translate(-50%, -50%);
	}
</style>
