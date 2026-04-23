# TOS AI Development Standards

Rules for any AI agent working in this codebase. This document covers **how** to develop—not **what** to develop. Feature specs, roadmaps, and architectural details live in `docs/`, `TOS_Beta-0_Roadmap.md`, and `task.md`.

---

## 1. Use the Makefile

The top-level `Makefile` is the single source of truth for building, testing, and running TOS. AI agents must use it instead of running raw `cargo`, `npm`, or `npx` commands directly.

**Key targets:**

| Action | Command |
|---|---|
| Full workspace syntax check | `make check` |
| Build everything | `make build-all` |
| Run all tests | `make test` |
| Run Brain core tests only | `make test-core` |
| Run shell integration tests | `make test-shell` |
| Run system-level test | `make test-system` |
| Run E2E (Playwright) | `make test-e2e` |
| Run UI component tests | `make test-ui-component` |
| Run orchestration health check | `make test-health` |
| Launch Brain + Web Face (dev) | `make run-web-dev` |
| Format code | `make fmt` |
| Lint (Clippy) | `make lint` |

Run `make help` for the full list.

---

## 2. Testing Requirements

Every task must include tests at **all applicable tiers** before it is considered complete.

### 2.1 Test Tiers

1. **Unit Tests** — Validate a single function or struct in isolation. No I/O, no state, no side effects.
   - Rust: inline `#[cfg(test)]` modules.
   - Svelte/TS: adjacent `.spec.ts` files.

2. **Integration Tests** — Validate interactions between subsystems headlessly (e.g., IPC command → state mutation → JSON delta).
   - Location: `tests/` at the workspace root, or `<crate>/tests/`.

3. **System / E2E Tests** — Validate the full stack end-to-end.
   - Brain system test: `make test-system`
   - Playwright E2E: `make test-e2e`
   - Orchestration health: `make test-health`

### 2.2 Definition of Done

A task is **not complete** until:

1. Tests for the new behavior exist at every applicable tier.
2. **Registration testing passes**: run `make test` (all unit + integration tests) and confirm zero failures.
3. `make check` passes with no errors.
4. If Svelte/TS files were changed: `make build-face-web` succeeds.

Do not commit code that breaks any of these gates.

---

## 3. Coding Standards

### 3.1 Rust

- Use enums for state, not strings or booleans.
- Use `Result` / `Option`. Do not use `unwrap()` in production paths.
- Do not use `#[allow(warnings)]` or blanket `unsafe` without explicit approval.
- Report errors through `LogManager` with appropriate `LogType` and region context.
- Every public function and struct must have a doc comment.

### 3.2 Svelte / TypeScript

- Component logic that can be tested independently should be extracted into `.ts` files with corresponding `.spec.ts` tests.
- UI components must use the project's CSS custom properties (`var(--color-primary)`, etc.)—do not hardcode colors or spacing.

### 3.3 Cross-Cutting

- All physical input must translate to a `SemanticEvent`. Never bind logic directly to raw key/mouse events.
- Platform-specific code stays behind the `Renderer`, `InputSource`, and `SystemServices` traits. Do not leak platform types into core logic.

---

## 4. Commit Standards

- **Format**: `type(scope): description` (e.g., `feat(brain): add offline AI queue`).
- **Atomic**: One logical change per commit.
- **Green state only**: All commits are gated by `scripts/pre-commit.sh`. If `make check` or `make test` fails, the commit is rejected.
- **Auto-commit on task completion**: When a task in `task.md` is marked complete and the workspace is green, commit immediately using the standard format.

---

## 5. AI Agent Conduct

1. **Don't reference this document in code.** No comments like `// per TOS AI Development Standards`. Comments should describe *what* and *why*, not cite process docs.
2. **Don't reference roadmap phase names in code.** Use descriptive feature/module names, not "Phase 10" or "Stage 4."
3. **Be proactive.** If a change requires a new CSS rule, a doc update, or a test fixture—do it in the same pass.
4. **Run the gates.** After any Rust change: `make check` then `make test`. After any Svelte/TS change: `make build-face-web` then `make test`. Don't wait to be asked.
5. **Commit when green.** Once all gates pass, immediately create a commit using the `type(scope): description` format. Do not leave passing work uncommitted.
