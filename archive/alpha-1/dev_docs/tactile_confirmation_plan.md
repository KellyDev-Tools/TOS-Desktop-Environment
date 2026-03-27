# Tactile Confirmation Plan: Security Modal Implementation

**Goal:** Implement the full *Tactile Confirmation System* for dangerous commands across the TOS interface as defined in §14 of the Core v1.0 and v1.2 specifications. This plan ensures secure command approval via physical/tactile interaction, with accessible fallbacks.

---

## 🏗️ Phase 1: Core UI Components & Visual Foundation
*Focus: Building reusable tactile confirmation components using LCARS aesthetic.*

| Task ID | Task Title | Description | Acceptance Criteria |
|--------|------------|-------------|----------------------|
| TC-01 | **LCARS Modal Styling** | Define modal container, overlay, and backdrop in `variables.css` with glassmorphism effect. | • `backdrop-filter: blur(10px)` with semi-transparent dark overlay.<br>• Modal positioned center or attached to bezel.<br>• Uses `--lcars-orange`, `--lcars-red` color progression. |
| TC-02 | **Tactile Slider Component** | Render horizontal slider track with draggable thumb for approval. (See §14.1) | • TDD: Test `render_slider(percentage: f32)`.<br>• Thumb tracks from 0% to 100%.<br>• Smooth CSS transitions on drag. |
| TC-03 | **Color Progression System** | Implement dynamic color shifts based on danger level and slider state. | • Low-risk: `--lcars-orange`.<br>• Medium-risk: `--lcars-gold`.<br>• High-risk: `--lcars-red` at 100%.<br>• Smooth gradient transitions. |
| TC-04 | **Multi-Button Confirmation (Accessibility)** | Render 3-button grid (Cancel, Approve, Dangerous) for non-slider interaction. (See §14.2) | • Button labels clearly indicate action.<br>• Requires **2 of 3** buttons for approval.<br>• Red glow on Dangerous button. |
| TC-05 | **Modal Title & Risk Description** | Display command summary and danger level with icon hints. | • Shows actual command being approved.<br>• Lists affected resources (e.g., `rm -rf /home/user/docs`).<br>• Risk level badge: Low/Medium/High/Critical. |
| TC-06 | **Cancel & Timeout Handles** | Render cancel button and countdown timer (30s default). | • Dismisses modal and cancels pending command.<br>• Timer visually decrements on modal.<br>• Auto-cancel on timeout. |

## 🕹️ Phase 2: Integration with SecurityManager
*Focus: Wiring tactile confirmation to the security policy system.*

| Task ID | Task Title | Description | Acceptance Criteria |
|--------|------------|-------------|----------------------|
| TC-07 | **SecurityManager State Machine** | Extend `SecurityManager` to track confirmation UI state. | • New states: `AwaitingConfirmation`, `Confirmed`, `Denied`, `TimedOut`.<br>• Stores pending command and risk metadata. |
| TC-08 | **Risk Classification Logic** | Implement `classify_command_risk()` to assign Low/Medium/High/Critical. | • TDD: Test risk classification for destructive patterns (`rm -rf`, `dd`, `>` overwrite).<br>• Returns `RiskLevel` enum and description. |
| TC-09 | **Dangerous Command Interception** | Hook `ShellAPI` to trigger tactile confirmation for flagged commands. | • TDD: Test that `rm -rf /` triggers modal.<br>• Command execution halted until approval.<br>• Logged via `LogManager::log(LogType::Security)`. |
| TC-10 | **Command Approval & Execution** | Execute pending command after confirmation threshold met. | • Slider at 100% **or** 2-of-3 buttons pressed.<br>• Log approval with timestamp and user context.<br>• Return success/failure to shell. |

## 🎯 Phase 3: Input Handling & Interaction Model
*Focus: Mapping diverse inputs to confirmation actions.*

| Task ID | Task Title | Description | Acceptance Criteria |
|--------|------------|-------------|----------------------|
| TC-11 | **Mouse/Trackpad Slider Drag** | Handle mouse dragging of slider thumb to 100%. | • Smooth tracking at 60fps.<br>• Momentum scrolling optional (e.g., flick to approve).<br>• TDD: Simulate drag events to 100%. |
| TC-12 | **Keyboard Chord Confirmation** | Implement three-key sequence (`Ctrl+Enter+Space`) for slider fallback. | • TDD: Test chord entry order validation.<br>• Visual feedback on each key press.<br>• Timer does not reset during chord retry. |
| TC-13 | **Gamepad Analog Stick** | Support analog stick (if gamepad feature enabled) for slider control. | • LY analog axis maps to slider position.<br>• Button A confirms at threshold.<br>• Button B cancels. |
| TC-14 | **Voice Command Fallback** | Allow "approve" voice command to confirm (if voice-system feature). | • Requires explicit utterance (e.g., "I approve this action").<br>• Logged with confidence score.<br>• Fallback to slider if confidence < 90%. |
| TC-15 | **XR Hand Gesture** | Support pinch or grab gesture for slider on XR platforms. | • Hand position maps to slider 0-100%.<br>• Haptic feedback on confirmation.<br>• Fallback to voice if tracking lost. |

## 🛡️ Phase 4: Accessibility & Inclusive Design
*Focus: Ensuring all users can approve commands safely.*

| Task ID | Task Title | Description | Acceptance Criteria |
|--------|------------|-------------|----------------------|
| TC-16 | **Screen Reader Announcements** | Integrate with `AccessibilityManager` for modal narration. | • Modal title, risk level, command, and timer announced.<br>• Button labels clear and distinct.<br>• State changes (e.g., "50% approved") announced. |
| TC-17 | **Switch Access Scanning** | Support switch scanning (single-key/dwell) on modal buttons. | • Auto-highlight each button in sequence.<br>• Dwell time configurable (1-5s).<br>• Cancel always accessible as first scan target. |
| TC-18 | **High-Contrast Mode** | Ensure modal is readable in high-contrast themes. | • Uses `--high-contrast-*` CSS variables.<br>• Slider thumb has clear border.<br>• Text has sufficient color contrast (WCAG AA). |
| TC-19 | **Motor Control Options** | Provide togglable slider stiffness and button hold-time requirements. | • Sticky slider mode (click-to-move).<br>• Hold duration before button registers (500ms-2s).<br>• Double-tap to reset slider. |
| TC-20 | **Cognitive Load Reduction** | Simplify modal UI for users with cognitive disabilities. | • Optional "Expert Mode" toggle in settings.<br>• Simple mode: Just Cancel/Approve buttons.<br>• Removes timer initially; offer extension. |

## 📊 Phase 5: Edge Cases, State Persistence & Recovery
*Focus: Robustness and correctness.*

| Task ID | Task Title | Description | Acceptance Criteria |
|--------|------------|-------------|----------------------|
| TC-21 | **Concurrent Command Blocking** | Prevent multiple dangerous commands queuing. | • Only one confirmation modal at a time.<br>• Subsequent commands queued with timeout.<br>• User notified via earcon + toast. |
| TC-22 | **Session Activity Timeout** | Invalidate confirmation if user inactive for 5 minutes. | • Clears pending command and modal.<br>• Logs timeout event with security context.<br>• Suggests re-entry of command. |
| TC-23 | **Modal Dismissal Edge Cases** | Handle window loss-of-focus, alt-tab, fullscreen exit. | • Modal remains visible if in focus mode.<br>• Pauses timer if window loses focus.<br>• Resumes on refocus (no timeout reset). |
| TC-24 | **Error Recovery** | Handle approval failure (e.g., permission denied at execution). | • Modal closes; error message shown in status bar.<br>• Offer retry or dismiss.<br>• Log failure reason via `LogManager`. |
| TC-25 | **Audit Trail & Compliance** | Comprehensive logging for security reviews. | • Stores: command, risk level, approval method, timestamp, user context.<br>• Log file in `self.log_directory/security/`.<br>• Encrypted/restricted read permissions. |

## 🧪 Phase 6: Testing, Validation & Performance
*Focus: Quality assurance and polish.*

| Task ID | Task Title | Description | Acceptance Criteria |
|--------|------------|-------------|----------------------|
| TC-26 | **Unit Tests: Risk Classification** | Test `classify_command_risk()` against pattern library. | • `cargo test --lib security::risk_classification` passes.<br>• Coverage: 100% of risk patterns.<br>• Zero panics on malformed input. |
| TC-27 | **Integration Tests: Modal Flow** | Test end-to-end approval workflows. | • TDD: Integration test in `tests/tactical_confirmation.rs`.<br>• Test slider to 100%, button chords, voice approval.<br>• Verify command executes post-confirmation. |
| TC-28 | **Performance Audit** | Ensure modal rendering does not cause jank. | • Modal appears within 100ms of trigger.<br>• Slider drag at 60fps (no dropped frames).<br>• Memory: < 5MB for modal state. |
| TC-29 | **Accessibility Compliance** | Verify WCAG 2.1 AA compliance. | • All interactive elements keyboard-accessible.<br>• Color contrast ratios meet AA standard.<br>• Screen reader test with NVDA/JAWS. |
| TC-30 | **Visual Polish & Animation** | Refine appearance and micro-interactions using LCARS keyframes. | • Slider thumb uses `ease-out` momentum on release (1.2s deceleration).<br>• Buttons pulse (`--lcars-pulse` keyframe) on hover for sighted users (400ms).<br>• Modal entrance: `recursive-zoom` scale from 0.8 to 1.0 (300ms).<br>• Modal exit: `recursive-zoom` scale from 1.0 to 0.8 (200ms).<br>• Slider track glow intensifies with danger level (smooth 100ms transitions). |

---

### 🚀 Execution Strategy
1. **TDD First**: Begin each phase with failing tests in `alpha-1/tests/tactical_confirmation.rs`.
2. **Modular Implementation**: 
   - UI rendering: `src/ui/render/confirmation.rs`
   - State logic: `src/system/security/confirmation.rs`
   - Integration: `src/cli.rs` (command interception)
3. **Compiler Protections**: Zero `unwrap()`, zero warnings. Use `Result` and `Option` throughout.
4. **Feature-Gated**: Confirmation UI behind `#[cfg(feature = "gui")]` if headless-passthrough needed.
5. **Iterative Validation**: After each phase, run full test suite and manual smoke tests.

---

**Dependency Chain:**
- TC-01 through TC-06: Parallel (no dependencies)
- TC-07 through TC-10: Require TC-01-06 complete
- TC-11 through TC-15: Require TC-01-06 complete; TC-11 highest priority
- TC-16 through TC-20: Require TC-01-06 complete
- TC-21 through TC-25: Require TC-07-10 complete
- TC-26 through TC-30: Final validation; can start after TC-10

**Status Indicators:**
- 🔴 Not Started
- 🟡 In Progress
- 🟢 Complete
- 🛡️ Security Review Required
