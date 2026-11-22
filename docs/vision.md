# TOS — The Vision

*This page is the anchor for the core vision. It exists so the vision
stays visible and accounted for while the work proceeds in slices.*

*It is not the spec: definitions and behaviour live in the spec suite
(the [spec suite](spec/architecture.md) and its companions), and where this page and the
spec differ on a definition, the spec governs. It is not the roadmap:
sequence and gates live there, and roadmaps are transient by design.
This page changes slowly, and only deliberately.*

**The one rule of this page: nothing leaves it silently.** An element's
status changes only by an edit that names the decision behind the
change. If something is dropped, superseded, or parked, it stays on
this page saying so — that is how we know we have not lost track of it.

Status vocabulary:

- **Live** — built, and part of the current product.
- **In progress** — an active campaign phase is carrying it.
- **Parked** — defined and deliberately waiting; the ledger names what
  revives it. Parking is not deleting.
- **Delegated** — the host desktop's job, by design. A boundary, not a
  gap.
- **Superseded** — replaced by a later decision; the ledger names the
  decision. The old text lives in git history, which is the archive.

---

## The vision

**Terminal first.** TOS's founding philosophy (§1) is that all modes
are overlays on the terminal. The 2026-11-15 pivot made the overlay the
whole product: TOS is an **ambient layer inside any Linux desktop** —
not a desktop environment of its own. The host (GNOME, KDE, Sway,
Hyprland, COSMIC) keeps window management, application launching and
lifecycle, notifications, and the login session. TOS does the one
thing it is for, on top.

**Three surfaces.** The face is thin and has exactly three:

- the **terminal as animated background** — a live terminal session
  rendered as the desktop's background layer, dimmed and ambient at
  rest; the zoomed-out state of the hierarchy, literally;
- a **chips overlay** — glanceable, touch-friendly, never stealing
  focus;
- a **prompt overlay** — the Unified Prompt, summoned on demand,
  zooming or sliding in, taking focus while used, dismissing just as
  fast. The summon is the product's signature interaction: it is how
  the user builds spatial memory of where the terminal lives. The
  prompt is docked to nothing; it acts independently of the terminal
  it summons over.

**One Brain, truth in one place.** A single headless Brain holds the
sessions, sectors, and state. Faces are renderers of Brain-held truth,
which is what lets a face be thin, replaceable, and everywhere. The
Brain may run on the machine in front of you or on a server — **remote
is the default topology, not a special case** — so the workspace stops
belonging to a machine and starts following the user. The protocol
between face and Brain is deliberately narrow: **three streams only**
(terminal, chip state, prompt), carried inside a mesh VPN so identity
and encryption stay boring.

**Chips: a logical, dynamic user experience.** Chips are a
touch-friendly accelerator overlay whose whole goal is to increase
access and reduce typing. They are *derived by rule from state the
Brain already holds* — never scraped from terminal output, which stays
a completely normal terminal's output. The Brain issues chip sets in
**generations** as context changes; the panels react to the user,
including filtering against the word being typed. Tapping a chip does
the typing for you.

**Depth you can move through.** Sectors and hubs organise work
spatially, and the hierarchy (Global Overview → Command Hub →
Application Focus → System Monitor) is a space you zoom through, not
a set of windows you stack. Where an app *runs* (host or server) and
where it *displays* (host screen or XR) are independent axes the Brain
routes; how visible that choice is remains an open preference
experiment. XR is a face the design must never preclude — panels
rendered locally on the headset, content streamed, **content never
the viewpoint** — and not a launch target.

**Anticipation, when the foundation earns it.** Behind the Brain sit
the satellite providers — search, heuristics, priority, a local AI —
whose job is to make the surfaces anticipate: predictive chips,
intuitive fillers, learned patterns. They are parked, deliberately,
until the interaction they would amplify feels right. The module
system and marketplace are how all of this extends without touching
the core.

**The point.** TOS has really big ideas, and the measure of all of
them is one thing: the parts working together through an elegant user
interaction. A solid foundation that works, carrying the vision one
frozen slice at a time — never the other way round.

---

## The ledger

### Core

| Element | What it is | Status | Defined in |
|---|---|---|---|
| Terminal as animated background | The session as the desktop's living background layer | **Live** (Phase 1; layer-shell face; GNOME shim is Phase 4) | Architecture, Direction Update + §7.2; Visual Design §5 |
| Brain holds the truth; faces render | One headless daemon; thin faces per machine | **Live** | Architecture §3 |
| Remote Brain as default topology | Brain on a server (e.g. tkserver); workspace follows the user | **In progress** (Phase 2: config, reconnect, narrowing landed; tkserver end-to-end remains) | Direction Update; §12 reframed |
| Three-stream protocol | Terminal, chip state, prompt — nothing else crosses remotely | **Live** (PR #16) | Architecture, "The remote surface" |
| Mesh-VPN transport | Tailscale/WireGuard carries identity + encryption; SSH fallback is break-glass | **In progress** (settles at the tkserver run) | Scope Split |
| Prompt overlay (Unified Prompt) | One input for command, search, AI; summoned, independent, docked to nothing | **Live** for CMD (Phases 1/1.6) | Architecture §7.1 (as corrected by the 0.1.1 note); Features §1 |
| Chips overlay | Glanceable status + accelerator strip | **Live** | Architecture §7.3–§7.7 |
| Chip lifecycle | Chip sets in generations; older sets deprecate and fall away | **Live** (0.1.1 model; Phase 1.6) | Architecture §7.7 |
| Dynamic chips | Panels filter against in-progress typing; dotfiles switch on the panel | **Live** (Phase 1.7, PR #18) | Architecture, 0.1.2 note |
| Face terminal rendering | Terminal emulation lives in the Brain: the Brain parses the PTY stream into a per-session grid and publishes display state; faces paint cells and send raw input — no emulation, no input encoding, no stream rendering in any face (Timothy's decision, 2026-11-22, after the face-era rendering faults) | **In progress** (Spec 0.1.4; the build is Phase 1.7's closing work) | Architecture §26.3, 0.1.4 note |
| Sectors & hubs | Spatial organisation of work in the Brain's model | **Live** in the model; face expression grows by slices | Architecture §5, §10 |
| Session persistence | A face restart must not blank the terminal; scrollback replays | **Live** (Phase 1.5) | Features §2 |

### What populates chips — the catalogue

Chips derive from Brain-held state only. Each family below is a
*provider* of chip sets; the lifecycle (§7.7) is shared.

| Family | Populates from | Status |
|---|---|---|
| Directory | The Brain's listing of the hub's current directory (breadcrumbs + entries) | **Live** |
| Status / warning | Brain and connection state (LOCAL/REMOTE, warnings) | **Live** |
| Process | Brain session state — PID + process name; a tap dives to Level 4 pre-focused | **Parked** — arrives with its provider |
| Search | Search scopes, filters, quick actions | **Parked** — provider: tos-searchd |
| Predictive / AI | Predictions and "Intuitive Fillers" (§28) | **Parked** — providers: tos-heuristicd, tos-priorityd, AI module |
| Prompt modes SEARCH / AI | The prompt's non-CMD modes | **Parked** — providers: tos-searchd; ZAYA / Cortex |

### Depth and surfaces

| Element | What it is | Status | Defined in |
|---|---|---|---|
| Dual-Sided Chip Layout | The full two-column chip form around the canvas | **Defined; first form live** — the ambient face's strip is its current expression | Architecture §7.4; Visual Design §5 |
| Level 1 — Global Overview | Sector tiles as mini command hubs; the zoomed-out space | **Parked** (view parked; may return as a zoom level) | Architecture §6 |
| Level 3 — Application Focus / Tactical Bezel | Slot-based bezel around a focused app | **Superseded as a TOS-owned level** (Direction Update: apps belong to the host); the bezel slot architecture is **Parked** | Architecture §8 |
| Level 4 — System Monitor | Sector-scoped process monitor; tactical reset | **Parked** — with the process provider | Architecture §9 |
| Expanded Bezel command surface | The bezel the chips and prompt surfaces descend from | **Parked** in full form; its descendants are the live overlays | Features §1 |
| Split viewports & panes | Multiple content panes inside a hub | **Parked** | Architecture §11 |
| Kanban / sector boards | Project context and shared boards in sectors | **Parked** | Architecture §10.1; Features §7 |

### Intelligence and extension

| Element | What it is | Status | Revives with |
|---|---|---|---|
| Search (tos-searchd) | Prompt search beyond the Brain's local reach | **Parked** — first daemon to revive | Prompt search outgrowing the Brain |
| Heuristics (tos-heuristicd) | Learned behaviour and prediction | **Parked** | The summon/motion model settled |
| Priority (tos-priorityd) | The priority stack behind chip ordering | **Parked** | The chip system outgrowing generation order |
| ZAYA local AI module + personas | A local assistant in the Brain, at home on a server | **Parked** | The first slice solid |
| Cortex orchestration | Agent stacking and behaviour modules over the Brain | **Parked** | Architecture §4; Features §4 |
| Modules & marketplace | Sandboxed modules; sector templates; signing | **Parked** | Other people writing modules (Ecosystem spec) |
| TOS Log (tos-loggerd) | Unified structured log | **Parked** — Brain logs to files/journal meanwhile | Architecture §19 |
| Command trust & confirmation | Dangerous-command handling and trust levels | **Parked** — defined, not in the current slice | Architecture §17.2 |
| Collaboration | Host-owned sharing, presence, following | **Parked** | Architecture §13 |
| Onboarding & first run | Guided first experience | **Parked** | Features §3 |
| Input breadth | Voice grammar, controller mapping, accessibility hooks | **Parked** | Architecture §14 |

### Placement and faces

| Element | What it is | Status |
|---|---|---|
| App placement experiment | Run location × display location; how visible the choice is by default | **Open experiment** (Phase 3) — Scope Split's four-cell table |
| XR face (Quest) | Locally rendered panels, streamed content; never the viewpoint | **Slot kept** (platform trait exists); Phase 5 — Architecture §15 |
| Electron face | The May remote-Brain vehicle | **Parked** — lessons carried into the ambient face |
| Android faces (Flutter + handheld) | Thin remote faces for phone / handheld | **Parked** |
| Full workspace memory | Sessions, layouts, histories across devices | **Parked** beyond terminal persistence — Features §2 |

### Delegated — the host's job, by design

Window and app management · application launching and lifecycle ·
login/session (no TOS session, no GDM integration) · system settings
(TOS keeps only its own small preference set) · notifications.

### Superseded — replaced, on the record

| Element | Replaced by | Decision |
|---|---|---|
| Chip population by output scraping / prompt sniffing | Chips derived from Brain-held state (§7.7) | Timothy's ruling, 2026-11-17; spec 0.1.1. Old text: git history, pre-0.1.1 spec |
| LCARS as the default aesthetic identity | The face's own design language (Visual Design §5.5); LCARS kept as a design influence and a possible future theme module | Timothy's ruling, 2026-11-22 |
| TOS as a standalone desktop environment (own session/compositor) | The ambient layer inside a host desktop | The pivot, 2026-11-15; Scope Split |
| `face-wayland-linux` crate; `archive/` prototypes | The layer-shell ambient face; git history | Dropped, Phase 0 |

---

## Keeping this page true

- A status changes only with the decision that changed it — a ruling,
  a merged PR, a Scope Split revision — cited in the edit.
- When a roadmap retires, its open items are harvested **here or into
  the spec** before the roadmap is cleared (the harvest rule).
- At the start of each new campaign, this page is re-read against the
  spec; drift is fixed in the open, in one of the two documents, never
  by quietly editing around it.

*Drafted 2026-11-21 from the spec suite (0.1.2), the Scope Split, and
Timothy's recorded decisions — for Timothy's correction. The vision is
his; this page is bookkeeping.*
