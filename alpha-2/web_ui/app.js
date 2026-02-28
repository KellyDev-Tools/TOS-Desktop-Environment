/**
 * TOS UI Controller
 * High-Fidelity Rendering & State Sync
 */

class TosUI {
    constructor() {
        this.state = null;
        this.currentMode = 'global';
        this.init();
    }

    init() {
        this.setupEventListeners();
        this.simulateState();

        // Internal state refresh loop
        setInterval(() => this.simulateState(), 3000);

        console.log("TOS UI // ALPHA-2.1 // LOADED");
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
        document.getElementById('zoom_out')?.addEventListener('click', () => this.handleCommand('zoom_out:'));
    }

    async setMode(mode) {
        const target = document.getElementById('state-render-target');
        if (target) target.classList.add('transitioning');

        // Cinematic Transition Delay
        await new Promise(r => setTimeout(r, 200));

        this.currentMode = mode;
        document.querySelectorAll('.mode-btn').forEach(btn => {
            btn.classList.toggle('active', btn.dataset.mode === mode);
        });

        this.render();

        if (target) target.classList.remove('transitioning');

        // Theme auto-switching based on mode (Demonstration)
        if (mode === 'sectors') this.setTheme('tactical');
        else if (mode === 'global') this.setTheme('default');
    }

    setTheme(theme) {
        document.body.classList.remove('theme-tactical', 'theme-alert');
        if (theme !== 'default') {
            document.body.classList.add(`theme-${theme}`);
        }
    }

    handleCommand(cmd) {
        const log = document.getElementById('mini-log');
        log.innerText = `TRANSMITTING: ${cmd.toUpperCase()}`;
        log.style.color = 'var(--color-primary)';

        // AI Intent Sniffing (UI-side simulation)
        if (cmd.toLowerCase().includes('help') || cmd.length > 20) {
            log.innerText = "INVOKING AI ENGINE...";
            log.style.color = 'var(--color-accent)';
        }

        console.log(`[TOS IPC] -> ${cmd}`);

        // Visual feedback
        setTimeout(() => {
            log.innerText = "COMMAND ACKNOWLEDGED // 12ms LATENCY";
            log.style.color = 'var(--color-success)';
        }, 500);
    }

    handleSectorClick(index) {
        this.handleCommand(`switch_sector:${index}`);

        // Optimistic UI state update for the mock prototype
        if (this.state && this.state.sectors[index]) {
            this.state.active_sector_index = index;
            this.setMode('hubs');
        }
    }

    simulateState() {
        // Mocking the TosState structure for Alpha-2.1 Prototype
        this.state = {
            sys_prefix: "TOS // SYSTEM-BRAIN",
            sys_title: "ALPHA-2.1 // INTEL-DRIVEN",
            sys_status: "BRAIN: ONLINE",
            sys_ready: "SYSTEM READY.",
            brain_time: new Date().toTimeString().split(' ')[0], // Mocking backend time string
            active_sector_index: 0,
            sectors: [
                { name: "Primary", hubs: [{ mode: 'Command' }] },
                { name: "Intelligence", hubs: [{ mode: 'Ai' }] },
                { name: "Staging", hubs: [{ mode: 'Directory' }] }
            ],
            system_log: [
                "BRAIN CORE ONLINE",
                "NETWORK STACK INITIALIZED",
                "COGNITIVE ENGINE LOADED"
            ],
            terminal_output: [
                { text: "> SYSTEM INITIALIZED...", color: "var(--color-success)" },
                { text: "> AWAITING INPUT FLOW...", color: "var(--color-primary)" }
            ]
        };
        this.render();
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
        if (logEl && logEl.innerText === "") {
            logEl.innerText = this.state.sys_ready || "";
        }

        if (!target || !title) return;

        if (this.currentMode === 'global') {
            title.innerText = "GLOBAL OVERVIEW";
            if (footer) footer.style.display = 'none';
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
                <div class="log-area" style="margin-top: 1.875rem; border-top: 1px solid var(--glass-border); padding-top: 0.9375rem">
                    <div style="font-size: 0.8rem; font-family: var(--font-display); margin-bottom: 0.625rem; color: var(--color-secondary)">BRAIN LOG_STREAM</div>
                    ${this.state.system_log.map(log => `
                        <div style="font-size: 0.75rem; margin-bottom: 0.25rem; font-family: monospace; opacity: 0.8">> ${log}</div>
                    `).join('')}
                </div>
            `;
        } else if (this.currentMode === 'hubs') {
            title.innerText = "HUB VIEW // COMMAND";
            if (footer) footer.style.display = 'block';
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
            if (footer) footer.style.display = 'block';
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
