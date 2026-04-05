# TOS Feature Specifications

**Purpose:** This document consolidates the detailed specifications for major TOS features that extend the core architecture. Each section is self-contained and cross-references the [Architecture Specification](./TOS_beta-0_Architecture.md) and [Ecosystem Specification](./TOS_beta-0_Ecosystem.md) as needed.

---

## Table of Contents

1. [Expanded Bezel Command Surface](#1-expanded-bezel-command-surface)
2. [Session Persistence & Workspace Memory](#2-session-persistence--workspace-memory)
3. [Onboarding & First-Run Experience](#3-onboarding--first-run-experience)
4. [Ambient AI & Skills System](#4-ambient-ai--skills-system)
5. [Marketplace Discovery & Browse Experience](#5-marketplace-discovery--browse-experience)
6. [TOS Editor](#6-tos-editor)

---

## 1. Expanded Bezel Command Surface

*Extends Architecture Specification §7.1 & §8.1*

### 1.1 Philosophy

The Persistent Unified Prompt exists at every level. It is always visible, always reachable. But in its collapsed state it is just an input — it does not surface output, context chips, or navigation. The Expanded Bezel Command Surface unlocks the full power of the prompt without requiring the user to navigate anywhere.

This is not a new level. It is a persistent overlay state that sits above whatever level the user is currently on. It can be invoked at Level 1 while surveying sectors, at Level 3 while a graphical app is in focus, or anywhere in between. The user's current view zooms back spatially to make room. Output appears. Action chips offer what to do next. The user decides whether to dive in or dismiss and continue.

### 1.2 Triggers

| Trigger | Description |
|:---|:---|
| **Tap bottom bezel** | Tap anywhere on the collapsed bottom bezel bar |
| **Swipe up from bottom edge** | Upward swipe gesture from the bottom bezel edge |
| **Split button in Top Bezel** | A dedicated expand button in the Top Bezel Center section |

All three triggers are equivalent and produce the same expansion animation and state.

### 1.3 Expansion Animation

When triggered, the current view undergoes a **spatial zoom-out**: the content scales down slightly along the z-axis, as if the user has stepped back from the screen. This uses the same depth language as level transitions.

The bottom bezel animates upward, revealing:
- The full **Persistent Unified Prompt** with all active bezel overlays visible
- The **left and right chip columns** populated with context from the current sector
- All active **ambient hint chips**, **AI skill chips**, and **warning chips**

The expanded surface occupies the lower portion of the viewport. The zoomed-out current view remains visible behind it — dimmed slightly but not occluded.

```
┌─────────────────────────────────────────────────────┐
│  TOP BEZEL                              [⊞ Split]   │
├─────────────────────────────────────────────────────┤
│                                                     │
│   [  Current view — zoomed back, dimmed  ]          │
│   [  Level 1 / 2 / 3 content visible     ]         │
│   [  Swipe ← → to move between L3 apps  ]          │
│                                                     │
├──────────────┬──────────────────────┬───────────────┤
│  LEFT CHIPS  │  PROMPT              │  RIGHT CHIPS  │
│  (context)   │  > _                 │  (AI / warn)  │
└──────────────┴──────────────────────┴───────────────┘
│  BOTTOM BEZEL (expanded)                            │
└─────────────────────────────────────────────────────┘
```

### 1.4 Level 3 App Navigation

While the Expanded Bezel Command Surface is open, the zoomed-out content layer becomes a **lateral swipe surface** for Level 3 applications. The user can swipe left/right (or press `←`/`→`) to cycle through open Level 3 applications without closing the expanded bezel.

The lateral navigation does not change the active sector. The same app that was in focus when the bezel was opened remains the active Level 3 context for the prompt's shell.

### 1.5 Shell Context

#### 1.5.1 Active PTY Available

If the current sector's active Command Hub PTY is idle, the submitted command is routed to that PTY. This is the default case.

#### 1.5.2 Active PTY Busy

If the active PTY is currently running a command, the prompt area displays three options as chips:

```
┌──────────────────────────────────────────────────────────────┐
│  [⏹ Stop (Ctrl+C)]   [⧉ New Terminal]   [⏳ Wait...]        │
└──────────────────────────────────────────────────────────────┘
```

- **[⏹ Stop (Ctrl+C)]** — sends `SIGINT` to the running process, freeing the PTY.
- **[⧉ New Terminal]** — spawns a fresh ephemeral shell pane in the current sector. Commands run in this pane are associated with the sector but do not interfere with the running process. The new pane closes automatically when the bezel is dismissed unless promoted (see §1.8).
- **[⏳ Wait...]** — dismisses the chip overlay and returns to a waiting state.

The Stop button is always visible regardless of which chip is selected.

### 1.6 Output Display

When a command completes, its output is rendered by the active Terminal Output Module in an overlay panel that expands upward from the prompt. Maximum height: 40% of the viewport. Long outputs are scrollable within the panel.

#### 1.6.1 Action Chips

| Chip | Action |
|:---|:---|
| **[→ Command Hub]** | Zooms into the sector's Level 2 Command Hub with the output visible. The bezel collapses. |
| **[⊞ Split View]** | Opens a split layout — output and terminal on one side, current Level 3 app on the other. |
| **[✕ Dismiss]** | Collapses the bezel. Output is saved to the sector's terminal history. |
| **[⧉ Keep Open]** | Pins the output panel open and returns focus to the background view. |

Action chips appear after output completes, not during streaming.

#### 1.6.2 Error Output

If the command exits with a non-zero code, the output panel border renders in amber. The AI Passive Observer chip (if installed and active) may surface a correction or explanation chip alongside the standard action chips.

### 1.7 Dismiss Behaviour

Configurable in **Settings → Interface → Expanded Bezel**:

| Setting | Behaviour |
|:---|:---|
| **Stay open** | The bezel remains expanded after output. |
| **Auto-collapse on complete** | The bezel collapses when a command completes with exit code 0. Errors keep it open. |
| **Auto-collapse after timeout** | The bezel collapses N seconds after output completes if no further input is detected (default: 5s). |

Manual collapse: tap the Top Bezel split button again, swipe down on the expanded bezel, or press `Esc`.

### 1.8 Ephemeral Pane Promotion

When a **[⧉ New Terminal]** pane is created from the busy-PTY state, it is ephemeral by default. If the user runs commands they want to keep, a **[⊞ Promote to Split]** chip appears after the first command completes. Tapping it converts the ephemeral pane into a persistent split pane in the sector's hub layout.

### 1.9 Architectural Position

The Expanded Bezel Command Surface is **not a level**. It does not change `active_level` in the session state. The Brain tracks a single boolean flag: `bezel_expanded`. This flag is not persisted to the session file — the bezel always opens collapsed on launch.

### 1.10 IPC Contracts

| Message | Effect |
|:---|:---|
| `bezel_expand` | Opens the Expanded Bezel Command Surface |
| `bezel_collapse` | Collapses the surface back to standard bezel |
| `bezel_output_action:<action>` | Triggers a post-output action chip (`hub`, `split`, `dismiss`, `keep`) |
| `bezel_pane_promote` | Promotes the ephemeral pane to a persistent sector split |
| `bezel_swipe:<direction>` | Navigates between Level 3 apps (`left` / `right`) |

---

## 2. Session Persistence & Workspace Memory

*Supplements Architecture Specification*

### 2.1 Philosophy

TOS should feel like it never forgot you left. When a user returns after closing the system, their sectors are where they left them, their terminals have their history, and their workspace is ready.

Two concepts drive this spec:

- **Live State** — continuously auto-saved snapshot of the current TOS environment. Written silently on every meaningful state change. Restored silently on every launch.
- **Named Sessions** — portable, exportable sector-scoped snapshots the user explicitly saves and switches between.

Both sit on top of the existing Settings Daemon, which continues to own user preferences. Session state is deliberately kept separate from preference state — the two never mix.

### 2.2 What Persists

#### 2.2.1 Per-Sector Persistent State

| State Field | Description |
|---|---|
| `name` | Sector display name |
| `position` | Index in Level 1 grid |
| `cwd` | Last working directory |
| `shell` | Active shell module ID |
| `terminal_module` | Active terminal output module ID |
| `environment` | Sector-level env vars |
| `hub_layout` | Multi-terminal layout within the sector (see §2.4) |
| `terminal_histories` | Scrollback buffers per terminal pane |
| `ai_chat_history` | AI chat conversation for this sector |
| `pinned_chips` | Pinned left/right chip entries |
| `sector_type` | Sector type module ID if non-default |
| `active_mode` | Last active hub mode (`CMD` / `SEARCH` / `AI`) |
| `frozen` | Whether the sector was frozen at close |

#### 2.2.2 Global Persistent State

| State Field | Description |
|---|---|
| `bezel_slots` | Component assignments for all Top / Left / Right slots |
| `active_sector_index` | Which sector was focused at close |
| `active_level` | Which hierarchy level was active at close |

#### 2.2.3 Separation from Settings Daemon

| Owned by Settings Daemon | Owned by Session Files |
|---|---|
| Theme module selection | Sector layout and positions |
| AI backend default | Terminal scrollback histories |
| AI behavior configs | AI chat histories per sector |
| Keybind mappings | Hub layouts (splits / tabs / panes) |
| Audio / haptic preferences | Working directories per pane |
| Accessibility settings | Pinned chips |
| Sandboxing tier rules | Active mode per sector |
| Module installation state | Bezel slot assignments |
| Agent persona library | **Active workflow state (project-level)** |
| | **Kanban board assignments (project-level)** |
| | **Agent terminal pane mappings** |
| | **LLM interaction history per task** |

**New in this version:** Session files now support project-level workflow context. Multiple sectors can reference the same project's kanban board and running agents.

### 2.3 Session Files

#### 2.3.1 Format & Location

```
~/.local/share/tos/sessions/
├── _live.tos-session          # Auto-saved live state (always present)
├── rust-project.tos-session   # Named session
├── client-work.tos-session    # Named session
└── experiments.tos-session    # Named session
```

#### 2.3.2 Schema

```json
{
  "tos_session_version": "1.1",
  "name": "rust-project",
  "created_at": "2025-03-01T09:14:00Z",
  "saved_at": "2025-03-04T17:42:11Z",
  "global": {
    "active_sector_index": 1,
    "active_level": 2,
    "bezel_slots": {
      "top_center": ["clock", "cpu_usage", "memory_usage"],
      "left": ["minimap"],
      "right": ["priority_stream"]
    }
  },
  "sectors": [
    {
      "id": "sector_a1b2",
      "name": "dev",
      "position": 0,
      "cwd": "/home/user/projects/torpedo",
      "shell": "fish",
      "terminal_module": "cinematic-triangular",
      "active_mode": "CMD",
      "frozen": false,
      "environment": { "RUST_LOG": "info" },
      "hub_layout": {
        "type": "splits",
        "panes": [
          { "id": "pane_1", "weight": 0.6, "cwd": "/home/user/projects/torpedo" },
          { "id": "pane_2", "weight": 0.4, "cwd": "/home/user/projects/torpedo/src" }
        ]
      },
      "terminal_histories": {
        "pane_1": ["cargo build", "cargo test", "cargo run"],
        "pane_2": ["ls", "vim main.rs"]
      },
      "active_view": "pane_1",
      "ai_chat_history": [
        { "role": "user", "content": "explain this error", "timestamp": "2025-03-04T17:40:00Z" },
        { "role": "assistant", "content": "The error is a borrow checker violation...", "timestamp": "2025-03-04T17:40:02Z" }
      ],
      "pinned_chips": {
        "left": ["~/projects", "/etc"],
        "right": ["cargo build", "cargo test"]
      },
      "project_context": {
        "project_path": "/home/user/projects/torpedo",
        "project_id": "torpedo-v1",
        "shared_kanban_board": true
      },
      "active_workflow": {
        "kanban_board_ref": "project:.tos/kanban.tos-board",
        "tasks_visible": ["task_001", "task_002", "task_003"],
        "agents_active": [
          {
            "agent_id": "careful_bot",
            "task_id": "task_001",
            "terminal_pane": "pane_workflow_1",
            "state": "running"
          }
        ]
      }
    }
  ]
}
```

#### 2.3.3 Auto-Save Triggers

The live state file (`_live.tos-session`) is written automatically on every significant state change, debounced at 2 seconds.

| Trigger | What Gets Written |
|---|---|
| Command submitted to PTY | `terminal_histories` for affected pane |
| `cd` or directory change | `cwd` for affected pane |
| Sector created, closed, or renamed | Full sector list |
| Sector frozen / unfrozen | `frozen` flag |
| Bezel slot reconfigured | `global.bezel_slots` |
| AI chat message sent or received | `ai_chat_history` for affected sector |
| Mode switch | `active_mode` for affected sector |
| Hub layout changed | `hub_layout` for affected sector |
| TOS graceful shutdown | Full synchronous write of all state |

**Crash Recovery:** `tos-sessiond` writes to a temp file (`_live.tos-session.tmp`) and atomically renames it on success. A corrupted or incomplete temp file is discarded on next startup; the previous good state is used instead.

### 2.4 Multi-Terminal Hub Layout

Each sector can contain multiple terminal instances. Their arrangement is defined by the `hub_layout` object.

#### 2.4.1 Layout Types

| Layout Type | Description |
|---|---|
| `splits` | Tiled panes with resizable dividers (default). |
| `tabs` | Stacked terminals, one visible at a time. Fallback for modules without `multi_terminal = true`. |
| `module_defined` | Layout fully controlled by the Terminal Output Module. |

#### 2.4.2 Splits Layout

A split layout defines a tree of panes, each with a weight (proportional size), its own `cwd`, its own shell instance, and its own terminal history. For the full splits interaction model, see Architecture §11.

### 2.5 Named Sessions

#### 2.5.1 Saving a Named Session

Named sessions are sector-scoped. To save: secondary select on a sector tile at Level 1 → **Save Session As...** → enter a name. Alternatively, tap the sector name chip in the Top Bezel Left section to open the **Sector Session Popover**:

```
┌─────────────────────────────┐
│  dev  ●  LIVE               │
├─────────────────────────────┤
│  ✦ rust-project      Mar 01 │
│    client-work       Feb 27 │
│    experiments       Feb 20 │
├─────────────────────────────┤
│  [Save Current]  [Export]   │
└─────────────────────────────┘
```

**[Export]** copies the `.tos-session` file to a user-selected path for portability.

#### 2.5.2 Loading a Named Session

Selecting a named session replaces the current sector's state with the saved one. The shell is re-spawned fresh in the restored `cwd` — running processes are not restored. The sector tile animates a brief reload pulse.

If a named session references a shell or terminal module that is no longer installed, TOS substitutes the system default and renders a yellow alert chip noting the substitution.

#### 2.5.3 Importing a Session

A `.tos-session` file can be imported by dropping it onto a sector tile at Level 1, or via **Settings → Sessions → Import**. `tos-sessiond` validates the file format and version before the Brain applies it.

### 2.6 Session Service (`tos-sessiond`)

A dedicated auxiliary daemon handles all session file I/O. It registers with the Brain's service registry on startup (ephemeral port).

**Responsibilities:**
- Maintains `_live.tos-session` with 2s debounced auto-save via atomic temp-file rename.
- Serves named session CRUD to the Brain via IPC.
- Validates session file schema on load and import.
- Detects and discards incomplete temp files on startup for crash recovery.

#### 2.6.1 IPC Contracts

| Message | Effect |
|---|---|
| `session_save:<sector_id>:<n>` | Saves current sector state as a named session |
| `session_load:<sector_id>:<n>` | Loads a named session into the specified sector |
| `session_delete:<sector_id>:<n>` | Deletes a named session |
| `session_list:<sector_id>` | Returns list of named sessions for a sector |
| `session_export:<sector_id>:<n>:<path>` | Exports a session file to the given path |
| `session_import:<path>` | Imports a `.tos-session` file |
| `session_live_write` | Forces an immediate synchronous live state write (used on graceful shutdown) |

#### 2.6.2 Startup & Restore Sequence

1. Brain starts and signals `tos-sessiond` to load `_live.tos-session`.
2. `tos-sessiond` reads and validates the file. If valid, returns the full state object to the Brain.
3. Brain reconstructs all sectors, hub layouts, and bezel slots.
4. Each sector's shell is re-spawned in its restored `cwd`. Terminal histories are loaded into each pane's output buffer before the shell prompt appears.
5. The Face receives the fully reconstructed state via the standard WebSocket state sync.
6. If `_live.tos-session` is missing or corrupt, the Brain starts with a single default sector and empty state.

**Silent by Design:** There is no restore notification, animation, or prompt. The system is simply there, as the user left it.

### 2.7 Terminal History Persistence

Terminal scrollback buffers are serialized as ordered arrays of strings per pane. On restore they are loaded into the terminal output module's buffer before the shell spawns, so history is visible immediately.

The existing `terminal_buffer_limit` setting (default 500 lines) governs both the live buffer and what is written to disk.

### 2.8 AI Chat History Persistence

Each sector's AI chat history is persisted as an ordered array of message objects. On restore, the active Chat Companion skill module receives the history via its `on_session_restore` callback.

Chat history is capped at 200 messages per sector in the session file.

### 2.9 Editor Pane Persistence

Editor pane state is persisted as part of the sector session. The following fields are added to the sector session schema:

```json
{
  "editor_panes": [
    {
      "pane_id": "pane_2",
      "file": "/8TB/tos/src/brain/main.rs",
      "scroll_line": 138,
      "cursor_line": 142,
      "mode": "viewer",
      "context_scope": "visible_range",
      "pending_edit_proposal_id": null,
      "unsaved_buffer": null
    }
  ]
}
```

`unsaved_buffer` holds the full unsaved content if the user has made edits but not saved. This ensures edits survive session switches and device handoffs even before an explicit save.

`pending_edit_proposal_id` references an AI chat turn ID. On session restore, the editor reconstructs the Diff Mode view from the chat history if a Vibe Coder proposal was pending approval.

### 2.9.1 Workflow & Agent State Persistence

In addition to editor panes, the session file preserves active workflow state:

```json
{
  "active_view": "pane_workflow_1",
  "active_workflow": {
    "kanban_board_path": "/home/user/projects/torpedo/.tos/kanban.tos-board",
    "tasks": {
      "task_001": {
        "id": "task_001",
        "title": "Fix borrow checker in session.rs",
        "assigned_agent": "careful_bot",
        "lane": "wip",
        "auto_accept": true,
        "state": {
          "steps_completed": 2,
          "steps_total": 5,
          "current_step": 3,
          "paused": true,
          "last_output": "..."
        },
        "llm_history": {
          "initial_decomposition": {
            "timestamp": "2025-04-04T10:22:00Z",
            "request": { ... },
            "response": { ... }
          },
          "step_interactions": [ ... ]
        }
      }
    },
    "agent_patterns": {
      "careful_bot": {
        "problem_type_lifetime_error": {
          "count": 5,
          "successful_approaches": [ ... ]
        }
      }
    }
  }
}
```

This structure enables:
- **Resumption:** Agents resume from exact step with full LLM context
- **Learning:** Patterns accumulate in `agent_patterns` per agent per sector
- **Transparency:** Complete LLM interaction archive preserved for audit/review

### 2.10 Cross-Device Session Handoff

The session system supports explicit device-to-device handoff — transferring active context from one Face to another without requiring both devices to share a file.

**Generating a handoff token:**

Any Face can request a handoff token for a sector:
```
session_handoff:<sector_id>  →  returns one-time token (expires 10 minutes)
```

**Claiming a handoff on a second Face:**

A connecting Face presents the token during the `face_register` handshake:
```json
{ "type": "face_register", "profile": "desktop", "handoff_token": "abc123xyz" }
```

The Brain responds with the full sector context: terminal history, AI chat, editor pane state (including pending proposals), cwd, active skill modules, and pinned chips. The second Face opens exactly where the first left off.

**Use case:** Approve a Vibe Coder step on your phone → generate a handoff → open your laptop → claim the handoff → continue approving remaining steps with a full keyboard and larger screen.

Handoff tokens are single-use and are invalidated after claim or expiry. They do not persist to disk.

---

## 3. Onboarding & First-Run Experience

*Supplements Architecture Specification*

### 3.1 Philosophy

TOS is a dense, powerful system. Its depth is a feature, not a bug — but that depth must not be a barrier. Three principles govern all onboarding design:

- **SKIP** — Respect the skip. Every onboarding element must be skippable. No forced flows, no unskippable animations beyond 2 seconds.
- **DO** — Teach through doing. Users learn TOS by using TOS, not by reading about it. Guided steps happen inside the live system, not in a sandbox.
- **FADE** — Fade gracefully. Onboarding hints become less visible as user confidence grows.

**Acceptance criterion for power users:** A user must be able to reach a live, unobstructed prompt within 5 seconds of launch. This is the single testable bar for the onboarding implementation.

### 3.2 Onboarding State Model

The Brain tracks a persistent onboarding state object in the Settings Daemon under `tos.onboarding`:

```toml
[onboarding]
first_run_complete   = false
wizard_complete      = false
hints_dismissed      = []
hint_suppressed      = false
sessions_count       = 0
commands_run         = 0
levels_visited       = []
```

The Brain evaluates this state on startup and passes relevant flags to the Face via the standard IPC state sync. The Face is responsible for rendering all onboarding UI elements; the Brain manages state persistence only.

### 3.3 First-Run Flow

| Stage | Name | Trigger | Duration | Skip? |
|:---|:---|:---|:---|:---|
| **S1** | Cinematic Intro | `first_run_complete = false` | ~12 seconds | YES — any key / tap |
| **S2** | Guided Demo | `wizard_complete = false` | ~4 minutes | YES — skip button always visible |
| **S3** | Ambient Hints | Ongoing (fadeable) | Indefinite | YES — per-hint or master off |

#### 3.3.1 Stage 1: Cinematic Intro

A short, skippable cinematic sequence that plays before the system is interactive. This is the user's first impression of TOS's aesthetic identity.

**Sequence:**
- **Frame 0–2s:** Black screen. The TOS wordmark fades in with amber glow. Subtle startup earcon plays.
- **Frame 2–5s:** LCARS grid lines sweep in from edges, forming the bezel skeleton.
- **Frame 5–9s:** Sector tiles fade in at Level 1, one by one. The Brain console output area activates with scrolling boot log text (**real Brain init output streamed live**). The system is visibly waking up.
- **Frame 9–12s:** Zoom transition inward to Level 2. The Command Hub assembles. Prompt cursor blinks. Text fades in: `SYSTEM READY.`
- **Frame 12s:** Cinematic ends. If `wizard_complete` is false, Stage 2 begins automatically.

**Skip Behavior:** Any keypress, mouse click, or touch immediately cuts to the end state (Level 2 Command Hub, live). `first_run_complete` is set to true. The skip is non-destructive: the Brain has already been initializing during the cinematic.

#### 3.3.2 Stage 2: Guided Demo Workflow

The Guided Demo is an interactive walkthrough that runs inside the live system. All commands run during the demo are real.

A non-blocking overlay panel appears in the bottom-left of the viewport above the bezel:
- **Style:** Glassmorphism card with LCARS amber accent border.
- **Controls:** `[NEXT →]`, `[SKIP TOUR]`, `[← BACK]`
- **"Show me" button:** Where applicable, auto-executes the step action.

**Guided Demo Steps:**

| Step | Instruction Shown | "Show Me" Action | What It Teaches |
|:---|:---|:---|:---|
| **1** | This is your Command Hub. The terminal is always here — it never goes away. | Highlights terminal output area | Core terminal-first identity |
| **2** | Type a command, any command. Try: `ls` | Auto-types `ls` in the prompt | Basic prompt interaction |
| **3** | Notice the chips that appeared? Click one to append it to your command. | Highlights nearest file chip | Directory context chips |
| **4** | Hold `Ctrl+Tab` to see all your Sectors at Level 1. | Triggers `Ctrl+Tab` zoom out | Level 1 navigation |
| **5** | Click your sector to zoom back in. | Highlights default sector tile | Level zoom mechanics |
| **6** | Press `Ctrl+M` to bring up the Minimap. | Fires `Ctrl+M` shortcut | Minimap / bezel slot |
| **7** | Type a question in plain English. Try: `show me running processes` | Auto-types query, switches to AI mode | AI Augmentation mode |
| **8** | You're ready. Explore freely — press `[?]` any time for help. | Dismisses overlay, pulses `?` badge | Completion + help shortcut |

The demo never blocks the user. At any step they can ignore the overlay and just use the system. The overlay tracks completion by detecting the relevant system event (e.g., a successful `ls` completing Step 2), not by enforcing sequence.

**Trust Setup:** During Stage 2 onboarding (inserted before Step 1), the user is presented with the trust configuration screen (Architecture §17.2.1) for each command class. This is the only time TOS actively prompts the user to think about trust posture.

On Step 8 or on `[SKIP TOUR]`, `wizard_complete` is set to true.

#### 3.3.3 Stage 3: Ambient Hints

After the guided demo, TOS continues to teach through non-blocking contextual hints — small tooltip-style overlays that appear when a user encounters a feature for the first time, and fade permanently once dismissed.

**Hint Anatomy:** Target element + brief label (max 12 words) + optional action link. Appear with 300ms fade-in. Never occlude the prompt or terminal output.

**Dismissal:** Clicking `[x]` adds the hint ID to `hints_dismissed` permanently. Performing the hinted action independently auto-dismisses the hint.

**Hint Decay:**

| Threshold | Opacity | Pulse |
|:---|:---|:---|
| Sessions 1–3 / Commands 0–50 | 100% | Amber pulse border active |
| Sessions 4–7 / Commands 51–200 | 70% | No pulse |
| Sessions 8–14 / Commands 200–499 | 40% | Whisper — barely visible |
| Sessions 15+ / Commands 500+ | Auto-suppressed | Re-enable manually in Settings |

**Hint Suppression:** A master toggle in **Settings → Interface → Onboarding** sets `hint_suppressed = true`, immediately hiding all active hints and preventing future ones.

**Initial Hint Registry:**

| Element | Tooltip Text | Condition |
|:---|:---|:---|
| Bezel Lateral Slots | These slots are configurable — dock any component here | First time Level 2 loads |
| `[AI]` Mode Button | Ask anything in plain English — commands are staged, never auto-run | First time CMD mode used without AI |
| Right-Click on Chip | Right-click any chip for deep options: inspect, signal, renice | First chip rendered |
| `Ctrl+Tab` | See all your Sectors from Level 1 | After 3 commands run |
| `Ctrl+Alt+Backspace` | Emergency recovery: Tactical Reset (God Mode) | After first error exit code |
| Status Badge (top-right) | Generate a secure link to share this session remotely | After session 2 |
| Earcons | Audio cues are configurable in Settings → Interface | First mode switch |
| `[?]` Help Badge | Replay the guided tour or browse the full manual | End of tour / any time |

### 3.4 Re-Access & Persistent Help

#### 3.4.1 The [?] Help Badge

A persistent `[?]` badge lives in the Top Bezel Right section. Always visible. Clicking it opens a Help Modal with three options:
- **Replay Tour** — restarts the Guided Demo overlay from Step 1 without resetting `wizard_complete` or any system state.
- **Open Manual** — opens the TOS User Manual in an Application Focus window (Level 3).
- **Reset Hints** — clears `hints_dismissed`, re-enabling all ambient hints.

#### 3.4.2 IPC Contracts

| Message | Effect |
|---|---|
| `onboarding_skip_cinematic` | Immediately ends Stage 1 |
| `onboarding_skip_tour` | Sets `wizard_complete = true`, dismisses overlay |
| `onboarding_advance_step` | Advances guided demo to next step |
| `onboarding_hint_dismiss:<hint_id>` | Permanently dismisses a specific hint |
| `onboarding_hints_suppress` | Sets `hint_suppressed = true` |
| `onboarding_replay_tour` | Re-opens guided demo overlay from Step 1 |
| `onboarding_reset_hints` | Clears `hints_dismissed` array |

### 3.5 Integration Points

- **Settings Daemon:** New `tos.onboarding` namespace. No schema conflicts with existing settings.
- **Brain IPC Handler:** New `onboarding_*` message prefix handled by a dedicated `OnboardingService` module.
- **Face (Web UI):** New `<OnboardingOverlay>` component rendered at the root level, z-indexed above content but below modals.
- **Earcon Service:** Two new earcons — `onboarding_start` (cinematic begin) and `onboarding_complete` (tour end).
- **Top Bezel Right Section:** `[?]` badge added as a permanent non-configurable slot element alongside the existing status badge.

**Dependency:** The cinematic intro requires the Brain to be fully initialized before Frame 5 begins. The init sequence should target completion within 4 seconds. If Brain init exceeds 4s, the cinematic holds on Frame 2–5 until ready.

---

## 4. Ambient AI & Skill System

*Supplements Architecture Specification & Ecosystem Specification*

### 4.1 Philosophy

AI in TOS is not a room you walk into. It is a layer that runs underneath everything, watching, learning context, and surfacing help exactly when it is useful — then getting out of the way.

Three principles govern all AI design in TOS:

- **STAGE, NEVER RUN.** The AI never executes a command without the user submitting it from the prompt. Every suggestion, correction, and prediction ends up staged — visible, editable, and under user control.
- **PLUGGABLE BY DEFAULT.** The LLM powering the system and the behaviors it exhibits are independent, swappable modules. The user is never locked into one model or one interaction style.
- **REMOVABLE.** All AI behavior wrappers, including the defaults, can be uninstalled or disabled. A user who wants zero AI involvement gets zero AI involvement.

### 4.2 Architecture Overview

The AI system is split into two independent module layers that compose at runtime:

```
┌─────────────────────────────────────────────────┐
│              AI Behavior Layer                  │
│  ┌───────────┐ ┌───────────┐ ┌───────────────┐  │
│  │ Passive   │ │   Chat    │ │   Predictor   │  │
│  │ Observer  │ │ Companion │ │   (Ghost)     │  │
│  │ [default] │ │ [Ollama▾] │ │ [default]    │  │
│  └───────────┘ └───────────┘ └───────────────┘  │
│         ↕ AI Engine API (JSON-RPC / IPC)        │
├─────────────────────────────────────────────────┤
│         AI Backend Layer (cascading)            │
│   System Default ──► Behavior Override          │
│  ┌──────────┐ ┌──────────┐ ┌──────────────┐    │
│  │  Ollama  │ │ OpenAI   │ │  Anthropic   │    │
│  │ (local)  │ │ (remote) │ │   (remote)   │    │
│  └──────────┘ └──────────┘ └──────────────┘    │
└─────────────────────────────────────────────────┘
```

- **AI Backend modules** (`.tos-ai`) define the LLM connection. One is the **system default**, selected in **Settings → AI → Backend**. Any skill module can override this and target a specific installed backend.
- **AI Behavior modules** (`.tos-skill`) define how the AI acts, when it speaks, and what UI it renders. Multiple skill modules can run simultaneously, each independently toggled in **Settings → AI → Skills**.

**Backend Resolution Order:**
1. **Behavior-level override** — if the skill module has a specific backend set in its config, use that.
2. **System default** — if no override is set, use the system default backend.

The Brain's `AIService` brokers all communication between skill modules and the active backend. Behavior modules never call the backend directly — they submit requests to the `AIService` via IPC.

### 4.3 AI Skill Modules (`.tos-skill`)

Behavior modules define a specific AI interaction pattern and own a specific region of the UI surface.

#### 4.3.1 Manifest Structure

```toml
name        = "Passive Observer"
version     = "1.0.0"
type        = "aibehavior"
description = "Watches terminal output and surfaces contextual chips silently"
author      = "TOS Core"
icon        = "observer.svg"

[behavior]
trigger     = "passive"           # passive | prompt_input | mode_switch | manual
ui_surface  = "chips"             # chips | ghost_text | chat_panel | thought_bubble
chip_color  = "secondary"         # primary | secondary | accent
runs_always = true

[capabilities_required]
function_calling = false
streaming        = true
vision           = false

[permissions]
terminal_read  = true
prompt_read    = true
prompt_write   = false
network        = false

[context_required]
# Only declared fields are sent, minimizing token usage
last_command = true
exit_code    = true
terminal_buffer_tail = true
```

#### 4.3.2 UI Surfaces

| Surface | Description | Default Color |
|:---|:---|:---|
| `chips` | AI-suggested chips in the left/right chip layout. Visually distinct via `secondary` color scheme. | Teal / cyan accent |
| `ghost_text` | Inline ghost text rendered in the prompt, dimmed, accepted with `Tab` or `→`. | 40% opacity prompt color |
| `chat_panel` | Replaces or augments the `[AI]` mode panel with a full chat interface. | Standard panel |
| `thought_bubble` | Floating dismissable card that appears above the prompt. Tapping expands into chat. | Glassmorphism dark card |

**Visual Identity:** AI chips always render in the secondary color (teal/cyan by default, themeable) with a subtle `✦` prefix glyph. This makes them immediately distinguishable from system-generated chips without any labeling required.

#### 4.3.3 The Behavior API

Behavior modules communicate with the Brain's `AIService` via IPC using the existing JSON-RPC format:

**Submitting a request:**
```json
{
  "behavior_id": "passive-observer",
  "trigger": "terminal_output",
  "context": {
    "last_command": "git psh origin main",
    "exit_code": 127,
    "cwd": "/home/user/project",
    "terminal_buffer_tail": ["git psh origin main", "git: 'psh' is not a git command"]
  },
  "request": "suggest correction",
  "stream": false
}
```

**Receiving a response (staged, never auto-run):**
```json
{
  "behavior_id": "passive-observer",
  "surface": "chips",
  "chips": [
    { "label": "git push origin main", "action": "stage_command", "color": "secondary" },
    { "label": "Explain error", "action": "open_chat", "color": "secondary" }
  ]
}
```

The Brain receives this response and instructs the Face to render the chips. The skill module never touches the Face directly.

### 4.4 Default Behavior Modules (Shipped with TOS)

#### 4.4.1 Passive Observer (`tos-observer`)

Watches the terminal output buffer and prompt passively. Surfaces contextual AI chips when it detects actionable moments. Never interrupts. Never opens a panel unprompted.

**Trigger conditions:**

| Condition | What It Surfaces |
|:---|:---|
| Exit code `127` (command not found) | Correction chip: closest matching command staged |
| Non-zero exit code with stderr output | "Explain error" chip + suggested fix chip |
| Partial command typed, 1.5s idle, unsubmitted | Ghost text completion or chip suggestion |
| Long-running command exceeds 30s | "Explain what this is doing" chip + "Cancel" chip |
| `cd` into directory with no prior visits | "What's in here?" chip (summarizes directory structure) |

**Settings:** `Settings → AI → Skills → Passive Observer` — toggle on/off, trigger sensitivity (Low/Medium/High), chip position.

#### 4.4.2 Chat Companion (`tos-chat`)

Provides the full chat interface within `[AI]` mode. When the user switches to `[AI]` mode, the terminal output area is replaced by a scrollable chat panel. Conversation history is maintained per-sector per-session.

**Chat panel features:**
- Full streaming responses with cursor animation.
- Code blocks in responses include a `[Stage →]` button that appends the code/command to the prompt.
- Context includes: current directory, last 20 terminal lines, active sector name, current shell.
- `[Clear]` button resets conversation history for the current sector.
- Switching away from `[AI]` mode preserves conversation history.

The chat panel is a skill module. It can be replaced by a marketplace alternative (DevOps chat companion, Git expert, documentation assistant) without changing any Brain or Face code.

### 4.5 Marketplace Behavior Module Archetypes

#### 4.5.1 Command Predictor

- **Surface:** `ghost_text`
- **Trigger:** `prompt_input` — fires on every keystroke with debounce
- **Behavior:** Renders dimmed ghost text inline in the prompt. `Tab` or `→` accepts. `Esc` dismisses.
- **Latency requirement:** must respond within 300ms or ghost text is suppressed for that keystroke cycle. Modules should declare `latency_profile = "local"` and warn users if connecting to a slow remote backend.

```
user types:  git che
ghost shows: git checkout feature/my-branch█
```

#### 4.5.2 Workflow Agent

- **Surface:** `thought_bubble` + `chat_panel`
- **Trigger:** `manual` — activated explicitly by the user
- **Behavior:** Plans and stages multi-step command sequences. Each step is presented as an ordered chip list with confirmation before any step is staged.
- **Safety contract:** Workflow agents MUST use the Brain's tactile confirmation API for any command that modifies the filesystem, network, or process state. The Brain enforces this — any `stage_command` call from a workflow agent for a flagged command class is automatically wrapped in confirmation.

```
User: "set up a new rust project called torpedo in ~/projects"

Agent proposes:
  Step 1 of 4: cd ~/projects          [Stage]
  Step 2 of 4: cargo new torpedo      [Stage]
  Step 3 of 4: cd torpedo             [Stage]
  Step 4 of 4: git init               [Stage]
```

#### 4.5.3 Domain Expert

- **Surface:** `chips` or `thought_bubble`
- **Trigger:** `passive` — activates based on detected context (e.g., presence of a `Dockerfile`, `.git`, `Cargo.toml`)
- **Behavior:** A specialist module with a narrowly scoped system prompt tuned to a domain. Examples: Git Expert, Docker Expert, Kubernetes Navigator, SQL Analyst.

```toml
[behavior]
trigger         = "passive"
context_signals = [".git", "Makefile", "Cargo.toml"]
ui_surface      = "chips"
```

#### 4.5.4 Thought Bubble Companion

- **Surface:** `thought_bubble`
- **Trigger:** `passive` — always watching
- **Behavior:** A floating, dismissable card that appears in the corner of the terminal when the AI has something to say. More conversational than the Passive Observer — can initiate, ask clarifying questions, and be expanded into a full chat.
- **Appearance:** Glassmorphism dark card with a cloud/bubble shape indicator. A small pulse animation indicates new content. `[×]` dismisses for the current session. Long-press `[×]` dismisses permanently until re-enabled.

### 4.6 AI Mode ([AI]) — Extended

The `[AI]` mode is preserved as a first-class Command Hub mode. Its behavior is now driven by whichever Chat Companion skill module is active. The mode itself is a surface contract, not an implementation.

- Switching to `[AI]` mode invokes the active Chat Companion module's `on_mode_enter` callback.
- If no Chat Companion module is installed, `[AI]` mode falls back to a minimal built-in text interface with a notice to install a Chat Companion from the Marketplace.
- `[AI]` mode remains one of three Command Hub modes: `[CMD]`, `[SEARCH]`, `[AI]`.

### 4.7 Context Passed to All Skill Modules

The `AIService` maintains a rolling context object automatically included with every skill module request. When an Editor pane is open in the sector, the Editor Context Object (Features §6.5.1) is also included.

```json
{
  "cwd": "/home/user/project",
  "sector_name": "dev-work",
  "shell": "fish",
  "terminal_buffer_tail": ["...last 20 lines..."],
  "last_command": "cargo build",
  "last_exit_code": 0,
  "active_mode": "CMD",
  "session_commands_run": 47,
  "os": "Linux",
  "env_hints": ["RUST_LOG=info", "NODE_ENV=development"],
  "editor_context": {
    "file": "/8TB/tos/src/brain/main.rs",
    "language": "rust",
    "visible_range": { "start_line": 138, "end_line": 185 },
    "cursor_line": 142,
    "diagnostics": [{ "line": 142, "severity": "error", "message": "cannot borrow..." }]
  }
}
```

Modules declare which context fields they consume in their manifest under `[context_required]`. The `AIService` only sends declared fields, minimizing token usage.

### 4.8 Vibe Coder Skill (`tos-vibe-coder`)

The Vibe Coder is the third built-in skill. It is disabled by default and activated by the user in **Settings → AI → Skills**.

- **Surface:** `chip_sequence` — an ordered list of actionable chips in the Right Bezel, one per planned step
- **Trigger:** `manual` — activated explicitly by the user typing a natural language intent in the prompt
- **Behavior:** Decomposes natural language intent into a reviewable multi-step plan. Each step is a file edit, command, or search operation. Steps are presented as chips in sequence — the user approves each before it executes.

```
User: "add error handling to the session loader in brain/session.rs"

Vibe Coder proposes:
  Step 1 of 3: Read brain/session.rs          [View] [✓]
  Step 2 of 3: Edit load_session() — add      [View] [✓]
               Result return type
  Step 3 of 3: Run cargo check                [Stage] [✓]
```

Multi-step edits spanning multiple files are presented as a full chip sequence. Each step can be approved individually. **Pending steps are persisted in the session file** — the user can approve Step 1 on their phone and continue from their laptop with the remaining steps intact.

File edits proposed by Vibe Coder are routed through the Editor AI Edit Flow (§6.6), presenting a diff before any write is committed.

**Context signals:** `.git`, `Cargo.toml`, `package.json`, `pyproject.toml` — activates automatically in development sector contexts.

### 4.9 Offline AI Queue

When the Brain cannot reach the configured AI backend (network loss, backend down, remote inference timeout), pending AI requests are queued rather than silently dropped.

- The queue is stored by `tos-sessiond` in the live session state.
- A **"N requests pending"** chip appears in the right bezel, replacing the normal AI chip output.
- When the backend connection restores, the queue drains in order. Each response is delivered to its originating skill module as if it had responded in real time.
- If the user navigates away from the sector while requests are queued, the queue persists and drains on return.
- Queued requests have a maximum age of 30 minutes. Requests older than 30 minutes are discarded with a notification.

**IPC:**
| Message | Effect |
|:---|:---|
| `ai_queue_status` | Returns count of pending queued requests |
| `ai_queue_flush` | Discards all queued requests |

### 4.10 Settings Integration

```
Settings → AI
├── Backend
│   ├── System Default:  [Ollama (local) ▾]
│   ├── Installed:       Ollama  |  OpenAI  |  Anthropic
│   └── Manage Backends → (opens marketplace filtered to .tos-ai)
├── Skills
│   ├── Passive Observer
│   │   ├── [●  ON]  [Remove]
│   │   ├── Backend: [System Default ▾]
│   │   └── Trigger Sensitivity: [Medium ▾]
│   ├── Chat Companion
│   │   ├── [●  ON]  [Remove]
│   │   ├── Backend: [OpenAI (gpt-4o) ▾]
│   │   └── Conversation Memory: [Per Session ▾]
│   ├── Vibe Coder
│   │   ├── [○  OFF]  [Remove]
│   │   ├── Backend: [System Default ▾]
│   │   └── Learned Patterns: [View / Clear]
│   ├── Command Predictor
│   │   ├── [●  ON]  [Remove]
│   │   ├── Backend: [Ollama (local) ▾]
│   │   └── Max Latency: [300ms ▾]
│   └── + Add Skill → (opens marketplace filtered to .tos-skill)
└── Global
    ├── AI Chip Color:        [Secondary (teal) ▾]
    ├── Ghost Text Opacity:   [40% ▾]
    ├── Disable All AI:       [ ] (master off switch)
    └── Context Sent:         [Standard ▾] (Standard / Minimal / Full)
```

### 4.11 IPC Contracts — AI System

| Message | Effect |
|:---|:---|
| `ai_skill_enable:<id>` | Enables a skill module by ID |
| `ai_skill_disable:<id>` | Disables a skill module by ID |
| `ai_skill_configure:<id>:<json>` | Updates a skill module's config |
| `ai_skill_load:<id>;<sector_id>` | Loads a skill into a specific sector |
| `ai_skill_unload:<id>;<sector_id>` | Unloads a skill from a specific sector |
| `ai_chip_stage:<command>` | Stages an AI-suggested chip command into the prompt |
| `ai_chip_dismiss:<chip_id>` | Dismisses an AI chip without staging |
| `ai_thought_expand` | Expands the active thought bubble into chat panel |
| `ai_thought_dismiss` | Dismisses the thought bubble for current session |
| `ai_thought_dismiss_permanent` | Dismisses thought bubble permanently |
| `ai_context_request` | Face requests current AI context object from Brain |
| `ai_context_sync:<sector_id>` | Remote Face requests full AI context for a sector |
| `ai_backend_set_default:<id>` | Sets the system default backend |
| `ai_backend_set_skill:<skill_id>:<backend_id>` | Sets a backend override for a specific skill module |
| `ai_backend_clear_skill:<skill_id>` | Removes the override, returns skill to system default |
| `ai_queue_status` | Returns count of pending queued requests |
| `ai_queue_flush` | Discards all queued requests |

### 4.12 Safety Contracts

- **No auto-execution.** No skill module may call `prompt_submit` directly. All command staging goes through `ai_chip_stage` or `stage_command`, placing text in the prompt without submitting.
- **Workflow agent confirmation.** Any workflow agent staging a command in classes `filesystem_write`, `network`, `process_kill`, or `privilege_escalation` must route through the Brain's tactile confirmation API. The Brain enforces this regardless of module implementation.
- **Context minimization.** Skill modules only receive context fields they declare in their manifest.
- **Backend isolation.** Skill modules never communicate with the LLM backend directly. All requests route through `AIService`.
- **Tool bundle enforcement.** Skill modules may only invoke Brain tools they declared in their `[tool_bundle]` manifest block. The Brain rejects undeclared tool calls at runtime.

---

## 5. Marketplace Discovery & Browse Experience

*Supplements Ecosystem Specification §2*

### 5.1 Philosophy

The marketplace is where TOS grows. Discovery is not a feature — it is the mechanism by which TOS becomes personal.

Two principles:

- **CURATED FIRST, EXHAUSTIVE SECOND.** The home view is a considered set of picks and categories. Exhaustive search is one tap away but is not the first thing a user sees.
- **THE DETAIL PAGE EARNS THE INSTALL.** A user should understand what a module does, what it looks like, and what it asks for before committing. The install button is the last thing on the page, not the first.

> **NOTE:** The marketplace is read from `tos-marketplaced` (ephemeral port, discovered via Brain service registry). This document describes the Face-side rendering of the data that daemon provides. No changes to the daemon or its package verification logic are required.

### 5.2 UI Surface

The marketplace opens as a **Level 3 Application Focus** and is accessed via:
- The **Web Portal satellite button** in the Top Bezel Right section — long press opens the Marketplace; short tap opens the Web Portal sharing overlay as before.
- The command `tos marketplace` typed in any Command Hub prompt.
- **Settings → [any module category] → Browse Marketplace** — opens pre-filtered to the relevant category.
- AI suggestion chips that fire when a missing or conflicting module is detected.

`Esc` or zoom-out returns the user to their previous level and sector without disrupting any running processes.

### 5.3 Home View

The marketplace home view is divided into two vertical sections: **Featured** (top) and **Categories** (below).

#### 5.3.1 Featured Strip

A horizontally scrollable strip of curated module cards, served by `tos-marketplaced` as a signed, versioned `featured.json` manifest that updates independently of TOS releases.

Each featured card shows: module name and type badge, a hero screenshot or animation, a one-line description (max 80 characters), install count and star rating, and a **[View]** button.

```
┌─────────────────────────────────────────────────────────────────────┐
│  FEATURED                                              [← →  1/6]  │
├──────────────────┬──────────────────┬──────────────────────────────┤
│  Void Theme      │  Git Expert      │  Starship Shell               │
│  THEME           │  AI BEHAVIOR     │  SHELL                        │
│  Dark LCARS      │  Git-aware chips │  Rust prompt                  │
│  ★ 4.8  12.4k   │  ★ 4.6  8.1k    │  ★ 4.9  21k                  │
│  [View]          │  [View]          │  [View]                       │
└──────────────────┴──────────────────┴──────────────────────────────┘
```

#### 5.3.2 Category Grid

| Category Tile | Module Type | Description |
|:---|:---|:---|
| Themes | `.tos-theme` | Visual appearance, color palettes, LCARS variants |
| Shells | `.tos-shell` | Shell implementations with OSC integration |
| Terminal Output | `.tos-terminal` | Terminal rendering modules |
| AI Backends | `.tos-ai` | LLM connections — local and remote |
| AI Skills | `.tos-skill` | Task-specific AI behaviors, tool bundles, and learned patterns |
| Sector Types | `.tos-sector` | Workspace presets and specialized sector logic |
| Bezel Components | `.tos-bezel` | Dockable bezel slot components |
| Audio Themes | `.tos-audio` | Earcon sets and ambient audio layers |
| Languages | `.tos-language` | Editor syntax highlighting and LSP configurations |

### 5.4 Category Browse View

A scrollable grid of **Browse Cards** for each category.

**Browse Card:**
- Screenshot — a static preview image
- Name and type badge
- One-line description (max 80 characters)
- Star rating and download count
- **[View]** — navigates to the detail page. There is no install button on the browse card.

**Sort/Filter bar:**
- **Sort:** Most Downloaded / Highest Rated / Newest
- **Filter:** All / Free / By the TOS Team / Compatible with current setup
- **Search:** filters the current category in real time

"Compatible with current setup" filters out modules that declare capability requirements the user's current AI backend or platform cannot satisfy.

### 5.5 Module Detail Page

Tapping **[View]** navigates to the module's full detail page.

**Layout:**
- Horizontally scrollable screenshot gallery at the top (module authors must submit at least one screenshot).
- Name, author, type badge, star rating, install count, version, update date, license.
- **[Review Permissions & Install]** — single call to action, positioned in the top-right metadata block.
- Full description, changelog, compatibility notes.
- **Permissions section** — human-readable permission statements (shown before installation, not as a surprise during it).
- Ratings & reviews.

**Permission display:**

| Manifest Permission | Displayed As |
|:---|:---|
| `terminal_read = true` | Can read your terminal output |
| `prompt_read = true` | Can read your current prompt |
| `network = true` | Can make network requests |
| `filesystem = true` | Can read and write files |
| No permissions declared | No special permissions required |

### 5.6 Install Flow

#### 5.6.1 Permission Review Step

Tapping **[Review Permissions & Install]** opens a modal showing the full permissions list for the module. This is the same information shown in the detail page permissions section, now presented as a formal review before commitment.

For modules with no permissions, the review step is lightweight — it primarily confirms the author and signature validity. **[Install]** triggers the download immediately.

#### 5.6.2 Download Progress

After **[Install]** is tapped:
- A download progress bar appears at the bottom of the detail page.
- The install button becomes **[Installing... x%]** and is non-interactive.
- The user can navigate away; the download continues in the background via `tos-marketplaced`.

#### 5.6.3 Completion

When the install completes:
- A notification is pushed to the TOS Log: `Module installed: <Name> — available in Settings → <Category>`.
- If the user is still on the detail page, the install button becomes **[Installed ✓]**.
- If the user has navigated away, the log notification is the only signal. No toast, no alert.
- The module is immediately available in its relevant Settings category.

#### 5.6.4 Install Failure

If the download or signature verification fails:
- The progress bar turns amber and shows the failure reason.
- The install button returns to **[Review Permissions & Install]** so the user can retry.
- A failure notification is pushed to the TOS Log.

### 5.7 Search

Global marketplace search is accessible from the home view via a persistent search bar. Search queries are sent to `tos-marketplaced` which searches across all module types simultaneously. Results are grouped by category.

The AI integration from Ecosystem Specification §2.4 is preserved: typing a natural language query in the search bar triggers the AI-assisted discovery path.

### 5.8 Installed State in Browse

When a user browses a category and encounters an already-installed module:
- Browse card shows **[Installed ✓]** badge.
- Detail page shows **[Installed ✓]** with a secondary link: **[Manage in Settings →]**.

There is no separate "My Modules" section in the marketplace. Installed module management lives in Settings.

### 5.9 IPC Contracts — Marketplace UI

| Message | Effect |
|:---|:---|
| `marketplace_home` | Returns featured manifest and category counts |
| `marketplace_category:<type>` | Returns paginated module list for a category |
| `marketplace_detail:<id>` | Returns full module metadata, screenshots, permissions |
| `marketplace_search:<query>` | Full-text search across all module types |
| `marketplace_search_ai:<query>` | AI-assisted natural language discovery |
| `marketplace_install:<id>` | Initiates install after permission review |
| `marketplace_install_cancel:<id>` | Cancels an in-progress download |
| `marketplace_install_status:<id>` | Returns current install progress (0–100, or error) |

---

## 6. TOS Editor

*Supplements Architecture Specification §11 (Split Viewports) and §30 (UI Module Interaction APIs)*

### 6.1 Philosophy

The TOS Editor exists because code and terminal output are inseparable. A developer running `cargo build` in one pane should be able to see the failing file in the next pane without switching applications, losing focus, or losing AI context.

Three principles govern all editor design decisions:

- **OUTPUT AREA FIRST.** The editor is primarily a *viewer* of the file currently active in the terminal context. Editing is a secondary capability that activates on demand. The terminal drives; the editor follows.
- **AI CONTEXT IS ALWAYS LIVE.** Every visible file, every selection, every cursor position is live context for the AI system. The AI knows what you are looking at without you pasting anything.
- **NO ESCAPE FROM TOS.** The editor never opens a separate window or requires leaving the hierarchy. It is always a pane, an overlay, or a Level 3 surface — always surrounded by the Tactical Bezel, always connected to the Brain.

### 6.2 Surface Modes

| Mode | Description | Activation |
|:---|:---|:---|
| **Viewer** | Read-only. Displays file, scrolls to relevant lines, highlights syntax, provides AI context. No cursor, no input. | Automatic — triggered by terminal events or `editor_open` IPC |
| **Editor** | Full cursor, keyboard input, syntax-aware editing, save operations. | User taps into the editor surface or sends `editor_activate` |
| **Diff** | Side-by-side comparison. Left = current file; right = proposed or historical state. | `editor_diff` IPC or automatically by Vibe Coder AI Edit Flow (§6.6) |

### 6.3 Editor Output Area — Level 2 Integration

The editor integrates into the Command Hub as a named output area — a peer to the terminal pane, not subordinate to it.

#### 6.3.1 Hub Layout Integration

`pane_type: "editor"` is a first-class pane type alongside `"terminal"` (Architecture §11.2). Editor panes are persisted in the session file (§2.9).

#### 6.3.2 Auto-Open Triggers

The Brain automatically opens or updates the editor pane in response to terminal events:

| Terminal Event | Editor Response |
|:---|:---|
| Command exits non-zero with a file path + line number in output | Opens file, scrolls to error line, highlights in amber |
| `cd` to a directory | Editor shows directory listing in Viewer Mode |
| User types a file path in the prompt | Editor previews the file before submission |
| AI Passive Observer identifies a relevant file | Editor opens file with AI annotation overlay |
| `git diff` or `git show` executed | Editor switches to Diff Mode |

Auto-open is configurable per sector in **Settings → Editor → Auto-Open Triggers**.

#### 6.3.3 Focus Rules

- Keyboard input always goes to the terminal pane by default.
- Clicking or tapping the editor pane switches keyboard focus to Editor Mode.
- `Ctrl+E` / swipe right-to-left toggles focus between terminal and editor panes.
- In mobile Face profile, focus follows the active tab.

#### 6.3.4 Editor Pane Header

```
┌──────────────────────────────────────────────────────┐
│  📄 src/brain/main.rs  ●  Rust  │ Ln 142  │ [AI] [⊞] │
└──────────────────────────────────────────────────────┘
```

- **●** — unsaved changes indicator (amber dot)
- **[AI]** — opens the AI Context Panel (§6.5) for this file
- **[⊞]** — promotes editor pane to Level 3 Application Focus

### 6.4 Editor Application Focus — Level 3

When promoted to Level 3, the editor becomes a full-screen surface wrapped in the Tactical Bezel.

```
┌─────────────────────────────────────────────────────┐
│  TOP BEZEL — File path, branch, dirty indicator     │
├──────────────┬──────────────────────┬───────────────┤
│  LEFT BEZEL  │                      │  RIGHT BEZEL  │
│  File tree   │   EDITOR SURFACE     │  AI Context   │
│  (optional)  │                      │  Panel (§6.5) │
├──────────────┴──────────────────────┴───────────────┤
│  BOTTOM BEZEL — Persistent Unified Prompt           │
└─────────────────────────────────────────────────────┘
```

The Persistent Unified Prompt remains active at all times. Commands typed while the editor is at Level 3 route to the sector's active PTY — the editor never intercepts shell commands.

The **File Tree** is an optional Bezel Component (`.tos-bezel`) docked to the Left Bezel slot showing the sector's cwd as a collapsible tree. A **minimap** of the current file occupies the Right Lateral Bezel slot when at Level 3.

### 6.5 AI Context System

#### 6.5.1 Editor Context Object

The Brain maintains an Editor Context Object for each open editor pane. This object is automatically included in every AI query from the same sector (§4.7).

```json
{
  "editor_context": {
    "file": "/8TB/tos/src/brain/main.rs",
    "language": "rust",
    "visible_range": { "start_line": 138, "end_line": 185 },
    "cursor_line": 142,
    "cursor_col": 18,
    "selection": null,
    "unsaved_changes": false,
    "git_status": "modified",
    "diagnostics": [
      { "line": 142, "severity": "error", "message": "cannot borrow `state` as mutable" }
    ]
  }
}
```

By default only the visible range and diagnostics are included to keep context tokens bounded. The user can expand scope in the AI Context Panel.

#### 6.5.2 AI Context Panel

Dockable Right Bezel slot component showing the live relationship between the current file and the AI:

```
┌─────────────────────────────────┐
│  AI CONTEXT          [⟳] [✕]   │
├─────────────────────────────────┤
│  📄 main.rs : Ln 142            │
│  ⚠ 1 error in context           │
│                                 │
│  CONTEXT SCOPE                  │
│  ○ Visible range (default)      │
│  ● Selection only               │
│  ○ Full file                    │
│  ○ Full file + imports          │
│                                 │
│  ACTIVE ANNOTATIONS             │
│  › Line 142 — borrow error      │
│    [Ask AI] [Explain] [Fix]     │
│                                 │
│  RECENT AI EDITS                │
│  › refactor_session_handler     │
│    2 mins ago · [Undo]          │
└─────────────────────────────────┘
```

#### 6.5.3 Context Send Actions

| Action | How | Result |
|:---|:---|:---|
| Send visible range | Tap `[AI]` in pane header | Visible lines sent as user message |
| Send selection | Select text → right-click → "Ask AI about this" | Selected text sent with ±10 lines context |
| Send full file | Context Panel → Full file | Full content sent; AI warned if > 32K tokens |
| Send error | Click annotation chip → [Ask AI] | Error + surrounding lines sent |
| Send diff | In Diff Mode → [Ask AI about diff] | Both sides sent |

#### 6.5.4 Inline AI Annotations

When the AI references specific lines, the Brain renders annotation chips in the editor's right margin. Annotations are ephemeral — tied to the AI chat turn that generated them.

#### 6.5.5 Semantic Scroll Sync

When the AI references a line in its response, the editor automatically scrolls to that line with a brief amber pulse. Scrolling the editor updates the AI context object so the next message automatically has the correct visible range.

### 6.6 AI Edit Flow (Vibe Coder Integration)

#### 6.6.1 Triggering an Edit

Initiated by:
- Natural language edit request in the prompt while editor is open
- Vibe Coder (§4.8) decomposing a task into a file edit step
- Clicking **[Fix]** on an annotation chip
- Selecting text → right-click → "Ask AI to rewrite this"

#### 6.6.2 Diff Review

The editor automatically switches to Diff Mode when an edit proposal is received:

```
┌──────────────────────────────────────────────────────┐
│  PROPOSED EDIT — fix borrow error       [Apply] [✕]  │
├─────────────────────┬────────────────────────────────┤
│  CURRENT            │  PROPOSED                      │
│  let state = self   │  let Some(state) = self        │
│    .sessions        │    .sessions                   │
│    .get_mut(&id)    │    .get_mut(&id) else {        │
│    .expect("...");  │      return Err(...);          │
│                     │    };                          │
│  state.update();    │  state.update();               │
└─────────────────────┴────────────────────────────────┘
```

**[Apply]** writes the change and returns to Editor Mode. **[✕]** rejects. The user can also edit the proposed side directly before applying.

#### 6.6.3 Multi-File Edits

When Vibe Coder proposes edits spanning multiple files, a chip sequence appears in the Right Bezel:

```
PROPOSED EDITS  (3 files)
─────────────────────────
✓ 1. main.rs — null check     [View] [Apply]
○ 2. session.rs — error type  [View] [Apply]
○ 3. lib.rs — re-export       [View] [Apply]

[Apply All]  [Reject All]
```

Each step can be applied individually. **Pending steps are persisted in the session file (§2.9)** — the user can apply steps across devices and sessions.

#### 6.6.4 Undo

All AI-applied edits are recorded in the undo stack with a distinct AI label. `Ctrl+Z` undoes them like any other edit. The AI Context Panel maintains a Recent AI Edits list (last 10) with per-edit undo buttons.

### 6.7 Multi-Device Rendering

| Profile | Default Hub Layout | Editor Behavior |
|:---|:---|:---|
| `desktop` | Vertical split — terminal left, editor right | Full editor with minimap, file tree, keyboard shortcuts |
| `mobile` | Tab layout — terminal tab / editor tab | Editor in second tab; AI Context Panel as bottom sheet; long-press line number sends line to AI |
| `vr` | Single pane | Editor as spatial panel; spatial gestures for navigation |

On mobile, tapping a line number in the margin sends that line to the AI as context. This is the primary mobile AI interaction with the editor — no text selection required.

When the virtual keyboard appears on mobile, the editor shrinks to accommodate it. Bezel slots collapse automatically. The prompt remains accessible via a floating pill above the keyboard.

### 6.8 File Management

| Action | Behavior |
|:---|:---|
| `edit <path>` in prompt | Opens file in Editor Mode |
| `view <path>` in prompt | Opens file in Viewer Mode |
| `Ctrl+S` | Save to current path |
| `Ctrl+Shift+S` | Save As — opens path input chip in prompt |
| Close pane with unsaved changes | Warning chip: `[Save] [Discard] [Cancel]` |
| Files outside sector cwd | Trust confirmation chip required before write |
| Binary files | Viewer Mode only — hex dump via Terminal Output Module |
| Files > 10MB | Viewer Mode only with warning chip |
| Image files | Rendered inline if Face supports it; AI vision context if backend declares `vision = true` |

### 6.9 Language & Syntax Support

**Built-in languages (Tree-sitter):** Rust, TypeScript, JavaScript, Python, Bash/Zsh, JSON, TOML, YAML, Markdown, HTML, CSS, SQL, Dockerfile, Go, C, C++.

**Language detection priority:**
1. File extension
2. Shebang line
3. Content heuristics
4. Manual override via language badge in pane header

**LSP Integration:** When an LSP server is available in the sector's PATH for the current language, the editor activates diagnostics, hover, go-to-definition, and completion. LSP diagnostics are forwarded to the Editor Context Object automatically.

Additional languages can be added via `.tos-language` modules (Ecosystem §1.10).

### 6.10 Session Persistence

See §2.9 for the full editor pane session schema.

### 6.11 IPC Contracts
### 6.12 Accessibility

TOS Editor follows the global accessibility guidelines (§24.1). Specific enhancements include:
- Screen reader announces line numbers and indentation level.
- High-contrast syntax highlighting mode.
- Keyboard-only navigation for all editor actions.

---

## 7. Kanban-Driven Agent Orchestration

*Introduces project-level workflow management, agent personas, and multi-sector collaboration*

### 7.1 Philosophy

TOS workflows evolve beyond simple linear pipelines into **project-scoped kanban boards**. Multiple sectors can open and work on the same project simultaneously. AI agents with distinct personas decompose tasks into executable steps, run commands in parallel, and learn from completed work. The system preserves all LLM reasoning for future learning and human audit.

Key principles:
- **One board, multiple agents, multiple sectors** — real team collaboration
- **User-defined kanban states** — not locked to a single model
- **Complete observability** — user watches every step, every LLM decision
- **Knowledge accumulation** — completed tasks feed a running project memory

### 7.2 Kanban Board Model

A kanban board is a **project-level artifact** stored in `<project_root>/.tos/kanban.tos-board`. Any sector opening that project sees and can interact with the same board.

#### 7.2.1 Board Definition

```json
{
  "tos_kanban_version": "1.0",
  "project_id": "tos-desktop-v0.5",
  "project_name": "TOS Desktop Environment - v0.5 Sprint",
  "board_definition": {
    "lanes": [
      {
        "id": "backlog",
        "name": "Backlog",
        "description": "Unscheduled work",
        "auto_promotion": {
          "enabled": false
        }
      },
      {
        "id": "planned",
        "name": "Planned",
        "description": "Ready to start",
        "auto_promotion": {
          "enabled": true,
          "target_lane": "wip",
          "condition": "available_bandwidth",
          "max_source_items": 10
        }
      },
      {
        "id": "wip",
        "name": "WIP",
        "description": "Currently in progress",
        "auto_promotion": {
          "enabled": false,
          "max_concurrent": 3
        }
      },
      {
        "id": "blocked",
        "name": "Blocked",
        "description": "Waiting on external factor",
        "auto_promotion": {
          "enabled": false
        }
      },
      {
        "id": "review",
        "name": "Review",
        "description": "Awaiting human approval",
        "auto_promotion": {
          "enabled": false
        }
      },
      {
        "id": "done",
        "name": "Done",
        "description": "Completed work",
        "auto_promotion": {
          "enabled": true,
          "action": "archive_to_history"
        }
      }
    ]
  },
  "tasks": {
    "task_001": {
      "id": "task_001",
      "title": "Fix borrow checker in session.rs",
      "description": "Address compiler error on line 142. Add proper error handling.",
      "lane": "wip",
      "assigned_agent": "careful_bot",
      "auto_accept": true,
      "depends_on": [],
      "tags": ["backend", "critical"],
      "acceptance_criteria": [
        "cargo check passes with no warnings",
        "test_load_session() passes",
        "Code reviewed by maintainer"
      ]
    }
  }
}
```

#### 7.2.2 User-Defined Lanes

Users fully customize kanban lanes. Settings → Workflows → Board Lanes:

```toml
[board_lanes]
# Customize lane names, descriptions, and auto-promotion rules
lane_1 = "backlog"
lane_2 = "planned"
lane_3 = "wip"
lane_4 = "blocked"
lane_5 = "review"
lane_6 = "done"

[auto_promotion]
# Planned → WIP: when WIP lane has room
planned_to_wip = { enabled = true, condition = "max_wip < 3" }

# WIP → Review: manual (agent completes work)
wip_to_review = { enabled = false }

# Done → Archive: automatic after 7 days or manual
done_to_archive = { enabled = true, days = 7 }
```

#### 7.2.3 Task Lifecycle

A task moves through lanes:

```
BACKLOG → PLANNED → WIP → BLOCKED (optional) → REVIEW → DONE → ARCHIVED

- Backlog: User adds task, not yet scheduled
- Planned: Task is scheduled, waiting for bandwidth
- WIP: Agent actively working (agent assigned, decomposing/executing steps)
- Blocked: Agent hit a blocker, paused and waiting for manual intervention
- Review: Agent completed work, awaiting human approval
- Done: Work approved and merged
- Archived: Historical record (auto-moved after N days)
```

Auto-promotion rules trigger state transitions automatically:
- `planned → wip`: When WIP lane has capacity (`max_concurrent < 3`)
- `done → archived`: After N days (configurable)

### 7.3 Agent Personas

An agent persona is a **markdown-based strategy definition** that any AI can read and interpret. Personas define how an agent approaches task decomposition and execution.

#### 7.3.1 Persona Format

Agent personas live in `~/.local/share/tos/personas/` and are discovered by the system.

**Example: careful_bot.md**

```markdown
# Agent Persona: careful_bot

## Identity
- **Name:** careful_bot
- **Role:** Methodical, thorough, risk-averse
- **Best for:** Critical path work, security-sensitive code, test-driven development
- **Cost:** Slower (runs full suite), higher token cost (validates thoroughly)

## Core Strategies

### Testing Strategy
- **Rule:** Always run full test suite before advancing
- **Implementation:** After any file write, run `cargo test --all`. Fail if exit code ≠ 0.
- **Override:** User can click [Skip tests] to bypass on a step-by-step basis

### Error Handling
- **Rule:** Halt on first error, report full context
- **Implementation:** On command exit code ≠ 0, pause workflow and stage investigative commands to prompt
- **Suggested next steps:** 
  - `cargo build --message-format=json` (detailed error)
  - `git log --oneline -5` (context)

### Step Sizing
- **Rule:** Many small steps (validate each change in isolation)
- **Implementation:** Limit file edits to <50 lines per step. Use multiple steps for larger refactors.

### Output Validation
- **Rule:** Verify output matches intent before advancing
- **Implementation:** After running a command, ask: "Did this produce the expected result?" before moving to next step.

## Tool Bundle
- `read_file`, `write_file`, `exec_cmd`, `list_dir`
- `run_tests` (synthetic: invokes test runner and parses results)
- `git_*` (commit, log, diff, rebase)

## Backend Preference
- **Preferred:** Local (fast LLM for iteration)
- **Fallback:** OpenAI GPT-4 (if complex reasoning needed)

## Learned Patterns
- Tracks which test failures are common for this codebase
- Learns which code patterns are "idiomatic" vs. "fragile"
- Stores in `~/.local/share/tos/personas/careful_bot/patterns.json`
```

#### 7.3.2 Built-in Personas

TOS ships with three reference personas:

| Persona | Style | Best For | Speed |
|---|---|---|---|
| **careful_bot** | Test-first, thorough validation, halt on error | Critical backend, security | Slow |
| **fast_bot** | Large steps, parallel validation, retry-on-error | Performance, feature work | Fast |
| **creative_bot** | Suggest alternatives, experiment, low-risk paths | Exploration, prototyping | Variable |

#### 7.3.3 Custom Personas

Users compose custom personas by mixing strategies or writing entirely new ones:

```markdown
# Agent Persona: balanced_bot

Combines:
- Testing strategy from **careful_bot** (run tests, but only affected ones)
- Step sizing from **fast_bot** (larger chunks, faster iteration)
- Error handling from **creative_bot** (suggest alternative approaches, not just halt)

## Custom Rule: Documentation
- **Rule:** Generate docstring for any public API added
- **Implementation:** After `write_file` adding a `pub fn`, ask: "Add docstring?"
```

#### 7.3.4 Persona Discovery & Management

Personas are discoverable in Settings → Workflows → Agent Personas:

```
├─ Built-in
│  ├─ careful_bot [details]
│  ├─ fast_bot [details]
│  └─ creative_bot [details]
├─ Custom
│  ├─ balanced_bot [details] [edit] [delete]
│  └─ deploy_bot [details] [edit] [delete]
├─ Marketplace
│  ├─ Browse Marketplace
│  └─ [installed marketplace personas]
└─ [+ New Persona]
```

### 7.4 Task Definition & Roadmap System

Tasks are defined in `.tos-task` format or auto-generated by the Roadmap Skill.

#### 7.4.1 Task Format

```yaml
# roadmap.tos-tasks (or roadmaps/v0.5.tos-roadmap)

version: "1.0"
roadmap_id: "v0.5"
roadmap_name: "Session Persistence & Workspace Memory"

tasks:
  - id: "task_001"
    title: "Fix borrow checker in session.rs"
    description: |
      Address compiler error on line 142 of brain/session.rs.
      The load_session() function needs proper error handling.
    source: "github://user/tos-desktop/issues/456"
    depends_on: ["task_000"]
    tags: ["backend", "critical", "session-persistence"]
    acceptance_criteria:
      - "cargo check passes with no warnings"
      - "test_load_session() passes"
      - "Code reviewed by @maintainer"
```

#### 7.4.2 Roadmap Skill

A built-in skill that generates tasks from external sources (GitHub issues, Jira, manual input).

**Skill: roadmap_planner** (marketplace: `.tos-skill`)

- **Surface:** Thought bubble + chat panel
- **Trigger:** Manual (user invokes explicitly)
- **Behavior:** Decomposes epics/issues into kanban tasks, estimates effort, suggests agents

User flow:
```
$ User: "Plan v0.5 from GitHub issues"

→ Roadmap Planner fetches GitHub issues
→ Groups by epic
→ Creates tasks in Backlog lane
→ Suggests agent assignments (based on task type)
→ Surfaces: "Created 12 tasks. Assign to roadmap_v0.5?"

$ User clicks [✓ Create]

→ Tasks appear in kanban Backlog lane
```

### 7.5 Agent Decomposition & Execution

When a task moves to WIP, the assigned agent decomposes it into executable steps using the LLM.

#### 7.5.1 Decomposition Process

```
1. Agent reads task + acceptance criteria
2. Agent loads its persona (markdown strategy)
3. Agent loads codebase context (recent commits, similar tasks)
4. Agent invokes LLM: "Decompose this task using my persona strategy"
5. LLM responds with step-by-step plan
6. Agent presents plan to user: "Ready to proceed with steps 1-5?"
7. On approval, agent begins execution
```

**LLM is provided:**
- Full task description
- Agent persona (markdown)
- Recent patterns learned for this codebase
- Tool bundle available
- Any codebase context (recent changes, related files)

**LLM responds with:**
```json
{
  "reasoning": "This is a lifetime issue. The function returns Session directly...",
  "steps": [
    {
      "step_id": "step_1",
      "title": "Read the error and surrounding context",
      "instruction": "Use read_file to examine lines 130-150 of brain/session.rs",
      "tool_calls": ["read_file(brain/session.rs, 130, 150)"],
      "expected_outcome": "Understand the lifetime mismatch"
    },
    ...
  ]
}
```

#### 7.5.2 Step Execution

Each step:
1. Stages commands to the agent's **isolated terminal prompt**
2. User (or auto-accept setting) approves
3. Command executes in agent's PTY
4. Output captured and archived
5. Agent observes output, determines next step

**Auto-accept setting** (per-task):
- If `auto_accept: true`, commands from this agent run without user approval
- User can still inspect logs, pause, or abort the agent
- Paused steps remain in prompt for manual execution

#### 7.5.3 Error Handling & User Intervention

When a step fails (non-zero exit):
- Agent pauses workflow
- Displays error output + agent interpretation
- User can:
  - **[Inspect]** — see full logs
  - **[Retry]** — retry same step (agent might try different command)
  - **[Suggest]** — tell agent a different approach
  - **[Skip]** — skip to next step (manual override)
  - **[Abort]** — abort task, move to BLOCKED

### 7.6 Workflow Manager Pane

The Workflow Manager is a **per-sector Level 2 pane** (sibling to terminal and editor) that displays the kanban board and active agent terminals.

#### 7.6.1 Layout

```
┌────────────────────────────────────────────────────────┐
│  KANBAN BOARD                                          │
├────────┬──────────┬─────────┬──────────┬────────────────┤
│BACKLOG │ PLANNED  │  WIP    │ BLOCKED  │ REVIEW | DONE  │
├────────┼──────────┼─────────┼──────────┼────────┼────────┤
│ • #456 │ • Fix    │ • Perf  │ • Need   │ • Code │ ✓Tests │
│   @rm  │   borrow │   opts  │  review  │ review │  @care │
│        │   @care  │ @fast   │ @careful │ @human │        │
│        │ [2/5]    │ [↗ new] │ [pause]  │        │        │
└────────┴──────────┴─────────┴──────────┴────────┴────────┘

Tab bar: [@careful #1] [@fast #2] [@creative #3]  [+]

Active agent terminal (@careful):
┌─────────────────────────────────────────────────────────┐
│ Fix borrow checker in session.rs                        │
│ Status: IN PROGRESS (step 2/5)                          │
│ [⏸ Pause] [→ Step] [⏹ Abort]                            │
├─────────────────────────────────────────────────────────┤
│ $ cargo clippy --all-targets                            │
│ Checking brain/session.rs...                            │
│ warning: unused variable: 'x'                           │
│                                                         │
│ Agent observation:                                      │
│ "Style warning detected. Proceeding with next step...  │
│  The fix is correct and passes clippy."                │
├─────────────────────────────────────────────────────────┤
│ [Previous output collapsed] [Show full log] [Copy]      │
└─────────────────────────────────────────────────────────┘
```

#### 7.6.2 Agent Terminal Tabs

Each active agent gets a tab. Users can:
- Click tab to switch view
- `Ctrl+Shift+W` to focus pane, `←/→` to cycle agents
- Right-click tab for options (pause, inspect, abort, view patterns)

Agent terminal is **isolated** — each agent has its own:
- Terminal pane / PTY context
- Output history
- Pending steps

#### 7.6.3 Kanban Board Interaction

**Drag to reassign:**
- Drag task card to different lane
- Auto-promotion rules trigger as appropriate

**Right-click task card:**
- View task details
- Reassign agent
- Edit task properties
- View task history (completed, previous attempts)

**View task decomposition:**
- Click task card to expand
- Shows agent's step-by-step plan
- Shows LLM reasoning (collapsible section)
- Shows acceptance criteria + current status

### 7.7 Multi-Agent Concurrency & Sandboxing

Multiple agents can work on different tasks simultaneously. To ensure data integrity and prevent collision:

- **Agent Sandboxing**: Each agent works within a transient filesystem overlay or sandbox. Changes are not directly written to the project's primary tree until the task is closed.
- **Isolated Context**: Each agent operates independently in its own PTY terminal context.
- **Merge Strategy**: Upon moving a task to the `REVIEW` or `DONE` lane, the agent's staged changes must be merged into the project tree. Conflicts are surfaced to the user via the Editor’s Diff Mode.

#### 7.7.1 Concurrency Model

```json
{
  "agents_active": [
    {
      "agent_id": "careful_bot",
      "task_id": "task_001",
      "state": "running",
      "current_step": 2,
      "terminal_pane": "pane_workflow_1"
    },
    {
      "agent_id": "fast_bot",
      "task_id": "task_002",
      "state": "running",
      "current_step": 1,
      "terminal_pane": "pane_workflow_2"
    }
  ]
}
```

**Key properties:**
- Each agent stages commands to **its own prompt** (not the sector's main prompt)
- Agents **do not block** each other
- Commands execute in parallel (limited only by system resources)
- Terminal output is **isolated per agent** (visible in tabs or split view)

#### 7.7.2 Multi-Agent Terminal Display

**Option A: Tabbed view**
```
Tabs: [@careful task_001] [@fast task_002] [@creative task_003]
Current view: @careful (task_001)
```

**Option B: Split view**
```
┌──────────────────────────────┬──────────────────────────────┐
│  @careful (task_001)         │  @fast (task_002)            │
│  $ cargo clippy...           │  $ hyperfine before.rs...    │
│  [step 2/5]                  │  [step 1/3]                  │
│                              │                              │
└──────────────────────────────┴──────────────────────────────┘

Or 3-way split for critical work.
```

User switches: `Ctrl+Tab` cycles through agent views.

#### 7.7.3 Resource Contention (Optional)

If multiple agents try to access the same resource (e.g., `git push`), optional centralized queueing:

```toml
[concurrent_execution]
# Resource-level queueing (prevents simultaneous git pushes)
exclusive_resources = ["git_push", "db_migrate"]

# Agent-level limits
max_agents = 3
max_parallel_file_writes = 2
```

Most work (cargo build, file edits, tests) can be truly parallel. Only explicitly exclusive operations are queued.

### 7.8 Persistence & Session Continuity

#### 7.8.1 Complete LLM Interaction Archive

### 7.9 Bulk Trust & Task Generation

To avoid approval fatigue when generating large roadmaps or multi-step plans:

- **Bulk Trust Chip**: When a Roadmap Skill or agent proposes 5+ tasks/steps at once, a single "Bulk Trust" warning chip appears. 
- **Expanded Review**: Tapping the chip shows a summary of all proposed actions, allowing the user to approve the entire set with one action.
- **Individual Override**: Users can still deselect or edit specific items within the bulk review before approving.

All LLM requests and responses are preserved in the session file under each task:

```json
{
  "llm_history": {
    "initial_decomposition": {
      "timestamp": "2025-04-04T10:22:00Z",
      "request": {
        "task_title": "Fix borrow checker in session.rs",
        "task_description": "...",
        "persona_md": "[full careful_bot persona markdown]",
        "codebase_context": { ... }
      },
      "response": {
        "reasoning": "This is a lifetime issue...",
        "plan": "Step 1: Read error...",
        "steps": [ ... ]
      }
    },
    "step_interactions": [
      {
        "step_id": "step_1",
        "executed_at": "2025-04-04T10:23:15Z",
        "command_executed": "read_file(brain/session.rs, 130-150)",
        "command_output": "fn load_session() -> Session { ... }",
        "agent_observation_request": "What do you see?",
        "agent_response": "The function returns Session directly...",
        "next_step_confirmation": "Should we proceed to step 2?"
      }
    ]
  }
}
```

#### 7.8.2 Workflow Manager UI Access to LLM History

The Workflow Manager pane includes an expandable **LLM reasoning panel** per step:

```
🔄 Step 1: Read error and context [✓ Done]
┌──────────────────────────────────────────────────┐
│ Agent observation (LLM-generated):               │
│ "The function returns Session directly, but the │
│ caller expects Result<Session>. This is the     │
│ mismatch."                                      │
│                                                 │
│ [Show full command output]                      │
└──────────────────────────────────────────────────┘
```

Expandable sections show:
- Agent reasoning (why this step?)
- Command output (what did it do?)
- Agent observations (what did it learn?)
- Decision points (should we continue?)

#### 7.8.3 Resumption with Full Context

When resuming a paused task:

```
User closes TOS while @careful_bot is mid-task:
- Current state saved: step 2/5, last command output, pending steps

User reopens TOS:
- Kanban board reloaded with all tasks in their lanes
- @careful_bot task still shows "IN PROGRESS"
- [Resume] chip appears: "Resume @careful_bot on Fix borrow checker?"
- User clicks [Resume] → agent continues from step 3 with full context
```

Agent resumes with:
- Full LLM decomposition (original plan)
- All past LLM interactions (observations from steps 1-2)
- Current step (3) with full context
- Learned patterns from similar tasks

#### 7.8.4 Multi-Sector Synchronization

When multiple sectors have the same project open:

```
Sector A (laptop) completes task_001:
  → Agent finishes, moves task to REVIEW

Sector B (desktop) sees update instantly:
  → Kanban board refreshes
  → task_001 shows in REVIEW lane
  → Can now review code

Sector C (tablet) reviewing code:
  → Approves, moves task to DONE
  → All sectors see DONE state
```

Synchronization via:
- Shared project directory (`.tos/kanban.tos-board`)
- File system watches (inotify / FSEvents)
- IPC updates between sectors on same machine
- For remote: TOS Remote Server protocol (see Architecture §12)
