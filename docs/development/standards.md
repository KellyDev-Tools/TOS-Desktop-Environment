# TOS Development Standards

How TOS is developed: build and test gates, testing tiers, coding
and commit standards, and the Stable Common contract. This document
covers **how** to develop — **what** to develop lives in the
[spec suite](../spec/architecture.md), sequenced by the
[Roadmap](../roadmap.md). [AGENTS.md](../../AGENTS.md) is the entry
point that points here.

*This document is the 2026-11-21 merge of the former root
`TOS AI Development Standards.md` and `docs/guides/Development-
Standards.md`, which had drifted apart. Where they disagreed, the
verified facts won: `make test` is an alias for the full `test-all`
chain, and CI runs cargo and Playwright directly (see §6).*

---

## 1. Use the Makefile

The top-level `Makefile` is the single source of truth for building,
testing, and running TOS during development. Use it instead of
running raw `cargo`, `npm`, or `npx` commands directly.

**Key targets:**

| Action | Command |
|---|---|
| Full workspace syntax check | `make check` |
| Build everything | `make build-all` |
| Run all tests | `make test` (alias for the full `test-all` chain) |
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

Create new tests for every feature or bug fix. Every task must
include tests at **all applicable tiers** covering the new behavior
before it is considered complete.

### 2.1 Test Tiers

1. **Unit Tests** — Validate a single function or struct in
   isolation. No I/O, no state, no side effects.
   - Rust: inline `#[cfg(test)]` modules.
   - Svelte/TS: adjacent `.spec.ts` files.

2. **Integration Tests** — Validate interactions between subsystems
   headlessly (e.g., IPC command → state mutation → JSON delta).
   - Location: `tests/` at the workspace root, or `<crate>/tests/`.

3. **System / E2E Tests** — Validate the full stack end-to-end.
   - Brain system test: `make test-system`
   - Playwright E2E: `make test-e2e`
   - Orchestration health: `make test-health`

**Don't ask if you should make a new test; the answer is always
yes.**

### 2.2 Definition of Done

A task is **not complete** until:

1. Tests for the new behavior exist at every applicable tier.
2. **Registration testing passes**: run `make test` (all unit +
   integration tests) and confirm zero failures.
3. `make check` passes with no errors.
4. If Svelte/TS files were changed: `make build-face-web` succeeds.

Do not commit code that breaks any of these gates.

---

## 3. The Stable Common Contract

`tos-common` is the foundation of the entire workspace. Changes
there ripple through every service (Brain, daemons, faces), so it
is held to a stability-first contract:

- **Deprecate, don't delete.** If a public API in `tos-common`
  must change, add the new method or field and mark the old one
  `#[deprecated]` instead of removing it immediately.
- **Backward compatibility.** Existing binaries and tests must
  keep compiling against the changed core.
- **Verification responsibility.** If you modify `tos-common`, you
  are responsible for verifying all dependents — the full
  `make test` chain, not just the crate's own tests.

---

## 4. Coding Standards

### 4.1 Rust

- Use enums for state, not strings or booleans.
- Use `Result` / `Option`. Do not use `unwrap()` in production
  paths.
- Do not use `#[allow(warnings)]` or blanket `unsafe` without
  explicit approval.
- Report errors through `LogManager` with appropriate `LogType`
  and region context.
- Every public function and struct must have a doc comment.

### 4.2 Svelte / TypeScript

- Component logic that can be tested independently should be
  extracted into `.ts` files with corresponding `.spec.ts` tests.
- UI components must use the project's CSS custom properties
  (`var(--color-primary)`, etc.) — do not hardcode colors or
  spacing.

### 4.3 Cross-Cutting

- All physical input must translate to a `SemanticEvent`. Never
  bind logic directly to raw key/mouse events.
- Platform-specific code stays behind the `Renderer`,
  `InputSource`, and `SystemServices` traits. Do not leak platform
  types into core logic.

---

## 5. Commit Standards

- **Format**: `type(scope): description` (e.g.,
  `feat(brain): add offline AI queue`).
- **Atomic**: One logical change per commit.
- **Green state only**: commits are gated by
  `scripts/pre-commit.sh`. If `make check` or `make test` fails,
  the commit is rejected.
- **Commit when green**: once all gates pass, commit immediately
  in the standard format. Do not leave passing work uncommitted.

---

## 6. Automated Guardrails and CI

- **Pre-commit hook.** The build installs a Git pre-commit hook
  (via `scripts/pre-commit.sh`) that runs the check and test
  gates. A failing gate blocks the commit; fix the regression
  rather than reaching for `--no-verify`, which only defers the
  failure to CI.
- **CI (GitHub Actions).** The blocking gates are the Rust job
  (`cargo check --workspace --all-targets` +
  `cargo test --workspace --no-fail-fast`) and the gated
  Playwright job (the slice, chip-lifecycle, remote, reconnect,
  and narrowing specs). The Svelte type-check and legacy E2E jobs
  are non-blocking baselines by design — their redness is a gauge,
  not a gate.

---

## 7. AI Agent Conduct

1. **Don't reference process documents in code.** No comments like
   `// per the development standards`. Comments describe *what*
   and *why*, not process.
2. **Don't reference roadmap phase names in code.** Use
   descriptive feature/module names, not "Phase 10" or "Stage 4."
3. **Be proactive.** If a change requires a new CSS rule, a doc
   update, or a test fixture — do it in the same pass.
4. **Run the gates.** After any Rust change: `make check` then
   `make test`. After any Svelte/TS change:
   `make build-face-web` then `make test`. Don't wait to be asked.
5. **Start from the reading order.** [AGENTS.md](../../AGENTS.md)
   gives it: Vision, then spec, then Roadmap. Never work from an
   archived roadmap or task list — the archive is history, not a
   queue.

---

## 8. Multi-Environment & Linux UI Development

TOS is developed across multiple environments; Linux UI work and
Linux-specific build targets belong on the maintainer's NixOS
host, whose environment is documented in
[nixos.md](./nixos.md).

- The NixOS host is the primary Linux development workspace,
  linked to the same origin repository.
- It hosts a persistent `tmux` session (typically named `tos`).
  Attach to it (`tmux attach -t tos`) to run services, monitor
  logs, and execute build targets in the background.
- **NixOS dev & testing rule**: compilation checks (`make check`),
  testing (`make test`), linting, and background services for
  Linux UI work run inside that persistent session, for
  high-fidelity Linux compatibility.
