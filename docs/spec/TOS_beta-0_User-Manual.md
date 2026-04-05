# TOS User Manual

## 1. System Philosophy: The Augmented Desktop Entity

TOS (**Terminal On Steroids**) is not a static workspace; it is a **dynamic augmented desktop entity**. Inspired by LCARS design principles, it prioritizes hierarchical depth, multi-sensory feedback, and context-aware rendering. The command line is not one feature among many — it is the permanent heart of the system. Every visual augmentation exists to empower the terminal, never to replace it.

---

## 2. Navigation Architecture: Hierarchical Levels

TOS uses a 4-level depth system to allow rapid transitions between high-level oversight and low-bit buffer inspection.

| Level | Name | Description | Visualization |
|:---|:---|:---|:---|
| **LVL 1** | **Global Overview** | Tactical map of all active system sectors. | Sector tiles + System Output Area (Brain console) |
| **LVL 2** | **Command Hub** | The primary workspace for shell and data interaction. | Dual-column chip-terminal |
| **LVL 3** | **Application Focus** | Dedicated window surface for a single graphical process. | Chrome-window overlay |
| **LVL 4** | **Deep Inspection & Recovery** | Detail View (metadata), Buffer View (hex dump, privileged), and Tactical Reset (God Mode wireframe recovery). | Property chips / hex viewer / wireframe map |

**Navigation is always vertical** — Zoom In or Zoom Out. There is no lateral level navigation.

---

## 3. The Command Hub (LVL 2) Modes

The mode selector at the left side of the Persistent Unified Prompt switches the terminal canvas context:

- **[CMD] Command Mode** — Standard interactive PTY terminal. Chips populate with command history, autocomplete suggestions, and tool flags.
- **[SEARCH] Search Mode** — Semantic or global filesystem indexing with instant results. Chips populate with search scopes, filters, and quick-action buttons.
- **[AI] AI Augmentation** — Natural language shell queries with AI explanation and command staging. The AI never executes commands without your confirmation from the prompt.
- **Directory Context** — Triggered automatically by `ls` or `cd`. Shows real-time file and folder chips for rapid prompt building. File and image previews where applicable.
- **Activity Context** — Triggered automatically by `top` or `ps`. Shows process-handling action chips (kill, renice, monitor):
  - **Live View:** 10Hz snapshots for active, graphical applications.
  - **Resource View:** App icon and name for inactive or non-graphical applications.
  - **System View:** Symbolic placeholders for background and system processes.

---

## 4. The Persistent Unified Prompt

The bottom bezel is the permanent command interface — it is always visible, always accessible, and always ready. It has three sections:

- **Left (Origin):** Universal Mode Selector (CMD, SEARCH, AI, ACTIVITY). Not removable.
- **Center:** The active command input field. Always reflects the command about to be executed.
- **Right:** Microphone (voice input) and Stop/Kill switch.

The prompt expands to full interactive mode at Level 2. At Level 3, it is collapsed but can be expanded by tapping or hovering the bottom bezel. At Level 4, it is locked during inspection (or disabled entirely during Tactical Reset).

### 4.1 Expanded Bezel Command Surface

From any level, tap the bottom bezel or swipe up from the bottom edge to open the **Expanded Bezel Command Surface**. This overlay brings the full prompt — with chip columns, AI skills chips, and warning chips — to the foreground, while the current view zooms back slightly. You can run commands from Level 3 without leaving your application.

---

## 5. Slot Architecture: Bezel Docking & Projection

The UI is partitioned into **Bezel Segments** with modular slots.

### Sidebar Slots (Left/Right)

Components like the **Minimap**, **Priority Alert Section**, and **Mini-Log** can be docked into lateral slots.

- **Bezel Projection:** Clicking a bezel segment expands its associated docked component inward into the viewport without shifting the stable bezel frame.

### Top Bezel Segments

Specifically partitioned for high-frequency awareness:
- **Left (Handles + Context):** Screen title and hierarchy level mapping. Expand/collapse handle for the left lateral segment.
- **Center (Telemetry):** Real-time Brain clock and system performance metrics. Configurable component slots.
- **Right (Controls):** Global toggles, Settings Access, Web Portal satellite button, and the **[?] Help Badge**.

---

## 6. Trust & Dangerous Commands

TOS does not decide what is dangerous — you do. When you stage a command in a **WARN** class (e.g., `sudo`, `rm -r` with many files), a non-blocking warning chip appears above the prompt:

```
⚠  sudo apt remove nginx          [Trust Class]
   Privilege escalation — runs as root
```

You can press Enter immediately and the command runs. The chip is information, not a gate. Tapping **[Trust Class]** permanently promotes that command class so the chip never appears for it again. You can revert trust at any time in **Settings → Security → Trust**.

---

## 7. Split Viewports

Any pane can be split to run multiple terminals or applications side by side:

| Action | Shortcut |
|---|---|
| Split focused pane (auto-orientation) | `Ctrl+\` |
| Move focus between panes | `Ctrl+Arrow` |
| Close focused pane | `Ctrl+W` |
| Equalize pane weights | Double-click any divider |

Split orientation is determined automatically by your display's aspect ratio (landscape = vertical split, portrait = horizontal split). You can override with `Shift+Ctrl+\`.

Pane management (fullscreen, swap, detach to sector, save layout as template) is available via the Expanded Bezel Command Surface when at Level 3.

---

## 8. AI Skills System

TOS ships with three built-in AI skills:

- **Passive Observer** — Watches your terminal silently. If a command fails, it surfaces a correction chip. If a command runs too long, it offers an explanation chip. Always passive, never intrusive.
- **Chat Companion** — Provides a full chat interface in `[AI]` mode. Ask anything in plain English; the AI stages commands for you to review and submit.
- **Vibe Coder** — Accepts a natural language intent ("add error handling to session loader") and decomposes it into a reviewable chip sequence of file edits and commands. Each step requires your approval before it runs. Disabled by default — enable in **Settings → AI → Skills**.

Additional AI skills and backends can be installed from the Marketplace. All AI is removable via **Settings → AI → Skills**.

**Safety guarantee:** The AI never executes a command. Every suggestion ends up staged in the prompt — visible, editable, under your control.

---

## 8a. The TOS Editor

The TOS Editor is a code and text viewer/editor that lives alongside your terminal as a split pane. It is not a separate application — it is part of the Command Hub.

**In Level 2 (Command Hub):** The editor occupies a split pane next to the terminal. It automatically opens the relevant file when your terminal produces an error with a file path and line number. When you type a file path in the prompt, the editor previews it before you run the command.

**In Level 3 (Application Focus):** Promote the editor to full screen via the `[⊞]` button in the pane header. The Left Bezel shows a file tree; the Right Bezel shows the AI Context Panel.

**AI integration:** The editor is always live context for the AI. Every visible file, cursor position, and error annotation is automatically included in AI queries — you never have to paste code into a chat window.

**Vibe Coder + Editor:** When Vibe Coder proposes a file edit, the editor switches to Diff Mode showing the proposed change. Tap **[Apply]** to commit or **[✕]** to reject. Multi-file edits are shown as a chip sequence — approve each step individually, even across devices.

**Shortcuts:**

| Shortcut | Action |
|---|---|
| `edit <path>` in prompt | Open file in Editor Mode |
| `view <path>` in prompt | Open file in Viewer Mode |
| `Ctrl+E` | Toggle focus between terminal and editor panes |
| `Ctrl+S` | Save file |
| `Ctrl+Shift+S` | Save As |

---

## 9. Working with Workflows & Agent Orchestration

TOS includes a powerful workflow management system that lets you organize multi-step tasks, assign AI agents to execute them, and learn from completed work.

### Opening a Project's Kanban Board

A project is a directory containing code and a `.tos/` metadata folder.

```bash
# Navigate to your project
$ cd ~/projects/tos-desktop

# Open the kanban board (replaces the sector's terminal view with Workflow Manager)
$ tos workflow open

# Or:
$ Ctrl+Shift+W to focus the Workflow Manager pane
```

### Understanding the Kanban Board

The kanban board shows tasks in columns (lanes):

```
BACKLOG │ PLANNED │ WIP │ BLOCKED │ REVIEW │ DONE

BACKLOG:   Work that hasn't been scheduled yet
PLANNED:   Ready to start, waiting for bandwidth
WIP:       Currently in progress (agent is working)
BLOCKED:   Agent hit a problem, waiting for your input
REVIEW:    Agent finished, waiting for your code review
DONE:      Work approved and complete
```

**Customizing lanes:** Settings → Workflows → Board Lanes

### Creating Tasks

Tasks are work items on the kanban board. You can create them:

**Option 1: Manual (create a task directly)**
```
In Workflow Manager, drag the [+] button to the Backlog lane.
Enter: Title, description, acceptance criteria, tags.
```

**Option 2: Roadmap Skill (auto-generate from GitHub/issues)**
```
Type: "Plan v0.5 from GitHub issues"

Roadmap Planner will:
- Fetch GitHub issues
- Group by epic
- Create tasks in Backlog
- Suggest suitable agents
```

**Option 3: YAML file (.tos-task)**
Create `roadmap.tos-tasks`:
```yaml
version: "1.0"
roadmap_id: "v0.5"

tasks:
  - id: task_001
    title: "Fix borrow checker"
    description: "Line 142 of session.rs needs error handling"
    depends_on: []
    tags: ["backend", "critical"]
    acceptance_criteria:
      - "cargo check passes"
      - "tests pass"
      - "Code reviewed"
```

Then: Workflow Manager → [Import tasks] → `roadmap.tos-tasks`

### Assigning Tasks to Agents

When you move a task to the WIP lane, assign an agent.

**Available agents:**
- `careful_bot` — slow but thorough (test-first, validates everything)
- `fast_bot` — quick iteration (large steps, parallel testing)
- `creative_bot` — exploratory work (suggests alternatives)
- Custom personas you've created or installed

```
Right-click task card → Assign agent → Select "careful_bot"
```

### Auto-Accept Setting

By default, agents propose commands but wait for your approval. You can enable auto-accept per task:

```
Task → [Settings] → Auto-Accept: ON

Now commands will execute immediately (you can still pause/inspect).
```

### Watching an Agent Work

Once assigned, an agent:

1. **Decomposes the task** — reads your task description, loads its persona (strategy), generates a step-by-step plan
2. **Shows the plan** — displays: "I'll do this in 5 steps. Proceed?"
3. **Executes each step** — runs commands, observes output, decides next step
4. **Reports progress** — shows which step it's on, what's happening

**Your workflow:**
```
Terminal pane (main work):
$ your commands here

Workflow Manager pane:
┌─ @careful_bot (task: Fix borrow checker)
│  Step 2/5: cargo clippy --all-targets
│  [⏸ Pause] [→ Next] [⏹ Abort]
│  $ Checking brain/session.rs...
└─ Agent observation: "Warning detected, continuing to step 3..."
```

### Pausing & Inspecting

If something looks wrong, pause the agent:

```
Click [⏸ Pause] in the agent's terminal

The agent stops. You can:
- [Inspect]: View full command output, LLM reasoning
- [Retry]: Retry the current step with a different approach
- [Suggest]: Tell the agent a different approach
- [Skip]: Skip this step, move to next
- [Abort]: Abort the entire task
```

### Agent Reasoning (LLM History)

Every step includes the **agent's reasoning** (what the LLM decided):

```
Step 1: Read error and context [✓ Done]
┌────────────────────────────────────────┐
│ Agent observation (LLM reasoning):     │
│ "The function returns Session directly,│
│ but the caller expects Result<Session>.│
│ This is the mismatch."                │
│                                        │
│ [Show full command output]             │
└────────────────────────────────────────┘
```

Expand any step to see the full LLM conversation. This is useful for:
- Understanding why the agent made a decision
- Learning from the agent's analysis
- Auditing the work

### Multi-Agent Work

When multiple tasks are in WIP with different agents:

```
Tab bar: [@careful task_001] [@fast task_002] [@creative task_003]

Click each tab to see that agent's work.
Or drag to split view to see multiple agents simultaneously.
```

Agents work in **parallel** — they don't block each other.

### Resuming Work Across Sessions

If you close TOS while an agent is working:

```
You close TOS while @careful_bot is on step 2/5.

Later, you reopen:
→ Workflow Manager shows the kanban board
→ task_001 is still in WIP with @careful_bot
→ [Resume] chip appears: "Resume @careful_bot on Fix borrow checker?"

Click [Resume] → agent continues from step 3 with full context
```

The agent remembers:
- What it discovered in steps 1-2
- What it was planning to do
- All previous LLM reasoning

### Code Review Before Merge

When an agent finishes, the task moves to REVIEW lane:

```
Right-click task → View task history

Shows:
- All steps executed
- Full LLM reasoning
- Files changed (link to diff)
- Test results
```

Review the code, then:
- **[Approve]** → task moves to DONE
- **[Request Changes]** → move back to WIP with notes

### Project Memory & Learning

After completing tasks, consolidate your project's learnings:

```
$ tos dream consolidate ~/projects/tos-desktop

Processing 5 completed tasks...
✓ task_001: Fix borrow checker → pattern: lifetime_error_resolution
✓ task_002: Optimize PTY perf → pattern: buffer_pooling_effectiveness
...

Project memory updated: .tos/memory/project_memory.md
```

Future agents will:
- Read your project memory
- Learn from past successful decompositions
- Suggest similar approaches for similar problems
- Get better (faster, more accurate) over time

### Viewing Project Memory

```
Settings → Workflows → Project Memory

Or open the file directly:
~/projects/tos-desktop/.tos/memory/project_memory.md
```

Project memory includes:
- **Quick patterns** — common problems + solutions
- **Completed tasks index** — all past work
- **Cross-task dependencies** — how tasks relate
- **Emergent recommendations** — lessons learned
- **Metrics** — project statistics (speed, test coverage, agents used)

---

## 10. Multi-Sensory Interface

TOS uses immersive feedback loops to minimize cognitive load:

- **Earcons** — Distinct audio cues for mode switches, level zooms, modal actions, and data commits.
- **Haptic Pulses** — Physical confirmation of virtual actions on supported hardware.
- **Alert Levels** — Green (normal), Yellow (caution — ambient audio shifts), Red (critical — repeating tone, haptic escalation).

All audio and haptic feedback is configurable in **Settings → Interface → Audio** and can be disabled entirely.

---

## 11. Global Shortcuts

| Shortcut | Action |
|---|---|
| `Ctrl + [` | Zoom out (move up one level) |
| `Ctrl + ]` | Zoom in (move down one level) |
| `Ctrl + Space` | Expand/Collapse the Top Bezel |
| `Ctrl + /` | Switch to AI mode |
| `Ctrl + T` | Create a new sector |
| `Alt + [1-9]` | Switch between first 9 sectors |
| `Ctrl + M` | Show/Hide the Tactical Mini-Map |
| `Ctrl + \` | Split focused pane |
| `Ctrl + W` | Close focused pane |
| `Ctrl + Alt + Backspace` | Trigger Tactical Reset (Level 4 God Mode) |
| Long-press / Right-Click | Open Secondary Select context menu on any chip |

All shortcuts can be remapped in **Settings → Interface → Keyboard**.

---

## 12. Configuration: System Settings

Access the **System Settings** modal (⚙ icon, Top Bezel Right) to adjust:

1. **Appearance** — Theme, Terminal Output Module, font size, color palette.
2. **AI** — Backend selection, skills, ghost text, disable master switch.
3. **Editor** — Auto-open triggers, font size, minimap, LSP integration per language.
4. **Security** — Trust configuration for command classes, per-sector overrides, deep inspection toggle.
5. **Interface** — Audio/haptic feedback, animation speeds, Expanded Bezel behaviour, split viewport snap settings.
6. **Network** — Remote access port, mDNS advertisement, view port map.
7. **Sessions** — Import/export session files, browse named sessions.
8. **System** — Default shell, sandboxing tiers, resource limits per sector.
9. **Onboarding** — Replay the guided tour, reset hints, suppress hints.

---

## 13. Session Persistence

TOS automatically saves your workspace state continuously. When you return, your sectors, terminals, histories, and AI chat are exactly where you left them — with no restore notification, no animation, no prompt.

**Named Sessions** allow you to save and recall distinct workspace states per sector (e.g., "rust-project", "client-work"). Save via: secondary select on a sector tile → **Save Session As...**

Session files (`.tos-session`) are portable — copy them to another machine and load them directly via **Settings → Sessions → Import** or by dropping the file onto a sector tile.

---

## 14. Deep Inspection & Recovery (LVL 4)

Level 4 provides three sub-views:

- **Detail View** — Structured metadata: CPU/memory, event history, config, security audit.
- **Buffer View** — Hex dump of the target process's memory (read-only, disabled by default, requires privilege elevation).
- **Tactical Reset (God Mode)** — Low-overhead wireframe diagnostics of the entire system. Press `Ctrl+Alt+Backspace` from anywhere, or use the bezel button.

During Tactical Reset, the prompt is locked and the Expanded Bezel is disabled. Force Kill and other destructive actions require re-authentication. Remote guests cannot initiate or interact with Tactical Reset.

---

*TOS Terminal On Steroids // User Manual*
