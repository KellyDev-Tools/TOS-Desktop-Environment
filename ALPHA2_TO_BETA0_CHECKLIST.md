# Alpha-2 to Beta-0 Promotion Checklist

Based on the codebase analysis and your Production Roadmap, here's what needs to be done for the beta-0 promotion:

## Current Status

| Component | Status |
|-----------|--------|
| Rust Build | ✅ Passes `cargo check` & `cargo build --release` |
| Rust Tests | ✅ 57/58 pass (1 orchestration test requires external services) |
| Svelte UI | ⚠️ Dependencies not installed |
| Playwright Tests | ⚠️ npm version too old (9.2.0, needs 20+) |

---

## Phase 1: Code Quality & Cleanup (Pre-Beta)

### 1.1 Rust Code Quality
| Task | Priority |
|------|----------|
| Run `cargo fix` on warnings | Medium |
| Update `cargo.lock` with latest patches | High |
| Fix any remaining compiler warnings | Medium |
| Add `#[must_use]` to critical Result-returning functions | Low |
| Consider adding `deny(warnings)` in CI | Medium |

### 1.2 Svelte UI Dependencies
| Task | Priority |
|------|----------|
| Install `node_modules` with Node 20+ | High |
| Run `npm run build` and fix any errors | High |
| Run `npm run check` (Svelte type checking) | High |
| Update `playwright.config.js` if needed | Medium |

### 1.3 Test Coverage
| Task | Priority |
|------|----------|
| Fix `test_service_orchestration_health` (requires services running) | Medium |
| Add integration tests for marketplace flow | Medium |
| Add component tests for Expanded Bezel | Low |
| Add E2E tests for full user journey | Low |

---

## Phase 2: Versioning & Release Prep

### 2.1 Version Bump
| File | Current | Beta-0 Suggestion |
|------|---------|-------------------|
| `Cargo.toml` | `0.1.0` | `0.1.0-beta.0` or `1.0.0-beta.0` |
| `svelte_ui/package.json` | `0.0.1` | `0.1.0-beta.0` |

### 2.2 Documentation
| Task | Priority |
|------|----------|
| Create `CHANGELOG.md` with Alpha-2 features | High |
| Update README with Beta-0 announcement | High |
| Update `dev_docs/` roadmap status | Medium |
| Add "Upgrading from Alpha-2" guide | Medium |
| Verify all `.tos-aibehavior` references replaced with `.tos-skill` in codebase | High |
| Document LSP server requirements per language | Medium |

### 2.3 Asset Management
| Task | Priority |
|------|----------|
| Generate production design tokens | High |
| Optimize and bundle marketplace assets | High |
| Pre-generate session templates | Medium |

---

## Phase 3: Production Readiness

### 3.1 Security
| Task | Priority |
|------|----------|
| Audit `unsafe` code blocks | High |
| Verify manifest signature verification works | High |
| Test Trust Service command blocking | High |
| Review credential handling in AI backends | High |
| Audit skill tool bundle enforcement — verify Brain rejects undeclared tool calls | High |
| Verify AI skill writes route through trust chip system for paths outside sector cwd | High |

### 3.2 Performance
| Task | Priority |
|------|----------|
| Profile and optimize Brain state serialization | Medium |
| Profile Wayland renderer performance | Medium |
| Optimize startup time (Brain init < 2s) | High |
| Add startup metrics logging | Low |

### 3.3 Monitoring
| Task | Priority |
|------|----------|
| Add crash reporting infrastructure | Medium |
| Add startup timing metrics | Medium |
| Add memory usage tracking | Low |
| Add IPC latency thresholds in production | Low |

---

## Phase 4: Native Platform Readiness

### 4.1 Linux (Wayland)
| Task | Priority |
|------|----------|
| Test `LinuxRenderer` with real Wayland compositor | High |
| Verify frame buffer sharing works | High |
| Test mDNS discovery | Medium |
| Verify remote connection flow | Medium |

### 4.2 Documentation for Native Faces
| Task | Priority |
|------|----------|
| Complete Linux Face integration guide | Medium |
| Document OpenXR platform requirements | Low |
| Document Android NDK requirements | Low |

### 4.3 AI Skills System
| Task | Priority |
|------|----------|
| Verify `.tos-skill` module type accepted by Marketplace — `.tos-aibehavior` rejected | High |
| Test Passive Observer correction chips after command failure | High |
| Test Chat Companion full chat flow with streaming responses | High |
| Test Vibe Coder chip sequence end-to-end: intent → steps → diff → apply | High |
| **Gate test:** No skill can auto-submit — all suggestions staged only | Critical |
| Test skill tool bundle enforcement — Brain rejects undeclared tools | High |
| Test skill context minimization — only declared fields received | Medium |
| Test offline AI queue: queue on disconnect, drain on reconnect, 30min expiry | Medium |
| Verify learned patterns stored locally, viewable and clearable in Settings | Medium |

### 4.4 Editor
| Task | Priority |
|------|----------|
| Editor pane renders in split layout alongside terminal pane | High |
| Auto-open on build error: correct file and line highlighted | High |
| Viewer Mode: read-only, no cursor, scrolls to target line | High |
| Editor Mode: keyboard input, syntax highlighting, save works | High |
| Diff Mode: renders side-by-side, Apply commits, reject discards | High |
| Multi-file edit chip sequence: individual Apply/Skip per step | High |
| Pending edit proposal persists to session file and restores | High |
| Session handoff: editor state reconstructs on claiming device | High |
| LSP diagnostics appear as annotation chips (requires LSP binary in PATH) | Medium |
| Mobile: tap line number sends line to AI context | Medium |
| Trust chip fires for writes outside sector cwd | High |
| Editor pane state persists and restores across sessions | High |

---

## Phase 5: Release Artifacts

### 5.1 Build Pipeline
| Task | Priority |
|------|----------|
| Create release build script | High |
| Generate signed release assets | High |
| Create Docker image for Brain daemon | Medium |
| Create installation scripts | Medium |

### 5.2 Packaging
| Task | Priority |
|------|----------|
| Create `.deb` package for Debian/Ubuntu | Medium |
| Create `.rpm` package for Fedora/RHEL | Low |
| Create Homebrew formula | Low |
| Create AUR package | Low |

---

## Recommended Sequence for Beta-0

1. **Week 1**: Code cleanup, dependencies, fix warnings
2. **Week 2**: Version bump, documentation, changelog
3. **Week 3**: Security audit, performance profiling
4. **Week 4**: Native platform testing, release artifacts

---

## Immediate Actions (This Week)

**High Priority:**
1. Install Svelte dependencies and run `npm run build`
2. Generate `CHANGELOG.md` documenting all Alpha-2.2 features
3. Update version numbers to `0.1.0-beta.0`
4. Audit `unsafe` code blocks (sandbox, LinuxRenderer)
5. Test Trust Service command blocking end-to-end
6. Find and replace all `.tos-aibehavior` references with `.tos-skill` in codebase
7. Implement Brain Tool Registry with runtime enforcement of skill tool bundles

---

## Notes

- The production roadmap at `dev_docs/TOS_alpha-2.2_Production-Roadmap.md` shows all Phase 1-4 items marked as complete ([x])
- One test (`zoom_to_jumps_directly`) was fixed in the most recent commit
- Empty `state/mod.rs` was removed from the codebase
- Service orchestration test failure is expected (requires external services running)
