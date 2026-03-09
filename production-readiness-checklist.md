# Production Readiness Checklist

## Current Build Status

| Component | Status |
|-----------|--------|
| Rust Build | ✅ Passes `cargo check` & `cargo build --release` |
| Rust Tests | ⚠️ 57/58 pass (1 orchestration test requires live services — tier classification needed) |
| Svelte UI | ⚠️ Dependencies not installed (Node 20+ required) |
| Playwright Tests | ⚠️ npm version too old (9.2.0, needs 20+) |

---

## Hard Gates — Cannot Ship Without These

| Gate | Reference |
|------|-----------|
| 100% Tier 1 & 2 test pass rate (resolve `test_service_orchestration_health` tier first) | Developer Ref §4.5 |
| Cold launch → interactive prompt ≤ 5 seconds | Features §3.1 |
| No AI module can auto-submit a command — staging only, always editable | Features §4 |
| All input goes through `SemanticEvent` — zero direct physical bindings | Standards §1.2 |
| All errors routed through `LogManager` with correct `LogType` | Standards §2.1 |
| No undocumented `unsafe` blocks | Standards §2.1 |
| IPC round-trip latency < 16ms in local testing | Developer Ref §4.5 |
| Manifest signature verification passes end-to-end | Ecosystem §1.0 |

---

## Phase 1: Code Quality & Standards Compliance

### 1.1 Rust Code Quality

- [ ] Update `cargo.lock` with latest patches
- [ ] Run `cargo fix` on all warnings
- [ ] Fix remaining compiler warnings
- [ ] Add `deny(warnings)` to CI pipeline (only after all warnings are clear)
- [ ] Add `#[must_use]` to critical `Result`-returning functions
- [ ] Run `cargo audit` against all Rust dependencies for known CVEs

### 1.2 Svelte UI Dependencies

- [ ] Install `node_modules` with Node 20+
- [ ] Run `npm run build` and fix any errors (required pipeline gate)
- [ ] Run `npm run check` (Svelte type checking)
- [ ] Update `playwright.config.js` if needed

### 1.3 Architecture Standards Audit

- [ ] Audit all input handlers — confirm no direct physical key/mouse bindings; all input must translate to a `SemanticEvent`
- [ ] Audit all error paths — confirm every error routes through `LogManager` with correct `LogType`; no stray `eprintln!` or `println!` paths
- [ ] Audit for `#[allow(unused_imports)]` — replace with commented-out imports and explanatory notes
- [ ] Audit spec cross-reference markers — public functions and structs touching specced behaviour must carry `// See §X.Y` comments
- [ ] Verify no `#[allow(warnings)]` or undocumented `unsafe` blocks remain

### 1.4 TDD Process Gate

- [ ] Audit feature code for test-first coverage — any code without a corresponding prior failing test must be retroactively covered
- [ ] Confirm `test_service_orchestration_health` tier classification — if Tier 2 (integration), it is a blocker requiring 100% pass rate

---

## Phase 2: CI/CD Pipeline

- [ ] Set up pipeline (GitHub Actions / GitLab CI)
- [ ] Wire all test targets: `make test-core`, `test-shell`, `test-sec`, `test-brain`, `test-ui-component`, `test-health`
- [ ] Gate PR merges on 100% test pass rate
- [ ] Implement build artifact signing for module distribution
- [ ] Add `criterion` benchmarks for IPC round-trip latency (target: <16ms)
- [ ] Confirm cold-start ≤5s test is wired and blocking CI in `tests/headless_brain.rs`
- [ ] Automate 60 FPS (desktop) and 90 FPS (VR) regression gates

---

## Phase 3: Versioning & Release Prep

### 3.1 Version Bump

| File | Current | Target |
|------|---------|--------|
| `Cargo.toml` | `0.1.0` | `0.1.0-beta.0` |
| `svelte_ui/package.json` | `0.0.1` | `0.1.0-beta.0` |

### 3.2 IPC Schema Versioning

- [ ] Version the `tos-protocol` IPC schema
- [ ] Implement graceful incompatibility detection for mismatched Face/Brain versions
- [ ] Document migration path for breaking IPC schema changes
- [ ] Define Brain and daemon update strategy without losing active sector state

### 3.3 Asset Management

- [ ] Generate production design tokens — central JSON/TOML consumed by both Web CSS and native Vulkan/GLES shaders (both consumers must be validated)
- [ ] Optimize and bundle marketplace assets
- [ ] Pre-generate sector session templates

### 3.4 Documentation

- [ ] Create `CHANGELOG.md` documenting all Alpha-2.2 features
- [ ] Update README with Beta-0 announcement
- [ ] Update `dev_docs/` roadmap status
- [ ] Add "Upgrading from Alpha-2" guide
- [ ] Complete Linux Face integration guide
- [ ] Document OpenXR platform requirements
- [ ] Document Android NDK requirements

---

## Phase 4: Security Hardening

- [ ] Audit all `unsafe` blocks (sandbox, `LinuxRenderer`) — document justification for each
- [ ] Verify manifest Ed25519 signature verification end-to-end
- [ ] Test Trust Service WARN and TRUST command paths
- [ ] Test trust edge cases: implicit bulk detection, per-sector overrides
- [ ] Attempt sandbox escape via Standard Tier module (adversarial test)
- [ ] Review credential handling in all AI backend modules
- [ ] Integration-test that the audit log (`/var/log/tos/audit.log`) is non-disableable
- [ ] Load/concurrency test collaboration token expiry (target: 30 min inactivity)

---

## Phase 5: Performance & Monitoring

### 5.1 Performance

- [ ] Optimize Brain init to < 2s cold start (required to hit the 5s user-facing prompt target)
- [ ] Profile and optimize Brain state serialization
- [ ] Profile Wayland renderer frame rate under load (splits, AI streaming)
- [ ] Verify Tactical Alert triggers correctly on sustained FPS drops below target
- [ ] Add startup timing metrics

### 5.2 Monitoring & Graceful Degradation

- [ ] Add crash reporting infrastructure (opt-in)
- [ ] Add memory usage tracking
- [ ] Add IPC latency threshold alerts (target: < 16ms round-trip)
- [ ] Define and implement UX for daemon registration failure at startup
- [ ] Test degraded-state behaviour when `tos-settingsd` or `tos-loggerd` fail to connect to the Brain
- [ ] Test and document all auto-Tactical Reset trigger conditions (>500ms latency, Brain deadlock)

---

## Phase 6: Native Platform & Feature Validation

### 6.1 Native Face Headless Stubs

- [ ] Implement string-buffer renderer stub for `LinuxRenderer` so visual states and layout can be validated headlessly
- [ ] Implement equivalent stubs for OpenXR and Android faces

### 6.2 Linux (Wayland)

- [ ] Test `LinuxRenderer` with real Wayland compositor
- [ ] Verify `dmabuf` frame buffer sharing for Level 3 app embedding
- [ ] Test mDNS discovery via Avahi
- [ ] Verify remote connection flow end-to-end

### 6.3 Onboarding & First-Run

- [ ] Verify cinematic intro is skippable at any point and completes within 12s
- [ ] Test guided demo — all steps run inside the live system, no sandbox
- [ ] Confirm ambient hints appear, can be dismissed per-hint or globally, and fade with use
- [ ] **Gate test:** Measure time from cold launch to interactive prompt with `wizard_complete = true` — must be ≤ 5 seconds
- [ ] Trust configuration wizard complete with no pre-selected defaults (ONB-05)

### 6.4 Session Persistence

- [ ] Test live state auto-save: sectors, terminal histories, AI chat, hub layout, pinned chips
- [ ] Validate named session save / load / export / import via tile drop and Settings panel
- [ ] Verify crash recovery: `_live.tos-session.tmp` atomic rename on success; corrupt temp file discarded on next launch
- [ ] Confirm restore is silent — no notification, animation, or prompt on launch

### 6.5 AI Co-Pilot System

- [ ] Passive Observer surfaces correction and explanation chips after command failure
- [ ] Chat Companion: AI mode staging, editing, and submission flow works correctly
- [ ] **Critical gate:** Confirm no AI module can auto-submit a command — staging only, always editable
- [ ] Test backend switching (e.g. Ollama, OpenAI) and per-sector behavior module overrides
- [ ] Validate context minimization — behavior modules only receive fields declared in their manifest
- [ ] Test ghost text and thought bubble display behaviors
- [ ] Verify AI chat history restores correctly when returning to a sector

### 6.6 Marketplace

- [ ] End-to-end permission review flow: scroll-to-consent gate active before Install button enables
- [ ] Test download progress display, cancellation, and failure recovery
- [ ] Verify signature verification and sideloading with a custom developer public key
- [ ] Confirm installed state badge renders correctly in both browse and detail views

### 6.7 Split Viewports

- [ ] Test automatic split orientation based on aspect ratio
- [ ] Verify `Shift+Ctrl+\` orientation override and minimum pane size blocking
- [ ] Test Expanded Bezel pane actions: fullscreen, swap, detach to sector, save layout
- [ ] Verify split state persists to session file and restores correctly on relaunch

### 6.8 Collaboration & Remote Sectors

- [ ] Test one-time token invite flow — token expires after 30 min inactivity
- [ ] Verify role promotion (Viewer → Operator) takes effect immediately
- [ ] Test following mode viewport synchronization
- [ ] Confirm all guest actions are tagged with guest identity in the TOS Log
- [ ] Test remote sector disconnect handling and 5s auto-close timer
- [ ] Load/concurrency test token expiry for race conditions

### 6.9 Deep Inspection & Recovery

- [ ] Confirm Buffer View is disabled by default and requires explicit privilege elevation
- [ ] Test Tactical Reset (God Mode): prompt locks, Expanded Bezel disables
- [ ] Verify remote guests cannot initiate or interact with Tactical Reset

### 6.10 Multi-Sensory Feedback

- [ ] Verify earcons fire on mode switches, level zooms, and alert escalations
- [ ] Test haptic patterns on supported hardware
- [ ] Confirm alert levels shift audio and visual cues correctly (Green → Yellow → Red)

### 6.11 Accessibility

- [ ] Test full keyboard navigation across all UI elements
- [ ] Verify screen reader announcements via AT-SPI (Linux) / TalkBack (Android)
- [ ] Test high-contrast themes and colourblind filter modes
- [ ] Verify dwell-clicking at default 500ms threshold
- [ ] Test switch scanning (single and multi-switch)
- [ ] Run automated axe-core accessibility audit via Playwright
- [ ] Ensure all accessibility user stories have passing Playwright assertions before launch

---

## Phase 7: Release Artifacts

### 7.1 Build Pipeline

- [ ] Create release build script
- [ ] Generate signed release assets
- [ ] Create Docker image for Brain daemon
- [ ] Create installation scripts

### 7.2 Packaging

- [ ] Create `.deb` package for Debian/Ubuntu
- [ ] Create `.rpm` package for Fedora/RHEL
- [ ] Create Homebrew formula
- [ ] Create AUR package

---

## Phase 8: End-User Documentation

- [ ] Write trust system explainer — rationale for no defaults, how WARN chips work
- [ ] Document the onboarding flow end-to-end
- [ ] Write Marketplace usage guide (install, permissions review, sideloading)
- [ ] Write FAQ covering surprising UX decisions (trust, WARN chips, no pre-selections)
- [ ] Review and expand User Manual for Beta-0 scope
