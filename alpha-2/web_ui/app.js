/**
 * TOS UI Controller
 * High-Fidelity Rendering & State Sync
 */

class TosUI {
    constructor() {
        this.state = null;
        this.currentMode = 'global';
        this.ws = null;
        this.pendingRequests = new Map();

        // Alpha-2.1 UI State (Symmetrical Bezel Segments)
        // Alpha-2.1 UI State (Configurable Slot Architecture)
        this.sidebarExpanded = false;
        this.sidebarRightExpanded = false;

        this.componentStates = {
            minimap: { projected: false },
            priority: { projected: false },
            telemetry: { projected: false },
            minilog: { projected: false },
            screen_title: { projected: false },
            brain_status: { projected: false },
            status_badges: { projected: false }
        };

        this.slotConfigs = {
            left: ['minimap'],
            right: ['priority', 'minilog'],
            top_left: ['screen_title'],
            top_center: ['brain_status', 'telemetry'],
            top_right: ['status_badges']
        };

        this.settingsActiveTab = 'global';
        this.init();
    }

    init() {
        this.setupEventListeners();
        this.setupWebSocket();
        this.setMode('global'); // Establish default view layering
        this.syncState();

        // High-frequency state synchronization loop
        setInterval(() => this.syncState(), 1000);

        console.log("TOS UI // ALPHA-2.1 // OPERATIONAL");
    }

    setupEventListeners() {
        const cmdInput = document.getElementById('cmd-input');
        if (cmdInput) {
            cmdInput.addEventListener('keypress', (e) => {
                if (e.key === 'Enter') {
                    this.handleCommand(cmdInput.value);
                    cmdInput.value = '';
                }
            });
        }

        // Mode Navigation (Hierarchy Levels)
        document.querySelectorAll('.level-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                const level = btn.dataset.level;
                // Map level names to modes
                const modeMap = {
                    'GlobalOverview': 'global',
                    'CommandHub': 'hubs',
                    'ApplicationFocus': 'sectors',
                    'DetailView': 'detail',
                    'BufferView': 'buffer',
                    'SpatialOverview': 'spatial'
                };
                this.setMode(modeMap[level] || 'global');
            });
        });

        // Settings Modal Listeners
        document.getElementById('settings-close')?.addEventListener('click', () => this.toggleSettingsModal(false));
        document.querySelectorAll('.settings-nav-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                this.settingsActiveTab = btn.dataset.tab;
                this.renderSettings();
                document.querySelectorAll('.settings-nav-btn').forEach(b => b.classList.toggle('active', b === btn));
            });
        });

        document.getElementById('settings-save')?.addEventListener('click', () => {
            this.commitSettings();
        });

        // Portal Modal Listeners
        document.getElementById('portal-close')?.addEventListener('click', () => this.togglePortalModal(false));
        document.getElementById('portal-copy')?.addEventListener('click', () => this.copyPortalLink());
        document.getElementById('portal-revoke')?.addEventListener('click', () => this.togglePortalModal(false));


        // Global Keyboard Shortcuts (§14, §22)
        document.addEventListener('keydown', (e) => {
            // Projected Components
            if (e.ctrlKey && e.key === 'm') {
                e.preventDefault();
                this.toggleMinimap();
            }
            if (e.ctrlKey && e.key === 'p') {
                e.preventDefault();
                this.togglePriority();
            }
            // Lateral Segment Expansion
            if (e.ctrlKey && e.key === 's') {
                e.preventDefault();
                this.toggleSidebar();
            }
            if (e.ctrlKey && e.key === 'r') {
                e.preventDefault();
                this.toggleSidebarRight();
            }
            // Settings shortcut
            if (e.key === 'Escape') {
                this.toggleSettingsModal(false);
                this.togglePortalModal(false);
            }
        });

        // 6.4 Bezel Commands
        document.getElementById('bezel-expand-left')?.addEventListener('click', () => this.toggleSidebar());
        document.getElementById('bezel-expand-right')?.addEventListener('click', () => this.toggleSidebarRight());
    }

    toggleSettingsModal(show) {
        const modal = document.getElementById('settings-modal');
        if (!modal) return;

        if (show) {
            modal.classList.remove('hidden');
            this.renderSettings();
            this.playEarcon('modal_open');
            this.triggerHaptic('click');
        } else {
            modal.classList.add('hidden');
            this.playEarcon('modal_close');
        }
    }

    renderSettings() {
        const pane = document.getElementById('settings-pane-content');
        if (!pane || !this.state || !this.state.settings) return;

        let html = '';
        const settings = this.state.settings;

        if (this.settingsActiveTab === 'global') {
            html = `
                <div class="settings-group">
                    <div class="settings-group-title">GLOBAL CORE PARAMETERS</div>
                    ${Object.entries(settings.global || {}).map(([k, v]) => `
                        <div class="settings-item">
                            <label class="settings-label">${k.toUpperCase()}</label>
                            <input type="text" class="settings-input" data-key="${k}" data-scope="global" value="${v}">
                        </div>
                    `).join('')}
                    ${Object.keys(settings.global || {}).length === 0 ? '<div style="opacity:0.5; font-style:italic">NO GLOBAL SETTINGS DEFINED</div>' : ''}
                    <div class="settings-item">
                        <label class="settings-label" style="color:var(--color-accent)">ADD NEW PARAMETER</label>
                        <div style="display:flex; gap:0.5rem">
                            <input type="text" id="new-param-key" class="settings-input" placeholder="KEY..." style="width:6rem">
                            <input type="text" id="new-param-val" class="settings-input" placeholder="VALUE..." style="width:6rem">
                        </div>
                    </div>
                </div>
            `;
        } else if (this.settingsActiveTab === 'sectors') {
            const currentSector = this.state.sectors[this.state.active_sector_index];
            const sectorId = currentSector?.id;
            const sectorSettings = sectorId ? (settings.sectors[sectorId] || {}) : {};

            html = `
                <div class="settings-group">
                    <div class="settings-group-title">SECTOR PARAMETERS // ${currentSector?.name.toUpperCase() || 'UNKNOWN'}</div>
                    ${Object.entries(sectorSettings).map(([k, v]) => `
                        <div class="settings-item">
                            <label class="settings-label">${k.toUpperCase()}</label>
                            <input type="text" class="settings-input" data-key="${k}" data-scope="sector" data-id="${sectorId}" value="${v}">
                        </div>
                    `).join('')}
                    ${Object.keys(sectorSettings).length === 0 ? '<div style="opacity:0.5; font-style:italic">NO SECTOR-SPECIFIC OVERRIDES</div>' : ''}
                </div>
            `;
        } else if (this.settingsActiveTab === 'interface') {
            html = `
                <div class="settings-group">
                    <div class="settings-group-title">INTERFACE CALIBRATION</div>
                    <div class="settings-item">
                        <label class="settings-label">UI FEEDBACK SCALE</label>
                        <input type="range" class="settings-input" style="width:10rem">
                    </div>
                    <div class="settings-item">
                        <label class="settings-label">CINEMATIC TRANSITIONS</label>
                        <button class="status-badge active">ENABLED</button>
                    </div>
                    <div class="settings-item">
                        <label class="settings-label">HAPTIC OVERRIDE</label>
                        <button class="status-badge">DISABLED</button>
                    </div>
                </div>
            `;
        }

        pane.innerHTML = html;
    }

    async commitSettings() {
        const inputs = document.querySelectorAll('.settings-input[data-key]');
        const log = document.getElementById('mini-log');

        for (const input of inputs) {
            const key = input.dataset.key;
            const val = input.value;
            const scope = input.dataset.scope;

            if (scope === 'global') {
                await window.__TOS_IPC__(`set_setting:${key};${val}`);
            } else if (scope === 'sector') {
                const id = input.dataset.id;
                await window.__TOS_IPC__(`set_sector_setting:${id};${key};${val}`);
            }
        }

        // Handle new global parameter if filled
        const newKey = document.getElementById('new-param-key')?.value;
        const newVal = document.getElementById('new-param-val')?.value;
        if (newKey && newVal) {
            await window.__TOS_IPC__(`set_setting:${newKey};${newVal}`);
        }

        if (log) {
            log.innerText = "SETTINGS COMMITTED.";
            log.style.color = 'var(--color-success)';
            this.playEarcon('data_commit');
            this.triggerHaptic('success');
        }

        this.syncState();
        this.toggleSettingsModal(false);
    }

    toggleComponent(id) {
        if (!this.componentStates[id]) return;
        this.componentStates[id].projected = !this.componentStates[id].projected;
        this.render();
        console.log(`[TOS UI] Component ${id}: ${this.componentStates[id].projected ? 'PROJECTED' : 'DOCKED'}`);
    }

    toggleMinimap() { this.toggleComponent('minimap'); }
    togglePriority() { this.toggleComponent('priority'); }
    toggleTelemetry() { this.toggleComponent('telemetry'); }
    toggleMinilog() { this.toggleComponent('minilog'); }
    toggleScreenTitle() { this.toggleComponent('screen_title'); }
    toggleBrainStatus() { this.toggleComponent('brain_status'); }
    toggleStatusBadges() { this.toggleComponent('status_badges'); }

    toggleSidebar() {
        this.sidebarExpanded = !this.sidebarExpanded;
        const sidebar = document.querySelector('.lcars-sidebar:not(.lcars-sidebar-right)');
        if (sidebar) sidebar.classList.toggle('expanded', this.sidebarExpanded);
    }

    toggleSidebarRight() {
        this.sidebarRightExpanded = !this.sidebarRightExpanded;
        const sidebar = document.querySelector('.lcars-sidebar-right');
        if (sidebar) sidebar.classList.toggle('expanded', this.sidebarRightExpanded);
    }

    async setMode(mode) {
        const target = document.getElementById('state-render-target');
        if (target) target.classList.add('transitioning');

        // Cinematic Transition Delay
        await new Promise(r => setTimeout(r, 200));

        this.currentMode = mode;

        // Spec 6: Manage UI Layering classes
        document.body.classList.toggle('view-global', mode === 'global');
        document.body.classList.toggle('view-hubs', mode === 'hubs');
        document.body.classList.toggle('view-sectors', mode === 'sectors');
        document.body.classList.toggle('view-detail', mode === 'detail');
        document.body.classList.toggle('view-buffer', mode === 'buffer');
        document.body.classList.toggle('view-spatial', mode === 'spatial');

        if (window.__TOS_IPC__) {
            window.__TOS_IPC__(`set_mode:${mode}`);
            this.playEarcon('nav_switch');
        }
        document.querySelectorAll('.level-btn').forEach(btn => {
            // Map level button state
            const modeMap = {
                'GlobalOverview': 'global',
                'CommandHub': 'hubs',
                'ApplicationFocus': 'sectors',
                'DetailView': 'detail',
                'BufferView': 'buffer',
                'SpatialOverview': 'spatial'
            };
            btn.classList.toggle('active', modeMap[btn.dataset.level] === mode);
        });

        this.render();

        if (target) target.classList.remove('transitioning');

        // Theme auto-switching based on mode or state
        if (this.state && this.state.pending_confirmation) this.setTheme('alert');
        else if (mode === 'sectors') this.setTheme('tactical');
        else if (mode === 'global') this.setTheme('default');
    }

    setTheme(theme) {
        document.body.classList.remove('theme-tactical', 'theme-alert');
        if (theme !== 'default') {
            document.body.classList.add(`theme-${theme}`);
        }
    }

    setupWebSocket() {
        if (window.__TOS_IPC__) return; // Use native bridge if available

        console.log("TOS UI // Initializing WebSocket Bridge (127.0.0.1:7001)...");
        this.ws = new WebSocket('ws://127.0.0.1:7001');

        this.ws.onopen = () => {
            console.log("TOS UI // WebSocket Bridge: CONNECTED ✅");
            // Define polyfill for IPC calls
            window.__TOS_IPC__ = (cmd) => {
                return new Promise((resolve, reject) => {
                    const id = Math.random().toString(36).substring(2, 9);
                    this.ws.send(cmd);
                    this.pendingRequests.set(id, resolve);

                    // Request Timeout
                    setTimeout(() => {
                        if (this.pendingRequests.has(id)) {
                            this.pendingRequests.delete(id);
                            reject("IPC TIMEOUT");
                        }
                    }, 5000);
                });
            };
        };

        this.ws.onmessage = (event) => {
            // In this simple bridge, we just take the first pending request
            const [id, resolve] = this.pendingRequests.entries().next().value || [null, null];
            if (resolve) {
                resolve(event.data);
                this.pendingRequests.delete(id);
            }
        };

        this.ws.onclose = () => {
            console.warn("TOS UI // WebSocket Bridge: DISCONNECTED. Retrying in 5s...");
            window.__TOS_IPC__ = null;
            setTimeout(() => this.setupWebSocket(), 5000);
        };
    }

    async handleCommand(cmd) {
        console.log(`[TOS IPC] -> ${cmd}`);

        const log = document.getElementById('mini-log');
        if (log) {
            log.innerText = `TRANSMITTING: ${cmd.toUpperCase()}`;
            log.style.color = 'var(--color-primary)';
        }

        // Transmit to Rust Brain via IPC Bridge
        if (window.__TOS_IPC__) {
            try {
                // Bezel Commands Interception
                if (cmd === 'bezel:settings') {
                    this.toggleSettingsModal(true);
                    return;
                }
                if (cmd === 'bezel:portal') {
                    this.togglePortalModal(true);
                    return;
                }


                // Configurable Slot Reassignment via command (Mock Logic)
                if (cmd.startsWith('dock:')) {
                    const [_, compId, segmentId] = cmd.split(':');
                    if (this.componentStates[compId] && this.slotConfigs[segmentId]) {
                        // Remove from old segment
                        Object.keys(this.slotConfigs).forEach(s => {
                            this.slotConfigs[s] = this.slotConfigs[s].filter(id => id !== compId);
                        });
                        // Add to new segment
                        this.slotConfigs[segmentId].push(compId);
                        this.render();
                    }
                }

                const response = await window.__TOS_IPC__(`prompt_submit:${cmd}`);

                // Parse the real latency from the response (e.g., "SUBMITTED (122µs)")
                const latencyMatch = response.match(/\(([^)]+)\)/);
                const latency = latencyMatch ? latencyMatch[1] : "OK";

                if (log) {
                    log.innerText = `ACK // ${latency}`;
                    log.style.color = 'var(--color-success)';
                    this.lastFeedback = Date.now();
                }
            } catch (e) {
                if (log) {
                    log.innerText = "IPC CONNECTION FAILURE";
                    log.style.color = 'var(--color-warning)';
                }
            }
        }
    }

    handleSectorClick(index) {
        this.handleCommand(`switch_sector:${index}`);

        // Optimistic UI state update for the mock prototype
        if (this.state && this.state.sectors[index]) {
            this.state.active_sector_index = index;
            this.setMode('hubs');
        }
    }

    async syncState() {
        try {
            // IPC bridge to the Rust Brain core
            if (window.__TOS_IPC__) {
                let rawState = await window.__TOS_IPC__("get_state:");

                // Strip the Rust diagnostic duration suffix e.g. "JSON (123µs)"
                if (rawState.includes(' (')) {
                    rawState = rawState.substring(0, rawState.lastIndexOf(' ('));
                }

                this.state = JSON.parse(rawState);
                this.render();
            } else {
                // Fallback for standalone development (Mock data)
                this.state = this.state || this.getDefaultState();
                this.state.brain_time = new Date().toTimeString().split(' ')[0];
                this.render();
            }
        } catch (e) {
            console.error("IPC Sync Failure:", e);
        }
    }

    getDefaultState() {
        return {
            sys_prefix: "TOS // SYSTEM-OFFLINE",
            sys_title: "AWAITING BRAIN LINK...",
            sys_status: "BRAIN: DISCONNECTED",
            sys_ready: "LINK FAILURE.",
            collab_presence: [],
            active_sector_index: 0,
            sectors: [{ name: "Local", hubs: [{ mode: 'Command' }] }],
            system_log: [{ text: "NO CONNECTION TO BRAIN", priority: 1, timestamp: new Date().toISOString() }],
            terminal_output: [],
            settings: { global: {}, sectors: {}, applications: {} }
        };
    }

    render() {
        if (!this.state) return;

        const target = document.getElementById('state-render-target');
        const title = document.getElementById('view-title');
        const footer = document.querySelector('.lcars-footer');

        // Render Collaboration Indicators (6.4)
        const collabEl = document.querySelector('.collab-indicators');
        if (collabEl && this.state.collab_presence) {
            collabEl.innerHTML = this.state.collab_presence.map(u =>
                `<div class="collab-dot" style="background: ${u.color}"></div>`
            ).join('');
        }

        // Dynamic Slot Rendering
        this.renderAllSlots();

        if (!target || !title) return;

        if (this.currentMode === 'global') {
            title.innerText = "GLOBAL OVERVIEW";
            target.innerHTML = `
                <div class="state-grid">
                    ${this.state.sectors.map((s, i) => `
                        <div class="sector-tile ${i === this.state.active_sector_index ? 'active' : ''}" onclick="window.tos.handleSectorClick(${i})">
                            <div style="font-size: 0.7rem; opacity: 0.6">S0${i}</div>
                            <div style="font-family: var(--font-display); font-weight: 700; color: var(--color-primary)">${s.name.toUpperCase()}</div>
                            <div style="font-size: 0.8rem; margin-top: 0.3125rem">TYPE: ${s.type ? s.type.toUpperCase() : 'STANDARD'}</div>
                            <div style="font-size: 0.8rem">STATUS: ${s.status ? s.status.toUpperCase() : 'ACTIVE'}</div>
                        </div>
                    `).join('')}
                </div>
            `;
        } else if (this.currentMode === 'hubs') {
            title.innerText = "HUB VIEW // COMMAND";

            let activeHub = null;
            if (this.state.sectors && this.state.sectors[this.state.active_sector_index]) {
                const sector = this.state.sectors[this.state.active_sector_index];
                if (sector.hubs && sector.hubs[sector.active_hub_index]) {
                    activeHub = sector.hubs[sector.active_hub_index];
                }
            }

            const termOutputs = (activeHub && activeHub.terminal_output ? activeHub.terminal_output : (this.state.terminal_output || []))
                .map(l => `<div style="color: ${l.color || 'inherit'}">${l.text || ''}</div>`).join('');

            let leftColumnHTML = '';

            if (activeHub) {
                if (activeHub.json_context) {
                    const ctx = activeHub.json_context;
                    leftColumnHTML = `
                        <div class="context-chip glass-panel">
                            <div class="chip-title">JSON CONTEXT // ${ctx.type || 'DATA'}</div>
                            <div class="chip-row"><strong>NAME:</strong> ${ctx.name || '--'}</div>
                            ${ctx.state ? `<div class="chip-row"><strong>STATE:</strong> <span class="status-badge active" style="display:inline-block; padding:0.125rem 0.625rem; margin-top:0.2rem; background: #000; color: var(--color-success); border-radius: 1rem; border: 1px solid rgba(255, 255, 255, 0.1); font-size: 0.75rem;">${ctx.state}</span></div>` : ''}
                            ${ctx.active_file ? `<div class="chip-row"><strong>FILE:</strong> ${ctx.active_file}</div>` : ''}
                            ${ctx.metadata ? `<div class="chip-metadata">
                                ${Object.entries(ctx.metadata).map(([k, v]) => `<div><strong>${k.toUpperCase()}:</strong> ${v}</div>`).join('')}
                            </div>` : ''}
                        </div>
                    `;
                } else if (activeHub.shell_listing) {
                    const dir = activeHub.shell_listing;
                    leftColumnHTML = `
                        <div class="context-chip glass-panel">
                            <div class="chip-title" style="color: var(--color-primary)">DIR PREVIEW // ${dir.path}</div>
                            <div class="directory-list">
                                ${dir.entries.map(e => `
                                    <div class="chip-row" style="font-family: var(--font-mono); font-size: 0.8rem; display: flex; align-items: center; justify-content: space-between;">
                                        <span><span style="color: ${e.is_dir ? 'var(--color-primary)' : 'inherit'}; min-width: 40px; display: inline-block;">${e.is_dir ? '[DIR]' : '&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;'}</span> <span style="font-weight: ${e.is_dir ? '700' : '300'}">${e.name}</span></span>
                                        <span style="opacity: 0.5">${e.is_dir ? '' : e.size + ' B'}</span>
                                    </div>
                                `).join('')}
                            </div>
                        </div>
                    `;
                } else if (activeHub.activity_listing) {
                    const act = activeHub.activity_listing;
                    leftColumnHTML = `
                        <div class="activity-chip-stack">
                            <div class="chip-title" style="color: var(--color-warning)">SYSTEM ACTIVITY LUNGS // RECENT</div>
                            <div style="overflow-y: auto; flex-grow: 1; padding-right: 0.5rem;">
                                ${act.processes.slice(0, 10).map(p => `
                                    <div class="context-chip glass-panel activity-chip ${p.snapshot ? 'has-snapshot' : ''}" style="margin-bottom: 0.75rem; flex-direction: row; align-items: center; justify-content: space-between; min-height: 4rem;">
                                        <div class="process-meta">
                                            <div class="chip-row"><strong style="color: var(--color-accent)">PID ${p.pid}:</strong> ${p.name.toUpperCase()}</div>
                                            <div class="chip-row" style="font-size: 0.75rem; opacity: 0.7; font-family: var(--font-mono)">CPU: ${p.cpu_usage.toFixed(1)}% | MEM: ${(p.mem_usage / 1024 / 1024).toFixed(1)} MB</div>
                                        </div>
                                        ${p.snapshot ? `
                                            <div class="process-snapshot" style="width: 4rem; height: 2.5rem; border: 1px solid rgba(255,255,255,0.1); border-radius: 0.25rem; overflow: hidden; background: #000; display: flex; align-items: center; justify-content: center;">
                                                <img src="${p.snapshot}" style="width: 100%; height: 100%; object-fit: cover; opacity: 0.8;" alt="Process View" />
                                            </div>
                                        ` : `
                                            <div class="process-snapshot" style="width: 4rem; height: 2.5rem; border: 1px solid rgba(255,255,255,0.05); border-radius: 0.25rem; background: rgba(0,0,0,0.3); display: flex; align-items:center; justify-content:center; font-size: 0.5rem; color: rgba(255,255,255,0.2)">
                                                NO FRAME
                                            </div>
                                        `}
                                    </div>
                                `).join('')}
                            </div>
                        </div>
                    `;
                } else {
                    leftColumnHTML = `
                        <div class="context-chip glass-panel empty-chip" style="display: flex; align-items: center; justify-content: center; height: 100%;">
                            <div style="opacity: 0.5; text-align: center; font-style: italic; font-weight: 300;">AWAITING CONTEXT EXPORT...</div>
                        </div>
                    `;
                }
            }

            target.innerHTML = `
                <div class="dual-column-layout">
                    <div class="left-chip-column">
                        ${leftColumnHTML}
                    </div>
                    <div class="right-terminal-column">
                        <div class="terminal-container" style="font-family: var(--font-mono); font-size: 0.9rem; line-height: 1.6; height: 100%;">
                            ${termOutputs}
                            <div style="animation: blink 1s infinite">_</div>
                        </div>
                    </div>
                </div>
            `;
        } else if (this.currentMode === 'sectors') {
            const currentSector = this.state.sectors[this.state.active_sector_index];
            title.innerText = `APPLICATION FOCUS // ${currentSector?.name.toUpperCase() || 'UNKNOWN'}`;
            target.innerHTML = `
                <div class="focal-app-container glass-panel">
                    <div class="app-chrome">
                        <div class="app-title">MAIN_SURFACE // BINARY_X86_64</div>
                        <div class="app-controls">
                            <span class="dot"></span>
                            <span class="dot"></span>
                        </div>
                    </div>
                    <div class="app-content-overlay">
                        <div class="terminal-container" style="padding: 2rem; font-size: 1.1rem; color: var(--color-success);">
                            [APPLICATION SURFACE ACTIVE]
                            <br/><br/>
                            $ ./run_sector_agent --mode=autonomous
                            <br/>
                            INITIALIZING NEURAL NET... OK
                            <br/>
                            MAPPING TOPOLOGICAL SPACE... OK
                            <br/>
                            AWAITING OPERATOR INPUT_
                        </div>
                    </div>
                </div>
            `;
        } else if (this.currentMode === 'detail') {
            title.innerText = "DETAIL INSPECTOR // LEVEL 4";
            target.innerHTML = `
                <div class="dual-column-layout" style="gap: 2rem;">
                    <div class="glass-panel" style="flex: 1; padding: 1.5rem;">
                        <div class="chip-title">METADATA PROPERTIES</div>
                        <div class="chip-row"><strong>NAME:</strong> TOS_SEC_0.BIN</div>
                        <div class="chip-row"><strong>TYPE:</strong> ENCRYPTED_BUFFER</div>
                        <div class="chip-row"><strong>SIZE:</strong> 4.2 MB</div>
                        <div class="chip-row"><strong>CREATED:</strong> 2026-03-01T20:32:15Z</div>
                        <div class="chip-row"><strong>OWNER:</strong> SYSTEM (TRUST_TIER_5)</div>
                    </div>
                    <div class="glass-panel" style="flex: 1; padding: 1.5rem;">
                        <div class="chip-title">CRYPTO HASHING</div>
                        <div class="chip-row" style="word-break: break-all; font-family: var(--font-mono); font-size: 0.8rem;">
                            SHA256: 4e3b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b
                        </div>
                        <div class="chip-row"><strong>STATUS:</strong> VERIFIED ✅</div>
                    </div>
                </div>
            `;
        } else if (this.currentMode === 'buffer') {
            title.innerText = "RAW DATA BUFFER // LEVEL 5";
            let hex = '';
            for (let i = 0; i < 400; i++) {
                hex += Math.floor(Math.random() * 256).toString(16).padStart(2, '0') + ' ';
                if ((i + 1) % 16 === 0) hex += '<br/>';
            }
            target.innerHTML = `
                <div class="buffer-hex-view glass-panel" style="padding: 1.5rem; font-family: var(--font-mono); font-size: 0.75rem; color: var(--color-primary); overflow-y: auto; height: 100%;">
                    <div class="chip-title" style="margin-bottom: 1rem; color: var(--color-warning)">TELEMETRY_STREAM_HEX_0XFC23A</div>
                    ${hex}
                </div>
            `;
        } else if (this.currentMode === 'spatial') {
            title.innerText = "SPATIAL TOPOLOGY // 3D SHELL";
            const sectors = this.state.sectors || [];
            target.innerHTML = `
                <div class="spatial-container" style="perspective: 1200px; height: 100%; width: 100%; display: flex; align-items: center; justify-content: center; background: radial-gradient(circle at center, rgba(34, 154, 255, 0.05) 0%, transparent 70%);">
                    <div class="spatial-layer" style="transform-style: preserve-3d; width: 80%; height: 80%; display: flex; flex-wrap: wrap; gap: 4rem; justify-content: center; transform: rotateY(-15deg) rotateX(10deg);">
                        ${sectors.map((s, idx) => `
                            <div class="spatial-panel glass-panel" style="width: 15rem; height: 10rem; transform: translateZ(${idx * 40}px); flex-shrink: 0; display: flex; flex-direction: column; padding: 1rem; border: 2px solid var(--color-primary); box-shadow: 0 0 30px rgba(0,0,0,0.5); animation: float ${3 + idx}s ease-in-out infinite alternate;">
                                <div class="chip-title" style="font-size: 0.9rem; margin-bottom: 0.5rem; border-bottom: 1px solid rgba(255,255,255,0.1)">SECTOR_${idx} // ${s.name.toUpperCase()}</div>
                                <div style="font-size: 0.7rem; opacity: 0.8; margin-top: auto;">
                                    <div>LOAD: 12%</div>
                                    <div>LATENCY: 0.2ms</div>
                                    <div class="status-badge active" style="font-size: 0.6rem; margin-top: 0.5rem; width: fit-content; background: #000; color: var(--color-success); border-radius: 1rem; padding: 0.1rem 0.5rem;">STABLE</div>
                                </div>
                            </div>
                        `).join('')}
                    </div>
                </div>
            `;
        }
    }

    renderAllSlots() {
        const containers = {
            left: document.getElementById('left-slots'),
            right: document.getElementById('right-slots'),
            top_left: document.getElementById('top-left-slots'),
            top_center: document.getElementById('top-center-slots'),
            top_right: document.getElementById('top-right-slots')
        };

        Object.keys(this.slotConfigs).forEach(slotId => {
            const container = containers[slotId];
            if (!container) return;

            container.innerHTML = this.slotConfigs[slotId].map(compId => this.renderComponent(compId)).join('');
        });
    }

    renderComponent(id) {
        const state = this.componentStates[id];
        const isProjected = state?.projected;
        const classNames = `slot - component ${isProjected ? '' : 'toggled-off'} `;

        if (id === 'minimap') {
            return `
                        < div class="${classNames}" id = "comp-minimap" >
                            <div class="minimap-container ${isProjected ? '' : 'toggled-off'}" onclick="window.tos.toggleMinimap()">
                                <div class="minimap-content">
                                    ${this.state.sectors.map((s, i) => `
                                <div class="minimap-sector ${i === this.state.active_sector_index ? 'active' : ''}">
                                    S${i} <div class="minimap-hub ${i === this.state.active_sector_index ? 'active' : ''}"></div>
                                </div>
                            `).join('')}
                                </div>
                            </div>
                </div > `;
        }

        if (id === 'priority') {
            return `
                        < div class="${classNames}" id = "comp-priority" >
                            <div class="minimap-container ${isProjected ? '' : 'toggled-off'}" onclick="window.tos.togglePriority()">
                                <div class="minimap-content">
                                    ${this.state.system_log.slice(-3).map(log => `
                                <div class="minimap-sector" style="border-left: 2px solid var(--color-warning)">
                                    ${log.text.toUpperCase()}
                                </div>
                            `).join('')}
                                </div>
                            </div>
                </div > `;
        }

        if (id === 'telemetry') {
            return `
                        < div class="${classNames}" id = "comp-telemetry" onclick = "window.tos.toggleTelemetry()" >
                    <div class="status-badge active" style="cursor:pointer">TELEMETRY</div>
                    <div class="slot-overlay glass-panel">
                        <div style="color:var(--color-primary); font-weight:800">SYSTEM TELEMETRY</div>
                        <div style="font-size:0.8rem; margin-top:0.5rem">
                            CPU: 12% | MEM: 4.2GB | NET: 1.2MB/s
                        </div>
                    </div>
                </div > `;
        }

        if (id === 'minilog') {
            const lastLog = this.state.system_log && this.state.system_log.length > 0
                ? this.state.system_log[this.state.system_log.length - 1]
                : { text: "SYSTEM READY.", priority: 0 };
            const text = (lastLog.text || "").toUpperCase();

            return `
                        < div class="${classNames}" id = "comp-minilog" onclick = "window.tos.toggleMinilog()" >
                            <div class="minimap-container ${isProjected ? '' : 'toggled-off'}" style="height: 0.5rem; border-color: var(--color-accent)">
                                <div class="minimap-content" style="font-size: 0.6rem; white-space: nowrap; overflow: hidden; color: var(--color-success)">
                                    ${text}
                                </div>
                            </div>
                </div > `;
        }

        if (id === 'screen_title') {
            const prefix = this.state.sys_prefix || "";
            const title = this.state.sys_title || "";
            return `
                        < div class="${classNames} lcars-title-area" id = "comp-screen-title" onclick = "window.tos.toggleScreenTitle()" >
                    <span class="lcars-prefix" style="color:var(--color-primary); opacity: 0.7; font-weight: 300">${prefix}</span>
                    <span class="lcars-title" style="font-weight:700; margin-left:1rem">${title}</span>
                </div > `;
        }

        if (id === 'brain_status') {
            const time = this.state.brain_time || "--:--:--";
            const status = (this.state.sys_status || "DISCONNECTED").toUpperCase();
            return `
                <div class="${classNames} lcars-status" id="comp-brain-status" onclick="window.tos.toggleBrainStatus()" style="display:flex; align-items:center">
                    <span style="font-size: 0.6rem; opacity: 0.5; margin-right: 0.5rem">BRAIN TIME</span>
                    <span id="status-clock" style="font-family:monospace; margin-right:1rem">${time}</span>
                    <span class="status-badge active" style="background: #000; color: var(--color-success); padding: 0.125rem 0.625rem; border-radius: 1rem; font-size: 0.75rem; border: 1px solid rgba(255, 255, 255, 0.1)">${status}</span>
                </div>`;
        }

        if (id === 'status_badges') {
            return `
                <div class="${classNames} lcars-status" id="comp-status-badges" style="display:flex; gap:0.5rem; align-items:center">
                    <button class="v-btn bezel-btn" title="Toggle Terminal" onclick="event.stopPropagation(); window.tos.handleCommand('bezel:term-toggle')">👁</button>
                    <button class="v-btn bezel-btn" title="Web Portal" onclick="event.stopPropagation(); window.tos.handleCommand('bezel:portal')">📡</button>
                    <button class="v-btn bezel-btn" title="Settings" onclick="event.stopPropagation(); window.tos.handleCommand('bezel:settings')">⚙</button>
                </div>`;
        }

        return '';
    }

    togglePortalModal(show) {
        const modal = document.getElementById('portal-modal');
        if (!modal) return;
        if (show) {
            modal.classList.remove('hidden');
            // Generate a random token for the mock UI
            const token = Math.random().toString(36).substring(2, 6) + '-' + Math.random().toString(36).substring(2, 6);
            const input = document.getElementById('portal-link-input');
            if (input) input.value = `https://tos.live/portal/${token}`;
            this.playEarcon('modal_open');
        } else {
            modal.classList.add('hidden');
            this.playEarcon('modal_close');
        }
    }

    copyPortalLink() {
        const input = document.getElementById('portal-link-input');
        if (input) {
            input.select();
            document.execCommand('copy');
            const btn = document.getElementById('portal-copy');
            if (btn) {
                const oldText = btn.innerText;
                btn.innerText = "COPIED!";
                btn.style.background = "var(--color-success)";
                this.playEarcon('bezel_tap');
                this.triggerHaptic('click');
                setTimeout(() => {
                    btn.innerText = oldText;
                    btn.style.background = "var(--color-primary)";
                }, 2000);
            }
        }
    }
    playEarcon(name) {
        if (window.__TOS_IPC__) {
            window.__TOS_IPC__(`play_earcon:${name}`).catch(e => console.error("Audio trigger failed:", e));
        }
    }
    triggerHaptic(cue) {
        if (window.__TOS_IPC__) {
            window.__TOS_IPC__(`trigger_haptic:${cue}`).catch(e => console.error("Haptic trigger failed:", e));
        }
    }
}

// Initializer
document.addEventListener('DOMContentLoaded', () => {
    window.tos = new TosUI();
});
