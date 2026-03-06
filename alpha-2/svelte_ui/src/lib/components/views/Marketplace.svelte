<script lang="ts">
	import { fade, fly, scale } from 'svelte/transition';
	import { getTosState } from '$lib/stores/ipc.svelte';

	const state = $derived(getTosState());
	
	let selectedCategory = $state('Featured');
	let searchQuery = $state('');

	const categories = ['Featured', 'AI Behaviors', 'Shell Modules', 'Themes', 'Terminal Modules'];
	
	const modules = [
		{ id: 'tos-observer', name: 'Passive Observer', type: 'AI Behavior', rating: 4.8, status: 'Installed' },
		{ id: 'tos-chat', name: 'Chat Companion', type: 'AI Behavior', rating: 4.9, status: 'Installed' },
		{ id: 'tos-shell-fish', name: 'Fish Shell', type: 'Shell', rating: 4.7, status: 'Installed' },
		{ id: 'tos-aurora-theme', name: 'Aurora Borealis', type: 'Theme', rating: 4.5, status: 'Available' },
		{ id: 'tos-grid-term', name: 'Grid Terminal', type: 'Output Module', rating: 4.2, status: 'Available' },
	];

	const filteredModules = $derived(
		modules.filter(m => {
			const matchesCategory = selectedCategory === 'Featured' || m.type.includes(selectedCategory.slice(0, -1));
			const matchesSearch = m.name.toLowerCase().includes(searchQuery.toLowerCase());
			return matchesCategory && matchesSearch;
		})
	);
</script>

<div class="marketplace-view" transition:fade>
	<header class="market-header">
		<div class="market-title">SYSTEM // MARKETPLACE</div>
		<div class="market-search">
			<input 
				type="text" 
				placeholder="SEARCH_MODULES..." 
				bind:value={searchQuery}
				class="glass-input"
			/>
		</div>
	</header>

	<nav class="market-nav">
		{#each categories as category}
			<button 
				class="nav-btn" 
				class:active={selectedCategory === category}
				onclick={() => selectedCategory = category}
			>
				{category.toUpperCase()}
			</button>
		{/each}
	</nav>

	<main class="market-content">
		{#if selectedCategory === 'Featured'}
			<section class="featured-strip" in:fly={{ y: 20 }}>
				<div class="featured-card aurora">
					<div class="card-tag">NEW_RELEASE</div>
					<div class="card-title">AURORA BOREALIS</div>
					<div class="card-desc">Dynamic atmospheric theme with kinetic light pulses.</div>
					<button class="lcars-btn-sm">VIEW_DETAIL</button>
				</div>
				<div class="featured-card intelligence">
					<div class="card-tag">TRENDING</div>
					<div class="card-title">PREDICTIVE_SHELL</div>
					<div class="card-desc">LLM-driven command completion with heuristic caching.</div>
					<button class="lcars-btn-sm">INSTALL</button>
				</div>
			</section>
		{/if}

		<div class="module-grid">
			{#each filteredModules as module}
				<div class="module-card glass-panel" in:scale>
					<div class="module-icon">⊞</div>
					<div class="module-info">
						<div class="module-name">{module.name}</div>
						<div class="module-type">{module.type}</div>
						<div class="module-rating">★ {module.rating}</div>
					</div>
					<div class="module-action">
						{#if module.status === 'Installed'}
							<span class="status-badge installed">INSTALLED</span>
						{:else}
							<button class="lcars-btn-sm primary">INSTALL</button>
						{/if}
					</div>
				</div>
			{/each}
		</div>
	</main>
</div>

<style>
	.marketplace-view {
		width: 100%;
		height: 100%;
		display: flex;
		flex-direction: column;
		background: rgba(5, 5, 10, 0.4);
		color: var(--color-text);
		padding: 40px;
		gap: 30px;
	}

	.market-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.market-title {
		font-family: var(--font-display);
		font-weight: 700;
		font-size: 2rem;
		letter-spacing: 0.1em;
		color: var(--color-primary);
	}

	.glass-input {
		background: rgba(255, 255, 255, 0.05);
		border: 1px solid var(--color-border);
		padding: 10px 20px;
		color: white;
		font-family: var(--font-mono);
		border-radius: 4px;
		outline: none;
		width: 300px;
	}

	.market-nav {
		display: flex;
		gap: 20px;
		border-bottom: 1px solid rgba(255, 255, 255, 0.1);
		padding-bottom: 10px;
	}

	.nav-btn {
		background: transparent;
		border: none;
		color: var(--color-text-dim);
		font-family: var(--font-display);
		font-weight: 700;
		font-size: 0.8rem;
		cursor: pointer;
		padding: 5px 10px;
		transition: all 0.2s;
	}

	.nav-btn.active {
		color: var(--color-secondary);
		border-bottom: 2px solid var(--color-secondary);
	}

	.market-content {
		flex: 1;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		gap: 40px;
	}

	.featured-strip {
		display: flex;
		gap: 20px;
		height: 200px;
	}

	.featured-card {
		flex: 1;
		border-radius: 8px;
		padding: 30px;
		display: flex;
		flex-direction: column;
		justify-content: flex-end;
		position: relative;
		overflow: hidden;
		border: 1px solid rgba(255, 255, 255, 0.1);
	}

	.featured-card.aurora {
		background: linear-gradient(135deg, #1e3c72 0%, #2a5298 100%);
		box-shadow: 0 10px 30px rgba(42, 82, 152, 0.3);
	}

	.featured-card.intelligence {
		background: linear-gradient(135deg, #0f2027 0%, #203a43 50%, #2c5364 100%);
		box-shadow: 0 10px 30px rgba(44, 83, 100, 0.3);
	}

	.card-tag {
		position: absolute;
		top: 20px;
		right: 20px;
		font-family: var(--font-mono);
		font-size: 0.6rem;
		background: rgba(0, 0, 0, 0.5);
		padding: 2px 10px;
		border-radius: 10px;
	}

	.card-title {
		font-family: var(--font-display);
		font-weight: 700;
		font-size: 1.5rem;
		margin-bottom: 5px;
	}

	.card-desc {
		font-size: 0.85rem;
		opacity: 0.7;
		margin-bottom: 20px;
	}

	.module-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
		gap: 20px;
	}

	.module-card {
		padding: 20px;
		display: flex;
		align-items: center;
		gap: 20px;
		background: rgba(255, 255, 255, 0.02);
		border: 1px solid rgba(255, 255, 255, 0.05);
		transition: all 0.2s;
	}

	.module-card:hover {
		background: rgba(255, 255, 255, 0.05);
		border-color: var(--color-primary);
		transform: translateY(-2px);
	}

	.module-icon {
		width: 50px;
		height: 50px;
		background: rgba(247, 168, 51, 0.1);
		border: 1px solid var(--color-primary);
		border-radius: 4px;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 1.5rem;
		color: var(--color-primary);
	}

	.module-info { flex: 1; }

	.module-name {
		font-family: var(--font-display);
		font-weight: 700;
		font-size: 1rem;
	}

	.module-type {
		font-size: 0.7rem;
		color: var(--color-text-dim);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.module-rating {
		font-size: 0.7rem;
		color: var(--color-secondary);
		margin-top: 5px;
	}

	.status-badge {
		font-family: var(--font-display);
		font-weight: 700;
		font-size: 0.6rem;
		padding: 2px 8px;
		border-radius: 10px;
	}

	.status-badge.installed {
		background: rgba(76, 175, 80, 0.1);
		color: #4caf50;
		border: 1px solid #4caf50;
	}

	.lcars-btn-sm {
		background: var(--color-primary);
		color: black;
		border: none;
		font-family: var(--font-display);
		font-weight: 700;
		font-size: 0.65rem;
		padding: 5px 15px;
		cursor: pointer;
		border-radius: 2px;
	}

	.lcars-btn-sm.primary {
		background: var(--color-primary);
	}
</style>
