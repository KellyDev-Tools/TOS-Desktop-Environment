# TOS documentation

The map. If you read nothing else, read the first three documents
in the authority order.

**New here?** Don't start with the map — start with
[start-here.md](start-here.md), the timed route in: ten minutes
to know what TOS is, an hour to run it, half a day to understand
how it works.

## Authority order

1. [Vision](vision.md) — why TOS exists, and the ledger of every
   element's status. Changes slowly, and only deliberately.
2. [The spec suite](spec/architecture.md) — what TOS is, defined.
   On any question of definition or behaviour, **the spec governs**
   over every other document, this one included.
3. [Roadmap](roadmap.md) — the current campaign: sequence and
   gates only. Transient by design; between campaigns this file is
   an empty placeholder.
4. Decisions, product notes, guides, and standards — each governs
   only its own subject.

## Start here (the newcomer pack)

| Document | Role | Status |
|---|---|---|
| [Start Here](start-here.md) | The timed route into TOS | Living |
| [Concepts](concepts.md) | The whole system on one page: picture, terms, invariants, three traces | Living |
| [Quickstart](quickstart.md) | Run TOS: the user services, a smoke checklist, known traps | Living |
| [State of TOS](state.md) | What works today, what's unverified, what's rough | Snapshot; refreshed per campaign |
| [AI Collaboration](ai-collaboration.md) | How work is briefed, executed, handed back, and decided | Living |

## Direction

| Document | Role | Status |
|---|---|---|
| [Vision](vision.md) | The core vision + element ledger | Living anchor |
| [Roadmap](roadmap.md) | The current campaign | Active; see its own header |
| [Scope Split](decisions/2026-11-15-scope-split.md) | Keep / delegate / park / drop for every component | Frozen decision record, 2026-11-15 |
| [User Stories](product/user-stories.md) | User-story backlog | Reference; non-governing |

## Specification

The governing suite. Spec versions are per-document: each
document's version moves only when that document changes.
[spec/VERSION](spec/VERSION) is the source of truth — it records
the latest overall revision and each document's own version, and
each document's header mirrors its entry. Prior texts live in
git history, not in filenames.

- [Architecture](spec/architecture.md) — the source of truth:
  structure, contracts, hierarchy, the chip system, the remote
  surface.
- [Features](spec/features.md) — detailed feature specifications.
- [Ecosystem](spec/ecosystem.md) — modules, marketplace, service
  orchestration (largely parked; see the Vision ledger).
- [Visual Design](spec/visual-design.md) — the visual and motion
  language of the three surfaces.
- [Developer Reference](spec/developer-reference.md) — project
  structure and development reference for the suite's subject
  matter.
- [User Manual](spec/user-manual.md) — **legacy framing**: it
  describes the pre-pivot standalone-environment experience. Its
  concepts carry over to the ambient layer; its framing is
  superseded (see its own direction banner).

## Guides

Task-oriented how-tos. A guide's banner states whether its subject
is current or parked.

- [Linux Face Integration](guides/linux-face-integration.md)
- [OpenXR Platform](guides/openxr-platform.md) — parked face (XR)
- [Android NDK](guides/android-ndk.md) — parked faces (Android)
- [Packaging and Release](guides/packaging-and-release.md) —
  parked; Beta-0 era

## Development

- [AGENTS.md](../AGENTS.md) — the entry point for AI agents (and
  anyone new): reading order, standing rulings, how work is done.
- [Development Standards](development/standards.md) — Makefile
  gates, testing tiers, coding and commit standards, the Stable
  Common contract.
- [NixOS Development Environment](development/nixos.md) — working
  notes for the maintainer's NixOS development host.

## Archive

[The archive](archive/README.md) holds superseded documents under
their original names — old roadmaps, the 0.1 spec suite, completed
task lists, and one-off plans. Nothing there governs current work.
Git history is the deeper archive behind it.

## Rules of this tree

- **Stable role-based names.** A filename says what a document
  *is*; its location says its lifecycle; its header says its
  version and status. Lowercase kebab-case throughout, except the
  conventional root files (`README.md`, `AGENTS.md`,
  `CHANGELOG.md`).
- **Versions live in headers and git history, not filenames.** A
  new spec version edits the suite in place; git preserves every
  prior text. Spec versions are per-document: `spec/VERSION` is
  the source of truth, listing the latest overall revision and
  each document's own version, and every header mirrors its
  entry there. A document's version moves only when that
  document changes; an unchanged document keeps its version and
  gains no new note.
- **Historical documents are not renamed.** Once a document is
  superseded, it moves to `archive/` under the name it had, so the
  record reads as it was written.
- Component `README.md` files live beside their components and are
  not listed here. The files under `modules/personas/` are runtime
  assets, not documents.
