# State of TOS

*Snapshot, 2026-11-22 (Phase 2 closure). This page is a photograph, not a record:
it is refreshed at campaign boundaries and allowed to go stale
between them. The durable record of every element's status is
the [Vision](vision.md) ledger; the current sequence is the
[Roadmap](roadmap.md).*

## Works today (merged on main)

- **The ambient slice.** Brain + GTK4 layer-shell face as systemd
  user services under `tos.target`: terminal session as the
  desktop background, chips overlay, independent prompt overlay.
- **Session persistence.** The shell session survives face
  restarts; a reconnecting face resubscribes and repaints from
  the Brain's current grid — the snapshot is the whole truth,
  no replay protocol.
- **Accelerator chips.** Directory chips derived from Brain-held
  state (cwd + listing), minted in generations; taps type into
  the terminal. Breadcrumb navigation included.
- **Dynamic chips.** The strip filters against the word being
  typed (prefix matches first), and carries its own `.*` switch
  for dotfiles (default hidden, like `ls`).
- **Terminal rendering.** The Brain owns terminal emulation
  (Spec 0.1.4): it parses the PTY stream once into a per-session
  grid and publishes display state; the face paints cells on
  canvas and sends raw key events and text back for the Brain
  to encode. xterm is gone. The on-device retest passed
  2026-11-22 — typing renders exactly as typed.
- **Remote Brain.** The default topology, proven end to end on
  tkserver over the tailnet (Phase 2, closed 2026-11-22):
  remote configuration (off = loopback-only, fail closed),
  reconnect with backoff and grid sync, and the **three-stream
  narrowing** — over a remote connection only the terminal,
  chip-state, and prompt streams are dispatched; everything
  else, including the retired raw-stream commands, is refused
  loudly.
- **CI.** Blocking gates: the Rust job and the gated Playwright
  specs (the grid spec joined the gate with PR #24). Type-check
  and legacy E2E run as non-blocking baselines.

## Verified by hand

- The slice end-to-end on the maintainer's NixOS desktop (nested
  Sway under GNOME): commands run normally, chips refresh after
  commands, chips filter while typing.
- Dynamic chips on device (2026-11-22 retest): the strip
  filters while typing, the `.*` switch and breadcrumb/entry
  taps all behave, and commands execute exactly once.
- The grid face on device (2026-11-22 retest): typing renders
  exactly as typed — no staircase, no doubling — and a
  submitted command shows once.
- The narrowing gate over a real remote WebSocket connection.
- **The remote end-to-end run** (2026-11-22): Brain on tkserver
  over the tailnet, face on the NixOS box — the run sheet's
  checks all passed. Phase 2 closed on this run.

## Not yet verified

- **Summon/dismiss feel** — the prompt's motion and focus
  behaviour have never had a dedicated assessment.
- **A full day of use** without a Brain crash.
- **The fish trial** — whether fish becomes the TOS default
  shell is an open question with a written trial plan, not a
  decision.
- **Chips-surface painting on device** — the ghost-rectangle
  fault from the 2026-11-22 retest is unconfirmed either way
  since the probe build (composited layers removed) landed;
  it needs one more device look.

## Knowingly rough

- Parts of the spec suite beyond the Architecture describe
  parked systems (marketplace, kanban orchestration, the AI
  module) in present tense. Banners mark the worst of it; the
  Vision ledger is the authority on what exists.
- The User Manual's framing predates the ambient-layer pivot;
  its concepts carry over, its world doesn't.
- The legacy Svelte E2E baseline is mostly red **by design** —
  it measures the old face's surface, not the current one's.

## Parked — do not build

The satellite daemons (search, heuristic, priority, marketplace,
logger), the Electron and Android faces, standalone packaging,
the ZAYA AI module, and XR implementation. The Vision ledger
names what revives each. Parking is a decision, not an
oversight.

## Next

Phase 3 is the placement experiment: the Brain runs remotely
by default now — when the prompt launches something, where
should it run, and does the user choose, or never think about
it? See the [Roadmap](roadmap.md).
