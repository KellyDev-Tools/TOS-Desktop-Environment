<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { fade, fly, scale } from 'svelte/transition';
	import { getTosState, marketplaceGetHome, marketplaceGetCategory, marketplaceGetDetail, marketplaceInstall, marketplaceGetStatus } from '$lib/stores/ipc.svelte';

	const state = $derived(getTosState());
	
	let selectedCategory = $state('Featured');
	let searchQuery = $state('');
    let homeData = $state<any>(null);
    let selectedModule = $state<any>(null); // Detail view
    let showPermissionModal = $state(false);
    let installProgress = $state<any>(null);
    let pollingInterval = $state<any>(null);

	onMount(async () => {
		homeData = await marketplaceGetHome();
        if (homeData) {
            categories = ['Featured', ...homeData.categories.map((c: any) => c.name)];
        }
	});

    onDestroy(() => {
        if (pollingInterval) clearInterval(pollingInterval);
    });

	let categories = $state(['Featured', 'AI Behaviors', 'Shell Modules', 'Themes', 'Terminal Modules']);
	
	const filteredModules = $derived(
		homeData ? (
            selectedCategory === 'Featured' ? homeData.featured : categoryModules
        ).filter((m: any) => m.name.toLowerCase().includes(searchQuery.toLowerCase()))
        : []
	);

    let categoryModules = $state<any[]>([]);

    async function handleCategorySelect(categoryName: string) {
        selectedCategory = categoryName;
        if (categoryName === 'Featured') {
            categoryModules = [];
            return;
        }
        const cat = homeData.categories.find((c: any) => c.name === categoryName);
        if (cat) {
            categoryModules = await marketplaceGetCategory(cat.id);
        }
    }

    async function openDetail(moduleId: string) {
        selectedModule = await marketplaceGetDetail(moduleId);
    }

    function initiateInstall() {
        showPermissionModal = true;
    }

    async function confirmInstall() {
        showPermissionModal = false;
        const res = await marketplaceInstall(selectedModule.summary.id);
        if (res === 'INSTALLING') {
            startPollingStatus(selectedModule.summary.id);
        }
    }

    function startPollingStatus(id: string) {
        installProgress = { module_id: id, progress: 0, status: 'Initializing' };
        pollingInterval = setInterval(async () => {
            const status = await marketplaceGetStatus(id);
            if (status) {
                installProgress = status;
                if (status.status === 'Complete' || status.status === 'Error') {
                    clearInterval(pollingInterval);
                }
            }
        }, 1000);
    }
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
				onclick={() => handleCategorySelect(category)}
			>
				{category.toUpperCase()}
			</button>
		{/each}
	</nav>

	<main class="market-content">
		{#if selectedCategory === 'Featured' && homeData}
			<section class="featured-strip" in:fly={{ y: 20 }}>
                {#each homeData.featured as module}
                    <div class="featured-card {module.name.toLowerCase().includes('aurora') ? 'aurora' : 'intelligence'}" onclick={() => openDetail(module.id)}>
                        <div class="card-tag">FEATURED</div>
                        <div class="card-title">{module.name.toUpperCase()}</div>
                        <div class="card-desc">{module.module_type} by {module.author}</div>
                        <button class="lcars-btn-sm" onclick={(e) => { e.stopPropagation(); openDetail(module.id); }}>VIEW_DETAIL</button>
                    </div>
                {/each}
			</section>
		{/if}

		<div class="module-grid">
			{#each filteredModules as module}
				<div class="module-card glass-panel" in:scale onclick={() => openDetail(module.id)}>
					<div class="module-icon">{module.icon || '⊞'}</div>
					<div class="module-info">
						<div class="module-name">{module.name}</div>
						<div class="module-type">{module.module_type}</div>
						<div class="module-rating">★ {module.rating}</div>
					</div>
					<div class="module-action">
						{#if module.installed}
							<span class="status-badge installed">INSTALLED</span>
						{:else}
							<button class="lcars-btn-sm primary" onclick={(e) => { e.stopPropagation(); openDetail(module.id); }}>{module.price}</button>
						{/if}
					</div>
				</div>
			{/each}
		</div>
	</main>

    {#if selectedModule}
        <div class="detail-overlay" transition:fade onclick={() => selectedModule = null}>
            <div class="detail-card glass-panel" onclick={(e) => e.stopPropagation()} in:fly={{ y: 50 }}>
                <header class="detail-header">
                    <div class="detail-icon">{selectedModule.summary.icon || '⊞'}</div>
                    <div class="detail-meta">
                        <h2>{selectedModule.summary.name}</h2>
                        <div class="author">By {selectedModule.summary.author}</div>
                        <div class="tags">
                            <span class="tag">{selectedModule.summary.module_type}</span>
                            <span class="tag">VERIFIED</span>
                        </div>
                    </div>
                    <button class="close-btn" onclick={() => selectedModule = null}>✕</button>
                </header>

                <div class="detail-gallery">
                    {#each selectedModule.screenshots as ss}
                        <img src={ss} alt="Screenshot" />
                    {/each}
                </div>

                <div class="detail-tabs">
                    <div class="tab-content">
                        <h3>DESCRIPTION</h3>
                        <p>{selectedModule.description}</p>

                        <h3>PERMISSIONS</h3>
                        <ul class="perm-list">
                            {#each selectedModule.permissions as perm}
                                <li>✦ {perm.replace('_', ' ').toUpperCase()}</li>
                            {/each}
                        </ul>
                    </div>
                </div>

                <footer class="detail-footer">
                    {#if installProgress && installProgress.module_id === selectedModule.summary.id}
                        {#if installProgress.status === 'Error'}
                            <div class="install-progress-bar error">
                                <div class="progress-text">ERROR: {installProgress.error || 'INSTALLATION_FAILED'}</div>
                                <button class="retry-btn" onclick={() => initiateInstall()}>RETRY</button>
                            </div>
                        {:else}
                            <div class="install-progress-bar">
                                <div class="progress-fill" style="width: {installProgress.progress * 100}%"></div>
                                <div class="progress-text">{installProgress.status.toUpperCase()}... {(installProgress.progress * 100).toFixed(0)}%</div>
                            </div>
                        {/if}
                    {:else if selectedModule.summary.installed}
                        <button class="lcars-btn secondary" disabled>INSTALLED</button>
                    {:else}
                        <button class="lcars-btn primary" onclick={initiateInstall}>INSTALL MODULE</button>
                    {/if}
                </footer>
            </div>
        </div>
    {/if}

    {#if showPermissionModal}
        <div class="modal-overlay" transition:fade>
            <div class="modal-card glass-panel" in:scale>
                <h2>REVIEW PERMISSIONS</h2>
                <p>"{selectedModule.summary.name}" requires the following access:</p>
                
                <div class="perm-review">
                    {#each selectedModule.permissions as perm}
                        <div class="perm-item">
                            <div class="perm-name">{perm.toUpperCase()}</div>
                            <div class="perm-desc">Required for core functionality.</div>
                        </div>
                    {/each}
                </div>

                <div class="security-meta">
                    <div class="sig-status valid">✓ SIGNATURE VALID</div>
                    <div class="author-label">AUTHOR: {selectedModule.summary.author}</div>
                </div>

                <div class="modal-actions">
                    <button class="lcars-btn secondary" onclick={() => showPermissionModal = false}>DECLINE</button>
                    <button class="lcars-btn primary" onclick={confirmInstall}>ACCEPT & INSTALL</button>
                </div>
            </div>
        </div>
    {/if}
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
        position: relative;
        overflow: hidden;
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
        cursor: pointer;
        transition: transform 0.3s;
	}

    .featured-card:hover { transform: scale(1.02); }

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
        cursor: pointer;
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

    .lcars-btn {
		background: var(--color-primary);
		color: black;
		border: none;
		font-family: var(--font-display);
		font-weight: 700;
		font-size: 1rem;
		padding: 12px 30px;
		cursor: pointer;
		border-radius: 4px;
        transition: all 0.2s;
	}

    .lcars-btn.secondary {
        background: var(--color-secondary);
    }

    .lcars-btn:hover {
        filter: brightness(1.2);
        box-shadow: 0 0 15px currentColor;
    }

    /* Detail View Styles */
    .detail-overlay {
        position: absolute;
        top: 0; left: 0; right: 0; bottom: 0;
        background: rgba(0,0,0,0.8);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 100;
        padding: 40px;
    }

    .detail-card {
        width: 100%;
        max-width: 900px;
        background: var(--color-bg-dark);
        border: 1px solid var(--color-border);
        display: flex;
        flex-direction: column;
        max-height: 90vh;
        overflow: hidden;
    }

    .detail-header {
        padding: 30px;
        display: flex;
        gap: 30px;
        align-items: center;
        border-bottom: 1px solid var(--color-border);
        position: relative;
    }

    .detail-meta h2 {
        margin: 0;
        font-family: var(--font-display);
        font-size: 2rem;
    }

    .detail-icon {
        width: 80px;
        height: 80px;
        background: rgba(247, 168, 51, 0.1);
		border: 2px solid var(--color-primary);
        font-size: 3rem;
        display: flex; align-items: center; justify-content: center;
        color: var(--color-primary);
    }

    .tags { display: flex; gap: 10px; margin-top: 10px; }
    .tag {
        font-size: 0.6rem;
        background: rgba(255,255,255,0.1);
        padding: 2px 10px;
        border-radius: 10px;
        font-family: var(--font-mono);
    }

    .close-btn {
        position: absolute;
        top: 20px; right: 20px;
        background: transparent; border: none; color: white;
        font-size: 1.5rem; cursor: pointer;
    }

    .detail-gallery {
        height: 250px;
        background: #000;
        display: flex;
        overflow-x: auto;
        padding: 20px;
        gap: 20px;
    }

    .detail-gallery img {
        height: 100%;
        border-radius: 4px;
        border: 1px solid rgba(255,255,255,0.1);
    }

    .tab-content {
        padding: 30px;
        overflow-y: auto;
    }

    .tab-content h3 {
        font-family: var(--font-display);
        color: var(--color-primary);
        font-size: 0.9rem;
        letter-spacing: 0.1em;
        margin-top: 30px;
    }

    .perm-list {
        list-style: none;
        padding: 0;
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 10px;
    }

    .perm-list li {
        font-family: var(--font-mono);
        font-size: 0.75rem;
        color: var(--color-text-dim);
    }

    .detail-footer {
        padding: 30px;
        border-top: 1px solid var(--color-border);
        display: flex;
        justify-content: flex-end;
    }

    /* Modal Styles */
    .modal-overlay {
        position: absolute;
        top: 0; left: 0; right: 0; bottom: 0;
        background: rgba(0,0,0,0.9);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 200;
    }

    .modal-card {
        width: 450px;
        padding: 40px;
        display: flex;
        flex-direction: column;
        gap: 20px;
    }

    .perm-review {
        background: rgba(0,0,0,0.3);
        padding: 20px;
        border-radius: 4px;
    }

    .perm-item { margin-bottom: 15px; }
    .perm-name { font-family: var(--font-display); font-weight: 700; color: var(--color-primary); font-size: 0.8rem; }
    .perm-desc { font-size: 0.7rem; opacity: 0.6; }

    .security-meta {
        font-family: var(--font-mono);
        font-size: 0.7rem;
        border-left: 2px solid var(--color-secondary);
        padding-left: 15px;
    }

    .sig-status { color: var(--color-secondary); margin-bottom: 5px; }

    .modal-actions {
        display: flex;
        justify-content: space-between;
        margin-top: 20px;
    }

    /* Progress bar */
    .install-progress-bar {
        width: 100%;
        height: 40px;
        background: rgba(255,255,255,0.05);
        border: 1px solid var(--color-border);
        position: relative;
        overflow: hidden;
    }

    .install-progress-bar.error {
        background: rgba(255, 100, 100, 0.1);
        border-color: #ff5252;
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0 15px;
    }

    .install-progress-bar.error .progress-text {
        color: #ff5252;
        position: static;
        width: auto;
    }

    .retry-btn {
        background: #ff5252;
        color: white;
        border: none;
        font-family: var(--font-display);
        font-weight: 700;
        font-size: 0.7rem;
        padding: 4px 12px;
        cursor: pointer;
        border-radius: 2px;
    }

    .progress-fill {
        height: 100%;
        background: var(--color-primary);
        transition: width 0.3s;
    }

    .progress-text {
        position: absolute;
        top: 0; left: 0; width: 100%; height: 100%;
        display: flex; align-items: center; justify-content: center;
        font-family: var(--font-display); font-weight: 700; font-size: 0.8rem;
        color: black; mix-blend-mode: hard-light;
    }
</style>

</style>
