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
                    'ApplicationFocus': 'sectors'
                };
                this.setMode(modeMap[level] || 'global');
            });
        });

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
        });

        // 6.4 Bezel Commands
        document.getElementById('bezel-expand-left')?.addEventListener('click', () => this.toggleSidebar());
        document.getElementById('bezel-expand-right')?.addEventListener('click', () => this.toggleSidebarRight());
        document.getElementById('bezel-add-sector')?.addEventListener('click', () => this.handleCommand('bezel:add_sector'));
        document.getElementById('bezel-term-toggle')?.addEventListener('click', () => this.handleCommand('bezel:terminal_toggle'));
        document.getElementById('bezel-settings')?.addEventListener('click', () => this.handleCommand('bezel:settings'));
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

        if (window.__TOS_IPC__) {
            window.__TOS_IPC__(`set_mode:${mode}`);
        }
        document.querySelectorAll('.level-btn').forEach(btn => {
            // Map level button state
            const modeMap = {
                'GlobalOverview': 'global',
                'CommandHub': 'hubs',
                'ApplicationFocus': 'sectors'
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
            terminal_output: []
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
            title.innerText = "SECTOR TOPOLOGY";
            target.innerHTML = `
                <div class="loading-state">
                    [TOPOLOGICAL MAP DATA LOADING...]
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
        const classNames = `slot-component ${isProjected ? '' : 'toggled-off'}`;

        if (id === 'minimap') {
            return `
                <div class="${classNames}" id="comp-minimap">
                    <div class="minimap-container ${isProjected ? '' : 'toggled-off'}" onclick="window.tos.toggleMinimap()">
                        <div class="minimap-content">
                            ${this.state.sectors.map((s, i) => `
                                <div class="minimap-sector ${i === this.state.active_sector_index ? 'active' : ''}">
                                    S${i} <div class="minimap-hub ${i === this.state.active_sector_index ? 'active' : ''}"></div>
                                </div>
                            `).join('')}
                        </div>
                    </div>
                </div>`;
        }

        if (id === 'priority') {
            return `
                <div class="${classNames}" id="comp-priority">
                    <div class="minimap-container ${isProjected ? '' : 'toggled-off'}" onclick="window.tos.togglePriority()">
                        <div class="minimap-content">
                            ${this.state.system_log.slice(-3).map(log => `
                                <div class="minimap-sector" style="border-left: 2px solid var(--color-warning)">
                                    ${log.text.toUpperCase()}
                                </div>
                            `).join('')}
                        </div>
                    </div>
                </div>`;
        }

        if (id === 'telemetry') {
            return `
                <div class="${classNames}" id="comp-telemetry" onclick="window.tos.toggleTelemetry()">
                    <div class="status-badge active" style="cursor:pointer">TELEMETRY</div>
                    <div class="slot-overlay glass-panel">
                        <div style="color:var(--color-primary); font-weight:800">SYSTEM TELEMETRY</div>
                        <div style="font-size:0.8rem; margin-top:0.5rem">
                            CPU: 12% | MEM: 4.2GB | NET: 1.2MB/s
                        </div>
                    </div>
                </div>`;
        }

        if (id === 'minilog') {
            const lastLog = this.state.system_log && this.state.system_log.length > 0
                ? this.state.system_log[this.state.system_log.length - 1]
                : { text: "SYSTEM READY.", priority: 0 };
            const text = (lastLog.text || "").toUpperCase();

            return `
                <div class="${classNames}" id="comp-minilog" onclick="window.tos.toggleMinilog()">
                    <div class="minimap-container ${isProjected ? '' : 'toggled-off'}" style="height: 0.5rem; border-color: var(--color-accent)">
                        <div class="minimap-content" style="font-size: 0.6rem; white-space: nowrap; overflow: hidden; color: var(--color-success)">
                            ${text}
                        </div>
                    </div>
                </div>`;
        }

        if (id === 'screen_title') {
            const prefix = this.state.sys_prefix || "";
            const title = this.state.sys_title || "";
            return `
                <div class="${classNames} lcars-title-area" id="comp-screen-title" onclick="window.tos.toggleScreenTitle()">
                    <span class="lcars-prefix" style="color:var(--color-primary); opacity: 0.7; font-weight: 300">${prefix}</span>
                    <span class="lcars-title" style="font-weight:700; margin-left:1rem">${title}</span>
                </div>`;
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
                    <button class="v-btn bezel-btn" title="Settings" onclick="event.stopPropagation(); window.tos.handleCommand('bezel:settings')">⚙</button>
                </div>`;
        }

        return '';
    }
}

// Initializer
document.addEventListener('DOMContentLoaded', () => {
    window.tos = new TosUI();
});
