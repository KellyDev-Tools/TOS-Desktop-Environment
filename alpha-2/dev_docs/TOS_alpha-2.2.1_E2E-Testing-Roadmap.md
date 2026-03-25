# TOS Alpha-2.2.1 End-to-End Testing Roadmap

This roadmap defines the plan to graduate from the existing Playwright component tests (which mock the WebSocket connections) to a true End-to-End (E2E) testing suite that launches the production Rust Brain and dynamically interacts with real system state.

---

## Phase 1 — Framework Provisioning ✅
*Establish the structural capability to spawn the Svelte UI and Rust backend in tandem within the Playwright runner.*

- [x] **Dedicated Configuration:** `svelte_ui/playwright.e2e.config.ts`.
  - Override `testDir` to point to `svelte_ui/tests/e2e`.
- [x] **Global Setup Script:** `svelte_ui/tests/e2e/globalSetup.ts`.
  - Compiles Brain via `cargo build`, spawns `tos-brain --headless`, caches PID.
- [x] **Global Teardown Script:** `svelte_ui/tests/e2e/globalTeardown.ts`.
  - Graceful SIGTERM to cached Brain PID, prevents orphan processes.
- [x] **Makefile Integration:** `make test-e2e` in `alpha-2/Makefile`.
  - ⚠️ Run from `alpha-2/` directory: `cd alpha-2 && make test-e2e`
- [x] **Gitignore:** Added `/test-results` and `dom-e2e-dump.html` to `.gitignore`.

---

## Phase 2 — Core Infrastructure Validation 🟡
*The baseline smoke tests confirming data correctly hydrates the Face from the Brain.*

**File:** `tests/e2e/sanity.spec.ts`

- [x] **Sanity Hydration Test**
  - Verifies Svelte Face connects and `.status-badge` resolves to `BRAIN: ACTIVE`.
  - Bypasses onboarding cinematic via `Escape` + `SKIP TOUR` click.
  - Asserts first `.sector-tile` renders with the name `Primary` from the real Rust `TosState::default()`.
- **Known Issue:** The `SKIP TOUR` button click is intermittently intercepted by the `.global-overview` spatial layer during its fade-in animation. The onboarding overlay needs a higher z-index or `{ force: true }` click modifier.

---

## Phase 3 — PTY Execution Matrix 🟡
*Deep integration paths verifying shell inputs roundtrip from browser to system shell and back through WebSockets.*

**File:** `tests/e2e/terminal.spec.ts`

- [x] **3-A Command Roundtrip** *(previously passing, now blocked by onboarding race)*
  - Sends `echo "TOS_TEST_E2E_ALIVE"` via `#cmd-input` and asserts the output appears as a `.term-line`.
  - **Known Issue:** Depends on reliable onboarding bypass and sector-tile navigation to Level 2. The `bootToCommandHub` helper sets `tos.onboarding.*` localStorage keys, but the onboarding overlay animation still races the first render.
- [ ] **3-B Process Registration** *(blocked: ACT mode navigation)*
  - Spawns `sleep 15 &` and navigates to ACT mode; asserts `.activity-item .proc-name` lists `sleep`.
  - **Known Issue:** The IPC command `set_mode:activity` correctly switches hub mode on the Brain side, but the test currently tries to open the Expanded Bezel overlay via UI clicks rather than using `sendCommand('set_mode:activity')` through the WebSocket. The Expanded Bezel does not have a visible "ACT" button — mode switching must happen via IPC.
- [ ] **3-C Tactical Context-Menu Signal** *(blocked: depends on 3-B)*
  - Right-clicks `.activity-item`, clicks `[SIGNAL] Force Kill`, asserts process removal.

---

## Phase 4 — Edge Scenarios & Service Mesh Connect 🟡
*Testing system behaviors strictly contingent on the multi-daemon mesh environment.*

**File:** `tests/e2e/edge_scenarios.spec.ts`

- [x] **4-A Trust Confirmation Blockers** ✅ *(passing)*
  - Sends `sudo su`, the Brain's TrustService classifies it as `PrivilegeEscalation`, and emits `CONFIRMATION_REQUIRED`. The test currently passes because the trust confirmation element is now wired (the Brain sets `pending_confirmation` on the state and the test locator finds it).
- [ ] **4-B Split Detachment & Session Persistence** *(blocked: tos-sessiond not registered)*
  - Creates a split pane (`Ctrl+\`), waits for `tos-sessiond` to flush `_live.tos-session`.
  - **Known Issue:** `tos-sessiond not found in registry` — the session daemon is not yet started by the E2E harness. The split-pane creation via `Ctrl+\` also requires focus not to be on an input element.
- [ ] **4-C Marketplace AI Resolution** *(blocked: tos-heuristicd not registered)*
  - Sends a hallucinated command, expects `.staged-command-chip` from heuristic engine.

---

## Known Blockers & Next Steps

### 1. Onboarding Race Condition (blocks Phase 2 + 3)
The `SKIP TOUR` button becomes visible but is immediately obscured by the `.global-overview` spatial layer transitioning in. Fix options:
- **Option A:** Set `z-index` of `.onboarding-overlay` higher than `.spatial-layer`.
- **Option B:** Use `{ force: true }` on the Playwright click to bypass the interception check.
- **Option C:** Set the onboarding localStorage keys (`tos.onboarding.first_run_complete`, `tos.onboarding.wizard_complete`) to `'true'` via `addInitScript` to skip the overlay entirely.

### 2. ACT Mode Navigation (blocks Phase 3-B, 3-C)
The Expanded Bezel overlay does not contain an "ACT" button. The correct approach is to use `sendCommand('set_mode:activity')` via IPC, which the Brain already handles at `ipc_handler.rs:271`. The test should evaluate this through the WebSocket rather than trying to click a UI element.

### 3. ~~Service Daemon Registration~~ ✅ RESOLVED
The `SessionService` now has a **local-first persistence fallback**. When `tos-sessiond` is not registered, the Brain writes `_live.tos-session` directly to disk via atomic temp-file rename. Configured by `tos.toml`:
```toml
[session]
sessions_dir = ""          # default: ~/.local/share/tos/sessions/
local_persistence = true   # write directly without tos-sessiond
debounce_ms = 2000
```
Config resolution order: `--config <path>` → `$TOS_CONFIG` → `~/.config/tos/tos.toml` → `./tos.toml` → built-in defaults.

---

## Acceptance Criteria Summary

| Phase | Status | Passing | Notes |
|-------|--------|---------|-------|
| 1 — Framework | ✅ Complete | — | Brain orchestration + teardown stable |
| 2 — Hydration | 🟡 Regressed | 0/1 | Onboarding overlay race condition |
| 3 — PTY Matrix | 🟡 Partial | 0/3 | Echo blocked by onboarding; ACT mode needs IPC path |
| 4 — Edge Cases | 🟡 Partial | 1/3 | Trust test passing; session + heuristic blocked |

**Latest run:** 1 passed, 6 failed (2026-03-24)

---

## Running the Suite

```bash
# Always run from the alpha-2 subdirectory:
cd /home/tim/TOS-Desktop-Environment/alpha-2
make test-e2e
```

To run a specific spec only:

```bash
cd alpha-2/svelte_ui
npx playwright test --config playwright.e2e.config.ts tests/e2e/sanity.spec.ts
```

To view a Playwright trace for a failed test:

```bash
npx playwright show-trace test-results/<test-folder>/trace.zip
```
