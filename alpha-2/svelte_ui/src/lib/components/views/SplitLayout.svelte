<script lang="ts">
	import type { SplitNode, Hub } from '$lib/stores/tos-state.svelte';
	import SplitPaneView from './SplitPaneView.svelte';
	import SplitLayout from './SplitLayout.svelte';

	let { node, activeHub }: { node: SplitNode; activeHub: Hub | null } = $props();

	function isLeaf(n: SplitNode): n is { Leaf: any } {
		return 'Leaf' in n;
	}
</script>

{#if isLeaf(node)}
	<SplitPaneView pane={node.Leaf} {activeHub} />
{:else if 'Container' in node}
	{@const container = node.Container}
	<div 
		class="split-container {container.orientation.toLowerCase()}"
	>
		{#each container.children as child, i}
			<div 
				class="split-item" 
				style="flex: {(isLeaf(child) ? child.Leaf.weight : 1) || 1};"
			>
				<SplitLayout node={child} {activeHub} />
				
				{#if i < container.children.length - 1}
					<div class="split-divider {container.orientation.toLowerCase()}"></div>
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
		flex-basis: 0; /* Ensures flex weight is the primary sizing factor */
	}

	.split-divider {
		position: absolute;
		z-index: 10;
		background: var(--color-border);
		pointer-events: none; /* Just visual for now, dragging handled by container logic later */
	}

	.split-divider.vertical {
		top: 0;
		bottom: 0;
		right: 0;
		width: 1px;
	}

	.split-divider.horizontal {
		left: 0;
		right: 0;
		bottom: 0;
		height: 1px;
	}
</style>
