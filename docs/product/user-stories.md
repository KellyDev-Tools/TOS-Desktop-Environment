# Terminal On Steroids — User Story Backlog
**Development Reference**

> **About this backlog.** A *reference* backlog, not a governing
> document: definitions live in the [spec](../spec/architecture.md),
> element status in the [Vision](../vision.md) ledger. Sections
> carry status lines in the ledger's vocabulary (current / parked /
> superseded). §1, the Ambient Face, is the current product; much of
> the rest predates the 2026-11-15 ambient-layer pivot and is
> labelled accordingly. **Before editing this file, read
> [Maintaining this backlog](#maintaining-this-backlog).**

---

## Table of Contents

- [Maintaining this backlog](#maintaining-this-backlog)
1. [The Ambient Face](#1-the-ambient-face)
2. [Navigation & Hierarchy](#2-navigation--hierarchy)
3. [The Prompt Overlay](#3-the-prompt-overlay)
4. [Cortex Orchestration](#4-cortex-orchestration)
5. [Workflow Management & Kanban Boards](#5-workflow-management--kanban-boards)
6. [SEARCH Mode](#6-search-mode)
7. [Trust & Security Model](#7-trust--security-model)
8. [Multi-Sensory Feedback](#8-multi-sensory-feedback)
9. [Onboarding & First-Run Experience](#9-onboarding--first-run-experience)
10. [Marketplace & Module System](#10-marketplace--module-system)
11. [Collaboration](#11-collaboration)
12. [Editor](#12-editor)
13. [Performance & Accessibility](#13-performance--accessibility)
14. [Appendix A — Story ID Index](#appendix-a--story-id-index)

---

## Maintaining this backlog

Directions for keeping these stories true. They exist because
this file once described a product that no longer existed for
months without anyone noticing — the ambient pivot happened in
the spec, the roadmap, and the code, and never reached the
backlog.

**What this file is.** A record of user needs, in the user's
language. It is *not* a specification and *not* a plan: where a
story and the spec disagree, the spec governs and the story gets
fixed; where a section's status and the Vision ledger disagree,
the ledger governs and the story gets fixed.

**Status vocabulary** — the Vision ledger's, used at the top of
every section, and at story level when one story differs from
its section:

- **Current** — describes the product as it is being built now.
- **Parked** — the need stands; its provider is parked. The
  section names what revives it.
- **Superseded** — the need, or its mechanism, was replaced by a
  decision. The line names the date and what replaced it.
- **Rewritten** — the need survived; the mechanism changed. The
  section intro dates the rewrite and says why.

**When to update — four triggers.**

1. *A decision changes user-facing behaviour.* The story update
   rides in the same change as the decision or spec edit — never
   as a later cleanup.
2. *A feature ships.* The handoff names the stories it
   satisfies (see [AI Collaboration](../ai-collaboration.md)).
3. *Hands-on testing reveals a need, or kills one.* Real use
   outranks speculation: AMB-05's dotfiles switch and live
   filtering both came from a device test, not a design session.
4. *A campaign closes.* When a roadmap retires, its harvest pass
   includes this backlog, alongside the Vision ledger.

**Adding a story.**

- IDs are `<EPIC>-<number>`: the next free number under the
  epic's prefix. **IDs are never reused**, even after a story is
  superseded — the index is a history, not a seating chart.
- One need per story, in the table's three columns, in plain
  user language. No implementation nouns the user wouldn't say.
- Add the story to Appendix A in the same change.
- Flagship stories get acceptance criteria. Criteria must be
  *observable and mechanism-honest*: they may not prescribe a
  ruled-out mechanism (no output scraping, no command sniffing,
  nothing docked that the rulings say floats). If the only known
  mechanism is ruled out, the story waits for a provider and
  says so — it does not quietly soften the ruling.

**Changing or retiring a story.** Keep the ID. If the need
survives with a new mechanism, rewrite the row and date the
rewrite in the section intro. If the need is gone, mark it
superseded with the date and its replacement. **Never delete a
story** — deletion is how a backlog starts lying about what the
product once promised.

**Who.** The maintainer approves status changes and new epics.
Contributors and AI agents propose story updates inside the same
PR as the work that triggers them; every handoff answers *which
stories does this satisfy, change, or supersede?*

---

## 1. The Ambient Face

Stories covering the ambient layer itself — the terminal as
background, the chips overlay, the independent prompt, and the
remote Brain. Added 2026-11-21: until now the backlog described
only the pre-pivot standalone environment, and the current
product had no stories at all.

| ID | As a… | I want to… | So that… |
|---|---|---|---|
| AMB-01 | developer | have my terminal session live as my desktop background | it is always present, with no window to manage |
| AMB-02 | developer | have the background terminal behave exactly like my regular terminal — my shell, my config, my programs | TOS never feels like a simulation of my tools |
| AMB-03 | touch user | see shortcuts generated from the directory I am in | I can navigate and act without typing paths |
| AMB-04 | touch user | have a chip tap type for me — directories `cd`, files insert their name mid-command | I stay in flow, and nothing happens behind my back |
| AMB-05 | developer | have the strip narrow as I type, with dotfiles behind a switch on the panel itself | the right chip is one tap away, and the panel follows `ls` conventions |
| AMB-06 | power user | summon a command prompt from anywhere, attached to nothing | I can act without disturbing the session or my desktop |
| AMB-07 | developer | have my session and scrollback survive a face restart or reconnect, exactly as I left them | the face is replaceable; my work is not fragile |
| AMB-08 | developer | run my session on my server and use a thin face on whatever screen I am at — desktop, laptop, phone | my work is where I am, not where the machine is |
| AMB-09 | owner | have a remote face able to drive my terminal but not administer my machine | remoteness does not widen what a connection can do |
| AMB-10 | desktop user | have TOS leave windows, apps, and settings to my desktop environment | it never fights the environment it lives in |
| AMB-11 | developer | choose where a session runs — or never have to choose | placement matches my intent without becoming my job |

*AMB-11 is Phase 3's open question, kept as a story so the
placement experiment has a need to test against.*

### AMB-02 Acceptance Criteria
- The session is the user's real shell in a real PTY — not an
  emulator of one. Interactive programs (editors, `top`, `ssh`)
  run unmodified.
- TOS never alters, filters, or reinterprets terminal output.

### AMB-04 Acceptance Criteria
- A directory chip sends `cd '<name>'` plus Enter through the
  same input path as the user's own keystrokes. There is no
  separate chip API that changes directories.
- A file chip inserts the shell-quoted name plus a trailing
  space, unsent, into the command being typed.
- A tap against a stale chip generation is deprecated, never
  executed blindly — the directory may have changed under it.

### AMB-05 Acceptance Criteria
- Filtering follows the word being typed (after the last space,
  then after the last `/`): prefix matches rank first, then
  substring matches; directories first within each class.
- Dotfiles are hidden until the panel's own `.*` switch is on.
  The switch lives on the panel, not in a settings screen.

### AMB-07 Acceptance Criteria
- The shell process, its cwd, and its jobs live in the Brain and
  survive face restarts untouched.
- On reconnect, the face is replayed the scrollback buffer
  (256 KiB) as the whole truth before live output resumes.

### AMB-09 Acceptance Criteria
- Over a remote connection, only the three streams dispatch —
  terminal, chip state, prompt. Any other command receives a
  loud `ERROR:` refusal naming the narrowing.
- The same commands remain available over a local connection:
  the gate judges the connection, not the command.

---

## 2. Navigation & Hierarchy

Stories covering the five-level zoom model, sector management, and the core navigation contract.

> **Status — superseded in framing.** The five-level zoom model
> predates the ambient face (2026-11-15 pivot). Knowing where you
> are survives as the chips' breadcrumbs; context switching
> belongs to the host desktop and the Brain's sessions. Kept for
> the record, and for the parked bezel design it belonged to.

| ID | As a… | I want to… | So that… |
|---|---|---|---|
| NAV-01 | power user | zoom in from the Global Overview into a specific sector with a single gesture | I can move from bird's-eye oversight to active terminal work without touching a mouse |
| NAV-02 | power user | zoom back out to the Global Overview from any active sector | I can context-switch between projects in one fluid motion |
| NAV-03 | developer | drill into Deep Inspection (Level 4) from an active Command Hub | I can inspect raw buffer contents and process metadata without leaving the session |
| NAV-04 | new user | see a clear visual indicator of my current depth level | I always know where I am in the hierarchy and how to return |
| NAV-05 | operator | create a new sector and have it auto-labeled based on my working directory | I spend less time naming contexts and more time working |
| NAV-06 | operator | freeze, clone, or close a sector from the tile context menu | I can manage session lifecycle without disrupting other running workloads |
| NAV-07 | collaborator | see the navigation depth of remote guests in real time | I understand what each collaborator is currently viewing |

### NAV-01 Acceptance Criteria
- Zoom-in gesture triggers the `ZoomIn` semantic event, not a raw key binding.
- The `recursive-zoom` keyframe animation plays during the transition (< 300 ms).
- Focus lands on the previously active Command Hub of the target sector.

### NAV-05 Acceptance Criteria
- When `cwd` changes to a directory containing a known project file (e.g., `Cargo.toml`, `.git`), the sector label updates within 2 s.
- The user can lock the label via the tile context menu to prevent further auto-rename.
- Locked sectors display a padlock badge on the tile.

---

## 3. The Prompt Overlay

Stories covering the prompt — summoned on demand, independent of
the terminal and of the other surfaces.

> **Status — rewritten 2026-11-21.** This section previously
> described the Command Hub's persistent, docked prompt, and two
> mechanisms the project has since ruled out: detecting the typed
> command to switch modes (HUB-02) and scraping terminal output
> to populate chips (HUB-03). The needs that survived are
> restated below; the rest are accounted for under *Parked and
> superseded*, not deleted. The governing definitions are in the
> Architecture (§7.7) and [Concepts](../concepts.md).

| ID | As a… | I want to… | So that… |
|---|---|---|---|
| HUB-01 | developer | summon the prompt, type a command, and submit it — all without touching the terminal session | I can act from anywhere without disturbing what is running |
| HUB-02 | developer | have the chip strip refresh to the directory I am in after commands like `ls` or `cd` complete | I get file chips that match where I am, without changing my natural shell habits |
| HUB-06 | power user | summon the prompt from anywhere and dismiss it back to nothing | I never lose my command line — and it never occupies my screen uninvited |

*HUB-01 today is the CMD family. The SEARCH and AI prompt
families arrive with their providers — see §6 (parked) and §4
(parked) — and rejoin this section when they do.*

### HUB-02 Acceptance Criteria
- The refresh derives from state the Brain already holds — the
  shell's working directory (OSC 7 / cwd tracking) and its
  listing, refreshed at command completion. **The Brain never
  classifies the typed command** to decide chips are needed, and
  never inspects output.
- Chips for the new directory populate promptly after the
  command completes (target: within 500 ms).
- Tapping a folder chip types `cd` into the terminal; tapping a
  file chip inserts its name into the command being staged
  (see AMB-04).

### HUB-06 Acceptance Criteria
- The prompt is its own overlay surface, summoned through the
  face (see the quickstart's hotkey binding) — it is not docked
  to the terminal, the chips, or any window.
- Summoning and dismissing it never moves focus from, scrolls,
  or otherwise disturbs the terminal session beneath.

### Parked and superseded from the original section

- **HUB-03 — superseded (mechanism).** "Output scraper
  populates process chips" is ruled out: terminal output is
  never inspected. Acting on processes returns only with a
  Brain-side provider; see the Vision ledger's Process chip
  family (parked).
- **HUB-04 — defined, not yet live.** Favourites and pinned
  chips are in the chip catalogue (the dual-sided layout); no
  provider builds them yet.
- **HUB-05 — parked.** Predictive completions wait for the
  heuristic and AI providers.
- **HUB-07 — superseded (mechanism).** The "Focus Error" chip
  required tagging terminal output by priority. Error surfacing
  returns only via a Brain-side provider, never output
  inspection.
- **HUB-08 — parked.** Typo-correction chips wait for the
  heuristic provider.

---

## 4. Cortex Orchestration

Stories covering the Passive Observer, Chat Companion, and the AI safety boundary — the AI never executes commands directly.

> **Status — parked.** The AI module (Cortex / ZAYA) is parked;
> the [Vision](../vision.md) ledger names what revives it. The
> safety boundary these stories assume — the AI stages, the user
> submits — remains a standing ruling.

| ID | As a… | I want to… | So that… |
|---|---|---|---|
| AI-01 | developer | receive a non-blocking correction chip when a command fails | I can recover immediately without manually diagnosing the error |
| AI-02 | developer | ask a question in plain English in AI mode and receive a staged command | I can explore unfamiliar tools without memorizing syntax |
| AI-03 | developer | review and edit the Cortex-staged command before it executes | I stay in full control — nothing runs behind my back |
| AI-04 | operator | install an alternative Assistant from the Marketplace | I can use my preferred LLM provider or a local model |
| AI-05 | operator | toggle individual Agents on or off independently | I can disable the Chat Companion without losing the Passive Observer |
| AI-06 | developer | have the Cortex silently watch for long-running commands and surface an explanation chip | I understand what a hung process is doing without interrupting it |
| AI-07 | team lead | have Agent stack automatically activate based on project context signals | Domain-specific assistance appears without me configuring it manually per session |
| AI-08 | developer | have Cortex chat history restored when I return to a sector | I can resume multi-turn conversations without losing context |

### AI-03 Acceptance Criteria
- Every AI suggestion is placed into the prompt input field — it is never auto-submitted.
- The staged command is fully editable before the user submits.
- The AI explanation is visible in the terminal canvas alongside the staged command.

### AI-07 Acceptance Criteria
- Skills declare `context_signals` in their manifest (e.g., `.git`, `Cargo.toml`).
- The AI Engine evaluates signals against the current `cwd` and activates the matching skill.
- Activation is logged with the sector name and signal matched.

### AI-09 through AI-14 — Editor & Skills

| ID | As a… | I want to… | So that… |
|---|---|---|---|
| AI-09 | developer | have the editor automatically open the failing file when a build error occurs | I can see the error in context without typing a separate command |
| AI-10 | developer | see AI annotation chips in the editor margin when the AI identifies a problem | I know exactly which line to focus on without reading the full AI response |
| AI-11 | developer | describe a code change in plain English and have Vibe Coder propose it as a diff | I can make complex edits without knowing the exact syntax |
| AI-12 | developer | approve or reject each step of a multi-file Vibe Coder edit individually | I stay in control of every change and can stop mid-sequence |
| AI-13 | developer | have a pending Vibe Coder edit sequence survive switching from my phone to my laptop | I can start a workflow on one device and complete it on another |
| AI-14 | developer | see AI-queued requests drain automatically when my backend connection restores | I don't lose context when my connection drops momentarily |

### AI-11 Acceptance Criteria
- Vibe Coder decomposes the natural language intent into a chip sequence with at minimum a read step and a write step.
- The editor switches to Diff Mode before any write is committed.
- No file is modified until the user taps **[Apply]** on the specific diff.

### AI-12 Acceptance Criteria
- Each step in the chip sequence has its own **[Apply]** and **[Skip]** controls.
- Skipping a step does not cancel the remaining steps.
- Applied steps are recorded in the undo stack with an "AI" label.

### AI-13 Acceptance Criteria
- `session_handoff:<sector_id>` generates a token valid for 10 minutes.
- The claiming Face receives `pending_edit_proposal_id` and reconstructs the diff view.
- The handoff token is single-use and invalidated after claim.

---

## 5. Workflow Management & Kanban Boards

Stories covering project-scoped kanban boards, agent personas, multi-agent task execution, and project memory consolidation. Workflows enable teams to decompose complex work into steps, assign AI agents with distinct personalities to execute them, and learn from completed tasks.

> **Status — parked.** Kanban and agent orchestration are
> parked with the AI module they depend on.

| ID | As a… | I want to… | So that… |
|---|---|---|---|
| WF-01 | developer | open a project's kanban board from any sector | I can see all active work for that project regardless of which device I'm on |
| WF-02 | team lead | define custom kanban lanes (Backlog, Planned, WIP, Blocked, Review, Done) | The board reflects our team's actual workflow |
| WF-03 | developer | set auto-promotion rules (e.g., "move from Planned to WIP when capacity available") | Tasks automatically advance without manual intervention |
| WF-04 | operator | create a task from a GitHub issue using the Roadmap Planner skill | I don't manually type task descriptions and acceptance criteria |
| WF-05 | developer | assign an Agent stack (careful-bot, fast-bot, creative-bot) to a task | The agents execute the task using a strategy that matches the work |
| WF-06 | developer | see the agent's decomposition plan before execution | I understand how the agents will approach the task and can request changes |
| WF-07 | developer | enable auto-accept for a task so the agent's commands run without my approval | I can let the agents work autonomously while I focus on other tasks |
| WF-08 | developer | pause an agent mid-task and inspect its reasoning | I can understand why it made a decision and guide it if needed |
| WF-09 | developer | watch multiple agents work on different tasks simultaneously in separate terminal panes | I can monitor parallel progress without context-switching |
| WF-10 | developer | skip a step or retry with a different approach when an agent hits an error | I can manually correct course without aborting the entire task |
| WF-11 | team lead | have an agent stack automatically activate based on project context (e.g., presence of Cargo.toml) | I don't need to manually configure agent strategies per-project |
| WF-12 | developer | create custom Agents by mixing manifests or writing new ones | I can tune behavior to my team's preferences |
| WF-13 | developer | see all LLM reasoning archived for a completed task | I can audit how the agent decomposed and executed the work |
| WF-14 | developer | resume a task with an agent starting from exactly where it paused | I can continue without losing context even after closing TOS |
| WF-15 | team lead | have the kanban board synchronized across multiple sectors working on the same project | All team members see real-time updates when tasks move lanes |
| WF-16 | developer | view a project's accumulated knowledge in a running memory file | I understand patterns, lessons learned, and best practices from past tasks |
| WF-17 | developer | consolidate completed task LLM histories into project memory with `tos dream consolidate` | The project learns from what worked and future agents improve |
| WF-18 | developer | export a task's complete transcript (all steps, LLM reasoning, outputs) as markdown | I can share work with others or create documentation |
| WF-19 | developer | view learned patterns for an agent and see which strategies have worked most often | I can make informed decisions about agent assignment |
| WF-20 | team lead | install alternative agent personas from the Marketplace | I can use community-built strategies without writing my own |
| WF-21 | developer | drag a task to a different lane to change its status | The kanban board reflects real-time workflow progress |
| WF-22 | developer | see a task's acceptance criteria and validation status in the kanban card | I know what success looks like and whether the task is meeting its goals |
| WF-23 | team lead | have older completed tasks auto-archive after N days | The kanban board stays focused on current and recent work |
| WF-24 | developer | filter the kanban board by agent, tag, or lane | I can focus on relevant tasks without visual clutter |
| WF-25 | developer | import a `.tos-task` YAML file to populate the kanban board | I can define tasks in code and version control them |

### WF-01 Acceptance Criteria
- `tos workflow open ~/projects/tos-desktop` loads the kanban board from `~/.tos/kanban.tos-board`.
- The Workflow Manager pane replaces the terminal view (user can switch via tab or split view).
- If multiple sectors open the same project, they all see the same board state.

### WF-02 Acceptance Criteria
- Users customize lanes via Settings → Workflows → Board Lanes.
- Lane order, names, and descriptions are persisted to the project's kanban file.
- New lanes appear immediately in the Workflow Manager pane.

### WF-04 Acceptance Criteria
- Roadmap Planner skill reads GitHub issues from a repository.
- Issues are decomposed into tasks with titles, descriptions, and acceptance criteria.
- User review step: "Create 12 tasks in Backlog?" before confirmation.
- Created tasks appear in the Backlog lane within 2 seconds.

### WF-05 Acceptance Criteria
- Right-click a task card → "Assign agent" displays list of available personas.
- Selecting an agent immediately loads its persona markdown and prepares for task decomposition.
- Agent assignment is persisted to the task in the kanban file.

### WF-06 Acceptance Criteria
- When a task moves to WIP, the agent reads the task and persona, generates a step-by-step plan.
- Plan appears in the Workflow Manager pane: "Step 1: Read error context... Step 2: Search patterns... [Proceed?]"
- User can request changes before execution begins.
- Proceeding advances to step 1 execution.

### WF-08 Acceptance Criteria
- Clicking [⏸ Pause] in an agent's terminal stops execution at the current step.
- An expandable section shows the agent's LLM reasoning for that step (why it chose this step, expected outcome).
- User can [Inspect] command output, [Retry] with different approach, or [Suggest] an alternative.

### WF-09 Acceptance Criteria
- Each active agent is displayed as a tab in the Workflow Manager pane: [@careful-bot #task_001] [@fast-bot #task_002].
- Clicking a tab switches to that agent's isolated terminal output.
- Or, split the pane to see 2-4 agents simultaneously.
- Agents' commands execute in parallel without blocking each other.

### WF-12 Acceptance Criteria
- Users create personas in `~/.local/share/tos/personas/<n>.md`.
- Persona markdown defines strategies (testing, error handling, step sizing, etc.) in plain text.
- Any persona file is immediately discoverable and usable.
- Settings → Workflows → Agent Personas shows built-in, custom, and Marketplace personas.

### WF-13 Acceptance Criteria
- Every task's session file includes a `llm_history` object containing:
  - `initial_decomposition`: {request, response, reasoning}
  - `step_interactions`: [{step_id, executed_at, command_executed, command_output, agent_response}]
- Expanding a step in the Workflow Manager shows this history in an expandable LLM reasoning panel.
- Full history is exported when user exports task transcript.

### WF-14 Acceptance Criteria
- If TOS closes while an agent is on step 2/5, the session file saves: `{ paused_at_step: 2, steps_completed: 1 }`.
- On reopen, the Workflow Manager shows the task in WIP with [Resume] chip.
- Clicking [Resume] loads the full LLM context from `llm_history` and continues from step 3.

### WF-15 Acceptance Criteria
- Sector A (laptop) and Sector B (desktop) both open `~/projects/torpedo/.tos/kanban.tos-board`.
- Task state changes propagate via file system watches (inotify / FSEvents) in < 2 seconds.
- Kanban board refreshes in both sectors, showing task moved to new lane.
- For remote sectors, TOS Remote Server protocol broadcasts updates (see Architecture §12).

### WF-17 Acceptance Criteria
- `tos dream consolidate ~/projects/torpedo` reads all completed task LLM histories.
- Synthesizes patterns: "Problem type: lifetime_error → Solution: wrap_in_result → Test first"
- Generates `~/.tos/memory/project_memory.md` with:
  - Quick patterns (problem + solution)
  - Completed tasks index
  - Cross-task dependencies
  - Emergent recommendations
  - Project statistics (total tasks, duration, agents used, success rate)
- Future agents load project memory as few-shot examples for new decompositions.

### WF-19 Acceptance Criteria
- Settings → Workflows → Agent Personas → [careful-bot] → Learned Patterns shows:
  - Problem type: lifetime_error (count: 5)
  - Successful approaches: [wrap_in_result → test_first → validate (100% success rate)]
  - Historical trends
- Agent uses these patterns in future task decompositions to improve accuracy and speed.

### WF-21 Acceptance Criteria
- Drag-and-drop a task card between lanes.
- Auto-promotion rules evaluate: if target lane is "wip" and current count < max_concurrent, move succeeds.
- If rule prevents move (e.g., WIP is full), card snaps back and a tooltip explains the blocker.

### WF-25 Acceptance Criteria
- Import button in Workflow Manager accepts `.tos-task` YAML files.
- YAML is validated (schema check) before creating tasks.
- Tasks are created in the Backlog lane in the order specified.
- Any validation errors are reported with line numbers.

---

## 6. SEARCH Mode

Stories covering semantic and filesystem search, result chips, and cross-domain filtering.

> **Status — parked.** `tos-searchd` is the first parked daemon
> slated to revive; these stories wait with it, and the SEARCH
> prompt family rejoins §3 when it lands.

| ID | As a… | I want to… | So that… |
|---|---|---|---|
| SRC-01 | developer | switch to SEARCH mode and get instant results across the filesystem | I find files without leaving the terminal or opening a separate app |
| SRC-02 | developer | filter search results by scope (files, processes, history, docs) using chips | I narrow results without typing additional flags |
| SRC-03 | power user | use semantic natural-language queries in SEARCH mode | I can describe what I'm looking for without knowing the exact filename |
| SRC-04 | developer | execute a quick action (open, copy path, delete) directly from a search result chip | I act on results without staging a separate command |

### SRC-01 Acceptance Criteria
- Switching to SEARCH mode via the mode selector triggers the `SearchModeEnter` semantic event.
- First results appear within 300 ms of the first keystroke.
- The terminal canvas displays results; the left chip region populates with filter scopes.

---

## 7. Trust & Security Model

Stories covering the non-blocking warning system, explicit trust promotion, and role-based access in collaboration.

> **Status — mixed.** The trust classes (SEC-01…04) remain
> defined in the spec but are not yet built. The collaborator
> roles (SEC-05, SEC-06) are superseded with §11 — the remote
> boundary that replaced them is AMB-09's three-stream gate.

| ID | As a… | I want to… | So that… |
|---|---|---|---|
| SEC-01 | operator | see a non-blocking warning chip when I stage a command in a WARN trust class | I am informed about privilege escalation without being blocked from proceeding |
| SEC-02 | operator | permanently promote a command class to TRUST by tapping the chip label | I eliminate friction for commands I run routinely |
| SEC-03 | operator | revert a TRUST promotion back to WARN from Settings | I can tighten security after granting trust without reinstalling |
| SEC-04 | new user | configure trust classes during the first-run wizard without any pre-selected defaults | I make an explicit, informed decision for each class |
| SEC-05 | team lead | assign collaborators the Viewer, Commenter, Operator, or Co-owner role | Guests can contribute at the appropriate privilege level |
| SEC-06 | auditor | have all guest actions recorded in the host's TOS Log with guest identity | I can audit who did what and when during a collaboration session |

### SEC-01 Acceptance Criteria
- Warning chip renders above the prompt in the right bezel area — it does not block the input field.
- Pressing Enter while the chip is visible submits the command immediately.
- The chip is dismissed automatically when the command executes.

### SEC-02 Acceptance Criteria
- Tapping `[Trust Class]` on the chip triggers the `trust_promote` IPC message with the command class identifier.
- The Settings Daemon persists the promotion synchronously before the next command can stage.
- Subsequent commands in the same class run with no chip and no friction.

---

## 8. Multi-Sensory Feedback

Stories covering earcons, haptic pulses, alert levels, and the notification center.

> **Status — superseded in framing.** Earcons and alert levels
> were tied to the bezel and zoom-level model. Feedback for the
> three surfaces will be re-storied when the face's motion and
> sound language is designed (see the Visual Design spec).

| ID | As a… | I want to… | So that… |
|---|---|---|---|
| MSF-01 | power user | hear a distinct earcon for each mode switch and level zoom | I receive audio confirmation of my navigation state without looking at the screen |
| MSF-02 | power user | receive a haptic pulse when a command I submitted completes | I know the result without watching the terminal |
| MSF-03 | operator | have the interface shift to Red alert level with a repeating tone when a critical notification arrives | I cannot miss an urgent system event |
| MSF-04 | accessibility-focused user | disable all audio and haptic feedback from a single settings toggle | I can use TOS comfortably in silent environments |
| MSF-05 | operator | dismiss a Priority 3 notification with a single tap; critical notifications require explicit interaction | High-priority alerts cannot be accidentally cleared |

### MSF-03 Acceptance Criteria
- A Priority 5 system event sets the global alert level to Red.
- A repeating earcon plays at the audio engine level — not via the Face — to ensure it sounds even if the UI is partially frozen.
- The notification unfurls in the Right Lateral Bezel with a red border and remains until the user explicitly dismisses it.

---

## 9. Onboarding & First-Run Experience

Stories covering the cinematic intro, guided demo, ambient hints, and the power-user escape hatch.

> **Status — superseded in framing.** First run is no longer
> entering a new environment; it is installing a layer into the
> desktop you already have (see the
> [Quickstart](../quickstart.md)). ONB-03's spirit — never block
> the power user — survives as a standing expectation.

| ID | As a… | I want to… | So that… |
|---|---|---|---|
| ONB-01 | new user | watch a skippable cinematic intro that establishes the TOS aesthetic identity | I understand what kind of system I am entering before I interact with it |
| ONB-02 | new user | step through a guided demo that uses the live system, not a sandbox | I learn by doing in real context rather than a simulated environment |
| ONB-03 | power user | reach a live, unobstructed prompt within 5 seconds of launch | I am never forced through an onboarding flow I have already completed |
| ONB-04 | new user | dismiss individual ambient hints and have that choice persisted | Hints I have absorbed stop appearing without me disabling hints globally |
| ONB-05 | new user | configure trust classes during the wizard with no defaults pre-selected | I understand what I am authorizing before granting any trust |

### ONB-03 Acceptance Criteria
- If `wizard_complete = true` and `first_run_complete = true`, the system boots directly to Level 1 with no intro or guided overlay.
- From a cold start, the time from application launch to an interactive prompt must be ≤ 5 seconds on reference hardware.
- This criterion is enforced as a performance integration test in `tests/headless_brain.rs`.

---

## 10. Marketplace & Module System

Stories covering module discovery, installation, sandboxing, and the permission model.

> **Status — parked.** The marketplace daemon is parked;
> MKT-02's informed-consent pattern remains the model for any
> future permission surface.

| ID | As a… | I want to… | So that… |
|---|---|---|---|
| MKT-01 | developer | browse and install a new Terminal Output Module from the Marketplace | I can replace the default rectangular terminal with an alternative visual style |
| MKT-02 | operator | see the declared permissions before installing any module | I make an informed consent decision before granting system access |
| MKT-03 | developer | sideload a community module by adding its developer public key | I can use trusted unsigned packages without compromising the broader security model |
| MKT-04 | operator | have Standard Tier modules run in a sandboxed environment automatically | Third-party code cannot access system resources it did not declare |
| MKT-05 | developer | install an alternative AI backend that replaces the default LLM provider | I can point TOS at a local model without modifying core system files |

### MKT-02 Acceptance Criteria
- The permission review screen lists every entry in the module's `[permissions]` manifest block.
- The user must scroll to the bottom of the permission list or explicitly tap "I have reviewed permissions" before the Install button becomes active.
- If the manifest declares no permissions, the screen still renders with a "No special permissions required" notice.

---

## 11. Collaboration

Stories covering real-time presence, following mode, role management, and the audit trail.

> **Status — superseded.** The guest-collaboration model was
> replaced on 2026-11-15 by the owner's own remote faces behind
> the three-stream gate — see AMB-08 and AMB-09. TOS is a
> personal system; sharing a session with guests is not in the
> current direction.

| ID | As a… | I want to… | So that… |
|---|---|---|---|
| COL-01 | team lead | invite a collaborator to a sector via a secure one-time token | I can share a live session without exposing persistent credentials |
| COL-02 | viewer | see live avatar presence for each active collaborator on the Global Overview | I know who is working where without asking |
| COL-03 | collaborator | follow another user's viewport so it stays synchronized with theirs | I can observe a colleague's actions in real time during a pairing session |
| COL-04 | team lead | promote a Viewer to Operator and have the change take effect immediately | I can dynamically adjust access without ending the session |
| COL-05 | team lead | have all guest actions tagged with guest identity in the TOS Log | I maintain a full audit trail for compliance purposes |

### COL-01 Acceptance Criteria
- One-time token expires after 30 minutes of inactivity.
- Token generation triggers a `collaboration_session_created` log entry at `LogType::Security`.
- The invited user sees a privacy notice before joining.

---

## 12. Editor

Stories covering the TOS Editor pane, AI context integration, and the Vibe Coder edit flow.

> **Status — parked.** The Editor is a spec surface outside the
> ambient slice. Note before any revival: EDT-01's acceptance
> criteria parse terminal output, which the no-scraping ruling
> now forbids — they would need rewriting first.

| ID | As a… | I want to… | So that… |
|---|---|---|---|
| EDT-01 | developer | see the failing file open automatically in the editor pane when a build fails | I can inspect the error in context without a separate command |
| EDT-02 | developer | preview a file in the editor by typing its path in the prompt | I can read a file before deciding whether to open or edit it |
| EDT-03 | developer | switch the editor to Diff Mode when Vibe Coder proposes a change | I can review exactly what will be modified before approving |
| EDT-04 | developer | approve or reject each file in a multi-file edit sequence individually | I maintain granular control over every change |
| EDT-05 | developer | have my pending Vibe Coder edit sequence persist when I switch devices | I can start a workflow on my phone and finish it on my laptop |
| EDT-06 | mobile user | tap a line number in the editor to send that line to the AI as context | I can get AI help on specific lines without text selection on a small screen |
| EDT-07 | developer | save a file from the editor with a keyboard shortcut | I don't need to navigate to a menu to persist my changes |
| EDT-08 | developer | have LSP diagnostics from my language server appear as annotation chips in the editor margin | I see type errors and warnings in context without switching to a separate tool |

### EDT-01 Acceptance Criteria
- Brain parses PTY output for file path + line number patterns matching `<path>:<line>` or `<path>:<line>:<col>`.
- `editor_open:<path>;<line>` IPC message is sent to the Face within 500ms of error detection.
- The editor scrolls to the error line and renders it with an amber highlight.
- Auto-open can be disabled per sector in **Settings → Editor → Auto-Open Triggers**.

### EDT-03 Acceptance Criteria
- Editor switches to Diff Mode automatically when `editor_edit_proposal` IPC is received.
- Left column shows current file state; right column shows proposed state.
- **[Apply]** commits the write and clears Diff Mode. **[✕]** rejects and returns to previous mode.
- The user can edit the proposed (right) column before applying.

### EDT-05 Acceptance Criteria
- `pending_edit_proposal_id` is written to the session file when a proposal is pending.
- On session handoff claim, the Brain reconstructs the diff view from the referenced AI chat turn.
- The second Face enters Diff Mode with the pending proposal immediately after connecting.

---

## 13. Performance & Accessibility

Stories covering frame-rate targets, headless testing, and keyboard/screen reader accessibility.

> **Status — current.** Applies to the ambient face as written.
> PER-02's VR target waits with the parked XR face.

| ID | As a… | I want to… | So that… |
|---|---|---|---|
| PER-01 | desktop user | maintain 60 FPS during all UI transitions and mode switches | The system feels responsive and premium at all times |
| PER-02 | VR user | maintain 90 FPS in OpenXR environments under normal workloads | I avoid nausea caused by frame drops in immersive mode |
| PER-03 | operator | see a Tactical Alert if frame rate drops below target for more than 2 seconds | I am informed of performance degradation without it blocking my work |
| PER-04 | keyboard user | navigate the sector tile context menu entirely with the keyboard | I can manage sectors without a pointing device |
| PER-05 | screen reader user | have all sector tiles and mode buttons announced correctly by the system screen reader | TOS is operable with assistive technology |

### PER-03 Acceptance Criteria
- Frame rate is sampled every 500 ms by the rendering subsystem.
- If the rolling 2-second average falls below 60 FPS (desktop) or 90 FPS (VR), the Tactical Alert chip appears non-intrusively in the corner of the screen.
- The chip displays current FPS and a one-tap link to the Performance diagnostics panel.
- The alert auto-dismisses once frame rate recovers for 3 consecutive seconds.

---

## Appendix A — Story ID Index

| Story ID | Epic | Summary |
|---|---|---|
| AMB-01 | Ambient Face | Terminal session as desktop background |
| AMB-02 | Ambient Face | A completely normal terminal |
| AMB-03 | Ambient Face | Chips generated from the current directory |
| AMB-04 | Ambient Face | Chip tap types into the terminal |
| AMB-05 | Ambient Face | Live strip filtering + dotfiles switch |
| AMB-06 | Ambient Face | Prompt summoned from anywhere |
| AMB-07 | Ambient Face | Session + scrollback survive reconnect |
| AMB-08 | Ambient Face | Remote Brain, thin face on any screen |
| AMB-09 | Ambient Face | Remote face cannot administer the machine |
| AMB-10 | Ambient Face | Host desktop keeps windows, apps, settings |
| AMB-11 | Ambient Face | Session placement choice (Phase 3 question) |
| NAV-01 | Navigation | Zoom in to a sector from Global Overview |
| NAV-02 | Navigation | Zoom out to Global Overview from any sector |
| NAV-03 | Navigation | Drill into Deep Inspection (Level 4) |
| NAV-04 | Navigation | Visual depth level indicator |
| NAV-05 | Navigation | Auto-label sectors from cwd |
| NAV-06 | Navigation | Sector lifecycle via tile context menu |
| NAV-07 | Navigation | Remote guest depth visibility |
| HUB-01 | Prompt Overlay | Summon prompt and submit a command |
| HUB-02 | Prompt Overlay | Chips refresh from Brain-held cwd after commands |
| HUB-03 | Prompt Overlay | Process chips via output scraper (superseded mechanism) |
| HUB-04 | Prompt Overlay | Favourites and pinned chips (defined, not yet live) |
| HUB-05 | Prompt Overlay | Predictive completions (parked) |
| HUB-06 | Prompt Overlay | Prompt summonable from anywhere, dismissible to nothing |
| HUB-07 | Prompt Overlay | Focus Error chip (superseded mechanism) |
| HUB-08 | Prompt Overlay | Typo correction chips (parked) |
| AI-01 | AI Skills | Non-blocking correction chip on failure |
| AI-02 | AI Skills | Plain-English to staged command |
| AI-03 | AI Skills | Review and edit staged command |
| AI-04 | AI Skills | Install alternative AI backend |
| AI-05 | AI Skills | Toggle skills independently |
| AI-06 | AI Skills | Long-running command explanation chip |
| AI-07 | AI Skills | Context-signal skill activation |
| AI-08 | AI Skills | AI chat history restored on sector restore |
| AI-09 | AI Skills | Editor auto-opens failing file |
| AI-10 | AI Skills | AI annotation chips in editor margin |
| AI-11 | AI Skills | Vibe Coder proposes change as diff |
| AI-12 | AI Skills | Approve multi-file edit steps individually |
| AI-13 | AI Skills | Pending edit persists across device handoff |
| AI-14 | AI Skills | Offline AI queue drains on reconnect |
| WF-01 | Workflow | Open project kanban board from any sector |
| WF-02 | Workflow | Define custom kanban lanes |
| WF-03 | Workflow | Set auto-promotion rules for lanes |
| WF-04 | Workflow | Create task from GitHub issue via Roadmap Skill |
| WF-05 | Workflow | Assign agent persona to task |
| WF-06 | Workflow | View agent decomposition plan before execution |
| WF-07 | Workflow | Enable auto-accept for autonomous agent work |
| WF-08 | Workflow | Pause agent and inspect LLM reasoning |
| WF-09 | Workflow | Watch multiple agents work simultaneously |
| WF-10 | Workflow | Skip step or retry with different approach |
| WF-11 | Workflow | Agent auto-activation by project context signals |
| WF-12 | Workflow | Create custom agent personas |
| WF-13 | Workflow | Archive all LLM reasoning for completed task |
| WF-14 | Workflow | Resume task with agent from exact pause point |
| WF-15 | Workflow | Sync kanban board across multi-sector team |
| WF-16 | Workflow | View project accumulated knowledge in memory file |
| WF-17 | Workflow | Consolidate task histories into project memory |
| WF-18 | Workflow | Export task transcript with full LLM reasoning |
| WF-19 | Workflow | View learned patterns and strategy effectiveness |
| WF-20 | Workflow | Install agent personas from Marketplace |
| WF-21 | Workflow | Drag-drop task between kanban lanes |
| WF-22 | Workflow | View acceptance criteria and validation status |
| WF-23 | Workflow | Auto-archive completed tasks after N days |
| WF-24 | Workflow | Filter kanban board by agent/tag/lane |
| WF-25 | Workflow | Import .tos-task YAML to populate board |
| EDT-01 | Editor | Auto-open editor on build failure |
| EDT-02 | Editor | Preview file by typing path in prompt |
| EDT-03 | Editor | Diff Mode for Vibe Coder proposals |
| EDT-04 | Editor | Individual approval of multi-file edits |
| EDT-05 | Editor | Pending edit persists on device switch |
| EDT-06 | Editor | Tap line number to send line to AI (mobile) |
| EDT-07 | Editor | Save file with keyboard shortcut |
| EDT-08 | Editor | LSP diagnostics as editor annotation chips |
| SRC-01 | Search | Instant filesystem search results |
| SRC-02 | Search | Scope filter chips |
| SRC-03 | Search | Semantic natural-language queries |
| SRC-04 | Search | Quick actions from result chips |
| SEC-01 | Trust | Non-blocking WARN chip |
| SEC-02 | Trust | Promote command class to TRUST |
| SEC-03 | Trust | Revert TRUST to WARN |
| SEC-04 | Trust | First-run trust config with no defaults |
| SEC-05 | Trust | Assign collaborator roles |
| SEC-06 | Trust | Guest action audit log |
| MSF-01 | Multi-Sensory | Earcons on mode/level changes |
| MSF-02 | Multi-Sensory | Haptic on command completion |
| MSF-03 | Multi-Sensory | Red alert on critical notification |
| MSF-04 | Multi-Sensory | Disable all audio/haptic |
| MSF-05 | Multi-Sensory | Priority-gated notification dismissal |
| ONB-01 | Onboarding | Skippable cinematic intro |
| ONB-02 | Onboarding | Guided demo in live system |
| ONB-03 | Onboarding | Live prompt within 5 seconds |
| ONB-04 | Onboarding | Persistent ambient hint dismissal |
| ONB-05 | Onboarding | Trust config in wizard |
| MKT-01 | Marketplace | Install Terminal Output Module |
| MKT-02 | Marketplace | Permission review before install |
| MKT-03 | Marketplace | Sideload via developer public key |
| MKT-04 | Marketplace | Standard Tier sandboxing |
| MKT-05 | Marketplace | Install alternative AI backend |
| MKT-06 | Marketplace | Install AI Skill from Marketplace |
| MKT-07 | Marketplace | Install Language Module from Marketplace |
| COL-01 | Collaboration | One-time token invite |
| COL-02 | Collaboration | Live avatar presence |
| COL-03 | Collaboration | Following mode viewport sync |
| COL-04 | Collaboration | Dynamic role promotion |
| COL-05 | Collaboration | Guest identity in audit log |
| PER-01 | Performance | 60 FPS desktop target |
| PER-02 | Performance | 90 FPS VR target |
| PER-03 | Performance | Tactical Alert on FPS drop |
| PER-04 | Performance | Full keyboard nav of context menu |
| PER-05 | Performance | Screen reader accessibility |
