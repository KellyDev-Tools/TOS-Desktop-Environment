# TOS roadmap to success

TOS does not need more features. It needs one route that works end to end, feels right, and stays working. This roadmap turns the ambient-layer direction — terminal as animated background, chips overlay, prompt overlay, host desktop environment (DE) handling windows and apps — into a sequence of phases, each gated by a test you can actually run. No phase starts until the previous gate passes, and no calendar dates are promised: progress is measured in gates cleared, not weeks elapsed.

The companion decision document is `docs/spec/TOS_Scope-Split.md` in the TOS repo (PR #2, branch `docs/scope-split-ambient-layer`). This roadmap assumes that split: keep Brain and tos-common, delegate session and app management to GNOME/KDE, drop the Wayland face and `archive/`, park everything else.

## Success is a boring first slice, not a big launch

Success, in order:

1. You log into GNOME or KDE as normal, and TOS is simply *there* — a live terminal behind your windows, chips at the edge, prompt one gesture away.
2. That local slice survives daily use without surprises: no lost sessions, no focus fights, no mystery disconnects.
3. The same layer follows you to another machine because the Brain lives on tkserver, not on the laptop in front of you.
4. Only then does TOS widen — placement choices, a GNOME shim, revived daemons, and eventually an XR face — each addition landing on a foundation that is already trusted.

If a proposed feature does not serve the step you are on, it parks. That rule is the whole strategy.

## The starting point is stronger than it feels

The foundation check in November 2026 built and tested the current tree after five months untouched. The results shape everything below:

- The full Rust workspace compiles clean on current stable Rust — Brain, tos-common, all seven daemons, the signer, the faces — with only unused-dependency warnings.
- tos-common passes 97 of 97 tests. Brain, the integration tests crate, the daemons, and the signer all pass too.
- The only red test suite is the Wayland face, whose test code no longer compiles after `SurfaceConfig` in tos-common gained a `depth` field — core/face drift that nothing caught because there is no continuous integration (CI).
- The Svelte face carries 19 `svelte-check` errors, concentrated in CommandHub, SettingsModal, and the main page — the views the ambient face will reuse most.
- The Playwright end-to-end (E2E) suite last recorded run failed all 10 tests for an environment reason — Chromium could not launch on NixOS (missing `libglib-2.0.so.0`) — so it has likely never given a trustworthy green signal in the real environment.
- Roughly 496 `.unwrap()` / `.expect()` calls across the core Rust crates are a plausible source of the "feels buggy" experience in a long-running daemon, even where the logic is correct.

The conclusion matters: do not rebuild the foundation. Pin it down, narrow the scope to what it already does well, and fix the seams where faces and core meet.

## Three rules govern every phase

- **Freeze before you widen.** One Brain, one face, one terminal session, working — before remote, before a second daemon, before any new module. The commit history shows the opposite pattern (five faces, eight daemons, a new AI module as the final commit) and May shows the cost: weeks fighting connection, certificate, and protocol seams.
- **Gates, not dates.** Each phase ends with done-when checks. A phase is done when its checks pass on a clean run, not when time runs out.
- **Elegance is a requirement, not polish.** The zoom/slide summon motion and the always-accessible prompt are the product. If a shortcut makes the slice work but feel wrong, it is not done — that trade is what stalled the previous direction.

## Phase 0 — Clean the board and make breakage visible

Goal: the repo reflects the new scope, and the next breakage is caught the week it happens instead of five months later.

Work:

- Merge the scope split (PR #2) so the keep / delegate / park / drop decisions are the recorded baseline.
- Execute the drops: export a tagged snapshot of `archive/` (588 files) outside the working repo, then remove it; delete `fix-regression` (zero unique commits, 49 behind `main`); salvage any brain tests from `brain-test-situation` (2 unique commits, about 158 behind) into `tests/`, then delete that branch.
- Remove the Wayland face crate from the workspace per the split — its tests are the only red suite, and the layer it represented is delegated to the host. Salvage useful protocol notes into docs first.
- Add basic GitHub Actions CI: `cargo check`, `cargo test` for the kept workspace, and `svelte-check` for the Svelte face. Red must block merges to `main`.
- Fix the Playwright E2E environment (the NixOS Chromium / `libglib-2.0.so.0` launch failure) so the suite can run at all — even before it is green, it must be *runnable*.

Done when:

- [ ] `main` builds clean in CI with no dropped components in the workspace
- [ ] CI runs Rust tests and `svelte-check` on every push
- [ ] The E2E suite launches Chromium and completes a run (pass or fail on app logic, not on environment)
- [ ] `archive/` and both stale branches are gone from the active repo, with a snapshot preserved

## Phase 1 — Freeze the first slice: local, ambient, boring

Goal: Brain plus one ambient face on one machine, doing three things reliably — background terminal, chips, prompt. This is the slice the scope split recommends freezing end to end before anything is unparked.

Work:

- Run the Brain locally as an ordinary user service inside a normal GNOME/KDE (or other layer-shell compositor) session. No TOS session, no display-manager integration.
- Build the smallest layer-shell client that hosts the Svelte-derived UI in three surfaces:
  - *Background* — the existing `XtermTerminal` component rendering one persistent Brain session, dimmed at rest, organised by the sector model.
  - *Chips* — the bezel system (`ExpandedBezel`, `WarningChip`, Brain/session status views) reduced to a glanceable strip that never steals focus.
  - *Prompt* — CommandHub, summoned by hotkey, zooming in from the background or sliding in from an edge, taking focus while in use and dismissing back into the background.
- Fix the 19 Svelte type errors only in the components this slice uses (CommandHub, the terminal, the chips, the main page shell). SettingsModal and parked views wait.
- Point the repaired E2E suite at this slice: launch, summon, run a command, dismiss, reconnect.

Done when:

- [ ] Cold start to usable background terminal works inside a host DE session
- [ ] The prompt summons and dismisses with the intended motion, every time, without focus loss or resize loops
- [ ] One terminal session persists across face restarts (Brain keeps the session)
- [ ] `svelte-check` is clean for all slice components
- [ ] The E2E slice tests pass green in CI and locally on NixOS
- [ ] A full day of ordinary use produces no Brain crash — any panic found gets a test before it gets a fix

Explicitly not in this phase: remote Brain, GUI app launching, search daemon, AI views, editor, marketplace, any second face.

## Phase 2 — Make remote the default topology

Goal: the same ambient face, unchanged in feel, talking to a Brain on tkserver. Your workspace follows you; the layer stays thin and local.

Work:

- Narrow the remote protocol to exactly three streams: terminal streams, chip state, and prompt commands. Anything the May remote work carried beyond those three parks until a later phase asks for it.
- Run the Brain connection inside a mesh virtual private network (Tailscale or WireGuard) and let the network handle identity and encryption, instead of maintaining public TLS certificates and bypasses for a personal tool. Keep the existing SSH fallback in tos-common as the break-glass path.
- Carry over the useful lessons from the parked Electron face — Brain discovery and remote connection handling — without reviving Electron as the shell.
- Define reconnect behaviour as a feature: the face must survive sleep, network drops, and Brain restarts by reattaching to the same persistent sessions.

Done when:

- [ ] The Phase 1 slice passes its E2E tests with the Brain on tkserver, not just locally
- [ ] Disconnect and reconnect (including laptop sleep/wake) restore the same terminal session with no lost scrollback beyond a stated limit
- [ ] No public TLS or certificate workaround is required — connection works on the tailnet only
- [ ] Chips truthfully show connection state (local / remote / disconnected) at a glance

## Phase 3 — Placement: run location is not display location

Goal: launching from the prompt can place an app sensibly across host and server, and you learn by using it whether placement should be invisible or explicit.

Background: where an app *runs* (host or server) and where it *shows* (host screen or, later, XR) are independent. Terminals already split this way — the pseudo-terminal (PTY) lives on the server and renders anywhere. For graphical apps on the host screen, waypipe can proxy an individual Wayland app from the server so it appears as an ordinary window in the host DE.

Work:

- Implement the two flat-screen combinations first: host-run/host-show (TOS asks the host DE to launch, then steps aside) and server-run/host-show (terminals natively; GUI via waypipe).
- Ship placement as an experiment, not a law:
  - Sensible defaults per type — terminals and (later) AI on the server, heavy or GPU-sensitive local apps on the host.
  - A small glyph on each chip showing where that item is running.
  - An override in the prompt for the launches where you care.
  - A single preference controlling how prominent the choice is: invisible default, quiet glyph only, or explicit choice per launch.
- Live with each setting for real work before deciding. Record the decision in the architecture docs when it settles.

Done when:

- [ ] Both flat-screen combinations launch from the prompt without manual shell workarounds
- [ ] The placement preference exists, switches behaviour without a rebuild, and the default is chosen from actual use
- [ ] A mis-placed app can be identified from its chip and relaunched in the other location from the prompt

## Phase 4 — Widen carefully, one piece at a time

Goal: broaden where TOS runs and what the prompt can do — in an order where each step is small, tested, and reversible. Nothing in this phase runs in parallel.

Order:

1. **GNOME shim.** Mutter does not offer layer-shell to regular clients, so GNOME needs its own small path — a Shell extension or a desktop-window approximation. Pick one (open decision below), prototype it against the Phase 1 surfaces only, and hold it to the same E2E slice tests. KDE, Sway, Hyprland, and COSMIC continue on layer-shell.
2. **Revive searchd.** The first parked daemon to return, feeding prompt search once local in-Brain search outgrows itself. Revive behind the existing tos-common contracts and its passing test suite.
3. **Reduce the panic surface.** Work through the roughly 496 `.unwrap()` / `.expect()` calls in Brain and tos-common hot paths, converting daemon-killing panics into handled errors, highest-traffic code first. Each conversion lands with a test.
4. **One packaging target only.** Revive packaging for the distro you actually run, so installing the ambient layer on a new machine is one command. The other targets stay parked.

Done when (per step, in order):

- [ ] The slice works on GNOME via the chosen shim, passing the same slice tests
- [ ] Prompt search uses searchd with no change to the summon/dismiss feel
- [ ] Brain hot paths contain no unhandled panic paths found by audit; CI stays green throughout
- [ ] A fresh machine can be set up from the single packaging target plus the setup docs

Still parked after this phase unless a step explicitly revives it: priorityd, loggerd (as a daemon), heuristicd, marketplaced, tos-signer, the Android faces, and the ZAYA1-8B module. The module system and services registry stay in tos-common so any of them *can* return without core surgery.

## Phase 5 — XR: a protected slot, not a promise

Goal: keep the Quest / spatial face possible without letting it tax the phases above, then build it only once the flat-screen layer is trusted.

What is already protected: `platform/quest.rs` in tos-common, the `spatial` profile, controller mapping, and the OpenXR platform guide. Keeping that slot costs almost nothing now and avoids a redesign later.

The one hard rule, from the scope split: **stream content, never the viewpoint.** Panels are rendered locally on the headset with streamed content textures — the sector/panel model the core already uses. Terminals and text are the forgiving first content; fast-moving 3D is out of scope.

Work (only when this phase opens):

- Prove the panel model with one read-only surface first: chips in XR, streamed from the existing Brain, panels placed at the edge of vision.
- Add the terminal panel second: pull a panel close to read and work, push it back to ambient.
- Treat run/display placement from Phase 3 as the routing layer — server-run/XR-show and host-run/XR-show are configurations, not new subsystems.

Done when:

- [ ] Chips and one terminal panel render in passthrough with locally rendered frames and streamed content
- [ ] Head movement shows no streamed-viewpoint lag (panels stay world-locked)
- [ ] The same Brain session is continuous across flat screen and headset

No date, no commitment: if Phase 4 is where TOS happily lives for a year, that is success, not delay.

## How this fits a small time budget

The roadmap is sequenced so the scarce resource — your design judgement — is spent only where only you can decide, and everything else can be prepared between sessions:

| Needs you | Can be carried for you |
|---|---|
| Whether the summon motion feels right | CI setup, branch/archive cleanup, test repairs |
| Placement preference, from living with it | Svelte type-error fixes in named components |
| GNOME shim choice, once prototyped | E2E environment fix and slice test scaffolding |
| When a parked piece earns revival | Panic-surface audit lists, docs updates |

A useful rhythm: one gate at a time, reviewed in one sitting. Each phase above is sized so its done-when list fits on one screen — if a phase grows past that, split it before starting it.

## Risks worth naming

- **Layer-shell fragmentation.** "Any Linux environment" is really "layer-shell compositors, plus a GNOME exception." The Phase 4 shim exists precisely so this stays one exception, not five.
- **Remote scope creep.** The moment remote carries more than terminal, chips, and prompt, it becomes the May problem again. The three-stream limit is a gate, not a guideline.
- **Reviving too early.** Every parked daemon and face still compiles and mostly passes tests, which makes unparking tempting. Tempting is not the criterion — the current phase gate is.
- **Single-maintainer bus factor.** All 418 commits are yours. CI, the integration tests crate, and these two decision documents are the project's memory; keeping them current is part of every phase, not extra credit.

## Decisions still open

These were aired and deliberately not settled. Each phase names the one it needs, so none of them blocks starting Phase 0:

1. **GNOME shim approach** — Shell extension versus desktop-window approximation. Needed at Phase 4, step 1.
2. **Placement preference default** — invisible defaults versus explicit choice, and how prominent the running-location glyph should be. Settled by experiment in Phase 3.
3. **Daemon revival order after searchd** — priority, logging, heuristics, marketplace. Needed only after Phase 4, step 2.
4. **XR timing** — when the spatial face moves from reserved slot to active work. Needed only to open Phase 5.

## The first move

Merge PR #2, run Phase 0, and do not write a new feature until the board is clean and CI is watching. The foundation already compiles and its core tests already pass — the roadmap to success is mostly refusing to widen until the one route you use every day is boring, elegant, and green.
