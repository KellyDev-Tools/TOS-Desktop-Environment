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

---

## Notes

- The production roadmap at `dev_docs/TOS_alpha-2.2_Production-Roadmap.md` shows all Phase 1-4 items marked as complete ([x])
- One test (`zoom_to_jumps_directly`) was fixed in the most recent commit
- Empty `state/mod.rs` was removed from the codebase
- Service orchestration test failure is expected (requires external services running)
