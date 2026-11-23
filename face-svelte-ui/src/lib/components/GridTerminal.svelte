<script lang="ts">
    // The background terminal surface: a painter of Brain-given
    // display state, and a capture of raw input — nothing more
    // (Architecture §26.3). The Brain owns the terminal emulation:
    // it parses the PTY stream into a grid and publishes snapshots;
    // this component draws the cells it is given and sends key
    // events and text back uninterpreted, for the Brain to encode.
    import {
        addRawListener,
        removeRawListener,
        sendCommand,
        sendTerminalKey,
        sendTerminalText,
        getConnectionState,
        getPrediction,
        clearPrediction,
        type TerminalKeyEvent
    } from '../stores/ipc.svelte';

    interface GridSpan {
        text: string;
        fg: string;
        bg: string;
        bold: boolean;
        dim: boolean;
        italic: boolean;
        underline: boolean;
        strike: boolean;
    }

    interface GridSnapshot {
        cols: number;
        rows: number;
        seq: number;
        display_offset: number;
        cursor_col: number;
        cursor_row: number;
        cursor_visible: boolean;
        lines: GridSpan[][];
    }

    let wrapElement = $state<HTMLDivElement | null>(null);
    let canvasEl = $state<HTMLCanvasElement | null>(null);
    let inputEl = $state<HTMLInputElement | null>(null);
    let predictionText = $derived(getPrediction());

    const FONT_FAMILY = '"Fira Code", "JetBrains Mono", "Courier New", monospace';
    const FONT_SIZE = 13;
    let cellW = 8;
    let cellH = 17;
    let colors = { fg: '#e0f7fa', bg: '#0b1622', cursor: '#00e5ff' };

    let snapshot = $state<GridSnapshot | null>(null);
    let sessionId: string | null = null;
    let isComposing = false;

    function acceptPrediction() {
        if (predictionText) {
            sendTerminalText(predictionText);
            clearPrediction();
        }
    }

    function resolveColor(c: string): string {
        if (c === 'fg') return colors.fg;
        if (c === 'bg') return colors.bg;
        if (c === 'cursor') return colors.cursor;
        return c;
    }

    // Cell width of a character: East Asian wide characters take
    // two cells in the Brain's grid, the rest take one.
    function charCells(ch: string): number {
        const c = ch.codePointAt(0) ?? 0;
        if (
            (c >= 0x1100 && c <= 0x115f) ||
            (c >= 0x2e80 && c <= 0xa4cf) ||
            (c >= 0xac00 && c <= 0xd7a3) ||
            (c >= 0xf900 && c <= 0xfaff) ||
            (c >= 0xfe30 && c <= 0xfe4f) ||
            (c >= 0xff00 && c <= 0xff60) ||
            (c >= 0xffe0 && c <= 0xffe6) ||
            c >= 0x20000
        ) {
            return 2;
        }
        return 1;
    }

    function paint() {
        if (!canvasEl || !snapshot) return;
        const ctx = canvasEl.getContext('2d');
        if (!ctx) return;
        const snap = snapshot;
        const cw = canvasEl.clientWidth;
        const ch = canvasEl.clientHeight;
        // Fit the Brain's grid cleanly into the drawable canvas: use
        // 1:1 measured cells when they fit, or scale down uniformly if
        // another face sized the session wider/taller than this viewport.
        const scaleX = snap.cols > 0 ? Math.min(1, cw / (snap.cols * cellW)) : 1;
        const scaleY = snap.rows > 0 ? Math.min(1, ch / (snap.rows * cellH)) : 1;
        const scale = Math.min(scaleX, scaleY);
        const drawCellW = cellW * scale;
        const drawCellH = cellH * scale;
        const drawFontSize = Math.max(8, FONT_SIZE * scale);

        ctx.fillStyle = colors.bg;
        ctx.fillRect(0, 0, cw, ch);
        ctx.textBaseline = 'top';
        for (let row = 0; row < snap.lines.length; row++) {
            const y = row * drawCellH;
            let col = 0;
            for (const span of snap.lines[row]) {
                const widthCells = [...span.text].reduce((n, c) => n + charCells(c), 0);
                if (span.bg !== 'bg') {
                    ctx.fillStyle = resolveColor(span.bg);
                    ctx.fillRect(col * drawCellW, y, widthCells * drawCellW, drawCellH);
                }
                ctx.font = `${span.italic ? 'italic ' : ''}${span.bold ? 'bold ' : ''}${drawFontSize}px ${FONT_FAMILY}`;
                ctx.fillStyle = resolveColor(span.fg);
                if (span.dim) ctx.globalAlpha = 0.6;
                let x = col * drawCellW;
                for (const c of span.text) {
                    ctx.fillText(c, x, y + 2 * scale);
                    x += charCells(c) * drawCellW;
                }
                ctx.globalAlpha = 1;
                if (span.underline) {
                    ctx.fillRect(col * drawCellW, y + drawCellH - 2, widthCells * drawCellW, 1);
                }
                if (span.strike) {
                    ctx.fillRect(col * drawCellW, y + drawCellH / 2, widthCells * drawCellW, 1);
                }
                col += widthCells;
            }
        }
        if (snap.cursor_visible && snap.display_offset === 0) {
            const cx = snap.cursor_col * drawCellW;
            const cy = snap.cursor_row * drawCellH;
            ctx.fillStyle = colors.cursor;
            ctx.globalAlpha = 0.85;
            ctx.fillRect(cx, cy, drawCellW, drawCellH);
            ctx.globalAlpha = 1;
            // Redraw the cell's character in the background colour.
            const row = snap.lines[snap.cursor_row];
            if (row) {
                let col = 0;
                for (const span of row) {
                    for (const c of span.text) {
                        if (col === snap.cursor_col) {
                            ctx.font = `${span.italic ? 'italic ' : ''}${span.bold ? 'bold ' : ''}${drawFontSize}px ${FONT_FAMILY}`;
                            ctx.fillStyle = colors.bg;
                            ctx.fillText(c, cx, cy + 2 * scale);
                        }
                        col += charCells(c);
                    }
                }
            }
        }
    }

    function applyDisplayMessage(message: string) {
        const kind = message.startsWith('display_snapshot:')
            ? 'display_snapshot:'
            : message.startsWith('display_update:')
              ? 'display_update:'
              : null;
        if (!kind) return;
        const rest = message.substring(kind.length);
        // The session id itself contains ':' — the JSON payload
        // starts at the first '{'.
        const jsonStart = rest.indexOf('{');
        if (jsonStart < 1) return;
        const sess = rest.substring(0, jsonStart - 1);
        try {
            const snap = JSON.parse(rest.substring(jsonStart)) as GridSnapshot;
            if (kind === 'display_update:') {
                // Updates belong to the subscribed session, in
                // sequence. (A snapshot, by contrast, is an
                // explicit resync: it always lands and resets the
                // baseline — after a Brain restart the session id
                // and sequence start over, and the snapshot is the
                // only thing that can say so.)
                if (sessionId && sess !== sessionId) return;
                if (snapshot && snap.seq < snapshot.seq) return;
            }
            sessionId = sess;
            snapshot = snap;
            paint();
        } catch {
            // A malformed push is ignored; the next update or a
            // resubscribe restores the truth.
        }
    }

    async function subscribe() {
        const res = await sendCommand('display_subscribe:');
        if (res) applyDisplayMessage(res);
    }

    function handleKey(e: KeyboardEvent) {
        if (e.metaKey || isComposing || e.isComposing) return; // leave OS/browser shortcuts & IME alone
        const fromInput = e.target === inputEl;
        const isTextKey = e.key.length === 1 && !e.ctrlKey && !e.altKey;
        if (fromInput && isTextKey) return; // arrives via the input event as text
        if (e.key === 'Tab' && predictionText && !e.ctrlKey && !e.altKey) {
            e.preventDefault();
            acceptPrediction();
            return;
        }
        e.preventDefault();
        const ev: TerminalKeyEvent = {
            key: e.key,
            code: e.code,
            ctrl: e.ctrlKey,
            alt: e.altKey,
            shift: e.shiftKey
        };
        sendTerminalKey(ev);
    }

    function handleTextInput(e?: Event) {
        if (!inputEl) return;
        if (isComposing || (e && 'isComposing' in e && (e as InputEvent).isComposing)) return;
        const v = inputEl.value;
        if (v) {
            sendTerminalText(v);
            inputEl.value = '';
        }
    }

    function handlePaste(e: ClipboardEvent) {
        const text = e.clipboardData?.getData('text');
        if (text) {
            e.preventDefault();
            sendTerminalText(text);
        }
    }

    let wheelAcc = 0;
    let wheelTimer: ReturnType<typeof setTimeout> | null = null;
    function queueScroll(deltaPx: number) {
        wheelAcc += deltaPx;
        if (wheelTimer) return;
        wheelTimer = setTimeout(async () => {
            wheelTimer = null;
            // Positive deltaPx (wheel up / finger drag down) looks at older lines.
            const lines = Math.trunc(wheelAcc / cellH);
            wheelAcc = 0;
            if (lines !== 0) {
                const res = await sendCommand(`display_scroll:;${lines}`);
                if (res) applyDisplayMessage(res);
            }
        }, 40);
    }

    function handleWheel(e: WheelEvent) {
        e.preventDefault();
        queueScroll(-e.deltaY);
    }

    let touchLastY: number | null = null;
    function handleTouchStart(e: TouchEvent) {
        if (e.touches.length === 1) {
            touchLastY = e.touches[0].clientY;
        }
    }
    function handleTouchMove(e: TouchEvent) {
        if (e.touches.length !== 1 || touchLastY === null) return;
        const y = e.touches[0].clientY;
        const dy = y - touchLastY;
        touchLastY = y;
        if (Math.abs(dy) >= 2) {
            e.preventDefault();
            // Dragging finger down pulls older lines into view.
            queueScroll(dy);
        }
    }
    function handleTouchEnd() {
        touchLastY = null;
    }

    async function jumpToLive() {
        const offset = snapshot?.display_offset ?? 0;
        if (offset > 0) {
            const res = await sendCommand(`display_scroll:;${-offset}`);
            if (res) applyDisplayMessage(res);
        }
    }

    $effect(() => {
        if (!wrapElement || !canvasEl) return;

        const style = window.getComputedStyle(document.documentElement);
        colors = {
            fg: style.getPropertyValue('--color-text').trim() || colors.fg,
            bg: style.getPropertyValue('--color-bg').trim() || colors.bg,
            cursor: style.getPropertyValue('--color-primary').trim() || colors.cursor
        };

        // Measure the cell from the actual font.
        const scratch = document.createElement('canvas').getContext('2d');
        if (scratch) {
            scratch.font = `${FONT_SIZE}px ${FONT_FAMILY}`;
            cellW = Math.max(1, scratch.measureText('MMMMMMMMMM').width / 10);
            cellH = Math.ceil(FONT_SIZE * 1.31);
        }

        // E2E/debug handle: the gated specs read what is on screen
        // through this — the grid's text rows, exactly as painted.
        const handle = {
            get session() {
                return sessionId;
            },
            get snapshot() {
                return snapshot;
            },
            rowsText: () =>
                (snapshot?.lines ?? []).map((row) => row.map((s) => s.text).join(''))
        };
        (wrapElement as unknown as { __tosGrid?: typeof handle }).__tosGrid = handle;

        const socketListener = (message: string) => {
            if (message === 'tos:reconnected') {
                // Resubscribe: the snapshot is the whole truth —
                // there is nothing to clear and nothing to replay.
                subscribe();
                return;
            }
            applyDisplayMessage(message);
        };
        addRawListener(socketListener);
        subscribe();
        // The mount can beat the connection: until the first
        // snapshot lands, keep asking whenever the store says we
        // are connected. (The stream-era surface never raced this
        // — the Brain pushed its replay at connect time.)
        const subscribeTimer = setInterval(() => {
            if (!sessionId && getConnectionState() === 'connected') subscribe();
        }, 500);

        let lastRows = 0;
        let lastCols = 0;
        let resizeTimeout: ReturnType<typeof setTimeout>;
        const resizeObserver = new ResizeObserver(() => {
            clearTimeout(resizeTimeout);
            resizeTimeout = setTimeout(() => {
                if (!wrapElement || !canvasEl) return;
                const w = canvasEl.clientWidth;
                const h = canvasEl.clientHeight;
                const dpr = window.devicePixelRatio || 1;
                canvasEl.width = Math.max(1, Math.floor(w * dpr));
                canvasEl.height = Math.max(1, Math.floor(h * dpr));
                const ctx = canvasEl.getContext('2d');
                ctx?.setTransform(dpr, 0, 0, dpr, 0, 0);
                const cols = Math.max(2, Math.floor(w / cellW));
                const rows = Math.max(2, Math.floor(h / cellH));
                if (rows !== lastRows || cols !== lastCols) {
                    lastRows = rows;
                    lastCols = cols;
                    sendCommand(`terminal_resize:${rows};${cols}`);
                }
                paint();
            }, 100);
        });
        resizeObserver.observe(wrapElement);

        return () => {
            clearTimeout(resizeTimeout);
            clearInterval(subscribeTimer);
            if (wheelTimer) clearTimeout(wheelTimer);
            removeRawListener(socketListener);
            resizeObserver.disconnect();
        };
    });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
    class="grid-terminal-wrapper"
    bind:this={wrapElement}
    data-testid="grid-terminal"
    tabindex="0"
    role="application"
    aria-label="Terminal"
    onkeydown={handleKey}
    onpaste={handlePaste}
    onwheel={handleWheel}
    ontouchstart={handleTouchStart}
    ontouchmove={handleTouchMove}
    ontouchend={handleTouchEnd}
    onclick={() => inputEl?.focus({ preventScroll: true })}
>
    <canvas bind:this={canvasEl} class="grid-terminal-canvas"></canvas>
    <!-- Soft-keyboard / IME path: committed text is forwarded as
         text events; the field itself never holds state. Keydown
         bubbles once to .grid-terminal-wrapper so non-text keys
         are never dispatched twice. -->
    <input
        bind:this={inputEl}
        class="grid-terminal-input"
        type="text"
        autocomplete="off"
        autocapitalize="none"
        spellcheck="false"
        aria-hidden="true"
        tabindex="-1"
        oncompositionstart={() => { isComposing = true; }}
        oncompositionend={() => { isComposing = false; handleTextInput(); }}
        oninput={handleTextInput}
    />
    {#if (snapshot?.display_offset ?? 0) > 0}
        <button
            type="button"
            class="scrollback-pill"
            data-testid="grid-scrollback-pill"
            aria-label="Jump to live terminal"
            onclick={(e) => { e.stopPropagation(); jumpToLive(); }}
        >
            ↑ {snapshot?.display_offset} · LIVE ↵
        </button>
    {/if}
    {#if predictionText}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <div class="autocomplete-chip" onclick={acceptPrediction} role="button" tabindex="0">
            <span class="prediction-text">{predictionText}</span>
            <span class="tab-badge">Tab to Accept</span>
        </div>
    {/if}
</div>

<style>
    .grid-terminal-wrapper {
        position: relative;
        width: 100%;
        height: 100%;
        min-height: 200px;
        padding: 8px;
        box-sizing: border-box;
        background: var(--color-bg);
        border: 1px solid var(--color-border);
        border-radius: 4px;
        overflow: hidden;
        outline: none;
        touch-action: none;
    }

    .grid-terminal-canvas {
        width: 100%;
        height: 100%;
        display: block;
    }

    .scrollback-pill {
        position: absolute;
        bottom: 12px;
        right: 12px;
        min-height: 44px;
        min-width: 44px;
        padding: 8px 14px;
        border-radius: 999px;
        border: 1px solid var(--color-primary, #00e5ff);
        background: rgba(11, 22, 34, 0.9);
        color: var(--color-primary, #00e5ff);
        font-family: var(--font-mono, monospace);
        font-size: 12px;
        font-weight: 600;
        cursor: pointer;
        z-index: 12;
        touch-action: manipulation;
    }

    .grid-terminal-input {
        position: absolute;
        bottom: 0;
        left: 0;
        width: 40px;
        height: 24px;
        opacity: 0.01;
        border: none;
        padding: 0;
        background: transparent;
        color: transparent;
        caret-color: transparent;
    }

    .autocomplete-chip {
        position: absolute;
        bottom: 8px;
        right: 12px;
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 4px 10px;
        background: rgba(11, 22, 34, 0.85);
        backdrop-filter: blur(8px);
        border: 1px solid var(--color-primary);
        border-radius: 20px;
        font-family: var(--font-mono, monospace);
        font-size: 11px;
        color: var(--color-primary);
        box-shadow: 0 4px 12px rgba(0, 229, 255, 0.15);
        pointer-events: auto;
        cursor: pointer;
        z-index: 10;
        animation: pulse 1.5s infinite alternate;
        transition: all 0.2s ease;
    }

    .autocomplete-chip:hover {
        background: var(--color-primary);
        color: var(--color-bg);
        border-color: var(--color-primary);
        box-shadow: 0 4px 16px var(--color-primary);
    }

    .autocomplete-chip:hover .tab-badge {
        background: var(--color-bg);
        color: var(--color-primary);
    }

    .tab-badge {
        padding: 2px 5px;
        background: var(--color-primary);
        color: var(--color-bg);
        border-radius: 3px;
        font-weight: bold;
        font-size: 9px;
        transition: all 0.2s ease;
    }

    @keyframes pulse {
        from {
            border-color: rgba(0, 229, 255, 0.4);
            box-shadow: 0 4px 12px rgba(0, 229, 255, 0.05);
        }
        to {
            border-color: rgba(0, 229, 255, 1);
            box-shadow: 0 4px 12px rgba(0, 229, 255, 0.35);
        }
    }
</style>
