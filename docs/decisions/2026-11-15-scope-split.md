# TOS scope split: keep / delegate / park / drop

> **Status — frozen decision record (2026-11-15).** This is the
> decision as made; it is not a current-state description, and its
> contemporary lines (such as "nothing here has been implemented")
> are part of the record. What has since been built is tracked in
> the [Vision](../vision.md) ledger and the [Roadmap](../roadmap.md).

TOS stops trying to be a desktop environment. The new direction is an **ambient layer that lives inside any Linux environment**: the terminal session as an animated background, a chips overlay for glanceable status, and a prompt overlay that zooms or slides in on demand and vanishes just as fast — always accessible at a moment's notice, never in the way. The host desktop environment (GNOME, KDE, or another compositor) keeps everything it is already good at: window management, app launching and placement, session and login, notifications, and system settings.

This document sorts every current component of TOS against that direction. It is a decision map to react to, not a rewrite plan. Nothing here has been implemented.

## How to read the verdicts

- **Keep** — part of the ambient-layer product. It carries forward and gets finished.
- **Delegate** — the job still needs doing, but the host desktop environment does it. TOS stops building, maintaining, and fighting this code.
- **Park** — not in the first ambient release, but the idea and the code stay in the repo, untouched, until there is a reason to revive them. Parking is not deleting.
- **Drop** — remove it. Dead weight, a duplicate, or a direction the pivot closes off for good.

The verdicts lean on two pieces of evidence from November 2026: the full commit history (418 commits on `main`, February–June 2026) and a foundation check that built and tested the current tree. The foundation is better than it feels in daily use — the whole Rust workspace compiles clean and almost every test passes — so most verdicts below are about scope, not about code quality.

## The split

### Core

| Component | Verdict | Rationale / what happens to it |
|---|---|---|
| Brain (`brain/`) | **Keep** | The headless daemon is the product's centre. It already runs without a display, holds sessions and state, and can live on a server. It becomes the one long-running process every face talks to. |
| tos-common — IPC and contracts (`ipc.rs`, `brain/ipc_handler.rs`, `cortex_registry.rs`) | **Keep** | The shared contract between Brain and faces. 97 of 97 tos-common tests pass; this is the best-tested code in the repo and the reason a thin face is possible at all. |
| tos-common — sectors and hierarchy (`brain/sector/`, `brain/hierarchy/`) | **Keep** | Sectors are the spatial organising idea behind the background and the zoom model. In the ambient design they organise terminal panels instead of windows. |
| tos-common — shell and PTY (`brain/shell/pty.rs`, `shell.rs`) | **Keep** | The terminal itself. The animated background *is* a rendered terminal session, so this is core, not plumbing. |
| tos-common — platform traits (`platform/traits.rs`, `headless.rs`, `mock.rs`) | **Keep** | The abstraction that lets one Brain serve a desktop face today and an XR face later. Headless and mock implementations also keep tests runnable without a display. |
| tos-common — remote (`platform/remote.rs`, `remote_server.rs`, `remote_session.rs`, `ssh_fallback.rs`) | **Keep** | Remote becomes a headline feature, not a side feature. See the remote section below; the protocol narrows, but the subsystem stays. |
| tos-common — platform, XR slot (`platform/quest.rs`) | **Keep** | Small today, but it reserves the XR face slot in the architecture. Keeping the trait costs almost nothing; retrofitting it later would cost a redesign. |
| tos-common — services registry (`services/registry.rs`, `daemon/`, `modules.rs`, `modules/sandbox.rs`) | **Keep** | The registry and sandboxed module system are how parked daemons and AI modules plug back in later without touching the core. Keep the mechanism even while most of its users are parked. |
| tos-common — UI-facing services (`services/bezel.rs`, `keybindings.rs`, `controller_mapping.rs`, `state.rs`, `config.rs`) | **Keep** | Bezel/chips state, keybindings, and shared state are exactly what the two overlays render. Controller mapping stays with the XR slot. |
| tos-common — host-facing services (`services/portal.rs`, `session.rs`, `capture.rs`, `audio.rs`, `accessibility.rs`, `haptic.rs`) | **Park** | These reach into host-desktop territory (portals, session, capture). The ambient face needs a small subset; the rest waits until a concrete feature asks for it. |
| tos-common — feature services (`services/ai/`, `search.rs`, `settings.rs`, `priority.rs`, `logger.rs`, `marketplace.rs`, `timeline.rs`, `lsp.rs`, `trust.rs`, `ssh.rs`, `heuristic.rs`) | **Park** | Each one backs a daemon or feature that is parked below. The shared types stay so the contracts do not rot; the behaviour revives with its daemon. |
| tos-common — collaboration (`collaboration.rs`) | **Park** | Interesting, tested, and unrelated to whether the ambient layer works. It waits. |

### Satellite daemons

All seven daemons are thin — most are a single `main.rs` — and every one of their test suites passes. None of them is broken. The question is only whether the ambient layer needs them on day one. It does not: two delegate to the host and the other five park.

| Component | Verdict | Rationale / what happens to it |
|---|---|---|
| tos-searchd | **Park** | Search feeds the prompt overlay, so this is the first daemon to revive — but the first slice can search locally inside the Brain. Revive when prompt search outgrows that. |
| tos-settingsd | **Delegate** | System settings belong to the host DE. TOS keeps only its own small preference set (including the placement preference below), stored as plain config, not a settings daemon. |
| tos-sessiond | **Delegate** | Login, logout, screen lock, and session lifecycle are the host's job once TOS is no longer a session. Terminal session persistence stays in the Brain, where it already lives. |
| tos-priorityd | **Park** | The priority stack is a distinctive TOS idea and has a Svelte view (`PriorityStack`). It is an overlay feature for later, not part of the terminal + chips + prompt slice. |
| tos-marketplaced | **Park** | A module marketplace is an ecosystem play. It matters only after other people write modules, which is well past the first slice. |
| tos-loggerd | **Park** | Structured logging is useful while debugging the seams, but it does not need its own daemon. The Brain can log to files/journal in the meantime. |
| tos-heuristicd | **Park** | Heuristics and learned behaviour are polish on top of an interaction that has to feel right first. Revive after the motion and summon model are settled. |

### Faces

Five faces was the clearest symptom of the old scope. The ambient direction needs exactly one face to launch, one shell pattern to reuse, and one slot kept open for XR.

| Component | Verdict | Rationale / what happens to it |
|---|---|---|
| Svelte face (`face-svelte-ui/`) | **Keep** | The source of the ambient face. CommandHub becomes the prompt overlay; `ExpandedBezel` and `WarningChip` become the chips overlay; `XtermTerminal` becomes the background layer. It has 19 type errors in `svelte-check`, concentrated in CommandHub, SettingsModal, and the main page — fixing those is part of finishing this face, not a reason to replace it. |
| Electron face (`face-electron-any/`) | **Park** | Electron was the vehicle for the remote-Brain work in May and still holds useful connection and window-state code. As a shell it duplicates what a native layer-shell client does with far less weight, so it parks — code kept, lessons (brain discovery, remote connection handling) carried into the ambient face. |
| Wayland face (`face-wayland-linux/`) | **Drop** | As a standalone face and would-be compositor client, this is the DE layer the pivot delegates away. It is also the only red in the whole test picture: its tests no longer compile because `SurfaceConfig` in tos-common gained a `depth` field and the test code was never updated. What replaces it is a much smaller layer-shell client hosting the Svelte-derived UI (below), not this crate. Salvage any useful protocol notes into docs, then remove the crate from the workspace. |
| Android Flutter face (`face-android-flutter/`) | **Park** | A phone/tablet face may make sense again as a thin remote face, and its Rust bridge stays compilable in the workspace. It is not part of the desktop ambient launch. |
| Android handheld face (`face-android-handheld/`) | **Park** | The handheld/controller direction pairs naturally with XR later. Parked with it, for the same reason: interesting, not first. |

### Session, packaging, AI, and the rest

| Component | Verdict | Rationale / what happens to it |
|---|---|---|
| GDM / display-manager session (`packaging/tos-session`, `packaging/tos.desktop`, `packaging/sway.config`, `make restart-gdm` flow) | **Delegate** | There is no TOS session to log into any more. The user logs into GNOME or KDE as normal, and the ambient layer starts as an ordinary user service or autostart entry inside that session. |
| Packaging — distro packages (`packaging/arch/`, `deb/`, `rpm/`, `homebrew/`) | **Park** | Packaging four distros is release work for a product that has not frozen its first slice. Keep the files; revive one target (the one you actually run) when the ambient face is worth installing. |
| tos-signer (`scripts/tos-signer/`, PKCS#11 / HSM signing) | **Park** | Carefully built in April (Stage 6.8) and passing its tests, but signing infrastructure serves a release pipeline and a marketplace — both parked. It waits with them. |
| AI assistant module — ZAYA1-8B (`modules/zaya/module.toml`, the final commit on `main`) | **Park** | A local assistant belongs in the Brain's future, especially on a server, but the last commit adding a *new* module while the seams were still buggy is the old pattern in miniature. The module system is kept; this module revives after the first slice is solid. |
| Personas (`modules/personas/` — careful, creative, fast bots) | **Park** | They ride with the AI module. No work needed; they are data, not code. |
| Editor and AI views in the Svelte face (`EditorPane`, `AiContextPanel`, `AiChat`, `ActiveThoughts`) | **Park** | A built-in editor and chat views widen the face back into an IDE. The prompt overlay can *invoke* AI later; it does not need to *contain* it for the first slice. |
| Remaining Svelte views (`GlobalOverview`, `SplitLayout`, `Marketplace`, `WorkflowManager`, `ApplicationFocus`, logs and inspector views) | **Park** | Overview and split views overlap the recursive-zoom idea and may return as zoom levels. Marketplace and workflow views ride with their parked daemons. Application focus is window management — that part delegates to the host in spirit, even though the view code itself simply parks. |
| Shell integration scripts (`scripts/tos_*_config.sh`, `tos_ps_config.ps1`) | **Keep** | Small, finished, and directly useful: they make ordinary shells TOS-aware, which is the ambient layer's home turf. |
| Tests — Rust integration crate (`tests/`) | **Keep** | Session persistence, service orchestration, sandbox isolation, transports, and the rest all pass. This suite is the contract that lets faces and daemons be parked and revived safely. |
| Tests — Playwright E2E (`face-svelte-ui/tests/`, Stage 6.9) | **Keep** | The suite itself is kept, but be clear about its state: its last recorded run failed all 10 tests because Chromium could not launch in the NixOS environment (missing `libglib-2.0.so.0`), not because of app logic. Fixing the environment so this suite can go green is part of the first slice, below. |
| `archive/` (588 of roughly 1,000 files at HEAD) | **Drop** | The February prototype generations (`tos-dream`, alpha-0/1/2, traditional) served their purpose during consolidation in March. Keeping them in the working repo doubles its size and search noise. Export a tagged snapshot or a separate archive repo if you want the history, then remove the folder. |
| Stale branches (`brain-test-situation`, `fix-regression`) | **Drop** | `fix-regression` has zero unique commits and is simply 49 behind `main`. `brain-test-situation` has 2 unique commits from March and sits about 158 behind. Salvage the brain tests if they are not already in `tests/`, then delete both branches. |

## How the ambient vision maps onto what you kept

The new face is three surfaces, and almost all of the UI already exists in the Svelte face. What is new is the shell they live in and the motion between states.

**The terminal as animated background.** A live terminal session rendered behind the desktop windows — dimmed and ambient at rest, so the desktop appears to float on top of your shell. This is `XtermTerminal` plus the sector model, presented as a background layer instead of a window. The recursive zoom hierarchy finally has a literal reading: the background is the zoomed-out state.

**The chips overlay.** Glanceable status — Brain connection, running sectors, warnings, priority items — shown as small chips that do not steal focus. This is the bezel system (`ExpandedBezel`, `WarningChip`, the module views such as `BrainStatus` and `MiniLog`) reduced to a thin strip or corner cluster.

**The prompt overlay.** CommandHub, summoned by hotkey. It zooms in from the background or slides in from an edge, takes focus while you use it, and dismisses back into the background. The summon gesture is the product's signature interaction, so the animation is not decoration: it is how the user builds the spatial memory of where the terminal lives.

**Where the surfaces live.** On Wayland, the layer-shell protocol exists precisely for backgrounds, panels, and overlays, and it works on KDE, Sway, Hyprland, COSMIC, and similar compositors. GNOME's Mutter does not offer layer-shell to regular clients, so GNOME support needs a separate small shim — a Shell extension, or a desktop-window approximation. That shim is the one deliberate platform exception in the design, and it is far smaller than the GDM session and compositor work it replaces.

## Remote gets stronger, not weaker

The Brain was always headless and network-facing, so the ambient pivot splits the system along a seam that already exists:

- **The Brain can live on the server** — tkserver — where persistent sessions, heavier compute, and eventually the local AI module belong.
- **The ambient layer is thin and local.** Each machine you sit at runs only the three surfaces above and connects to the same Brain. Your workspace stops belonging to a machine and starts following you.

That only works if the remote protocol stays narrow. The ambient design remotes exactly three things: terminal streams, chip state, and prompt commands. The May remote work felt buggy because it was remoting a whole desktop session's worth of interaction, with certificate bypasses and content-security workarounds to match. A three-stream protocol can be made boring and reliable.

Identity and encryption should be boring too: run the Brain connection inside a mesh VPN such as Tailscale or WireGuard and let the network decide which devices may connect, instead of maintaining public TLS certificates for a personal tool. The SSH fallback already in tos-common stays as the break-glass path.

## Where apps run is separate from where they show

Launching an app from the prompt raises two independent questions, and the design should keep them separate:

- **Run location** — host or server.
- **Display location** — host screen or XR headset.

That gives four combinations, all of which the kept architecture can express:

| Runs on | Shows on | How it works |
|---|---|---|
| Host | Host screen | The normal case. TOS asks the host DE to launch the app and steps aside. Best for browsers, local GUI tools, and anything GPU- or latency-sensitive on the machine in front of you. |
| Server | Host screen | Terminals already work this way — the PTY lives on the server and renders anywhere. For GUI apps, waypipe can proxy an individual Wayland app from the server so it appears as an ordinary window in the host DE, without streaming a whole desktop. Best for apps that live next to your data and persistent sessions. |
| Server | XR headset | Panels placed in space around you. The headset renders the panel frames locally; the content streams from the server. This is the distinctive XR case. |
| Host | XR headset | The local machine feeds panels to the headset when the content is local or the server is too far away. |

**Placement is a preference, not a per-launch chore.** Whether placement should be an invisible default or an explicit choice each time is still open — you wanted to play with both, and it reads as a preference setting rather than a fixed design law. A sensible starting shape: sensible defaults per app type (terminals and AI on the server, heavy local apps on the host), a small glyph on each chip showing where that item is running, and an override in the prompt for the times you care. The setting to experiment with is how visible that choice is by default.

## XR is a face the design must not preclude, not a launch target

The code already reserves this slot: tos-common's platform layer has `quest.rs` alongside `remote.rs` and `headless.rs`, the old `vr` profile was renamed to `spatial`, and there is an OpenXR platform guide in the docs. The split keeps that reservation and adds one hard design rule:

**Stream content, never the viewpoint.** Anything that moves with the user's head must be rendered locally on the headset. Panels are locally rendered surfaces with streamed content textures — which is exactly the sector/panel model the kept core already uses. Text and terminals are the forgiving case; fast-moving 3D is not, and TOS does not need it.

On a flat screen, zooming into the terminal background is an animation. In passthrough XR, the ambient layer sits at the edge of the room, chips at the edge of vision, and zooming is pulling a panel closer. The perspective that was hard to land on a monitor is native in a headset — which is a reason to protect the slot, and also a reason not to rush it: Quest development (Android toolchain, Meta SDK) is its own project. XR timing is an open decision, below.

## What this cut removes, and the first slice to freeze

The surface reduction is large even though most code is parked rather than deleted:

- One face to finish instead of five, and it reuses existing Svelte components.
- No session, compositor, or display-manager surface at all — that whole class of bugs delegates to the host.
- Seven daemons become zero required daemons at launch; the Brain alone must run.
- The working repo roughly halves when `archive/` leaves.

What remains is still a real product: Brain + tos-common + one ambient face, with remote as a designed-in property rather than an add-on.

**The recommended first vertical slice** — the smallest route that should be frozen end-to-end before anything is unparked:

1. Brain running locally, serving one persistent terminal session.
2. One ambient face (layer-shell first) showing that session as the background layer.
3. Chips overlay showing Brain connection and session state.
4. Prompt overlay (CommandHub) summoning and dismissing with the zoom/slide motion, able to run a command in that session.
5. The Svelte face's 19 type errors fixed in the components this slice uses, and the Playwright E2E environment fixed so the suite can actually run green against this slice.

No remote, no second daemon, no new features until that route is boringly reliable. Remote is slice two — the same face against a Brain on tkserver — because the protocol narrowing makes it a configuration of the same system, not a new one.

## Decisions left open for you

These were aired in discussion but deliberately not settled. The split above does not depend on their answers, except where noted.

1. **GNOME shim approach** — Shell extension versus desktop-window approximation for the ambient surfaces on GNOME. This decides how "any Linux environment" is honoured on the most common DE.
2. **Placement preference default** — invisible defaults versus explicit run/display choice, and how prominent the override and the running-location glyph should be. To be settled by playing with it, as you said.
3. **Daemon revival order** — searchd is the assumed first revival, for prompt search. The order after that (priority, logger, heuristics, marketplace) is open.
4. **XR timing** — when the Quest/spatial face moves from reserved slot to active work. The architecture protects it either way; the schedule is yours.
