# State of TOS

*Snapshot, 2026-11-21. This page is a photograph, not a record:
it is refreshed at campaign boundaries and allowed to go stale
between them. The durable record of every element's status is
the [Vision](vision.md) ledger; the current sequence is the
[Roadmap](roadmap.md).*

## Works today (merged on main)

- **The ambient slice.** Brain + GTK4 layer-shell face as systemd
  user services under `tos.target`: terminal session as the
  desktop background, chips overlay, independent prompt overlay.
- **Session persistence.** The shell session survives face
  restarts; a reconnecting face is replayed the terminal
  scrollback (256 KiB) as the whole truth.
- **Accelerator chips.** Directory chips derived from Brain-held
  state (cwd + listing), minted in generations; taps type into
  the terminal. Breadcrumb navigation included.
- **Dynamic chips.** The strip filters against the word being
  typed (prefix matches first), and carries its own `.*` switch
  for dotfiles (default hidden, like `ls`).
- **Terminal rendering.** The face renders through the xterm
  canvas renderer. An on-device retest (2026-11-22) shows the
  input-line fault persists in a new shape — redraws landing as
  new rows — so this is not counted as fixed; see the Roadmap's
  Phase 1.7 status.
- **Remote groundwork.** Brain remote configuration (off =
  loopback-only, fail closed), reconnect with backoff and
  scrollback truth, and the **three-stream narrowing**: over a
  remote connection, only terminal, chip-state, and prompt
  commands are dispatched; everything else is refused loudly.
- **CI.** Blocking gates: the Rust job and the gated Playwright
  specs. Type-check and legacy E2E run as non-blocking baselines.

## Verified by hand

- The slice end-to-end on the maintainer's NixOS desktop (nested
  Sway under GNOME): commands run normally, chips refresh after
  commands, chips filter while typing.
- Dynamic chips on device (2026-11-22 retest): the strip
  filters while typing, the `.*` switch and breadcrumb/entry
  taps all behave, and commands execute exactly once — the
  faults found were in painting, not behaviour.
- The narrowing gate over a real remote WebSocket connection.

## Not yet verified

- **The remote end-to-end run**: Brain on the home server over
  the tailnet, face on another machine. All the pieces are
  merged; the assembled run hasn't happened. It is Phase 2's
  last gate.
- **The canvas retest** ran on device 2026-11-22 and did not
  pass: dynamic chips verified working by hand, but terminal
  redraw painting and chips-surface painting are both faulty
  on device. Diagnostics are in progress (Roadmap Phase 1.7).
- **Summon/dismiss feel** — the prompt's motion and focus
  behaviour have never had a dedicated assessment.
- **A full day of use** without a Brain crash.
- **The fish trial** — whether fish becomes the TOS default
  shell is an open question with a written trial plan, not a
  decision.

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

Phase 2 closes with the server end-to-end run; Phase 3 is the
placement experiment (when the Brain runs remotely by default,
does the user choose where things run, or never think about
it?). See the [Roadmap](roadmap.md).
