<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { fade, fly, scale, slide } from 'svelte/transition';
	import { getTosState, marketplaceGetHome, marketplaceGetCategory, marketplaceGetDetail, marketplaceInstall, marketplaceGetStatus, sendCommand, marketplaceSearchAi } from '$lib/stores/ipc.svelte';

	const tosState = $derived(getTosState());
	
	let selectedCategory = $state('Featured');
	let searchQuery = $state('');
    let homeData = $state<any>(null);
    let selectedModule = $state<any>(null); // Detail view
    let showPermissionModal = $state(false);
    let installProgress = $state<any>(null);
    let pollingInterval = $state<any>(null);
    let isAiSearching = $state(false);
    let aiSearchResults = $state<any[]>([]);

	onMount(() => {
		fetchHome();
	});

    async function fetchHome() {
        if (homeData) return;
        homeData = await marketplaceGetHome();
        if (homeData) {
            categories = ['Featured', ...homeData.categories.map((c: any) => c.name)];
            isConnecting = false;
        } else {
            // Retry in 1s if failed to fetch (likely not connected yet)
            setTimeout(fetchHome, 1000);
        }
    }

    onDestroy(() => {
        if (pollingInterval) clearInterval(pollingInterval);
    });

    let categories = $state(['Featured']);
    let isConnecting = $state(true);
    let categoryModules = $state<any[]>([]);
	
	const filteredModules = $derived(
		aiSearchResults.length > 0 ? aiSearchResults : (
            homeData ? (
                selectedCategory === 'Featured' ? homeData.featured : categoryModules
            ).filter((m: any) => m.name.toLowerCase().includes(searchQuery.toLowerCase()))
            : []
        )
	);

    async function handleCategorySelect(categoryName: string) {
        selectedCategory = categoryName;
        resetAiSearch();
        if (categoryName === 'Featured') {
            categoryModules = [];
            return;
        }
        if (!homeData) return;
        const cat = homeData.categories.find((c: any) => c.name === categoryName);
        if (cat) {
            categoryModules = await marketplaceGetCategory(cat.id);
        }
    }

    function resetAiSearch() {
        isAiSearching = false;
        aiSearchResults = [];
        searchQuery = '';
    }

    async function handleAiSearch() {
        if (!searchQuery.trim()) return;
        isAiSearching = true;
        const res = await marketplaceSearchAi(searchQuery);
        if (res) {
            aiSearchResults = res;
        } else {
            aiSearchResults = [];
        }
        isAiSearching = false;
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
                onkeydown={(e) => e.key === 'Enter' && handleAiSearch()}
			/>
            <button class="ai-search-btn" class:loading={isAiSearching} onclick={handleAiSearch} title="Use AI to find matching modules">
                ✦ {isAiSearching ? 'SCANNING...' : 'AI_FIND'}
            </button>
		</div>
	</header>

	<nav class="market-nav">
		{#each categories as category}
			<button 
				class="nav-btn" 
				class:active={selectedCategory === category && !isAiSearching}
				onclick={() => handleCategorySelect(category)}
			>
				{category.toUpperCase()}
			</button>
		{/each}
        {#if isAiSearching}
            <button class="nav-btn active">AI_RESULTS</button>
        {/if}
	</nav>

	<main class="market-content">
		{#if selectedCategory === 'Featured' && homeData && !isAiSearching}
			<section class="featured-strip" in:fly={{ y: 20 }}>
                {#each homeData.featured as module}
                    <div role="button" tabindex="0" onkeydown={(e) => e.key === 'Enter' && openDetail(module.id)} class="featured-card {module.name.toLowerCase().includes('aurora') ? 'aurora' : 'intelligence'}" onclick={() => openDetail(module.id)}>
                        <div class="card-tag">FEATURED</div>
                        <div class="card-title">{module.name.toUpperCase()}</div>
                        <div class="card-desc">{module.module_type} by {module.author}</div>
                        <button class="lcars-btn-sm" onclick={(e) => { e.stopPropagation(); openDetail(module.id); }}>VIEW_DETAIL</button>
                        <div class="card-glow"></div>
                    </div>
                {/each}
			</section>
		{/if}

		<div class="module-grid" class:searching={isAiSearching}>
			{#each filteredModules as module}
				<div 
                    role="button" tabindex="0" onkeydown={(e) => e.key === 'Enter' && openDetail(module.id)}
                    class="module-card glass-panel" 
                    class:ai-result={aiSearchResults.some(a => a.id === module.id)}
                    in:fly={{ y: 20, duration: 300 }} 
                    onclick={() => openDetail(module.id)}
                >
					<div class="module-icon">{module.icon || '⊞'}</div>
					<div class="module-info">
						<div class="module-name">
                            {module.name}
                            {#if module.verified}
                                <span class="verified-badge-card">✓</span>
                            {/if}
                        </div>
						<div class="module-type">{module.module_type}</div>
						<div class="module-rating">
                            {'★'.repeat(Math.round(module.rating))}
                            <span class="rating-num">({module.rating})</span>
                        </div>
					</div>
					<div class="module-action">
						{#if module.installed}
							<div class="status-indicator">
                                <div class="pulse-dot-small"></div>
                                <span class="status-badge installed">INSTALLED</span>
                            </div>
						{:else}
							<button class="lcars-btn-sm price-tag" onclick={(e) => { e.stopPropagation(); openDetail(module.id); }}>
                                {module.price === '$0.00' ? 'FREE' : module.price}
                            </button>
						{/if}
					</div>
				</div>
            {:else}
                <div class="empty-state" in:fade>
                    <div class="empty-icon">⬡</div>
                    <div class="empty-text">NO MODULES FOUND IN THIS CLASS</div>
                </div>
			{/each}
		</div>
	</main>

    {#if selectedModule}
        <div class="detail-overlay" transition:fade onclick={() => selectedModule = null} role="button" tabindex="0" onkeydown={(e) => e.key === 'Escape' && (selectedModule = null)}>
            <div class="detail-card glass-panel" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.key === 'Enter' && e.stopPropagation()} in:fly={{ y: 100, duration: 500 }} role="dialog" aria-modal="true" tabindex="-1">
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

                <div class="detail-scroll-area">
                    <div class="detail-gallery">
                        {#each (selectedModule.screenshots || []) as ss}
                            <img src={ss} alt="Screenshot" />
                        {/each}
                    </div>

                    <div class="detail-sections">
                        <section class="detail-section">
                            <h3>DESCRIPTION</h3>
                            <p>{selectedModule.description}</p>
                        </section>

                        <section class="detail-section">
                            <h3>PERMISSIONS</h3>
                            <ul class="perm-list">
                                {#each (selectedModule.permissions || []) as perm}
                                    <li>✦ {perm.replace('_', ' ').toUpperCase()}</li>
                                {/each}
                            </ul>
                        </section>

                        <section class="detail-section">
                            <h3>USER REVIEWS</h3>
                            <div class="review-grid">
                                {#each (selectedModule.reviews || []) as review}
                                    <div class="review-card">
                                        <div class="review-meta">
                                            <span class="review-author">{review.author}</span>
                                            <span class="review-rating">{'★'.repeat(review.rating)}</span>
                                            <span class="review-date">{review.date}</span>
                                        </div>
                                        <p class="review-comment">"{review.comment}"</p>
                                    </div>
                                {:else}
                                    <div class="no-reviews">No reviews for this module yet.</div>
                                {/each}
                            </div>
                        </section>
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
            <div class="modal-card glass-panel" in:scale={{ start: 0.9 }}>
                <h2>REVIEW PERMISSIONS</h2>
                <p>"{selectedModule.summary.name}" requires the following access:</p>
                
                <div class="perm-review">
                    {#each (selectedModule.permissions || []) as perm}
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
		background: radial-gradient(circle at 50% 50%, rgba(15, 15, 30, 0.4) 0%, rgba(5, 5, 10, 0.8) 100%);
		color: var(--color-text);
		padding: 40px;
		gap: 30px;
        position: relative;
        overflow: hidden;
        backdrop-filter: blur(10px);
	}

	.market-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.market-title {
		font-family: var(--font-display);
		font-weight: 700;
		font-size: 2.2rem;
		letter-spacing: 0.15em;
		color: var(--color-primary);
        text-shadow: 0 0 20px rgba(247, 168, 51, 0.3);
	}

    .market-search {
        display: flex;
        gap: 15px;
        align-items: center;
    }

	.glass-input {
		background: rgba(255, 255, 255, 0.05);
		border: 1px solid var(--color-border);
		padding: 12px 25px;
		color: white;
		font-family: var(--font-mono);
		border-radius: 25px;
		outline: none;
		width: 350px;
        transition: all 0.3s;
        box-shadow: inset 0 2px 10px rgba(0,0,0,0.5);
	}

    .glass-input:focus {
        border-color: var(--color-primary);
        background: rgba(255, 255, 255, 0.1);
        width: 400px;
    }

    .ai-search-btn {
        background: linear-gradient(135deg, var(--color-accent), #7b61ff);
        color: white;
        border: none;
        padding: 10px 20px;
        border-radius: 20px;
        font-family: var(--font-display);
        font-weight: 700;
        cursor: pointer;
        transition: all 0.2s;
        display: flex;
        align-items: center;
        gap: 8px;
        box-shadow: 0 4px 15px rgba(123, 97, 255, 0.3);
    }

    .ai-search-btn:hover {
        transform: scale(1.05);
        box-shadow: 0 6px 20px rgba(123, 97, 255, 0.5);
    }

    .ai-search-btn.loading {
        opacity: 0.7;
        cursor: wait;
        animation: searchPulse 1s infinite alternate;
    }

    @keyframes searchPulse {
        from { filter: brightness(1); }
        to { filter: brightness(1.5); }
    }

	.market-nav {
		display: flex;
		gap: 30px;
		border-bottom: 2px solid rgba(255, 255, 255, 0.05);
		padding-bottom: 15px;
	}

	.nav-btn {
		background: transparent;
		border: none;
		color: var(--color-text-dim);
		font-family: var(--font-display);
		font-weight: 700;
		font-size: 0.9rem;
		cursor: pointer;
		padding: 5px 0;
		transition: all 0.3s;
        border-bottom: 2px solid transparent;
        letter-spacing: 0.1em;
	}

	.nav-btn.active {
		color: var(--color-secondary);
		border-bottom-color: var(--color-secondary);
        text-shadow: 0 0 10px rgba(92, 136, 218, 0.5);
	}

	.market-content {
		flex: 1;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		gap: 50px;
        padding-right: 15px;
	}

    .market-content::-webkit-scrollbar { width: 4px; }
    .market-content::-webkit-scrollbar-thumb { background: var(--color-border); border-radius: 2px; }

	.featured-strip {
		display: flex;
		gap: 30px;
		min-height: 220px;
	}

	.featured-card {
		flex: 1;
		border-radius: 12px;
		padding: 40px;
		display: flex;
		flex-direction: column;
		justify-content: flex-end;
		position: relative;
		overflow: hidden;
		border: 1px solid rgba(255, 255, 255, 0.1);
        cursor: pointer;
        transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1);
	}

    .featured-card:hover { 
        transform: translateY(-8px) scale(1.01); 
        box-shadow: 0 20px 40px rgba(0,0,0,0.5);
    }

	.featured-card.aurora {
		background: linear-gradient(135deg, #1e3c72 0%, #2a5298 100%);
	}

	.featured-card.intelligence {
		background: linear-gradient(135deg, #0f2027 0%, #203a43 50%, #2c5364 100%);
	}

    .card-glow {
        position: absolute;
        inset: 0;
        background: radial-gradient(circle at top right, rgba(255,255,255,0.1), transparent);
        opacity: 0.5;
    }

	.card-tag {
		position: absolute;
		top: 25px;
		right: 25px;
		font-family: var(--font-mono);
		font-size: 0.65rem;
		background: rgba(0, 0, 0, 0.6);
		padding: 4px 12px;
		border-radius: 20px;
        backdrop-filter: blur(5px);
        border: 1px solid rgba(255,255,255,0.1);
	}

	.card-title {
		font-family: var(--font-display);
		font-weight: 800;
		font-size: 1.8rem;
		margin-bottom: 8px;
        z-index: 1;
	}

	.card-desc {
		font-size: 0.9rem;
		opacity: 0.8;
		margin-bottom: 25px;
        z-index: 1;
	}

	.module-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
		gap: 25px;
	}

	.module-card {
		padding: 25px;
		display: flex;
		align-items: center;
		gap: 25px;
		background: rgba(255, 255, 255, 0.03);
		border: 1px solid rgba(255, 255, 255, 0.08);
		transition: all 0.3s;
        cursor: pointer;
        border-radius: 8px;
	}

	.module-card:hover {
		background: rgba(255, 255, 255, 0.07);
		border-color: var(--color-primary);
		transform: translateY(-4px);
        box-shadow: 0 10px 30px rgba(0,0,0,0.3);
	}

	.module-icon {
		width: 60px;
		height: 60px;
		background: rgba(247, 168, 51, 0.1);
		border: 1px solid var(--color-primary);
		border-radius: 8px;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 2rem;
		color: var(--color-primary);
	}

	.module-info { flex: 1; }

	.module-name {
		font-family: var(--font-display);
		font-weight: 700;
		font-size: 1.1rem;
        margin-bottom: 2px;
	}

	.module-type {
		font-size: 0.75rem;
		color: var(--color-text-dim);
		text-transform: uppercase;
		letter-spacing: 0.1em;
	}

    .verified-badge-card {
        font-size: 0.65rem;
        background: rgba(92, 136, 218, 0.2);
        color: var(--color-secondary);
        padding: 1px 6px;
        border-radius: 4px;
        margin-left: 8px;
        vertical-align: middle;
        border: 1px solid rgba(92, 136, 218, 0.3);
    }

	.module-rating {
		font-size: 0.75rem;
		color: var(--color-secondary);
		margin-top: 5px;
	}

    .rating-num { color: var(--color-text-muted); margin-left: 5px; font-family: var(--font-mono); }

    .price-tag {
        background: var(--color-primary) !important;
        font-weight: 800;
        box-shadow: 0 4px 10px rgba(247, 168, 51, 0.2);
    }

    .status-indicator {
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .pulse-dot-small {
        width: 6px;
        height: 6px;
        background: #4caf50;
        border-radius: 50%;
        animation: pulseFade 2s infinite;
    }

    @keyframes pulseFade {
        0% { opacity: 1; transform: scale(1); }
        50% { opacity: 0.4; transform: scale(1.2); }
        100% { opacity: 1; transform: scale(1); }
    }

    .empty-text { font-family: var(--font-display); font-weight: 700; letter-spacing: 0.1em; }

    /* AI Search Branding */
    .module-grid.searching {
        position: relative;
    }

    .module-grid.searching::after {
        content: "";
        position: absolute;
        inset: 0;
        background: linear-gradient(rgba(18, 16, 16, 0) 50%, rgba(0, 0, 0, 0.25) 50%), linear-gradient(90deg, rgba(255, 0, 0, 0.06), rgba(0, 255, 0, 0.02), rgba(0, 0, 255, 0.06));
        background-size: 100% 4px, 3px 100%;
        pointer-events: none;
        z-index: 10;
        opacity: 0.3;
        animation: scanline 10s linear infinite;
    }

    @keyframes scanline {
        from { background-position: 0 0; }
        to { background-position: 0 100%; }
    }

    .module-card.ai-result {
        border-color: var(--color-secondary);
        box-shadow: 0 0 15px rgba(92, 136, 218, 0.2);
        animation: aiPulsate 4s ease-in-out infinite;
    }

    @keyframes aiPulsate {
        0% { box-shadow: 0 0 10px rgba(92, 136, 218, 0.1); }
        50% { box-shadow: 0 0 25px rgba(92, 136, 218, 0.3); }
        100% { box-shadow: 0 0 10px rgba(92, 136, 218, 0.1); }
    }

    .ai-result .module-icon {
        background: rgba(92, 136, 218, 0.15);
        color: var(--color-secondary);
        border-color: var(--color-secondary);
    }

	.status-badge {
		font-family: var(--font-display);
		font-weight: 700;
		font-size: 0.65rem;
		padding: 4px 10px;
		border-radius: 4px;
	}

	.status-badge.installed {
		background: rgba(76, 175, 80, 0.1);
		color: #4caf50;
		border: 1px solid #4caf50;
	}

	.lcars-btn-sm {
		background: var(--color-primary);
		color: black !important;
		border: none;
		font-family: var(--font-display);
		font-weight: 700;
		font-size: 0.75rem;
		padding: 8px 20px;
		cursor: pointer;
		border-radius: 4px;
        transition: all 0.2s;
	}

    .lcars-btn-sm:hover { filter: brightness(1.2); }

    .lcars-btn {
		background: var(--color-primary);
		color: black !important;
		border: none;
		font-family: var(--font-display);
		font-weight: 700;
		font-size: 1.1rem;
		padding: 12px 40px;
		cursor: pointer;
		border-radius: 4px;
        transition: all 0.2s;
        letter-spacing: 0.05em;
	}

    .lcars-btn.secondary {
        background: var(--color-secondary);
    }

    .lcars-btn:hover {
        filter: brightness(1.2);
        box-shadow: 0 0 20px currentColor;
    }

    .lcars-btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    /* Detail View Styles */
    .detail-overlay {
        position: absolute;
        top: 0; left: 0; right: 0; bottom: 0;
        background: rgba(0,0,0,0.85);
        backdrop-filter: blur(8px);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 100;
        padding: 40px;
    }

    .detail-card {
        width: 100%;
        max-width: 1000px;
        background: var(--color-bg-dark);
        border: 1px solid var(--color-border);
        display: flex;
        flex-direction: column;
        max-height: 90vh;
        overflow: hidden;
        border-radius: 12px;
        box-shadow: 0 30px 60px rgba(0,0,0,0.8);
    }

    .detail-header {
        padding: 40px;
        display: flex;
        gap: 30px;
        align-items: center;
        border-bottom: 1px solid rgba(255,255,255,0.05);
        position: relative;
        background: rgba(255,255,255,0.02);
    }

    .detail-meta h2 {
        margin: 0;
        font-family: var(--font-display);
        font-size: 2.5rem;
        line-height: 1;
    }

    .detail-icon {
        width: 100px;
        height: 100px;
        background: rgba(247, 168, 51, 0.1);
		border: 2px solid var(--color-primary);
        font-size: 4rem;
        display: flex; align-items: center; justify-content: center;
        color: var(--color-primary);
        border-radius: 12px;
    }

    .tags { display: flex; gap: 10px; margin-top: 15px; }
    .tag {
        font-size: 0.7rem;
        background: rgba(255,255,255,0.1);
        padding: 3px 12px;
        border-radius: 20px;
        font-family: var(--font-mono);
        color: var(--color-text-dim);
    }

    .close-btn {
        position: absolute;
        top: 25px; right: 25px;
        background: rgba(255,255,255,0.1); 
        border: none; color: white;
        width: 35px; height: 35px;
        display: flex; align-items: center; justify-content: center;
        border-radius: 50%;
        font-size: 1rem; cursor: pointer;
        transition: all 0.2s;
    }
    .close-btn:hover { background: rgba(255,255,255,0.2); }

    .detail-scroll-area {
        flex: 1;
        overflow-y: auto;
        padding-bottom: 40px;
    }

    .detail-gallery {
        height: 300px;
        background: #000;
        display: flex;
        overflow-x: auto;
        padding: 30px;
        gap: 25px;
        border-bottom: 1px solid rgba(255,255,255,0.05);
    }

    .detail-gallery img {
        height: 100%;
        border-radius: 8px;
        border: 1px solid rgba(255,255,255,0.1);
        box-shadow: 0 10px 20px rgba(0,0,0,0.5);
    }

    .detail-sections {
        padding: 40px;
        display: flex;
        flex-direction: column;
        gap: 50px;
    }

    .detail-section h3 {
        font-family: var(--font-display);
        color: var(--color-primary);
        font-size: 1rem;
        letter-spacing: 0.2em;
        margin-bottom: 20px;
        border-left: 4px solid var(--color-primary);
        padding-left: 15px;
    }

    .detail-section p {
        line-height: 1.6;
        font-size: 1.05rem;
        color: var(--color-text-dim);
    }

    .perm-list {
        list-style: none;
        padding: 0;
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
        gap: 15px;
    }

    .perm-list li {
        font-family: var(--font-mono);
        font-size: 0.85rem;
        border-radius: 4px;
        background: rgba(255,255,255,0.03);
        padding: 10px;
        color: var(--color-text-dim);
    }

    .review-grid {
        display: flex;
        flex-direction: column;
        gap: 20px;
    }

    .review-card {
        background: rgba(255,255,255,0.03);
        padding: 20px;
        border-radius: 8px;
        border: 1px solid rgba(255,255,255,0.05);
    }

    .review-meta {
        display: flex;
        align-items: center;
        gap: 15px;
        margin-bottom: 10px;
    }

    .review-author { font-weight: 700; color: white; }
    .review-rating { color: var(--color-secondary); letter-spacing: 2px; }
    .review-date { font-size: 0.8rem; color: var(--color-text-muted); margin-left: auto; }
    .review-comment { font-style: italic; font-size: 0.95rem; line-height: 1.4; color: var(--color-text-dim); }

    .detail-footer {
        padding: 30px 40px;
        border-top: 1px solid rgba(255,255,255,0.05);
        display: flex;
        justify-content: flex-end;
        background: rgba(255,255,255,0.02);
    }

    /* Modal Styles */
    .modal-overlay {
        position: absolute;
        top: 0; left: 0; right: 0; bottom: 0;
        background: rgba(0,0,0,0.92);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 200;
        backdrop-filter: blur(15px);
    }

    .modal-card {
        width: 500px;
        padding: 50px;
        display: flex;
        flex-direction: column;
        gap: 25px;
        border-radius: 20px;
    }

    .modal-card h2 {
        font-family: var(--font-display);
        font-weight: 800;
        letter-spacing: 0.1em;
        text-align: center;
        color: var(--color-primary);
    }

    .perm-review {
        background: rgba(0,0,0,0.4);
        padding: 25px;
        border-radius: 8px;
        max-height: 300px;
        overflow-y: auto;
        border: 1px solid rgba(255,255,255,0.05);
    }

    .perm-item { margin-bottom: 20px; }
    .perm-item:last-child { margin-bottom: 0; }
    .perm-name { font-family: var(--font-display); font-weight: 700; color: var(--color-primary); font-size: 0.9rem; margin-bottom: 4px; }
    .perm-desc { font-size: 0.75rem; opacity: 0.6; line-height: 1.4; }

    .security-meta {
        font-family: var(--font-mono);
        font-size: 0.75rem;
        border-left: 3px solid var(--color-secondary);
        padding: 10px 20px;
        background: rgba(92, 136, 218, 0.05);
    }

    .sig-status { color: var(--color-secondary); margin-bottom: 5px; font-weight: 700; }

    .modal-actions {
        display: flex;
        justify-content: space-between;
        margin-top: 10px;
        gap: 20px;
    }

    .modal-actions .lcars-btn { flex: 1; padding: 12px; font-size: 0.9rem; }

    /* Progress bar */
    .install-progress-bar {
        width: 100%;
        height: 44px;
        background: rgba(255,255,255,0.05);
        border: 1px solid var(--color-border);
        position: relative;
        overflow: hidden;
        border-radius: 6px;
    }

    .install-progress-bar.error {
        background: rgba(255, 100, 100, 0.1);
        border-color: #ff5252;
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0 20px;
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
        font-size: 0.8rem;
        padding: 6px 20px;
        cursor: pointer;
        border-radius: 4px;
        transition: all 0.2s;
    }
    .retry-btn:hover { filter: brightness(1.2); }

    .progress-fill {
        height: 100%;
        background: linear-gradient(90deg, var(--color-primary), var(--color-accent));
        transition: width 0.3s cubic-bezier(0.16, 1, 0.3, 1);
        box-shadow: 0 0 20px rgba(247, 168, 51, 0.4);
    }

    .progress-text {
        position: absolute;
        top: 0; left: 0; width: 100%; height: 100%;
        display: flex; align-items: center; justify-content: center;
        font-family: var(--font-display); font-weight: 800; font-size: 0.9rem;
        color: black; mix-blend-mode: overlay;
        letter-spacing: 0.1em;
    }
</style>

