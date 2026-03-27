# Bezel Completion Plan: Tactical Unified Interface

**Goal:** Implement the full *Tactical Bezel* system across the Recursive Zoom Hierarchy as defined in §3 of the Core v1.0 and v1.2 specifications. This plan ensures a high-performance, LCARS-compliant, and accessible interface that guarantees navigational "escape" from any level.

---

## 🏗️ Phase 1: Core Rendering & Visual Foundation (Visual Excellence)
*Focus: Implementing the base SVG/CSS components using the curated LCARS palette (`--lcars-*`).*

| Task ID | Task Title | Description | Acceptance Criteria | Status |
|--------|------------|-------------|----------------------|--------|
| BZ-01 | **LCARS CSS Variable Integration** | Define core bezel themes in `variables.css` including elbow paths and glassmorphism filters. | • `backdrop-filter: blur()` applied to all bezel overlays.<br>• Colors match §3.1 of the AI Development Standards. | 🟢 |
| BZ-02 | **L3: Application Bezel (Collapsed)** | Render the thin, semi-transparent strip for Level 3 (Focus) with App Icon, Title, and Zoom-Out button. (See §3.2.1) | • TDD: Test `render_bezel(HierarchyLevel::Focus, BezelState::Collapsed)`.<br>• Bezel respects position config (Top/Left/Right). | 🟢 |
| BZ-03 | **L2: Command Hub Bezel (Output Mode)** | Implement the bezel for Level 2 including the "Output Mode Toggle" (Perspective vs. Rectangular) and Left Region Toggle. (See §2.1) | • Mode toggle button updates `Viewport::output_mode` state.<br>• Transitions are animated (200ms). | 🟢 |
| BZ-04 | **L1: Global Overview Bezel** | Implement Level 1 bezel with Settings Gear, Add Sector (+), and Collaboration avatars. (See §1.2) | • Gear icon opens Settings modal.<br>• Collaboration indicators show mock avatars for now. | 🟢 |

## 🕹️ Phase 2: Interaction Model & Recursive Navigation
*Focus: Mapping physical inputs to `SemanticEvent` and handling transitions.*

| Task ID | Task Title | Description | Acceptance Criteria | Status |
|--------|------------|-------------|----------------------|--------|
| BZ-05 | **Semantic Event Pipeline: Bezel-Born** | Wire bezel UI controls to `SemanticEvent::ToggleBezel`, `ZoomOut`, and `SplitView`. | • TDD: Integration test verifying "Hierarchy Round-Trip" on `ZoomOut`.<br>• Zero use of unwrap() in event handlers. | 🟢 |
| BZ-06 | **"Bezel-Born" Autocomplete Drawer** | Implement the downward-unfurling autocomplete overlay attached to the L2 top bezel. (See §2.5) | • Drawer appears when typing in Unified Prompt.<br>• Max height restricted to 3/4 of viewport. | 🟢 |
| BZ-07 | **Kinetic Expand/Collapse Handle** | Add logic for the down-chevron (Expand Handle) with dragging support and `Ctrl+Space` shortcut. | • Smooth 60fps animation during expansion.<br>• Uses `recursive-zoom` keyframes where appropriate. | 🟢 |

## 🛡️ Phase 3: Tactile Confirmation & Security (Tactical Reset)
*Focus: Infrastructure for dangerous command approval via the bezel.*

| Task ID | Task Title | Description | Acceptance Criteria | Status |
|--------|------------|-------------|----------------------|--------|
| BZ-08 | **Tactile Confirmation Slider** | Render the security slider within the expanded bezel for commands flagged by `SecurityManager`. (See §14) | • Requires full traversal to approve.<br>• Color shifts from `--lcars-orange` to `--lcars-red` on danger. | 🟢 |
| BZ-09 | **Security Integration (Blocking)** | Intercept dangerous commands in the `ShellAPI` and trigger the Bezel Confirmation UI. | • TDD: Test that `rm -rf /` is blocked until slider is 100%.<br>• Logged via `LogManager` as `LogType::Security`. | 🟡 |
| BZ-10 | **Chorded Key Confirmation** | Accessibility fallback for tactile slider using three-key chords (`Ctrl+Enter+Space`). | • Successfully triggers `SecurityManager::approve_command`.<br>• Announced via `AccessibilityManager`. | 🔴 |

## 📊 Phase 4: Priority HUD & Indicators
*Focus: Real-time visual feedback based on weighted priority scores (§5).*

| Task ID | Task Title | Description | Acceptance Criteria | Status |
|--------|------------|-------------|----------------------|--------|
| BZ-11 | **Border Chips & Chevrons** | Render pill-shaped accents and chevrons on bezel edges based on `PriorityScore`. (See §5.1) | • 1 chip = low, 4 chips = critical.<br>• Pulsing animation for scores > 80. | 🟢 |
| BZ-12 | **Glow & Luminance Effects** | Apply inner/outer glow to the bezel when priority thresholds are met. | • Intensity scales with focus recency/frequency.<br>• Configurable per-sector colors. | 🟢 |

## 🧪 Phase 5: Validation, i18n & Performance
*Focus: Ensuring quality and accessibility.*

| Task ID | Task Title | Description | Acceptance Criteria | Status |
|--------|------------|-------------|----------------------|--------|
| BZ-13 | **Hierarchy Round-Trip Verification** | Comprehensive integration tests for all bezel levels. | • `cargo test` passes for the new bezel-specific tests. | 🟢 |
| BZ-14 | **Globalization (i18n) Audit** | Move all bezel labels/tooltips to `resources/i18n/`. | • Supports English and mock translation labels. | 🔴 |
| BZ-15 | **Accessibility (AT-SPI / Switch)** | Bind bezel controls to the `AccessibilityManager` for screen reader support. | • All buttons have `aria-label` equivalent logic.<br>• Switch scanning works on bezel items. | 🟡 |

---

### 🚀 Execution Strategy
1. **TDD First**: Every task begins with a failing test case in `alpha-1/src/ui/render/bezel_test.rs`.
2. **Modular Rendering**: Use `src/ui/render/bezel.rs` for shared logic across all hierarchy levels.
3. **Compiler Protections**: Maintain zero warnings and zero `unwrap()` calls.
4. **Visual Polish**: Review every UI change against the LCARS modernized aesthetic (§3.1).

---
**Status Indicators:**
- 🔴 Not Started
- 🟡 In Progress
- 🟢 Complete
- 🛡️ Security Review Required

**Update Log (2026-02-23):**
- Centralized all bezel rendering logic into `src/ui/render/bezel.rs`.
- Implemented L1, L2, and L3 specific bezel views with hierarchy-aware labels.
- Added tactile confirmation slider for security-flagged commands.
- Added priority border chips and glassmorphism styling.
- Verified hierarchy round-trip and component rendering with updated test suite.

