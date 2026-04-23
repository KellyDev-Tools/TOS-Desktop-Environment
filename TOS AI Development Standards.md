# TOS AI Development Standards & Tactical Guidelines

This document dictates the standards, coding philosophy, and aesthetic requirements for AI-assisted development of the Tactical Operating System (TOS). Any AI agent working on this codebase must adhere to these principles to maintain architectural integrity and visual excellence.

---

## 1. Architectural Philosophy

### 1.1 The Recursive Hierarchy
Code must respect the vertical depth model. Navigation is never lateral; it is always **Zoom In** or **Zoom Out**.
- **Level 1 (Global)**: Sector Management & Configuration.
- **Level 2 (Hub)**: Tactical Command & Unified Prompt.
- **Level 3 (Focus)**: Active Application Surface.
- **Level 4 (Detail)**: Metadata & Inspection.
- **Level 5 (Buffer)**: Raw memory/privileged access.

### 1.2 Input Abstraction
**Never** bind logic directly to physical input (keyboard keys, mouse buttons).
- All physical input must translate to a `SemanticEvent`.
- Logic must respond to `SemanticEvent` (e.g., `AiSubmit`, `ZoomIn`, `StopOperation`).

### 1.3 Platform Abstraction
Maintain the "Unified Vision" by using traits defined in `src/platform/mod.rs`.
- Do not use platform-specific types (like `std::os::unix::...`) in core logic. Use `Renderer`, `InputSource`, and `SystemServices` abstractions.

---

## 2. Coding Standards (Rust)

### 2.1 Type Safety & Correctness
- Use custom enums for state management instead of strings or booleans (e.g., `CommandHubMode`).
- Favor `Result` and `Option` over `unwrap()`. Errors in tactical systems must be handled gracefully.
- **Never** disable compiler protections (e.g., do not use `#[allow(warnings)]` or `unsafe` blocks unless absolutely necessary, documented and explicitly approved).
- **Never** use `#[allow(unused_imports)]`. Instead, comment out the imports and document why they are unused or explain why they are needed.
- **Error Reporting**: All errors must be reported via the centralized log system (`src/system/log.rs`). Use `LogManager` to record errors with the appropriate log type (i.e. `LogType::System` or `LogType::Security`) events with appropriate region context.

### 2.2 Modularity
- Systems (AI, Search, Reset) belong in `src/system/`.
- UI rendering logic belongs in `src/ui/render/`.
- Features described in the Specification must be cross-referenced using section markers (e.g., `// See §3.4`).

### 2.3 Documentation
- Every public function and struct must have a doc comment explaining its role in the tactical environment.
- Use the `§` symbol when referencing sections from the Architectural Specification.

---

## 3. Visual Excellence & UI Standards

### 3.1 The LCARS Aesthetic
TOS is inspired by 24th-century tactical displays but modernized for current hardware.
- **Color Palette**: Use deep blacks, muted purples (#443366), tactical ambers (#FF9900), and LCARS blues (#6699FF).
- **Geometry**: Use large, bold corner radii (20px+) for major containers, contrasting with sharp 2px borders for internal widgets.
- **Typography**: Strictly use "Outfit" or "Inter" for UI labels, and "JetBrains Mono" or similar for tactical data.

### 3.2 Premium Materiality
- **Glassmorphism**: Use `backdrop-filter: blur(12px)` and semi-transparent backgrounds (`rgba(0,0,0,0.4)`) to create depth.
- **Tactical Micro-interactions**: Every click must feel substantial. Use `0.1s ease-out` transitions for hover states and `0.05s` for active states.
- **Layered Depth**: Use subtle box-shadows and inner glows to separate hierarchy levels.

### 3.3 The "Tactical Chip" System
- **Status Indicators**: Chips (like trust warnings or AI thoughts) must use high-contrast text on solid backgrounds.
- **Color Coding**: 
  - `var(--color-primary)` (Blue): Information / Hub Mode.
  - `var(--color-success)` (Green): Active Operation / Success.
  - `var(--color-warning)` (Amber): Tactical Alert / Trust Warning.
  - `var(--color-danger)` (Red): System Failure / Red Alert.

### 3.4 Animation Philosophy
- **Entrance**: Use `scaleIn` or `slideFade` for new panes.
- **Feedback**: Use subtle pulses (e.g., "amberPulse") for active AI reasoning or pending confirmations.
- **Efficiency**: Animations must never delay user input. They are visual feedback, not blocking transitions.

---

## 4. Testing & Validation

### 4.1 Test Taxonomy & Definitions
To maintain velocity and reliability, testing in TOS is strictly categorized:

1. **Unit Tests:** 
   - *Definition:* Validates a single, isolated function, struct, or pure logic sequence. No side effects, no file-system access, no network, no global state.
   - *Location:* Inline alongside the code within `#[cfg(test)]` modules (Rust) or adjacent `.spec.ts` files (Svelte logic).
   - *Execution:* Must execute in microseconds.

2. **Integration Tests:** 
   - *Definition:* Validates the interaction between multiple subsystems natively. Uses realistic but headless state (e.g., verifying an IPC string mutates the Brain and generates the correct JSON delta).
   - *Location:* The `tests/` directory at the workspace root (e.g., `tests/headless_brain.rs`).
   - *Execution:* Spins up local memory structures, completely bypassing UI/renderers.

3. **Component Tests:** 
   - *Definition:* Validates an individual sub-system, daemon, or UI module completely in isolation, independent of the rest of the system. Given input/state X, it must produce output/state Y. Essential for debugging the distributed TOS architecture.
   - *Location:* `svelte_ui/tests/` (for UI components), `tests/` (for isolated Brain modules like the `TrustService`), or standalone service tests (e.g., verifying `tos-marketplaced` without a Brain connection).
   - *Execution:* Fast execution using mocks for any external dependency (e.g., mocking the `brain.sock` or `SystemServices` trait).

### 4.2 Test-Driven Development (TDD) Protocols
Test-Driven Development is strictly required. No feature code should be written without a failing test being written and executed first.

**For Brain/Core (Rust):**
1. **Write the Test:** Add a test case to `tests/` (e.g., `tests/headless_brain.rs` or `tests/settings_schema.rs`). 
2. **Verify Failure:** Run the test using `cargo test --test <name>` to prove it fails exactly as expected.
3. **Implement Subsystem:** Write the minimal Rust code required to pass the test.
4. **Verify Success:** Run the test again to prove it passes.
5. *Rule:* Never use the Face (UI) to manually verify Brain state. Always use Headless testing bypassing the IPC socket or explicitly checking state JSON.

**For Web Face/Frontend (Svelte):**
1. **Write the Test:** Add a Playwright component/E2E test in `svelte_ui/tests/`.
2. **Verify Failure:** Run the test using `cd svelte_ui && npx playwright test` to observe failure.
3. **Implement UI:** Build the Svelte component or logic.
4. **Verify Success:** Run the Playwright test again.
5. *Rule:* Playwright tests must assert visual state, DOM presence, and CSS classes, not just logic.

**For Native Faces (Wayland/OpenXR/Android):**
1. **Write the Test:** Add a test case to the visual states suite (e.g., `tests/face_visual_states.rs`).
2. **Verify Failure:** Run the test using `cargo test --test face_visual_states` to observe failure.
3. **Implement Render Logic:** Update the platform-specific drawing or layout code (e.g., `src/platform/linux/wayland.rs` or common UI layout code).
4. **Verify Success:** Run the test again to prove it correctly simulates the rendering.
5. *Rule:* Native faces must provide a testing stub or string-buffer renderer so that visual states, dimensions, and text rendering logic can be validated headlessly in CI without requiring an active Compositor or XR runtime.

### 4.3 Pipeline Verification & Commit Standards
- **Commit Messages**: Must follow the `type(scope): description` format (e.g., `feat(brain): add sentiment analysis`).
- **Atomic Commits**: Each commit must represent a single logical unit of change.
- **Pipeline Compliance**: 
  - AI Agents must run `cargo check` and `cargo test` after any Rust file changes.
  - AI Agents must run `cd svelte_ui && npm run build` after any Svelte/TS file changes to ensure the SPA statically yields.
  - Do not commit code that breaks the compilation or testing pipeline.
  - All commits are gated by the `scripts/pre-commit.sh` hook, which runs workspace-wide checks and tests.
- **Automatic Commit on Task Completion**: Upon marking a task as complete in `task.md`, the AI Agent must automatically perform a commit of all relevant changes using the standardized commit format, provided the workspace is in a "Green" state.

---

## 5. Interaction Etiquette for AI Agents

1. **Be Proactive**: If a change requires a new CSS rule or a documentation update, do it immediately.
2. **Be Aesthetic**: If generating UI, ensure it looks "Tactical" and "Premium." Avoid generic layouts.
3. **Be Structured**: Use headers and bold text in responses to make technical details easy to parse.

## 6 AI Agent will not use Documentation name's and reff's in the code comments or doc comments.
1. Using "Phase 10" Is non-descriptive and confusing. Instead use the name of the feature or module for ease of understanding.
2. Using "TOS AI Development Standards" in code comments or doc comments. Is confusing and not needed.
