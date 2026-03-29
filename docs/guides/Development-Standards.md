# TOS Development Standards: Monorepo Stability

To maintain a high-fidelity and stable operating system, we follow a **Stability-First** development model. This guide outlines the standards for working with shared components and the automation that enforces them.

## 1. The "Stable Common" Contract

`tos-common` is the foundation of the entire workspace. Changes here ripple through all services (Brain, Search, Faces, etc.).

### 1.1 Non-Breaking Changes
- **Depreciate, Don't Delete:** If you need to change a public API in `tos-common`, add a new method/field and mark the old one as `#[deprecated]` instead of removing it immediately.
- **Backward Compatibility:** Ensure that existing binaries and tests can still compile against your changes.

### 1.2 Verification Responsibility
- If you modify `tos-common`, you are responsible for verifying **all** dependents.
- **REQUIRED:** Run `make test-all` before every commit.

## 2. Automated Guardrails

We use automated hooks to ensure that no "breaking" code enters the repository.

### 2.1 Git Pre-commit Hook
The build system automatically installs a Git pre-commit hook located at `.git/hooks/pre-commit`.
- **Trigger:** Happens automatically when you run `make build-all`, `make test-all`, or `make check`.
- **Action:** Runs `cargo check --workspace` and `make test-all`.
- **Failure:** If any check fails, the commit is blocked. You must fix the regression before committing.

### 2.2 Continuous Integration (CI)
Our GitHub Actions pipeline runs the same `make test-all` suite on every Push and Pull Request. A "Green" build is required for merging into `main`.

## 3. Workflow Summary

1. **Modify Code:** Make your changes in a service or common library.
2. **Local Check:** Run `make check` to ensure basic syntax/type stability.
3. **Commit:** The pre-commit hook will run the full test suite.
4. **Push:** CI will perform final verification.

---
> [!TIP]
> If you need to perform a quick commit for an urgent fix, you *can* use `git commit --no-verify`, but this is **strongly discouraged** as it will likely fail the CI pipeline anyway.
