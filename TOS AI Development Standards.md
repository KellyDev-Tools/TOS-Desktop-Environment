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
- **Colors**: Use the curated palette in `variables.css` (`--lcars-orange`, `--lcars-blue`, `--lcars-gold`, `--lcars-red`).
- **Glassmorphism**: Use semi-transparent backgrounds with `backdrop-filter: blur()` for overlays.
- **Layout**: Prefer high-density grids and "Elbow" paths over traditional window borders.

### 3.2 Premium Micro-Animations
Interfaces must feel "alive."
- Any state change (mode switch, zoom) must be accompanied by a subtle transition or animation.
- Use the `recursive-zoom` keyframes for level transitions.

---

## 4. Testing & Validation

### 4.1 Test-Driven Development (TDD) & "No Code Without Tests"
- **Test-Driven Development (TDD)** is strictly required. Write the test first, verify it fails, then write the tactical implementation to pass it.
- Every new feature must include a corresponding integration test in `tests/`.
- Tests should verify the "Hierarchy Round-Trip" (e.g., initializing state, zooming to a level, triggering an event, verifying state change).

### 4.2 Terminal Verification
- AI Agents must run `cargo check` and `cargo test` after every significant file change to ensure no regression.

---

## 5. Interaction Etiquette for AI Agents

1. **Be Proactive**: If a change requires a new CSS rule or a documentation update, do it immediately.
2. **Be Aesthetic**: If generating UI, ensure it looks "Tactical" and "Premium." Avoid generic layouts.
3. **Be Structured**: Use headers and bold text in responses to make technical details easy to parse.

## 5 AI Agent will not use Documentation name's and reff's in the code comments or doc comments.
1. Using "Phase 10" Is non-descriptive and confusing. Instead use the name of the feature or module for ease of understanding.
2. Using "TOS AI Development Standards" in code comments or doc comments. Is confusing and not needed.
