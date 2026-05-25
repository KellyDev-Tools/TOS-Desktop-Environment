<script lang="ts">
    import { Terminal } from 'xterm';
    import { FitAddon } from 'xterm-addon-fit';
    import { WebLinksAddon } from 'xterm-addon-web-links';
    import 'xterm/css/xterm.css';
    import { addRawListener, removeRawListener, sendCommand, getPrediction, clearPrediction } from '../stores/ipc.svelte';

    let terminalElement = $state<HTMLDivElement | null>(null);
    let term: Terminal | null = null;
    let fitAddon: FitAddon | null = null;
    let predictionText = $derived(getPrediction());

    function acceptPrediction() {
        if (predictionText) {
            const encoder = new TextEncoder();
            const hex = Array.from(encoder.encode(predictionText))
                .map((b) => b.toString(16).padStart(2, '0'))
                .join('');
            sendCommand(`terminal_input_hex:${hex}`);
            clearPrediction();
        }
    }

    $effect(() => {
        if (!terminalElement) return;

        // Fetch CSS custom properties to initialize terminal theme dynamically
        const style = window.getComputedStyle(document.documentElement);
        const colorPrimary = style.getPropertyValue('--color-primary').trim() || '#00e5ff';
        const colorText = style.getPropertyValue('--color-text').trim() || '#e0f7fa';
        const colorBg = style.getPropertyValue('--color-bg').trim() || '#0b1622';

        term = new Terminal({
            cursorBlink: true,
            fontFamily: 'Fira Code, JetBrains Mono, Courier New, Courier, monospace',
            fontSize: 13,
            theme: {
                background: colorBg,
                foreground: colorText,
                cursor: colorPrimary,
                cursorAccent: colorBg,
                selectionBackground: 'rgba(0, 229, 255, 0.3)',
            },
            convertEol: true
        });

        fitAddon = new FitAddon();
        term.loadAddon(fitAddon);

        // WebLinksAddon: intercepts file paths like "src/main.js:14" and triggers editor open
        const webLinksAddon = new WebLinksAddon((event, uri) => {
            console.log('[Terminal WebLink Clicked]', uri);
            const match = uri.match(/^(?:file:\/\/\/)?([^:]+)(?::(\d+))?$/);
            if (match) {
                const path = match[1];
                const line = match[2] || '1';
                sendCommand(`editor_open:${path};${line}`);
            } else {
                window.open(uri, '_blank');
            }
        }, {
            urlRegex: /(?:https?:\/\/|localhost:|127\.0\.0\.1:|[a-zA-Z0-9_\-\.\/]+?\.[a-zA-Z0-9_\-\.]+?)(?::\d+)?(?:\/[^\s"']*)?/
        });
        term.loadAddon(webLinksAddon);

        term.open(terminalElement);
        fitAddon.fit();

        // Intercept Tab key to support prediction acceptance and prevent focus navigation loss
        term.attachCustomKeyEventHandler((e) => {
            if (e.key === 'Tab') {
                e.preventDefault();
                if (e.type === 'keydown') {
                    if (predictionText) {
                        // Accept prediction: hex-encode predictionText and send to backend
                        const encoder = new TextEncoder();
                        const hex = Array.from(encoder.encode(predictionText))
                            .map((b) => b.toString(16).padStart(2, '0'))
                            .join('');
                        sendCommand(`terminal_input_hex:${hex}`);
                        clearPrediction();
                    } else {
                        // Send Tab key hex (09) to terminal for standard shell autocompletion
                        sendCommand(`terminal_input_hex:09`);
                    }
                }
                return false;
            }
            return true;
        });

        // Handle font loading delay: re-set fontFamily to clear cached cell measurements and refit
        const forceFontRemeasure = () => {
            try {
                if (term && fitAddon) {
                    const currentFont = term.options.fontFamily;
                    term.options.fontFamily = 'monospace';
                    term.options.fontFamily = currentFont;
                    fitAddon.fit();
                    term.refresh(0, term.rows - 1);
                }
            } catch (e) {
                console.error('[Terminal Font Re-measure Error]', e);
            }
        };

        if (typeof document !== 'undefined' && document.fonts) {
            document.fonts.ready.then(forceFontRemeasure);
        }
        const fontTimeout = setTimeout(forceFontRemeasure, 500);

        // Socket listener: hex-decode pty_output bytes and write to xterm.js
        const socketListener = (message: string) => {
            if (message.startsWith('pty_output:')) {
                const hexPayload = message.substring(11);
                // Hex-decode to Uint8Array
                const bytes = new Uint8Array(
                    hexPayload.match(/.{1,2}/g)?.map((byte) => parseInt(byte, 16)) || []
                );
                term?.write(bytes);
            }
        };
        addRawListener(socketListener);

        // Terminal typing -> hex-encode and send terminal_input_hex to backend
        const onDataDisposable = term.onData((data) => {
            // Hex-encode typing data
            const encoder = new TextEncoder();
            const hex = Array.from(encoder.encode(data))
                .map((b) => b.toString(16).padStart(2, '0'))
                .join('');
            sendCommand(`terminal_input_hex:${hex}`);
        });

        let lastRows = 0;
        let lastCols = 0;
        let resizeTimeout: ReturnType<typeof setTimeout>;

        // Setup resize listener
        const resizeObserver = new ResizeObserver(() => {
            clearTimeout(resizeTimeout);
            resizeTimeout = setTimeout(() => {
                try {
                    fitAddon?.fit();
                    if (term && (term.rows !== lastRows || term.cols !== lastCols)) {
                        lastRows = term.rows;
                        lastCols = term.cols;
                        sendCommand(`terminal_resize:${term.rows};${term.cols}`);
                    }
                } catch (e) {
                    console.error('[Terminal Fit Error]', e);
                }
            }, 100);
        });
        resizeObserver.observe(terminalElement);

        return () => {
            clearTimeout(resizeTimeout);
            clearTimeout(fontTimeout);
            onDataDisposable.dispose();
            removeRawListener(socketListener);
            resizeObserver.disconnect();
            term?.dispose();
        };
    });
</script>

<div class="xterm-wrapper">
    <div bind:this={terminalElement} class="terminal-container"></div>
    {#if predictionText}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
        <div class="autocomplete-chip" onclick={acceptPrediction} role="button" tabindex="0">
            <span class="prediction-text">{predictionText}</span>
            <span class="tab-badge">Tab to Accept</span>
        </div>
    {/if}
</div>

<style>
    .xterm-wrapper {
        position: relative;
        width: 100%;
        height: 100%;
        min-height: 200px;
        background: var(--color-bg);
        border: 1px solid var(--color-border);
        border-radius: 4px;
        overflow: hidden;
    }

    .terminal-container {
        width: 100%;
        height: 100%;
        padding: 8px;
    }

    :global(.xterm) {
        height: 100%;
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
