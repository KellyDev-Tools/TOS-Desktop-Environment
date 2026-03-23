# Alpha-2 → Beta-0 Promotion Checklist

---

## Current Status

| Component | Status |
|-----------|--------|
| Rust Build | ✅ Passes `cargo check` & `cargo build --release` |
| Rust Tests | ✅ 57/58 pass (1 orchestration test requires live services) |
| Svelte UI | ⚠️ Dependencies not installed (Node 20+ required) |
| Playwright Tests | ⚠️ npm version too old (9.2.0, needs 20+) |

---

## Phase 1: Code Quality & Standards Compliance

### 1.1 Rust Code Quality

| Task | Priority | Notes |
|------|----------|-------|
| Update `cargo.lock` with latest patches | High | — |
| Run `cargo fix` on all warnings | Medium | Must complete before adding `deny(warnings)` |
| Fix remaining compiler warnings | Medium | — |
| Add `deny(warnings)` to CI pipeline | Medium | Only after all warnings are clear |
| Add `#[must_use]` to critical `Result`-returning functions | Low | — |

### 1.2 Svelte UI Dependencies

| Task | Priority | Notes |
|------|----------|-------|
| Install `node_modules` with Node 20+ | High | — |
| Run `npm run build` and fix any errors | High | Required pipeline gate |
| Run `npm run check` (Svelte type checking) | High | — |
| Update `playwright.config.js` if needed | Medium | — |

### 1.3 Architecture Standards Audit

These are code review gates, not test tasks. Every item is a hard requirement from the development standards.

| Task | Priority | Spec Ref |
|------|----------|----------|
| Audit all input handlers — confirm no direct physical key/mouse bindings exist; all input must translate to a `SemanticEvent` | High | Standards §1.2 |
| Audit all error paths — confirm every error routes through `LogManager` with correct `LogType`; no stray `eprintln!` or `println!` error paths | High | Standards §2.1 |
| Audit for `#[allow(unused_imports)]` — replace with commented-out imports and explanatory note | High | Standards §2.1 |
| Audit spec cross-reference markers — public functions and structs touching specced behaviour must carry `// See §X.Y` comments | Medium | Standards §2.2 |
| Verify no `#[allow(warnings)]` or undocumented `unsafe` blocks remain | High | Standards §2.1 |

### 1.4 TDD Process Gate

| Task | Priority | Notes |
|------|----------|-------|
| Audit Alpha-2 feature code for test-first coverage — any feature code without a corresponding prior failing test must be retroactively covered | Medium | Developer Ref §4 |
| Confirm `test_service_orchestration_health` tier classification — if Tier 2 (integration), it is a Beta-0 blocker requiring 100% pass rate | High | Developer Ref §4.5 |

---

## Phase 2: Versioning & Release Prep

### 2.1 Version Bump

| File | Current | Beta-0 Target |
|------|---------|---------------|
| `Cargo.toml` | `0.1.0` | `0.1.0-beta.0` |
| `svelte_ui/package.json` | `0.0.1` | `0.1.0-beta.0` |

### 2.2 Documentation

| Task | Priority |
|------|----------|
| Create `CHANGELOG.md` documenting all Alpha-2.2 features | High |
| Update README with Beta-0 announcement | High |
| Update `dev_docs/` roadmap status | Medium |
| Add "Upgrading from Alpha-2" guide | Medium |
| Complete Linux Face integration guide | Medium |
| Document OpenXR platform requirements | Low |
| Document Android NDK requirements | Low |
| Verify all `.tos-aibehavior` references replaced with `.tos-skill` in codebase and docs | High |
| Add Editor pane type to Svelte Face hub layout renderer | High |
| Document LSP server requirements per language in Developer Reference | Medium |

### 2.3 Asset Management

| Task | Priority | Notes |
|------|----------|-------|
| Generate production design tokens — central JSON/TOML consumed by both Web CSS and native Vulkan/GLES shaders | High | Architecture §16.2 — both consumers must be validated |
| Optimize and bundle marketplace assets | High | — |
| Pre-generate sector session templates | Medium | — |

---

## Phase 3: Production Readiness

### 3.1 Security

| Task | Priority | Spec Ref |
|------|----------|----------|
| Audit all `unsafe` blocks (sandbox, `LinuxRenderer`) — document justification for each | High | Standards §2.1 |
| Verify manifest Ed25519 signature verification end-to-end | High | Ecosystem §1.0, §2.2 |
| Test Trust Service command blocking (WARN and TRUST paths) | High | Architecture §17.2 |
| Test trust edge cases: implicit bulk detection, per-sector overrides | Medium | Architecture §17.2.2, §17.2.4 |
| Attempt sandbox escape via Standard Tier module | Medium | Architecture §17.3 |
| Review credential handling in all AI backend modules | High | Architecture §17.2 |

### 3.2 Performance

| Task | Priority | Spec Ref |
|------|----------|----------|
| Optimize Brain init to < 2s cold start | High | Features §3.1 — required to hit the 5s user-facing prompt bar |
| Profile and optimize Brain state serialization | Medium | — |
| Profile Wayland renderer frame rate under load (splits, AI streaming) | Medium | Architecture §16 |
| Verify Tactical Alert triggers correctly on sustained FPS drops below target | Low | Architecture §16.4 |
| Add startup timing metrics | Medium | — |

### 3.3 Monitoring

| Task | Priority |
|------|----------|
| Add crash reporting infrastructure (opt-in) | Medium |
| Add memory usage tracking | Low |
| Add IPC latency threshold alerts (target: < 16ms round-trip) | Low |

---

## Phase 4: Native Platform & Feature Validation

### 4.1 Native Face Headless Stubs

These must exist before native platform tests can run in CI.

| Task | Priority | Spec Ref |
|------|----------|----------|
| Implement string-buffer renderer stub for `LinuxRenderer` so visual states and layout can be validated headlessly | High | Developer Ref §4 |
| Implement equivalent stubs for OpenXR and Android faces | Medium | Developer Ref §4 |

### 4.2 Linux (Wayland)

| Task | Priority | Spec Ref |
|------|----------|----------|
| Test `LinuxRenderer` with real Wayland compositor | High | Architecture §15.2 |
| Verify `dmabuf` frame buffer sharing for Level 3 app embedding | High | Architecture §15.2 |
| Test mDNS discovery via Avahi | Medium | Ecosystem §5 |
| Verify remote connection flow end-to-end | Medium | Ecosystem §5 |

### 4.3 Onboarding & First-Run

| Task | Priority | Spec Ref |
|------|----------|----------|
| Verify cinematic intro is skippable at any point and completes within 12s | High | Features §3.3.1 |
| Test guided demo — all steps run inside the live system, no sandbox | High | Features §3.3.2 |
| Confirm ambient hints appear, can be dismissed per-hint or globally, and fade with use | Medium | Features §3.3.3 |
| **Gate test:** Measure time from cold launch to interactive prompt with `wizard_complete = true` — must be ≤ 5 seconds | High | Features §3.1 |

### 4.4 Session Persistence

| Task | Priority | Spec Ref |
|------|----------|----------|
| Test live state auto-save: sectors, terminal histories, AI chat, hub layout, pinned chips | High | Features §2.3 |
| Validate named session save / load / export / import via tile drop and Settings panel | High | Features §2.5 |
| Verify crash recovery: `_live.tos-session.tmp` atomic rename on success; corrupt temp file discarded on next launch | Medium | Features §2.6 |
| Confirm restore is silent — no notification, animation, or prompt on launch | Medium | Features §2.6.2 |

### 4.5 AI Skills System

| Task | Priority | Spec Ref |
|------|----------|----------|
| Passive Observer surfaces correction and explanation chips after command failure | High | Features §4 |
| Chat Companion: AI mode staging, editing, and submission flow works correctly | High | Features §4 |
| **Gate test:** Confirm no AI skill can auto-submit a command — staging only, always editable | Critical | Features §4.12 |
| Test backend switching (e.g. Ollama, OpenAI) and per-sector skill overrides | High | Features §4 |
| Validate context minimization — skill modules only receive fields declared in their manifest | Medium | Features §4.12 |
| Test ghost text and thought bubble display behaviors | Medium | Features §4 |
| Verify AI chat history restores correctly when returning to a sector | High | Features §2.8 |
| Verify Vibe Coder chip sequence proposes steps in order and persists pending steps to session | High | Features §4.8 |
| Test skill tool bundle enforcement — Brain rejects undeclared tool calls at runtime | High | Ecosystem §1.4.3 |
| Verify skill learned patterns are stored locally and visible in Settings → AI → Skills | Medium | Ecosystem §1.4.4 |
| Test offline AI queue: requests queued on disconnect, drain on reconnect, expire after 30 min | Medium | Features §4.9 |
| Confirm `.tos-aibehavior` module type is fully replaced by `.tos-skill` in all code paths | High | Ecosystem §1.4 |

### 4.6 Marketplace Discovery

| Task | Priority | Spec Ref |
|------|----------|----------|
| End-to-end permission review flow: scroll-to-consent gate active before Install button enables | High | Features §5.6.1 |
| Test download progress display, cancellation, and failure recovery | High | Features §5.6.2–5.6.4 |
| Verify signature verification and sideloading with a custom developer public key | High | Ecosystem §1.0 |
| Confirm installed state badge renders correctly in both browse and detail views | Medium | Features §5.8 |
| Verify AI Skills category renders and filters to `.tos-skill` module type | High | Features §5.3.2 |
| Verify Languages category renders and filters to `.tos-language` module type | Medium | Features §5.3.2 |

### 4.7 Editor & AI Edit Flow

| Task | Priority | Spec Ref |
|------|----------|----------|
| Editor pane auto-opens on build error with correct file and line highlighted | High | Features §6.3.2, EDT-01 |
| File preview on path typed in prompt opens in Viewer Mode | Medium | Features §6.3.2, EDT-02 |
| Diff Mode renders correctly for Vibe Coder single-file proposals | High | Features §6.6.2, EDT-03 |
| Multi-file edit chip sequence renders with individual Apply/Skip per step | High | Features §6.6.3, EDT-04 |
| Pending edit proposal persists to session file and reconstructs on session restore | High | Features §2.9, EDT-05 |
| Session handoff token generates, claims, and reconstructs editor state on second device | High | Features §2.10, EDT-05 |
| Editor pane focus toggle `Ctrl+E` works correctly | Medium | Features §6.3.3 |
| Save (`Ctrl+S`) and Save As (`Ctrl+Shift+S`) work correctly | High | Features §6.8 |
| Trust confirmation chip fires for writes outside sector cwd | High | Features §6.8, Architecture §17.2 |
| LSP diagnostics appear as annotation chips in editor margin when LSP server available | Medium | Features §6.9, EDT-08 |
| Mobile: tap line number sends line to AI as context | Medium | Features §6.7, EDT-06 |
| Editor state (file, scroll, pending proposal) persists in session file and restores correctly | High | Features §2.9 |

### 4.7 Split Viewports

| Task | Priority | Spec Ref |
|------|----------|----------|
| Test automatic split orientation based on aspect ratio | High | Architecture §11 |
| Verify `Shift+Ctrl+\` orientation override and minimum pane size blocking | High | Architecture §11 |
| Test Expanded Bezel pane actions: fullscreen, swap, detach to sector, save layout | High | Architecture §11.8 |
| Verify split state persists to session file and restores correctly on relaunch | High | Architecture §11.9 |

### 4.8 Collaboration & Remote Sectors

| Task | Priority | Spec Ref |
|------|----------|----------|
| Test one-time token invite flow — token expires after 30 min inactivity | High | Architecture §13 |
| Verify role promotion (Viewer → Operator) takes effect immediately | High | Architecture §13 |
| Test following mode viewport synchronization | Medium | Architecture §13 |
| Confirm all guest actions are tagged with guest identity in the TOS Log | High | Architecture §13 |
| Test remote sector disconnect handling and 5s auto-close timer | Medium | Architecture §12 |

### 4.9 Deep Inspection & Recovery

| Task | Priority | Spec Ref |
|------|----------|----------|
| Confirm Buffer View is disabled by default and requires explicit privilege elevation | High | Architecture §9.5 |
| Test Tactical Reset (God Mode): prompt locks, Expanded Bezel disables | High | Architecture §9 |
| Verify remote guests cannot initiate or interact with Tactical Reset | High | Architecture §9 |

### 4.10 Multi-Sensory Feedback

| Task | Priority | Spec Ref |
|------|----------|----------|
| Verify earcons fire on mode switches, level zooms, and alert escalations | Medium | Architecture §23 |
| Test haptic patterns on supported hardware | Medium | Architecture §23.4 |
| Confirm alert levels shift audio and visual cues correctly (Green → Yellow → Red) | Medium | Architecture §23 |

### 4.11 Accessibility

| Task | Priority | Spec Ref |
|------|----------|----------|
| Test full keyboard navigation across all UI elements | High | Architecture §24 |
| Verify screen reader announcements via AT-SPI (Linux) / TalkBack (Android) | High | Architecture §24.1 |
| Test high-contrast themes and colourblind filter modes | Medium | Architecture §24.1 |
| Verify dwell-clicking at default 500ms threshold | Medium | Architecture §24.3 |
| Test switch scanning (single and multi-switch) | Low | Architecture §24.3 |

---

## Phase 5: Release Artifacts

### 5.1 Build Pipeline

| Task | Priority | Spec Ref |
|------|----------|----------|
| Create release build script | High | — |
| Generate signed release assets | High | Ecosystem §1.0 |
| Create Docker image for Brain daemon | Medium | — |
| Create installation scripts | Medium | — |

### 5.2 Packaging

| Task | Priority |
|------|----------|
| Create `.deb` package for Debian/Ubuntu | Medium |
| Create `.rpm` package for Fedora/RHEL | Low |
| Create Homebrew formula | Low |
| Create AUR package | Low |

---

## Hard Gates — Beta-0 Cannot Ship Without These

| Gate | Spec Ref |
|------|----------|
| 100% Tier 1 & 2 test pass rate (resolve `test_service_orchestration_health` tier classification first) | Developer Ref §4.5 |
| Cold launch → interactive prompt ≤ 5 seconds | Features §3.1 |
| No AI skill can auto-submit a command — staging only | Features §4.12 |
| All input goes through `SemanticEvent` — zero direct physical bindings | Standards §1.2 |
| All errors routed through `LogManager` with correct `LogType` | Standards §2.1 |
| No undocumented `unsafe` blocks | Standards §2.1 |
| IPC round-trip latency < 16ms in local testing | Developer Ref §4.5 |
| Manifest signature verification passes end-to-end | Ecosystem §1.0 |
| No `.tos-aibehavior` references remain in codebase — all module types use `.tos-skill` | Ecosystem §1.4 |
| Vibe Coder edit proposals never auto-apply — user must tap [Apply] in Diff Mode | Features §6.6.2 |
| Skill tool bundle enforcement verified — undeclared tool calls rejected by Brain at runtime | Ecosystem §1.4.3 |

---

## Notes

- All Phase 1–4 items from the Alpha-2.2 Production Roadmap (`dev_docs/TOS_alpha-2.2_Production-Roadmap.md`) are marked complete `[x]`
- `zoom_to_jumps_directly` test was fixed in the most recent commit
- Empty `state/mod.rs` was removed from the codebase
- Service orchestration test failure is expected when external services are not running — tier classification must be confirmed before treating as non-blocking
