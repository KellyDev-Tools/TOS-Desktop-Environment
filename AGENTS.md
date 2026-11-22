# AGENTS.md — working in this repo

Instructions for any AI agent (or human) picking up TOS cold. This
file is the entry point; the detail lives in the documents it points
at. It covers **how** work is done here and **where the truth
lives** — never a task list. (The current work is the Roadmap's
current phase, as directed by the maintainer.)

## Read in this order

1. [Vision](docs/vision.md) — what TOS is for, and the status of
   every element of it. Nothing leaves that ledger silently.
2. [Architecture Specification](docs/spec/architecture.md) — the
   definitions. **The spec is the source of truth**: where the spec
   and the roadmap differ, the spec governs.
3. [Roadmap](docs/roadmap.md) — the current campaign: sequence and
   gates only. Roadmaps are transient; when one retires, its open
   items are harvested into the spec or the Vision ledger first,
   and the file is cleared to an empty placeholder.
4. [Scope Split](docs/decisions/2026-11-15-scope-split.md) — the
   keep / delegate / park / drop decision, frozen as made on
   2026-11-15.

The full documentation map is [docs/README.md](docs/README.md).

## Standing design rulings (do not relitigate)

- TOS is an **ambient layer** inside a host desktop. The host keeps
  windows, apps, session, and settings; TOS is not a desktop
  environment and has no session of its own.
- The **terminal stays a completely normal terminal**. Its output
  is never scraped, sniffed, or reinterpreted to drive features.
- **Chips derive from state the Brain already holds** — cwd,
  directory listing, session state. Chips are a touch-friendly
  accelerator that types into the terminal (Architecture §7.7).
- The **prompt is an independent overlay**, docked to nothing.
- The remote surface is **three streams only** — terminal, chip
  state, prompt. Everything else is refused over remote
  connections; the local/UDS surface is unrestricted.
- Parked subsystems (tos-searchd, tos-heuristicd, tos-priorityd,
  tos-marketplaced, tos-loggerd, the ZAYA AI module) stay parked
  until the Roadmap revives them. Parking is not deleting.

## How to work

- Build and test through the **Makefile** (`make check`,
  `make test`). The full standards — testing tiers, coding rules,
  commit rules, definition of done — live in
  [docs/development/standards.md](docs/development/standards.md).
- CI's blocking gates are the Rust job (`cargo check --workspace`
  + `cargo test --workspace`) and the gated Playwright specs. The
  Svelte type-check and legacy E2E jobs are non-blocking baselines
  by design.
- Commits: `type(scope): description`, one logical change, green
  only. Do not leave passing work uncommitted.
- Do not reference process documents or roadmap phase names in
  code comments; comments say what and why.
- Documentation uses **stable role-based names**: a file's
  location states its lifecycle (`spec/` governs, `decisions/` is
  frozen, `guides/` instructs, `archive/` remembers), and versions
  live in document headers and git history — not in filenames.
  Historical documents keep their original names under
  `docs/archive/`.
- Linux UI development and testing happen on the maintainer's
  NixOS host — see
  [docs/development/nixos.md](docs/development/nixos.md).

When in doubt: the Vision says why, the spec says what, the
roadmap says in what order.
