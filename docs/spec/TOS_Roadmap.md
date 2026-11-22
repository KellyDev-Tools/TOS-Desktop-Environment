# TOS roadmap to success

TOS does not need more features. It needs one route that works end to end, feels right, and stays working. This roadmap turns the ambient-layer direction — terminal as animated background, chips overlay, prompt overlay, host desktop environment (DE) handling windows and apps — into a sequence of phases, each gated by a test you can actually run. No phase starts until the previous gate passes, and no calendar dates are promised: progress is measured in gates cleared, not weeks elapsed.

The companion decision document is `docs/spec/TOS_Scope-Split.md` in the TOS repo (PR #2, branch `docs/scope-split-ambient-layer`). This roadmap assumes that split: keep Brain and tos-common, delegate session and app management to GNOME/KDE, drop the Wayland face and `archive/`, park everything else.

**Source of truth:** the spec suite (`docs/spec/TOS_v0.1.2_Architecture.md` and its companions) defines what TOS is. This roadmap sequences the work to get there; on any question of definition or behaviour, the spec governs and this document defers to it.

**Lifecycle:** this roadmap is a campaign document. It serves one goal — the ambient layer, trusted in daily use — and when that goal is reached it will be *replaced* by a successor roadmap of greater refinement, not extended indefinitely; phase numbers scope to this document. Before it is retired, anything still open is harvested: open decisions and unmet done-when checks carry into the successor, and anything that has turned out to be a definition moves into the spec. Nothing durable should live only in this document — git preserves it, but preservation is not the same as being carried forward. When the final phase completes, the roadmap is **cleared**: this file persists as an empty placeholder between goals — a standing slot, holding no plan — until the next campaign's roadmap is written into it. The cleared campaigns remain in git history.

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
- [x] One terminal session persists across face restarts (Brain keeps the session) — delivered in Phase 1.5, below
- [ ] `svelte-check` is clean for all slice components
- [ ] The E2E slice tests pass green in CI and locally on NixOS
- [ ] A full day of ordinary use produces no Brain crash — any panic found gets a test before it gets a fix

Explicitly not in this phase: remote Brain, GUI app launching, search daemon, AI views, editor, marketplace, any second face.

## Phase 1.5 — Session persistence across face restarts

A small phase pulled forward from Phase 1's done-when list (and a precondition for Phase 2's reconnect story): a face restart must not blank the terminal.

Finding that shaped it: the shell was already Brain-owned and outlived faces — what was missing was the *display*. A reconnecting face paints only what it receives, and the Brain replayed nothing, so the ongoing session looked brand new.

Work:

- Retain the raw PTY output in the Brain (`ScrollbackBuffer`, 256 KiB, newline-aware trim) — appended in the read loop before each broadcast.
- On a new face connection, replay the retained output to that client alone as ordinary `pty_output`, before its live stream continues.
- Un-pin the slice E2E persistence test (it was a documented `test.fail()` during Phase 1) and hold it green in CI.

Done when:

- [x] A face restart (reload, crash, WebView bounce) repaints the terminal with the session's prior output, up to the stated 256 KiB limit
- [x] The slice E2E persistence test passes unpinned, locally and in CI

Explicitly not in this phase: persistence across *Brain* restarts (the buffer is in-memory; state restore already brings back the hub model, and seeding the buffer from it is a later refinement), per-hub shells, and the remote/reconnect behaviour Phase 2 defines.

## Phase 1.6 — Accelerator chips: the overlay earns its place

Placed before Phase 2 by decision (2026-11-17): the chip system is base to the whole idea, and remote (Phase 2) only carries whatever interaction exists — so the interaction comes first.

The model, stated plainly:

- **The terminal stays a completely normal terminal.** Normal output, any terminal app runs as it would anywhere else. Nothing is captured, converted, or reinterpreted.
- **The chip system is a touch-friendly accelerator that overlays.** Its whole goal is to increase access and reduce typing. Chips are generated from context the Brain already holds — never from transforming terminal output — and tapping a chip does the typing (or the acting) for you.
- **The prompt input acts independently of the terminal.** It is not docked onto the terminal view; the terminal is background and context, not the container. (Both points surfaced in the Phase 1 smoke test: the chips strip was empty at rest and the prompt was just a second terminal.)

Constraint that shapes the slice: **many of the chip system's information providers are frozen.** The satellite daemons (searchd, heuristicd, priorityd, marketplaced, loggerd) stay parked in this phase. V1 chips feed only from live Brain state and the services already running inside the Brain process.

The operative definition of this model is the spec — Architecture 0.1.2, the Spec 0.1.1 note and §7.7. The summary above is the phase's working statement; where the two differ, the spec governs.

Work:

- Keep the directory listing fresh in the Brain: refresh `shell_listing` when the shell's cwd changes and when a command completes, not only when a hub enters Directory mode — so the chips always describe the directory the terminal is actually in.
- Chips surface: alongside the always-on status/sector bar, render **directory accelerator chips** from the listing — tap a directory to `cd` into it, tap a file to insert its path — plus **cwd breadcrumb chips** (tap an ancestor to go there).
- Prompt surface: rebuild input-first — the dedicated CMD input bar as the overlay's primary, independent element, with the hub view as context. (SEARCH and AI modes stay inert: their providers are frozen.)
- Point the slice E2E at the new behaviour: chips for a known directory are present at rest, and tapping a directory chip changes the terminal's working directory.

Done when:

- [x] The chips strip is never empty at rest: status chips plus the current directory's accelerator chips
- [x] Tapping a directory chip `cd`s the terminal; tapping a file chip inserts its path — measurably less typing for the same navigation
- [x] Directory chips track the terminal's cwd across `cd` and across commands that change the directory's contents
- [x] The prompt summons as an independent input-first overlay, not a terminal with a bar attached
- [x] The slice E2E covers the above and stays green in CI

Explicitly not in this phase: heuristic suggestion chips (heuristicd), search chips (searchd), priority ordering (priorityd), marketplace chips (marketplaced) — each arrives when its provider unfreezes; parsing or transforming terminal output into chips, in any form.

**Status (2026-11-17): built, pending merge + CI.** Brain: `SectorManager::refresh_listing_for` refreshes the shell hub's listing at startup, on OSC 7 cwd changes, and on OSC 9002 command completions (entries sorted directories-first); OSC 7 paths are percent-decoded. One dependency to know: cwd tracking rides on the shell reporting OSC 7 — fish does natively; a shell that never reports it leaves the chips on the last known directory. Face: the chips surface renders an `AcceleratorChips` strip (status segment, cwd breadcrumbs, entry chips) and the prompt surface leads with an independent `PromptInputBar`. Two implementation calls worth recording: chips act by **typing into the terminal** (`terminal_input_hex`) rather than through the older model-based `dir_pick` commands, because the accelerator contract is about the real shell — the Brain's model catches up via OSC 7; and a file chip inserts the file's shell-quoted **name** (what you would type in that directory), not its absolute path. Also fixed on the way: a fresh session's hub now starts in the user's home directory instead of `/`. Slice E2E is 8 tests (chips at rest, prompt input, chip-tap `cd`, plus the Phase 1/1.5 suite) — green locally; the CI box above is checked when the PR's CI passes. Still Timothy's, unchanged from Phase 1: summon/dismiss motion and focus feel, and a full day of use.

**Chip lifecycle follow-up (2026-11-17, PR #11):** at Timothy's request, the listing now carries a **generation** counter — every issuance is a new chip set — and a PTY completion watcher re-issues it after *every* completed command in *any* shell (syncing the hub cwd from the shell process via /proc on Linux), so `ls` and even `pwd` produce a chip request without OSC integration. A **fake headless chip system** test (`tests/chips-lifecycle.spec.ts`) verifies the full sequence over the face protocol: `ls` → new request with the previous set deprecated and lowered in priority; directory-chip select → `cd` typed into the terminal → next request describes the new directory; repeated requests demote older sets until they fall off the visible chips. The deprecation/priority policy itself lives in the chip system (the consumer), with the Brain supplying generations — when the real face adopts the same decay, it implements this policy.

### What the Phase 1 smoke test established (2026-11-16, Timothy's NixOS box)

Collected from the first real-hardware run, since several findings shape this phase and the packaging:

- The slice runs end to end on a real machine: Brain plus the three-surface face, terminal fully functional. Session recipe that worked (reference implementation: `scripts/launch-session.sh` on the `smoke-test` branch): Sway as the compositor — nested inside the GNOME host session, since Mutter offers no layer-shell — with the prompt toggle bound to Mod4+Space / Mod4+Return / Ctrl+Space, and the Brain started `--headless --orchestrate`.
- **The face UI must be served over HTTP** (the launcher used 127.0.0.1:18701): the SvelteKit build's ES module imports do not load from a file path in WebKitGTK. The shipped systemd unit passes a file path (`--ui …/index.html`) — a packaging defect to fix by serving the build on loopback HTTP or having the host serve it.
- NixOS/WebKit environment requirements for the face: `GIO_EXTRA_MODULES` pointing at glib-networking (TLS inside WebKit), `WEBKIT_DISABLE_DMABUF_RENDERER=1`, and fontconfig available.
- The two interaction findings that motivate this phase (above): the chips strip empty at rest, and the prompt surface acting as a second terminal rather than an independent input.


## Phase 1.7 — Dynamic chips: the panel reacts to typing

Pulled forward from on-device use (2026-11-21, tkserver, during Phase 2 testing): the Phase 1.6 strip worked but was unwieldy in a real home directory — one long row of every entry, dotfiles included, with no way to narrow it short of running a command. Timothy's direction: the panel must carry a show/hide dotfiles switch on itself, close by, the way every good file viewer does — and the panels are dynamic: as he types, the list starts filtering for things that match.

The operative definition is the spec — Architecture 0.1.2, the Spec 0.1.2 note. The summary below is the phase's working statement; where the two differ, the spec governs.

Work:

- Brain: model the shell's input line from the terminal input stream (plain typing, backspace, Ctrl+U and Ctrl+W tracked exactly; unmodelable input — escape sequences, Tab completion, cursor moves — desyncs the model and clears the filter: no filter beats a wrong filter) and publish the current fragment as hub state (`chip_filter`) whenever it changes.
- Face: narrow the strip to the fragment's matches (prefix first, then substring; directories-first preserved inside each class), show the active filter on the strip, and say "no matches" rather than going bare.
- The dotfiles switch: a `.*` chip on the strip itself flips a Brain-held hub flag (`chips_show_hidden`, default off, matching `ls`), so every face flips at once. The listing stays complete — visibility is the chip system's concern.
- Companion fix from the same on-device round (separate PR): the terminal surface moves to xterm's canvas renderer — WebKitGTK leaves replaced DOM cells unpainted as the shell redraws the input line, smearing typed text on screen (`ls` rendered as `llsls`) while the shell receives it exactly. Defined as a face requirement in the Spec 0.1.2 note.

Done when:

- [x] Typing in the terminal narrows the chips to matches without running a command; clearing or submitting the line restores the full set
- [x] Unmodelable input (history recall, Tab completion) clears the filter rather than filtering wrongly
- [x] Dotfiles are hidden by default, and one tap on the panel's own switch shows them — in every face at once
- [x] The gated E2E covers the above (a headless protocol suite and a through-the-face slice test) and stays green in CI

Explicitly not in this phase: fuzzy ranking beyond prefix/substring, learning from history or frequency (heuristicd and priorityd stay parked), filtering against anything but the current directory's listing.

**Status (2026-11-21): built, pending merge + CI.** Brain: `brain::shell::input_line::InputLine` models the line inside `PtyShell::write` — the one funnel every keystroke takes — and publishes through `SectorManager::set_chip_filter`, which bumps versions only when the fragment actually changes. The toggle is one new IPC command (`chips_set_show_hidden`) on the active hub. Face: `AcceleratorChips` derives the visible entries from listing + flag + fragment, and carries the `.*` switch and the filter readout. E2E: chips-lifecycle gains a headless Phase 1.7 suite and slice gains a through-the-face test (a second page watches the strip narrow while the first types) — 12/12 locally across both specs. Still Timothy's, unchanged: the tkserver re-test once merged.

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

**Status (2026-11-17, PR #14): the config foundation is built.** The pre-audit's four traps are addressed in code: the Brain now honours its `[remote]` config (ports, bind address, remote-access toggle, mDNS gate, UDS path — remote off means loopback-only, fail closed); the TLS acceptor is built once instead of per connection, and the self-signed certificate covers the configured bind identity (e.g. the tailnet IP) plus the machine hostname; the face's two plain-`ws://` paths now use `wss://`; and chips show LOCAL / REMOTE / DISCONNECTED (the LOCAL case is asserted in the slice E2E). A new gated spec (`tests/remote.spec.ts`) proves the bind behaviour against a real Brain on a non-loopback address. Still open for this phase: the tkserver end-to-end run (needs the Brain deployed on tkserver with `remote_access = true` and `bind_address` set to its tailnet IP — the same `tos.toml` shape the spec uses) and the three-stream narrowing.

**Status (2026-11-17, PR #15): reconnect behaviour is built.** The face's fixed 2-second retry is now exponential backoff with jitter (1s doubling to a 30s cap, reset on connect), with immediate retries on network-return (`online`) and on the face becoming visible again — the sleep/wake proxies. On a reconnect (not the first connect) the IPC store signals `tos:reconnected` before any replay can arrive, and the terminal clears: the Brain's scrollback replay lands as the whole truth instead of appending to the frozen pre-drop content. A restarted Brain therefore presents a clean terminal (its scrollback starts empty) rather than a stale illusion of the old session; a merely-dropped connection replays the surviving session in full. The gated spec `tests/reconnect.spec.ts` proves it end to end: real Brain killed and restarted against the same state, face never reloaded — chips show DISCONNECTED while down, LOCAL after reattach, the new session works, the old content is gone.

**Status (2026-11-17, PR #16): three-stream narrowing is built.** The Brain's remote handlers (WebSocket and TCP) now enforce an allowlist — the nine prefixes of the three streams (terminal input/resize/signal, prompt submit ×2, state ×2, plus the `tos_ports` probe and `face_register`) — and refuse everything else with a loud `ERROR:` that keeps the caller's `cmd:` id, so a parked caller fails visibly instead of hanging. The unauthenticated OpenSearch HTTP shim on the TCP port is gone (403), WebRTC signalling is refused, and `ai_prediction_received` is no longer broadcast to remote clients. Grey-area commands (`set_mode`, sectors, settings/theme, `session_save`/`load`, portals) are parked over remote by explicit ruling; the local UDS path keeps the full surface untouched. The operative definition lives in the Architecture spec ("The remote surface: the three streams, defined"). The gated spec `tests/narrowing.spec.ts` proves it headlessly: allowed commands answer, terminal input reaches the shell, parked and grey-area commands are refused, and the same parked command still answers over UDS. Still open for this phase: only the tkserver end-to-end run (Timothy's deploy), which is also where the TLS-vs-mesh question gets settled in practice — the narrowing lands with WSS unchanged.

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
