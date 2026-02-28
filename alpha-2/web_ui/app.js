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
        this.startClock();
        this.simulateState();

        // Internal state refresh loop
        setInterval(() => this.simulateState(), 3000);

        console.log("TOS UI // ALPHA-3 // LOADED");
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

    simulateState() {
        // Mocking the TosState structure for Alpha-3 Prototype
        this.state = {
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
            ]
        };
        this.render();
    }

    render() {
        if (!this.state) return;

        const target = document.getElementById('state-render-target');
        const title = document.getElementById('view-title');
        if (!target || !title) return;

        if (this.currentMode === 'global') {
            title.innerText = "GLOBAL OVERVIEW";
            target.innerHTML = `
                <div class="state-grid">
                    ${this.state.sectors.map((s, i) => `
                        <div class="sector-tile ${i === this.state.active_sector_index ? 'active' : ''}">
                            <div style="font-size: 0.7rem; opacity: 0.6">S0${i}</div>
                            <div style="font-family: var(--font-display); font-weight: 700; color: var(--color-primary)">${s.name.toUpperCase()}</div>
                            <div style="font-size: 0.8rem; margin-top: 5px">TYPE: STANDARD</div>
                            <div style="font-size: 0.8rem">STATUS: ACTIVE</div>
                        </div>
                    `).join('')}
                </div>
                <div class="log-area" style="margin-top: 30px; border-top: 1px solid var(--glass-border); padding-top: 15px">
                    <div style="font-size: 0.8rem; font-family: var(--font-display); margin-bottom: 10px; color: var(--color-secondary)">BRAIN LOG_STREAM</div>
                    ${this.state.system_log.map(log => `
                        <div style="font-size: 0.75rem; margin-bottom: 4px; font-family: monospace; opacity: 0.8">> ${log}</div>
                    `).join('')}
                </div>
            `;
        } else if (this.currentMode === 'hubs') {
            title.innerText = "HUB VIEW // COMMAND";
            target.innerHTML = `
                <div class="terminal-container" style="font-family: monospace; font-size: 0.9rem; line-height: 1.6">
                    <div style="color: var(--color-success)">tim@tos-alpha3:~/dev$ ls -la</div>
                    <div>total 84</div>
                    <div>drwxr-xr-x 12 tim tim 4096 Feb 27 17:15 .</div>
                    <div>drwxr-xr-x  3 tim tim 4096 Feb 17 15:20 ..</div>
                    <div>drwxr-xr-x  2 tim tim 4096 Feb 27 17:15 <span style="color: var(--color-primary)">brain</span></div>
                    <div>-rw-r--r--  1 tim tim  828 Feb 17 18:15 Makefile</div>
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

    startClock() {
        const clock = document.getElementById('status-clock');
        const update = () => {
            const now = new Date();
            if (clock) clock.innerText = now.toTimeString().split(' ')[0];
        };
        update();
        setInterval(update, 1000);
    }
}

// Initializer
document.addEventListener('DOMContentLoaded', () => {
    window.tos = new TosUI();
});
