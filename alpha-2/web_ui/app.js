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

        // Mode Navigation
        document.querySelectorAll('.mode-btn').forEach(btn => {
            btn.addEventListener('click', () => {
                const mode = btn.dataset.mode;
                this.setMode(mode);
            });
        });

        // Zoom Controls
        document.getElementById('zoom-in')?.addEventListener('click', () => this.handleCommand('zoom_in:'));
        document.getElementById('zoom-out')?.addEventListener('click', () => this.handleCommand('zoom_out:'));

        // 6.4 Bezel Commands
        document.getElementById('bezel-expand')?.addEventListener('click', () => this.handleCommand('bezel:expand'));
        document.getElementById('bezel-add-sector')?.addEventListener('click', () => this.handleCommand('bezel:add_sector'));
        document.getElementById('bezel-term-toggle')?.addEventListener('click', () => this.handleCommand('bezel:terminal_toggle'));
        document.getElementById('bezel-settings')?.addEventListener('click', () => this.handleCommand('bezel:settings'));
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
        document.querySelectorAll('.mode-btn').forEach(btn => {
            btn.classList.toggle('active', btn.dataset.mode === mode);
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

        // Render headers from state
        const prefixEl = document.getElementById('sys-prefix');
        const titleEl = document.getElementById('sys-title');
        const statusEl = document.getElementById('status-brain');
        const logEl = document.getElementById('mini-log');
        const clockEl = document.getElementById('status-clock');

        if (prefixEl) prefixEl.innerText = this.state.sys_prefix || "";
        if (titleEl) titleEl.innerText = this.state.sys_title || "";
        if (statusEl) statusEl.innerText = this.state.sys_status || "";
        if (clockEl) clockEl.innerText = this.state.brain_time || "--:--:--";

        // Render Collaboration Indicators (6.4)
        const collabEl = document.querySelector('.collab-indicators');
        if (collabEl && this.state.collab_presence) {
            collabEl.innerHTML = this.state.collab_presence.map(u =>
                `<div class="collab-dot" style="background: ${u.color}"></div>`
            ).join('');
        }

        // Drive Mini-Log from system_log (Authoritative Telemetry)
        if (logEl && (!this.lastFeedback || Date.now() - this.lastFeedback > 3000)) {
            const lastLog = this.state.system_log && this.state.system_log.length > 0
                ? this.state.system_log[this.state.system_log.length - 1]
                : null;

            if (lastLog && typeof lastLog === 'object' && lastLog.text) {
                logEl.innerText = lastLog.text.toUpperCase();
                logEl.style.color = lastLog.priority >= 3 ? 'var(--color-warning)' : 'var(--color-accent)';
            } else if (typeof lastLog === 'string') {
                logEl.innerText = lastLog.toUpperCase();
                logEl.style.color = 'var(--color-accent)';
            } else {
                logEl.innerText = this.state.sys_ready || "SYSTEM READY.";
                logEl.style.color = 'var(--color-success)';
            }
        }

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
            const outputs = this.state.terminal_output ?
                this.state.terminal_output.map(l => `<div style="color: ${l.color || 'inherit'}">${l.text}</div>`).join('') : '';

            target.innerHTML = `
                <div class="terminal-container" style="font-family: monospace; font-size: 0.9rem; line-height: 1.6">
                    ${outputs}
                    <div style="animation: blink 1s infinite">_</div>
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
}

// Initializer
document.addEventListener('DOMContentLoaded', () => {
    window.tos = new TosUI();
});
