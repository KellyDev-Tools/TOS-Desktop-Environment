# TOS v0.1.1 Roadmap — Terminal Foundation & Feature Audits

This document defines the development roadmap for the TOS v0.1.1 release. The main objective is to harden the terminal core by integrating `xterm.js` for raw PTY rendering (while maintaining floating transparent autocomplete suggestion overlays) and to fix the six critical bugs identified in our Project Audit.

All work must comply with the `TOS AI Development Standards.md`.

---

## 📋 Status Tracking Requirement
To ensure high visibility and structured tracking, every section, component, and task in this roadmap must be explicitly prefixed with one of the following visual status indicators:
- `✅ Completed` — The task or component is fully implemented, verified, and merged.
- `🛠️ WIP` (Work In Progress) — Active development is currently underway.
- `⏳ Pending` — Scheduled for implementation, but not yet started.

These indicators must be updated as progress is made.

---

## 🗺️ Roadmap Progress Overview

- [✅] **Component 1: Dependencies Setup** (Status: `✅ Completed`)
- [✅] **Component 2: Rust Backend - Broadcaster & Raw Input** (Status: `✅ Completed`)
- [✅] **Component 3: Svelte Frontend - xterm.js Integration** (Status: `✅ Completed`)
- [✅] **Component 4: Cinematic Element Archive** (Status: `✅ Completed`)
- [✅] **Component 5: Hotkeys, FPS Monitor & Sector Management** (Status: `✅ Completed`)
- [✅] **Verification & Validation** (Status: `✅ Completed`)

---

## 🛠️ Component Breakdown & Checklist

### Component 1: Dependencies Setup
**Status**: `✅ Completed`

- [x] Add `"xterm": "^5.3.0"`, `"xterm-addon-fit": "^0.8.0"`, and `"xterm-addon-web-links": "^0.9.0"` to the Svelte Face dependencies.
- [x] Run package installation to lock dependencies in `face-svelte-ui/`.

---

### Component 2: Rust Backend (Broadcaster & Raw Input support)
**Status**: `✅ Completed`

- [x] Modify `IpcHandler` struct to hold `broadcasters: Mutex<Vec<UnboundedSender<String>>>` in `ipc_handler.rs`.
- [x] Add `register_broadcaster` and `broadcast` helper methods in `ipc_handler.rs implementation block.
- [x] Modify `PtyShell` struct in `pty.rs` to hold `ipc: Arc<Mutex<Option<Weak<IpcHandler>>>>` to prevent reference cycles.
- [x] Expose `pub fn set_ipc(&self, ipc: &Arc<IpcHandler>)` on `PtyShell` to dynamically assign the IPC weak reference after initialization.
- [x] Modify `PtyShell::write(&mut self, data: &[u8])` to write raw bytes directly, enabling terminal arrow keys and control characters.
- [x] Update `read_loop` in `pty.rs` to hex-encode PTY output bytes and broadcast them:
  - Format: `pty_output:<hex_bytes>`
  - Ensure legacy line-buffered reading still runs in parallel to feed telemetry and AI parsing without breaking existing systems.
- [x] In `ipc_handler.rs` `handle_request`:
  - Intercept `"ai_prediction_received"` and `"pty_output"` requests to broadcast them to frontends, returning `"OK"`.
  - Implement `"terminal_input_hex"` handler to decode incoming keystroke hex sequences and write raw bytes into the `PtyShell`.
  - Update `handle_prompt_submit` to write bytes using `.as_bytes()`.
- [x] Wire the IPC handler back to the shell in `brain/mod.rs` after initialization: `shell.lock().unwrap().set_ipc(&ipc)`.
- [x] Register the WebSocket connection sender in `remote_server.rs` via `self.ipc.register_broadcaster(mpsc_tx.clone())`.

---

### Component 3: Svelte Frontend (xterm.js integration)
**Status**: `✅ Completed`

- [x] Create `XtermTerminal.svelte` (`face-svelte-ui/src/lib/components/XtermTerminal.svelte`):
  - Mount a container div using Svelte 5 `$effect`.
  - Bind layout colors, text, and styles to theme tokens (e.g. `var(--color-primary)`).
  - Subscribe to `pty_output` socket events, hex-decode the payload, and write directly to `xterm.js`.
  - Bind key press and data entry events: hex-encode inputs and send `"terminal_input_hex:<hex>"`.
  - Integrate `FitAddon` for responsive element resizing.
  - Setup a custom `WebLinksAddon` that parses file paths and absolute workspace links with line numbers (e.g., `src/main.js:14`), opening them in the Svelte editor layout with `"editor_open:<path>"`.
- [x] Expose a subscriber registry for raw events in `ipc.svelte.ts`.
- [x] Mount `<XtermTerminal />` in `CommandHub.svelte`, replacing the legacy `.term-line` listing.
- [x] Mount `<XtermTerminal />` in `SplitPaneView.svelte`, enabling real-time terminal views.
- [x] Overlay transparent absolute-positioned floating glassmorphic chips above the terminal canvas for predictions and autocomplete suggestions.

---

### Component 4: Cinematic Element Archive
**Status**: `✅ Completed`

- [x] Create `docs/archive/cinematic_intro.svelte` and archive all old atmospheric intro Svelte markup, skip/start triggers, animations, and CSS classes.
- [x] Clean up `face-svelte-ui/src/routes/+page.svelte` to remove introductory animation stages and focus on the professional work environment.

---

### Component 5: Hotkeys, FPS Monitor & Sector Management
**Status**: `✅ Completed`

- [x] Remap shortcuts in `+page.svelte`:
  - Map `Ctrl+T` to trigger `sendCommand("sector_create:")`.
  - Map the terminal drawer toggle to `Ctrl+Shift+T`.
- [x] Resolve the FPS Monitor warning feedback loop:
  - Throttle terminal FPS warnings to a maximum of 1 occurrence per 30 seconds.
  - Log locally using `console.warn` instead of passing warnings back to the backend.
- [x] Implement Sector Add visual component:
  - Add a beautiful visual "+" dashed/glassmorphic card at the end of the active sectors list in `GlobalOverview.svelte`.
  - Hook card click handler to trigger `sendCommand("sector_create:")`.
- [x] Implement Sector Close/Freeze actions:
  - Ensure "Close Sector" and "Freeze Sector" triggers in `SectorContextMenu.svelte` use the correct `sectorId` and trigger `sendCommand("sector_close:<id>")` / `sendCommand("sector_freeze:<id>")`.

---

## 🧪 Verification Plan
**Status**: `✅ Completed`

### Automated Tests
- Run `make check` to ensure Svelte and Rust files compile clean.
- Run `make test` to verify backend integrity.
- Run `make build-face-web` to assert production frontend builds compile successfully.

### Manual Verification
1. Launch Dev Env: `make run-web-dev`.
2. Inspect `xterm.js` rendering: run interactive applications (e.g., `htop`, `vi`, `ls -la`) in split and main pane terminals. Assert sub-10ms response time.
3. Validate path links: Click paths with lines (e.g. `src/lib/stores/ipc.svelte.ts:25`) and verify the editor correctly highlights and displays them.
4. Verify Sector controls: Validate `Ctrl+T` / "+" card additions, and Context Menu closes/freezes.
5. Verify Hotkeys: Verify `Ctrl+Shift+T` toggles terminal panels cleanly.
6. Verify Autocomplete: Observe AI predictions floating exactly above the command line inputs.
7. Verify Heuristics: Query or witness heuristics suggestions updating.
8. Verify FPS Throttling: Confirm console warnings are throttled under heavy renders.
