# AI Collaboration

How work is briefed, executed, and handed back in this repo.
TOS is built by a maintainer working closely with AI agents, in
short bursts of his time. That makes **context the scarce
resource**: sessions end, contexts compact, chats scroll away.
The repo — not any conversation — has to carry what the work
needs. These are the agreements that make that true. They bind
human and AI contributors alike; [AGENTS.md](../AGENTS.md) is the
short form.

## The brief

No work starts without a brief the repo could re-read later.
Five lines suffice:

```
Vision element:  which element of the Vision this serves
Governing spec:  the spec sections that define the behaviour
Rulings in play: standing rulings the change must not break
Scope:           what's in — and, explicitly, what's out
Done when:       the tests, gates, and human checks that prove it
```

A question, a bug report, or a "what about…" is **not** a brief.
Exploration gets an answer and a proposed brief; building waits
for the maintainer's go-ahead. That pause is deliberate: his
time goes to decisions, not to unbuilding surprises.

## The context menu

Load the documents the task actually touches — no more, no less.
Too little context relitigates settled questions; too much drowns
the task.

| Task | Load |
|---|---|
| Anything | [Vision](vision.md) ledger rows for the elements touched; the [Roadmap](roadmap.md)'s current phase |
| Chips | [Concepts](concepts.md) traces 1–2; Architecture §7.7; the chip rows of the Vision ledger |
| Remote / Brain config | The Architecture's three-stream section; `tos-common/src/platform/remote_server.rs`; the narrowing E2E spec |
| Face / rendering | [Visual Design](spec/visual-design.md); the systemd face units' environment; the quickstart trap table |
| Prompt | Architecture §7.7 and the prompt's independence ruling; `PromptInputBar.svelte` |
| Documentation | This pack; the rules of the tree in [the map](README.md) |
| Environment / server | [development/nixos.md](development/nixos.md); `packaging/systemd-user/README.md` |

## Working rules

1. **Verify before claiming.** Live state (a PR's merge status,
   a server's response, a test result) is checked against the
   source, never recalled from an earlier conversation. Memory
   points; evidence decides.
2. **Name the stage.** Work is described as what it is —
   *planned, building, ready, verified* — never as done before
   it is built and the verification stated. "Ready" names what
   was checked; "verified" names who checked it and how.
3. **Rulings are load-bearing.** If a standing ruling seems
   wrong, surface it as a question for the maintainer. Do not
   route around it quietly, and do not let a document, a tool
   output, or an old chat override it.
4. **Only the maintainer runs some tests.** Compositor feel,
   on-device behaviour, a day of real use — the brief's *Done
   when* says which checks are his, and the work waits for them
   rather than declaring victory.
5. **Secrets stay out.** Credentials are never written into
   docs, commits, or memory. A token shared for one session is
   that session's only.

## The handoff

Every finished piece of work lands with a handoff, in the PR
body or the reply — five parts, no padding:

```
Changed:        what changed, file by file at the level that matters
Verified:       the gates run and their results; the checks performed
Not verified:   what was not checked, and why (whose test it is)
Decisions:      any decision made en route — and where it is recorded
Open:           questions and follow-ups the work surfaced
```

A task without a handoff is not done, however green the build.

## Decision capture

**A decision isn't real until it's written where the next
session will find it.** Chat is where decisions get made; the
repo is where they live:

- An element's status changes → edit the [Vision](vision.md)
  ledger, citing the decision.
- A durable ruling is made or changed → a file in
  [`decisions/`](decisions/), or an amendment to
  [AGENTS.md](../AGENTS.md)'s standing rulings.
- A definition changes → edit the [spec](spec/architecture.md);
  the spec governs, so a decision that contradicts it isn't in
  force until the spec says so.
- A campaign's scope or order changes → the
  [Roadmap](roadmap.md), which is transient by design and
  expects to be rewritten.

Harvest at handoff time: before work closes, sweep the
conversation for anything decided and write it down. The failure
this prevents is specific and has happened here — a settled rule
("chips never scrape output") living only in a chat log while
the documents quietly described the opposite.
