# Start here

The route in, sized by the time you have. The
[documentation map](README.md) lists everything; this page is
the order to read it in. TOS is developed by a human working
closely with AI — this route serves both.

## Ten minutes — know what it is

1. [Vision](vision.md) — why TOS exists, and the status of every
   element of it.
2. [Concepts](concepts.md) — the whole system on one page: the
   picture, the ten terms, the five invariants, three end-to-end
   traces.

After this you can say what TOS is, what it deliberately is not,
and what the chips actually do.

## One hour — run it

3. [Quickstart](quickstart.md) — install the user services,
   start `tos.target`, and work the smoke checklist until every
   line passes.

After this you have TOS alive on a desktop and a feel for the
three surfaces.

## Half a day — understand how it works

4. The [Architecture Specification](spec/architecture.md), in
   this order: the front matter and Direction Update (the target
   topology, and what they supersede), the system structure
   sections, §7.7 (chip requests and the chip lifecycle), and the
   three-stream narrowing of the remote surface. The rest of the
   suite — [Features](spec/features.md),
   [Ecosystem](spec/ecosystem.md),
   [Visual Design](spec/visual-design.md) — as the work touches
   them.
5. [State of TOS](state.md) — what actually works today, what
   hasn't been verified yet, and what's knowingly rough. Read it
   before believing any document, including the spec: parts of
   the suite describe parked systems in present tense.
6. The [Scope Split](decisions/2026-11-15-scope-split.md) — why
   whole subsystems are frozen, so you don't "helpfully" revive
   one.

## Your first change

7. [AI Collaboration](ai-collaboration.md) — how work here is
   briefed, executed, and handed back, whether the hands are
   human or AI. The brief template and the handoff format are
   the whole discipline.
8. [Development Standards](development/standards.md) — the
   Makefile gates, testing tiers, and definition of done. The
   short version: `make check`, `make test`, green only.
9. [AGENTS.md](../AGENTS.md) — the standing rulings, in one page.
   If a change would break one, the change is wrong or the
   ruling needs the maintainer's explicit revisit — never a
   quiet route around it.

## When you're lost

- *What's true?* The authority order: Vision, then spec, then
  Roadmap — [the map](README.md) states it.
- *What's the current phase?* The [Roadmap](roadmap.md).
- *Why is this frozen?* The [Vision](vision.md) ledger and the
  Scope Split.
- *It doesn't run.* The quickstart's trap table, then the
  systemd README in `packaging/systemd-user/`.
