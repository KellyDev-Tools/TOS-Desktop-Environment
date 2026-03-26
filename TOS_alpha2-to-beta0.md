# TOS Alpha-2 → Beta-0 Promotion

**Single authoritative reference for all gates, tasks, and validation required before Beta-0 ships.**

> **Living Document** — This file is the source of truth for the Alpha-2 → Beta-0 transition.
> It lives at the repository root (`/8TB/tos/TOS_alpha2-to-beta0.md`) and must be kept
> current as work progresses.
>
> ### Maintenance Rules
>
> 1. **Update on every change.** When you complete a task, fix a bug, or make any structural
>    change to the `alpha-2/` or `beta-0/` trees, update the relevant section of this
>    document in the same commit.
> 2. **Use status markers.** Prefix task rows with `✅` (done), `🔧` (in progress),
>    or `❌` (blocked). Leave unmarked rows as not yet started.
> 3. **Timestamp significant updates.** Add a dated entry to the _Audit Trail_ at the bottom
>    whenever the Build Status table, Hard Gates, or folder-migration readiness changes.
> 4. **Never delete history.** When a claim is corrected, strike-through the old value and
>    add the correction inline so reviewers can see what changed.
> 5. **Beta-0 is a "Pull" Destination.** We are not moving the `alpha-2/` folder wholesale.
>    Instead, `beta-0/` starts with the consolidated spec docs. Functionality is
>    systematically "pulled" from `alpha-2/` into `beta-0/` only after it has been
>    refactored to meet the Beta-0 specifications.

---

## Build Status

| Component | Status |
|---|---|
| Rust Build | ❌ `cargo check` **fails** — `src/bin/settingsd.rs` has 2 errors (`E0624` private method `load_local`, `E0282` type inference). 3 warnings (`handle_ai_submit` unused, `ShellApi` fields unused, `MockContent` never constructed). |
| Rust Tests | ⚠️ 16/~105 pass — only lib unit tests compile. Integration tests blocked by `settingsd.rs` build failure. `tos-protocol` tests also fail (`E0063` missing fields on `CommandHub`). `test_service_orchestration_health` tier classification still pending. |
| Svelte UI | ⚠️ Node v20.20.1 available — `node_modules` not installed, build not verified |
| Playwright Tests | ⚠️ npm 10.8.2 available — `node_modules` not installed, tests not run |

---

## Hard Gates — Beta-0 Cannot Ship Without These

| Gate | Spec Ref |
|---|---|
| 100% Tier 1 & 2 test pass rate — resolve `test_service_orchestration_health` tier classification first | Developer Ref §4.5 |
| Cold launch → interactive prompt ≤ 5 seconds with `wizard_complete = true` | Features §3.1 |
| No AI skill can auto-submit a command — staging only, always editable | Features §4.12 |
| All input routed through `SemanticEvent` — zero direct physical key/mouse bindings | Standards §1.2 |
| All errors routed through `LogManager` with correct `LogType` — no stray `eprintln!`/`println!` | Standards §2.1 |
| No undocumented `unsafe` blocks in codebase | Standards §2.1 |
| IPC round-trip latency < 16ms in local testing | Developer Ref §4.5 |
| Manifest Ed25519 signature verification passes end-to-end | Ecosystem §1.0 |
| No `.tos-aibehavior` references remain — all module types use `.tos-skill` | Ecosystem §1.4 |
| Vibe Coder proposals never auto-apply — user must tap [Apply] in Diff Mode | Features §6.6.2 |
| Skill tool bundle enforcement verified — undeclared tool calls rejected by Brain at runtime | Ecosystem §1.4.3 |

---

## Phase 0 — Selective Pull & Reconstruction

The Alpha-2 tree has structural and architectural debt. Rather than a bulk move, Beta-0 is being reconstructed in the `beta-0/` directory. Verified components are "pulled" from `alpha-2/`, refactored for the new spec, and staged in the target tree.

### 0.1 Problems in Alpha-2

| Problem | Detail |
|---|---|
| `dev_docs/` overcrowded | 27 files across four naming schemes — several superseded by canonical Beta-0 spec files |
| `src/platform/electron/` dead branch | Electron was explored in Alpha-2.2.1 and not chosen — ~25 files across `src/`, `tests/`, `resources/` + 5 config files of dead code |
| `src/brain/state/` empty directory | `state/mod.rs` was removed but the directory was not cleaned up |
| Root-level clutter | `demo.log`, `print_ws.js`, `meta.json`, root `package.json`, root `package-lock.json`, root `playwright.config.js` are artifacts |
| `scripts/demo_context_export.py` | Prototype tool misplaced in shell integration scripts directory |
| `src/common/mod.rs` | As `tos-protocol/` matures, this may be redundant — migration candidate |
| `modules/` at root | Only two stub `module.toml` files — development fixtures, not production modules |
| `src/bin/settingsd.rs` broken | Calls private method `load_local` on `SettingsService` — blocks all bin/integration test compilation |
| `tos-protocol` test stale | `protocol_tests.rs` missing required `is_running` and `last_exit_status` fields on `CommandHub` |

### 0.2 Target Beta-0 Tree

Changes from Alpha-2 are annotated inline.

```
.
├── Cargo.lock
├── Cargo.toml                          # version → 0.1.0-beta.0
├── Makefile
├── tos.toml
├── CHANGELOG.md                        # NEW — Alpha-2 → Beta-0 change log
├── README.md                           # UPDATE — Beta-0 announcement
│
├── assets/
│   └── design_tokens.json              # unchanged
│
├── dev/                                # RENAMED from modules/ — dev fixtures only
│   ├── fixtures/
│   │   ├── tos-ai-standard/
│   │   │   └── module.toml
│   │   └── tos-shell-fish/
│   │       └── module.toml
│   └── README.md                       # NEW — explains these are test fixtures
│
├── docs/                               # RENAMED from dev_docs/ — cleaner name
│   │
│   ├── spec/                           # NEW — canonical Beta-0 specs (from beta-0/dev_docs/)
│   │   ├── TOS_beta-0_Architecture.md
│   │   ├── TOS_beta-0_Developer.md
│   │   ├── TOS_beta-0_Ecosystem.md
│   │   ├── TOS_beta-0_Features.md
│   │   └── TOS_beta-0_User-Manual.md
│   │                                   # NOTE: TOS_User_Stories.md stays at repo root — version-agnostic
│   │
│   ├── guides/                         # NEW — operational guides
│   │   ├── Linux-Face-Integration.md
│   │   ├── OpenXR-Platform.md
│   │   ├── Android-NDK.md
│   │   └── Upgrading-from-Alpha-2.md
│   │
│   └── archive/                        # MOVED — all Alpha-2 dev_docs preserved
│       ├── TOS_alpha-2.0_Roadmap.md
│       ├── TOS_alpha-2.1_Brain-Roadmap.md
│       ├── TOS_alpha-2.1_Dependencies-Tree.md
│       ├── TOS_alpha-2.1_Ecosystem-Roadmap.md
│       ├── TOS_alpha-2.1_Face-Roadmap.md
│       ├── TOS_Alpha-2.1_User-Manual.md
│       ├── TOS_alpha-2.2.1_E2E-Testing-Roadmap.md
│       ├── TOS_alpha-2.2.1_Electron-Platform-Guide.md
│       ├── TOS_alpha-2.2.1_Platform-Options-Implementation-Plan.md
│       ├── TOS_alpha-2.2.1_Platform-Options.md
│       ├── TOS_alpha-2.2_AI-Copilot-Specification.md
│       ├── TOS_alpha-2.2_Expanded-Bezel-Specification.md
│       ├── TOS_alpha-2.2_Implementation-Plan.md
│       ├── TOS_alpha-2.2_Marketplace-Discovery-Specification.md
│       ├── TOS_alpha-2.2_Onboarding-Specification.md
│       ├── TOS_alpha-2.2_Production-Roadmap.md
│       ├── TOS_alpha-2.2_Session-Persistence-Specification.md
│       ├── TOS_alpha-2.2_Split-Viewport-Specification.md
│       ├── TOS_alpha-2.2_Trust-Confirmation-Specification.md
│       ├── TOS_alpha-2_Architecture-Specification.md
│       ├── TOS_alpha-2_Developer-SDK.md
│       ├── TOS_alpha-2_Development-Guide.md
│       ├── TOS_alpha-2_Display-Face-Specification.md
│       ├── TOS_alpha-2_Ecosystem-Orchestration.md
│       ├── TOS_alpha-2_Ecosystem-Specification.md
│       ├── TOS_alpha-2_Project-Structure.md
│       └── TOS_alpha-2_TDD-Plan.md
│
├── scripts/                            # unchanged — shell integration scripts only
│   ├── tos.bash
│   ├── tos.fish
│   └── tos.zsh
│
├── tools/                              # NEW — internal dev/prototype tooling
│   └── demo_context_export.py          # MOVED from scripts/
│
├── src/
│   ├── bin/                            # unchanged
│   │   ├── brain_node.rs
│   │   ├── heuristicd.rs
│   │   ├── loggerd.rs
│   │   ├── marketplaced.rs
│   │   ├── priorityd.rs
│   │   ├── searchd.rs
│   │   ├── sessiond.rs
│   │   ├── settingsd.rs
│   │   ├── system_test.rs
│   │   └── tos-pkg.rs
│   │
│   ├── brain/                          # unchanged except state/ cleanup
│   │   ├── hierarchy/
│   │   │   └── mod.rs
│   │   ├── ipc_handler.rs
│   │   ├── mod.rs
│   │   ├── module_manager.rs
│   │   ├── sector/
│   │   │   ├── mod.rs
│   │   │   └── tdp.rs
│   │   └── shell/
│   │       └── mod.rs
│   │                                   # REMOVED: brain/state/ (directory is empty)
│   │
│   ├── common/                         # unchanged — review for migration to tos-protocol
│   │   └── mod.rs
│   │
│   ├── config.rs
│   ├── face/
│   │   └── mod.rs
│   ├── lib.rs
│   ├── main.rs
│   │
│   ├── modules/
│   │   ├── mod.rs
│   │   └── sandbox/
│   │       └── mod.rs
│   │
│   ├── platform/
│   │   ├── linux/
│   │   │   ├── mod.rs
│   │   │   └── wayland.rs
│   │   ├── mock.rs
│   │   ├── mod.rs
│   │   ├── quest.rs
│   │   ├── remote.rs
│   │   ├── remote_server.rs
│   │   ├── remote_session.rs
│   │   └── ssh_fallback.rs
│   │                                   # REMOVED: platform/electron/ (entire subtree)
│   │
│   └── services/
│       ├── ai/
│       │   └── mod.rs
│       ├── audio.rs
│       ├── capture.rs
│       ├── haptic.rs
│       ├── heuristic.rs
│       ├── logger.rs
│       ├── marketplace.rs
│       ├── mod.rs
│       ├── portal.rs
│       ├── priority.rs
│       ├── registry.rs
│       ├── search.rs
│       ├── session.rs
│       ├── settings.rs
│       └── trust.rs
│
├── svelte_ui/                          # unchanged structure
│   ├── package.json                    # version → 0.1.0-beta.0
│   ├── package-lock.json
│   ├── playwright.config.ts
│   ├── playwright.e2e.config.ts
│   ├── README.md
│   ├── src/
│   │   ├── app.css
│   │   ├── app.d.ts
│   │   ├── app.html
│   │   ├── lib/
│   │   │   ├── actions/
│   │   │   │   └── longpress.ts
│   │   │   ├── assets/
│   │   │   │   └── favicon.svg
│   │   │   ├── components/
│   │   │   │   ├── DisconnectOverlay.svelte
│   │   │   │   ├── ExpandedBezel.svelte
│   │   │   │   ├── modules/
│   │   │   │   │   ├── BrainStatus.svelte
│   │   │   │   │   ├── MiniLog.svelte
│   │   │   │   │   ├── Minimap.svelte
│   │   │   │   │   ├── PriorityStack.svelte
│   │   │   │   │   └── Telemetry.svelte
│   │   │   │   ├── OnboardingOverlay.svelte
│   │   │   │   ├── PortalModal.svelte
│   │   │   │   ├── SectorContextMenu.svelte
│   │   │   │   ├── SettingsModal.svelte
│   │   │   │   ├── SystemOutput.svelte
│   │   │   │   ├── TacticalContextMenu.svelte
│   │   │   │   └── views/
│   │   │   │       ├── AiChat.svelte
│   │   │   │       ├── ApplicationFocus.svelte
│   │   │   │       ├── CommandHub.svelte
│   │   │   │       ├── DetailInspector.svelte
│   │   │   │       ├── GlobalOverview.svelte
│   │   │   │       ├── Marketplace.svelte
│   │   │   │       ├── SplitLayout.svelte
│   │   │   │       └── SplitPaneView.svelte
│   │   │   ├── index.ts
│   │   │   └── stores/
│   │   │       ├── ipc.svelte.ts
│   │   │       ├── tos-state.svelte.ts
│   │   │       └── ui.svelte.ts
│   │   └── routes/
│   │       ├── +layout.svelte
│   │       ├── +layout.ts
│   │       └── +page.svelte
│   ├── static/
│   │   ├── favicon.png
│   │   └── robots.txt
│   ├── svelte.config.js
│   ├── tests/
│   │   ├── e2e/
│   │   │   ├── edge_scenarios.spec.ts
│   │   │   ├── globalSetup.ts
│   │   │   ├── globalTeardown.ts
│   │   │   ├── index.spec.ts
│   │   │   ├── README.md
│   │   │   ├── roadmap.spec.ts
│   │   │   ├── sanity.spec.ts
│   │   │   └── terminal.spec.ts
│   │   ├── marketplace.spec.ts
│   │   ├── secondary_select.spec.ts
│   │   ├── ui_component.spec.ts
│   │   └── user_stories.spec.ts
│   ├── tsconfig.json
│   └── vite.config.ts
│
├── tests/                              # unchanged
│   ├── ai_integration.rs
│   ├── application_integration.rs
│   ├── brain_core/
│   │   └── main.rs
│   ├── face_visual_states.rs
│   ├── headless_brain.rs
│   ├── sandbox.rs
│   ├── security_manifest.rs
│   ├── service_extraction.rs
│   ├── service_orchestration.rs
│   ├── settings_schema.rs
│   ├── shell_integration/
│   │   └── main.rs
│   ├── stimulator_brain_node.rs
│   ├── stimulator.rs
│   └── ui_component.spec.js
│
├── tos-protocol/                       # unchanged
│   ├── Cargo.toml
│   ├── src/
│   │   ├── collaboration.rs
│   │   ├── ipc.rs
│   │   ├── lib.rs
│   │   ├── marketplace.rs
│   │   ├── modules.rs
│   │   └── state.rs
│   └── tests/
│       └── protocol_tests.rs
│
├── tos-android/                        # WORKSPACE CRATE — standalone Android Face
│   ├── Cargo.toml
│   └── src/
│       ├── face.rs
│       ├── input.rs
│       ├── lib.rs
│       ├── ndk_stubs.rs
│       └── services.rs
│
└── .gitignore
```

### 0.3 Change Inventory

**Removals**

| Path | Reason |
|---|---|
| `src/platform/electron/` (entire subtree) | Platform not chosen — ~25 files, dead code |
| `src/brain/state/` (empty directory) | `state/mod.rs` already removed; directory orphaned |
| `demo.log` | Build artifact (empty file), not source |
| `print_ws.js` | Prototype debug script; does not belong at root |
| `meta.json` | Unclear provenance; empty file, likely Electron-era artifact |
| `package.json` (root) | Electron-era root package; canonical JS lives in `svelte_ui/` |
| `package-lock.json` (root) | Same as above |
| `playwright.config.js` (root) | Electron-era Playwright config; Svelte UI has its own `playwright.config.ts` |
| `install_deps.sh` | Superseded by `Makefile` targets — confirm before removal (see §0.5) |

**Renames / Moves**

| From | To | Reason |
|---|---|---|
| `dev_docs/` | `docs/archive/` | Alpha-2 docs preserved but clearly archived |
| `modules/` | `dev/fixtures/` | Disambiguates dev fixtures from real module installs |
| `scripts/demo_context_export.py` | `tools/demo_context_export.py` | Not a shell integration script |

**New Directories**

| Path | Contents |
|---|---|
| `docs/spec/` | Canonical Beta-0 spec files (moved from `beta-0/dev_docs/`) |
| `docs/guides/` | Operational guides (Linux Face, OpenXR, Android, Upgrade) |
| `docs/archive/` | All Alpha-2 `dev_docs/` files |
| `dev/fixtures/` | Module stubs for development testing |
| `tools/` | Internal prototype and debug utilities |

**New Files**

| Path | Notes |
|---|---|
| `CHANGELOG.md` | Required by Phase 2 |
| `docs/archive/README.md` | One-line note: these docs are superseded by `docs/spec/` |
| `dev/fixtures/README.md` | Explains these are test fixtures, not production modules |
| `docs/guides/Linux-Face-Integration.md` | Required by Phase 2 |
| `docs/guides/Upgrading-from-Alpha-2.md` | Required by Phase 2 |
| `docs/guides/OpenXR-Platform.md` | Required by Phase 2 |
| `docs/guides/Android-NDK.md` | Required by Phase 2 |

**Version Bumps**

| File | Field | From | To |
|---|---|---|---|
| `Cargo.toml` | `version` | `0.1.0` | `0.1.0-beta.0` |
| `svelte_ui/package.json` | `version` | `0.0.1` | `0.1.0-beta.0` |

### 0.4 Execution Order

Beta-0 is built by pulling and refactoring functional blocks from Alpha-2.

1. **Initialize `beta-0/` project** — Copy `Cargo.toml`, `Makefile`, and `tos.toml` from `alpha-2/` into `beta-0/` and apply Beta-0 version bumps immediately.
2. **Setup `beta-0/` Docs** — Move the consolidated specs from `beta-0/dev_docs/` into their final `beta-0/docs/spec/` locations as defined in §0.2.
3. **Refactor & Pull `tos-protocol`** — Pull `tos-protocol/` into `beta-0/`. Fix `CommandHub` missing fields during the pull.
4. **Refactor & Pull Core Services** — Pull `src/services/` one by one. Fix `settingsd.rs` visibility issues during the pull into `beta-0/`.
5. **Reconstruct Brain** — Pull `src/brain/` and refactor to match the new `SemanticEvent` and `LogManager` standards (§1.3).
6. **Migrate Svelte Face** — Pull `svelte_ui/` and perform a clean `npm install` and build.
7. **Clean up Clutter** — Ensure no dead `electron/` code or root-level artifacts (`demo.log`, etc.) are pulled into the new tree.
8. **Verify §0.6 Pull Readiness Gate** for each module as it is landed in `beta-0/`.

### 0.5 Open Decisions

These are not blockers but need a call before execution.

**`install_deps.sh`** — Is this still the intended dependency install path, or has the `Makefile` fully replaced it? If `Makefile`, remove it. If still needed for bootstrap (before `make` is available), keep it and add a note to the README.

**`src/common/mod.rs`** — As `tos-protocol/` matures as the authoritative IPC schema crate, `src/common/` may become redundant. Worth reviewing whether its contents should migrate into `tos-protocol/src/` before Beta-0 or be explicitly left as a separate internal-only module.

**`tos-android/` crate** — This is now a real workspace member (listed in `Cargo.toml` members), not an empty placeholder. It contains `face.rs`, `input.rs`, `lib.rs`, `ndk_stubs.rs`, and `services.rs`. Decide whether it should remain a workspace member in Beta-0 or be published as a separate crate.

### 0.6 Pull Readiness Gate

**No functionality is considered "landed" in `beta-0/` until it satisfies these criteria.**

| # | Prerequisite | Status | Notes |
|---|---|---|---|
| 1 | `cargo check` passes in `beta-0/` | ❌ | Not started |
| 2 | Component lib unit tests pass | 🔧 | 16/16 verified in Alpha-2 |
| 3 | `tos-protocol` tests pass in `beta-0/` | ❌ | Blocks IPC verification |
| 4 | Root-level artifacts (§0.3) excluded from `beta-0/` | ❌ | |
| 5 | Code meets Standards §2.1 (no stray `println!`) | ❌ | |
| 6 | `unsafe` blocks carry justification comments | ❌ | |
| 7 | Version 0.1.0-beta.0 applied | ❌ | |
| 8 | `svelte_ui/` build is clean in `beta-0/` | ❌ | |

**Pull Procedure:** We are currently at Step 1 of §0.4. Once core files are staged in `beta-0/`, we verify the readiness of each subsystem against this gate.

---

## Phase 1 — Code Quality & Standards Compliance

### 1.1 Rust Code Quality

| Task | Priority | Notes |
|---|---|---|
| ❌ Fix `src/bin/settingsd.rs` build errors — make `load_local` public or refactor call site | **Critical** | Blocks all bin and integration test compilation |
| ❌ Fix `tos-protocol/tests/protocol_tests.rs` — add missing `is_running`, `last_exit_status` fields to `CommandHub` initializers | **Critical** | Blocks `tos-protocol` test suite |
| Update `cargo.lock` with latest patches | High | — |
| Run `cargo fix` on all warnings (3 current: `handle_ai_submit`, `ShellApi` fields, `MockContent`) | Medium | Must complete before adding `deny(warnings)` |
| Fix remaining compiler warnings | Medium | — |
| Add `deny(warnings)` to CI pipeline | Medium | Only after all warnings cleared |
| Add `#[must_use]` to critical `Result`-returning functions | Low | — |

### 1.2 Svelte UI Dependencies

| Task | Priority | Notes |
|---|---|---|
| Install `node_modules` with Node 20+ | High | Required pipeline gate |
| Run `npm run build` and fix any errors | High | Required pipeline gate |
| Run `npm run check` (Svelte type checking) | High | — |
| Update `playwright.config.js` if needed | Medium | — |

### 1.3 Architecture Standards Audit

Code review gates — every item is a hard requirement from the development standards.

| Task | Priority | Spec Ref |
|---|---|---|
| Audit all input handlers — confirm no direct physical key/mouse bindings; all input must flow through `SemanticEvent` | High | Standards §1.2 |
| Audit all error paths — confirm every error routes through `LogManager` with correct `LogType`; no stray `eprintln!` or `println!` | High | Standards §2.1 |
| Audit for `#[allow(unused_imports)]` — replace with commented-out imports and explanatory note | High | Standards §2.1 |
| Audit spec cross-reference markers — public functions and structs touching specced behaviour must carry `// See §X.Y` comments | Medium | Standards §2.2 |
| Verify no `#[allow(warnings)]` or undocumented `unsafe` blocks remain | High | Standards §2.1 |
| Confirm all `.tos-aibehavior` references replaced with `.tos-skill` in all code paths | High | Ecosystem §1.4 |

### 1.4 TDD Process Gate

| Task | Priority | Notes |
|---|---|---|
| Audit Alpha-2 feature code for test-first coverage — retroactively cover any feature without a prior failing test | Medium | Developer Ref §4 |
| Confirm `test_service_orchestration_health` tier classification — if Tier 2 (integration), it is a Beta-0 blocker | High | Developer Ref §4.5 |
| Add integration tests for marketplace install flow | Medium | — |
| Add component tests for Expanded Bezel | Low | — |

---

## Phase 2 — Versioning & Release Prep

### 2.1 Version Bump

| File | Current | Beta-0 Target |
|---|---|---|
| `Cargo.toml` | `0.1.0` | `0.1.0-beta.0` |
| `svelte_ui/package.json` | `0.0.1` | `0.1.0-beta.0` |

### 2.2 Documentation

| Task | Priority | Notes |
|---|---|---|
| Create `CHANGELOG.md` documenting all Alpha-2.2 features | High | — |
| Update README with Beta-0 announcement | High | — |
| Add "Upgrading from Alpha-2" guide | Medium | — |
| Complete Linux Face integration guide | Medium | — |
| Document LSP server requirements per language in Developer Reference | Medium | — |
| Document OpenXR platform requirements | Low | — |
| Document Android NDK requirements | Low | — |
| Add Editor pane type to Svelte Face hub layout renderer documentation | High | — |

### 2.3 Asset Management

| Task | Priority | Spec Ref |
|---|---|---|
| Generate production design tokens — central JSON/TOML consumed by both Web CSS and native Vulkan/GLES shaders | High | Architecture §16.2 |
| Optimize and bundle marketplace assets | High | — |
| Pre-generate sector session templates | Medium | — |

---

## Phase 3 — Production Readiness

### 3.1 Security

| Task | Priority | Spec Ref |
|---|---|---|
| Audit all `unsafe` blocks (sandbox, `LinuxRenderer`) — document justification for each | High | Standards §2.1 |
| Verify manifest Ed25519 signature verification end-to-end | High | Ecosystem §1.0, §2.2 |
| Test Trust Service command blocking — WARN and TRUST paths | High | Architecture §17.2 |
| Test trust edge cases: implicit bulk detection, per-sector overrides | Medium | Architecture §17.2.2, §17.2.4 |
| Verify skill tool bundle enforcement — Brain rejects undeclared tool calls at runtime | High | Ecosystem §1.4.3 |
| Verify AI skill file writes route through trust chip system for paths outside sector cwd | High | Architecture §17.2 |
| Attempt sandbox escape via Standard Tier module | Medium | Architecture §17.3 |
| Review credential handling in all AI backend modules | High | Architecture §17.2 |

### 3.2 Performance

| Task | Priority | Spec Ref |
|---|---|---|
| Optimize Brain init to < 2s cold start | High | Features §3.1 — required to hit the 5s user-facing prompt gate |
| Profile and optimize Brain state serialization | Medium | — |
| Profile Wayland renderer frame rate under load (splits, AI streaming) | Medium | Architecture §16 |
| Verify Tactical Alert triggers correctly on sustained FPS drops below target | Low | Architecture §16.4 |
| Add startup timing metrics | Medium | — |

### 3.3 Monitoring

| Task | Priority | Notes |
|---|---|---|
| Add crash reporting infrastructure (opt-in) | Medium | — |
| Add memory usage tracking | Low | — |
| Add IPC latency threshold alerts (target: < 16ms round-trip) | Low | — |

---

## Phase 4 — Native Platform & Feature Validation

### 4.1 Native Face Headless Stubs

Must exist before native platform tests can run in CI.

| Task | Priority | Spec Ref |
|---|---|---|
| Implement string-buffer renderer stub for `LinuxRenderer` — visual states and layout validated headlessly | High | Developer Ref §4 |
| Implement equivalent stubs for OpenXR and Android faces | Medium | Developer Ref §4 |

### 4.2 Linux (Wayland)

| Task | Priority | Spec Ref |
|---|---|---|
| Test `LinuxRenderer` with real Wayland compositor | High | Architecture §15.2 |
| Verify `dmabuf` frame buffer sharing for Level 3 app embedding | High | Architecture §15.2 |
| Test mDNS discovery via Avahi | Medium | Ecosystem §5 |
| Verify remote connection flow end-to-end | Medium | Ecosystem §5 |

### 4.3 Onboarding & First-Run

| Task | Priority | Spec Ref |
|---|---|---|
| Verify cinematic intro is skippable at any point and completes within 12s | High | Features §3.3.1 |
| Test guided demo — all steps run inside the live system, not a sandbox | High | Features §3.3.2 |
| Confirm ambient hints appear, can be dismissed per-hint or globally, and fade with use | Medium | Features §3.3.3 |
| **Gate test:** Measure cold launch → interactive prompt with `wizard_complete = true` — must be ≤ 5 seconds | High | Features §3.1 |

### 4.4 Session Persistence

| Task | Priority | Spec Ref |
|---|---|---|
| Test live state auto-save: sectors, terminal histories, AI chat, hub layout, pinned chips | High | Features §2.3 |
| Validate named session save / load / export / import via tile drop and Settings panel | High | Features §2.5 |
| Verify crash recovery: `_live.tos-session.tmp` atomic rename on success; corrupt temp file discarded on next launch | Medium | Features §2.6 |
| Confirm restore is silent — no notification, animation, or prompt on launch | Medium | Features §2.6.2 |

### 4.5 AI Skills System

| Task | Priority | Spec Ref |
|---|---|---|
| Passive Observer surfaces correction and explanation chips after command failure | High | Features §4 |
| Chat Companion: AI mode staging, editing, and submission flow works correctly | High | Features §4 |
| **Gate test:** Confirm no AI skill can auto-submit a command — staging only, always editable | Critical | Features §4.12 |
| Test backend switching (Ollama, OpenAI) and per-sector skill overrides | High | Features §4 |
| Validate context minimization — skill modules only receive fields declared in their manifest | Medium | Features §4.12 |
| Test ghost text and thought bubble display behaviors | Medium | Features §4 |
| Verify AI chat history restores correctly when returning to a sector | High | Features §2.8 |
| Verify Vibe Coder chip sequence proposes steps in order and persists pending steps to session | High | Features §4.8 |
| Test skill tool bundle enforcement — Brain rejects undeclared tool calls at runtime | High | Ecosystem §1.4.3 |
| Verify skill learned patterns are stored locally and visible in Settings → AI → Skills | Medium | Ecosystem §1.4.4 |
| Test offline AI queue: queued on disconnect, drain on reconnect, expire after 30 min | Medium | Features §4.9 |
| Verify `.tos-skill` module type accepted by Marketplace — `.tos-aibehavior` type rejected | High | Ecosystem §1.4 |

### 4.6 Marketplace

| Task | Priority | Spec Ref |
|---|---|---|
| End-to-end permission review flow: scroll-to-consent gate active before Install button enables | High | Features §5.6.1 |
| Test download progress display, cancellation, and failure recovery | High | Features §5.6.2–5.6.4 |
| Verify signature verification and sideloading with a custom developer public key | High | Ecosystem §1.0 |
| Confirm installed state badge renders correctly in both browse and detail views | Medium | Features §5.8 |
| Verify AI Skills category renders and filters to `.tos-skill` module type | High | Features §5.3.2 |
| Verify Languages category renders and filters to `.tos-language` module type | Medium | Features §5.3.2 |

### 4.7 Editor & AI Edit Flow

| Task | Priority | Spec Ref |
|---|---|---|
| Editor pane renders in split layout alongside terminal pane | High | Features §6, Architecture §11.2 |
| Auto-open on build error: correct file and line highlighted in amber | High | Features §6.3.2, EDT-01 |
| Viewer Mode: read-only, no cursor, scrolls to target line | High | Features §6.2 |
| Editor Mode: keyboard input, syntax highlighting, save works | High | Features §6.2 |
| Diff Mode: side-by-side renders correctly — Apply commits, reject discards | High | Features §6.6.2, EDT-03 |
| Multi-file edit chip sequence renders with individual Apply/Skip per step | High | Features §6.6.3, EDT-04 |
| Pending edit proposal persists to session file and reconstructs on restore | High | Features §2.9, EDT-05 |
| Session handoff token generates, claims, and reconstructs editor state on second device | High | Features §2.10, EDT-05 |
| Editor pane focus toggle `Ctrl+E` works correctly | Medium | Features §6.3.3 |
| Save (`Ctrl+S`) and Save As (`Ctrl+Shift+S`) work correctly | High | Features §6.8 |
| Trust confirmation chip fires for writes outside sector cwd | High | Features §6.8, Architecture §17.2 |
| File preview on path typed in prompt opens in Viewer Mode | Medium | Features §6.3.2, EDT-02 |
| LSP diagnostics appear as annotation chips in editor margin when LSP server is in PATH | Medium | Features §6.9, EDT-08 |
| Mobile: tap line number sends line to AI as context | Medium | Features §6.7, EDT-06 |
| Editor pane state (file, scroll, cursor, pending proposal) persists and restores correctly | High | Features §2.9 |

### 4.8 Split Viewports

| Task | Priority | Spec Ref |
|---|---|---|
| Test automatic split orientation based on display aspect ratio | High | Architecture §11.3 |
| Verify `Shift+Ctrl+\` orientation override | High | Architecture §11.3.3 |
| Verify minimum pane size blocking with amber flash and earcon | High | Architecture §11.5 |
| Test Expanded Bezel pane actions: fullscreen, swap, detach to sector, save layout | High | Architecture §11.8 |
| Verify split state persists to session file and restores correctly on relaunch | High | Architecture §11.9 |

### 4.9 Collaboration & Remote Sectors

| Task | Priority | Spec Ref |
|---|---|---|
| Test one-time token invite flow — token expires after 30 min inactivity | High | Architecture §13 |
| Verify role promotion (Viewer → Operator) takes effect immediately | High | Architecture §13 |
| Test following mode viewport synchronization | Medium | Architecture §13 |
| Confirm all guest actions are tagged with guest identity in TOS Log | High | Architecture §13 |
| Test remote sector disconnect handling and 5s auto-close timer | Medium | Architecture §12 |

### 4.10 Deep Inspection & Recovery

| Task | Priority | Spec Ref |
|---|---|---|
| Confirm Buffer View is disabled by default and requires explicit privilege elevation | High | Architecture §9.5 |
| Test Tactical Reset (God Mode): prompt locks, Expanded Bezel disables | High | Architecture §9 |
| Verify remote guests cannot initiate or interact with Tactical Reset | High | Architecture §9.3.3 |

### 4.11 Multi-Sensory Feedback

| Task | Priority | Spec Ref |
|---|---|---|
| Verify earcons fire on mode switches, level zooms, and alert escalations | Medium | Architecture §23 |
| Test haptic patterns on supported hardware | Medium | Architecture §23.4 |
| Confirm alert levels shift audio and visual cues correctly (Green → Yellow → Red) | Medium | Architecture §23 |

### 4.12 Accessibility

| Task | Priority | Spec Ref |
|---|---|---|
| Test full keyboard navigation across all UI elements | High | Architecture §24 |
| Verify screen reader announcements via AT-SPI (Linux) / TalkBack (Android) | High | Architecture §24.1 |
| Test high-contrast themes and colourblind filter modes | Medium | Architecture §24.1 |
| Verify dwell-clicking at default 500ms threshold | Medium | Architecture §24.3 |
| Test switch scanning (single and multi-switch) | Low | Architecture §24.3 |

---

## Phase 5 — Release Artifacts

### 5.1 Build Pipeline

| Task | Priority | Notes |
|---|---|---|
| Create release build script | High | — |
| Generate signed release assets | High | Ecosystem §1.0 |
| Create Docker image for Brain daemon | Medium | — |
| Create installation scripts | Medium | — |

### 5.2 Packaging

| Task | Priority | Notes |
|---|---|---|
| Create `.deb` package for Debian/Ubuntu | Medium | — |
| Create `.rpm` package for Fedora/RHEL | Low | — |
| Create Homebrew formula | Low | — |
| Create AUR package | Low | — |

---

## Notes

- All Phase 1–4 items from `dev_docs/TOS_alpha-2.2_Production-Roadmap.md` are marked complete
- `zoom_to_jumps_directly` test was fixed in the most recent Alpha-2 commit
- Empty `state/mod.rs` was removed from the codebase
- Service orchestration test failure is expected when external services are not running — tier classification must be confirmed before treating as non-blocking
- `tos-android/` is now a real workspace crate (not the `android/` placeholder directory previously described) with 5 source files and its own `Cargo.toml`
- No root `README.md` exists in `alpha-2/` — one must be created before migration
- Beta-0 spec files live in `/8TB/tos/beta-0/dev_docs/`, not at the `alpha-2/` project root

---

## Audit Trail

Dated log of significant validation events and status changes.

| Date | Event |
|---|---|
| 2026-03-26 | **Initial validation audit.** Build status corrected from ✅ to ❌ (`settingsd.rs` errors). Test count corrected from 57/58 to 16/~105. npm version corrected (10.8.2, not 9.2.0). `android/` placeholder replaced with `tos-android/` workspace crate. `platform/android.rs` removed from target tree (does not exist). Electron file count corrected to ~25. Root `playwright.config.js` added to removal list. Spec file source corrected to `beta-0/dev_docs/`. Folder Migration Gate (§0.6) added. Living document protocol added. |
| 2026-03-26 | **Strategy Pivot.** Migration model changed from " wholesale copy" to "Selective Pull." Beta-0 is now the primary integration target. All execution steps and gates updated to reflect refactoring and pulling code from Alpha-2 into the new Beta-0 structure based on consolidated specs. |
