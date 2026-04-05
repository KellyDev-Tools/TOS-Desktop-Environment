# TOS Specification Patch: Kanban-Driven Agent Orchestration & Project Memory

**Version:** 1.0  
**Date:** 2025-04-05  
**Target Specs:** TOS beta-0 Architecture, Features, Ecosystem, User-Manual  
**Status:** Ready for Integration

---

## Overview

This patch introduces:
1. **Project-level kanban boards** with user-defined lanes and auto-promotion rules
2. **Agent personas** (markdown-based, composable, discoverable)
3. **Multi-sector collaboration** on shared projects
4. **LLM interaction archival** (complete reasoning preserved)
5. **Dream consolidation** (synthesized project memory from completed tasks)
6. **Multi-agent terminal multiplexing** with isolated execution contexts

---

## File-by-File Integration Guide

| Spec File | Changes | Impact |
|---|---|---|
| **Features.md** | Add §7 (new), Update §2.2.2, §2.9 | Core workflow feature spec |
| **Ecosystem.md** | Add §1.6, §1.7 (new), Update §1.4 | Agent personas, roadmap skill |
| **Architecture.md** | Add §10.1 (subsection), §30.8 (new) | Project context, IPC contracts |
| **User-Manual.md** | Add new section on workflows | User-facing documentation |
| **Developer.md** | Add section on agent development | For extension authors |

---

# FEATURES.md PATCHES

## Patch 1: Update §2.2.2 Session Persistence Schema

**Location:** Features.md, §2.2.2 "Separation from Settings Daemon"

**Replace existing table with:**

```markdown
### 2.2.2 Separation from Settings Daemon

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
```

---

## Patch 2: Update §2.3.2 Session File Schema

**Location:** Features.md, §2.3.2 "Schema"

**Add to the sector object in the schema (after `"pinned_chips"`)**:

```json
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
```

---

## Patch 3: Expand §2.9 Editor Pane Persistence

**Location:** Features.md, §2.9 "Editor Pane Persistence"

**After the existing editor_panes schema, add:**

```markdown
### 2.9.1 Workflow & Agent State Persistence

In addition to editor panes, the session file preserves active workflow state:

```json
{
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
```

---

## Patch 4: New §7 - Kanban-Driven Agent Orchestration

**Location:** Features.md, after §6 "TOS Editor"

**Insert the following new section:**

```markdown
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

### 7.7 Multi-Agent Concurrency

Multiple agents can work on different tasks simultaneously. Each operates independently in its own terminal context.

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

---

# ECOSYSTEM.md PATCHES

## Patch 1: Update §1.4 AI Skill Modules - Add Agent Role

**Location:** Ecosystem.md, §1.4 "AI Skill Modules"

**After §1.4.2 Manifest, add new section:**

```markdown
#### 1.4.2.1 Agent Role (New in this version)

Skills with `role = "agent"` have additional capabilities for workflow orchestration:

```toml
name = "workflow_orchestrator"
version = "1.0.0"
type = "skill"
role = "agent"
description = "Coordinates multi-step task execution and agent decomposition"

[agent_capabilities]
can_spawn_subtasks = true           # Can invoke other skills
can_switch_backend = true           # Can override backend per step
can_manage_workflow = true          # Can read/write workflow definitions
requires_approval_for_tools = ["write_file", "exec_cmd"]

[tool_bundle]
allowed_tools = [
  "read_file", "write_file", "exec_cmd", "list_dir",
  "get_terminal_output", "get_sector_context", "search_codebase",
  "workflow_task_decompose", "workflow_task_step_next"
]
```

Agent skills:
- Can read `.tos-workflow` and `.tos-task` files
- Can invoke other skills as subtasks
- Can manage step-by-step task execution
- Can override AI backend per step (local → cloud or vice versa)
- Still respect user approval: staging (not auto-executing) commands
```

---

## Patch 2: New §1.6 - Agent Persona Modules

**Location:** Ecosystem.md, after §1.5 "Terminal Output Modules"

**Insert new section:**

```markdown
### 1.6 Agent Persona Modules (`.tos-persona`)

Agent Personas are markdown-based strategy definitions that AI agents read and interpret. They define how an agent approaches task decomposition, error handling, and decision-making.

#### 1.6.1 Persona Format & Discovery

Personas live in `~/.local/share/tos/personas/` and are discoverable by the system. Any persona file (markdown) is readable by an AI model without compilation or special parsing.

```markdown
# Agent Persona: careful_bot

## Identity
- **Name:** careful_bot
- **Role:** Methodical, thorough, risk-averse
- **Best for:** Critical path work, security-sensitive code, test-driven development

## Core Strategies

### Testing Strategy
- **Rule:** Always run full test suite before advancing
- **Implementation:** After any file write, run full test suite. Fail if any test fails.

### Error Handling
- **Rule:** Halt on first error, report full context
- **Implementation:** On exit code ≠ 0, pause and surface error to user.

### Step Sizing
- **Rule:** Many small steps (validate each in isolation)
- **Implementation:** Limit file edits to <50 lines per step.

## Tool Bundle
- `read_file`, `write_file`, `exec_cmd`, `list_dir`
- `run_tests`, `git_*`

## Backend Preference
- **Preferred:** Local Ollama (fast iteration)
- **Fallback:** OpenAI GPT-4 (complex reasoning)

## Learned Patterns
- Stores observations in `~/.local/share/tos/personas/careful_bot/patterns.json`
- Learns which test failures are common
- Learns which code patterns are idiomatic vs. fragile
```

**Built-in personas:**
- `careful_bot.md` — test-first, thorough validation
- `fast_bot.md` — large steps, parallel validation
- `creative_bot.md` — suggest alternatives, explore

#### 1.6.2 Persona Composition

Users compose custom personas by mixing strategies:

```markdown
# Agent Persona: balanced_bot

Combines:
- Testing from **careful_bot** (run tests, but only affected ones)
- Step sizing from **fast_bot** (larger chunks, faster iteration)
- Error handling from **creative_bot** (suggest alternatives)
```

#### 1.6.3 Persona Storage & Learned Patterns

Each persona can accumulate **learned patterns**:

```json
{
  "problem_type_lifetime_error": {
    "count": 5,
    "successful_approaches": [
      {
        "problem": "Function returns T instead of Result<T>",
        "solution_path": "wrap_in_result → write_test → validate",
        "success_rate": 1.0,
        "avg_steps": 4
      }
    ]
  },
  "file_pattern_session_rs": {
    "common_error_types": ["lifetime", "borrow_checker"],
    "effective_first_steps": ["read_error_context", "search_similar"]
  }
}
```

Stored in: `~/.local/share/tos/personas/<persona_id>/patterns.json`

Used in: Future task decompositions as few-shot examples to improve accuracy and speed.

#### 1.6.4 Persona Marketplace

Personas are discoverable in Settings → Workflows → Agent Personas → Marketplace.

Users can:
- Browse community personas
- Install new personas (`.tos-persona` package, if formalized)
- Edit and fork existing personas
- Export their custom personas for sharing

#### 1.6.5 Persona Manifest (Optional Formal Structure)

For marketplace distribution, personas can include a manifest:

```toml
name = "careful_bot"
version = "1.0.0"
type = "persona"
description = "Test-first, thorough validation approach"
author = "TOS Team"
icon = "careful_bot.png"

[strategies]
testing = "always_full_suite"
error_handling = "halt_and_report"
step_sizing = "many_small_steps"
output_validation = "always_verify"

[tool_bundle]
allowed_tools = ["read_file", "write_file", "exec_cmd", "run_tests", "git_*"]

[backend_preference]
preferred = "local_ollama"
fallback = "openai_gpt4"

[learning]
learns_patterns = true
pattern_storage = "~/.local/share/tos/personas/careful_bot/patterns.json"
```

---

## Patch 3: New §1.7 - Roadmap Skill System

**Location:** Ecosystem.md, after §1.6

**Insert new section:**

```markdown
### 1.7 Roadmap & Task Generation Skills (`.tos-skill`, role = "planner")

The Roadmap Skill is a built-in skill that decomposes epics/issues into kanban tasks. It can be extended via marketplace for integration with GitHub, Jira, GitLab, etc.

#### 1.7.1 Built-in Roadmap Planner

**Skill: roadmap_planner**

```toml
name = "Roadmap Planner"
version = "1.0.0"
type = "skill"
role = "planner"
description = "Decomposes issues and epics into actionable tasks"

[capabilities]
interaction_surface = "thought_bubble"
trigger = "manual"
can_create_tasks = true

[tool_bundle]
allowed_tools = [
  "read_github_issue",
  "create_task",
  "update_task",
  "assign_agent",
  "fetch_codebase_metadata"
]

[permissions]
network = true
filesystem = "read"
```

#### 1.7.2 Roadmap Planning Flow

User invokes: `"Plan v0.5 from GitHub issues"`

Roadmap Planner:
1. Fetches issues from GitHub API
2. Reads issue descriptions, labels, complexity hints
3. Groups by epic (using labels or manually)
4. Decomposes each issue into 1-5 tasks
5. Estimates effort (small/medium/large)
6. Suggests suitable agents (e.g., fast_bot for perf, careful_bot for critical)
7. Surfaces plan in thought bubble: "Create 12 tasks in Backlog?"
8. User approves → tasks created in kanban board

#### 1.7.3 Extension Points

Marketplace can provide:
- `github_roadmap_planner` — GitHub issues integration
- `jira_roadmap_planner` — Jira integration
- `gitlab_roadmap_planner` — GitLab integration
- Custom: domain-specific planners (e.g., "API design planner", "documentation planner")

---

## Patch 4: Update §1.4.3 Brain Tool Registry

**Location:** Ecosystem.md, §1.4.3 "Brain Tool Registry"

**Add to the tool table (after existing tools):**

```markdown
| `workflow_task_decompose(task_id)` | LLM decomposes task using agent persona | Always — request-response only |
| `workflow_task_step_next` | Agent advances to next step | Staged only — user confirm |
| `workflow_read_persona(persona_id)` | Agent loads its persona markdown | No |
| `workflow_read_patterns(persona_id)` | Load learned patterns from previous tasks | No |
```

---

# ARCHITECTURE.md PATCHES

## Patch 1: Add §10.1 Subsection - Project Context

**Location:** Architecture.md, §10 "Sectors and the Tree Model"

**After §10 introduction, add subsection:**

```markdown
### 10.1 Project Context & Shared Kanban Boards

Sectors can associate with a **project** to share a kanban board and collaborate with other sectors.

#### 10.1.1 Project Association

A sector optionally specifies a project context:

```json
{
  "id": "sector_laptop",
  "name": "dev-laptop",
  "project_context": {
    "project_path": "/home/user/projects/tos-desktop",
    "project_id": "tos-desktop-v0.5",
    "shared_kanban_board": true
  }
}
```

If `shared_kanban_board: true`:
- Multiple sectors can open the same project
- All sectors see the same kanban board (`.tos/kanban.tos-board`)
- Task state updates propagate across sectors in real-time
- Agents in different sectors can work on different tasks simultaneously

#### 10.1.2 Multi-Sector Synchronization

When multiple sectors reference the same project:

1. **File system watches** (`inotify`, FSEvents) detect changes to `.tos/kanban.tos-board`
2. **Brain broadcasts** task state changes via IPC to all connected sectors
3. **Face updates** kanban board view in real-time (cards move, agents show progress)

For remote sectors:
- Synchronization via TOS Remote Server protocol (§12)
- Conflicts resolved by "last write wins" or user prompt (configurable)

#### 10.1.3 Agent Isolation

Each agent operates in its own terminal context, even when multiple agents are active:

- **Isolated PTY**: Each agent's commands execute in a separate pseudo-terminal
- **Isolated output**: Terminal output is captured separately per agent
- **No contention**: Agents do not block each other (except for explicitly exclusive resources)

See §7.7 (Features) for concurrency details.
```

---

## Patch 2: Add §30.8 - Workflow IPC Contracts

**Location:** Architecture.md, §30 "UI Module Interaction APIs"

**After §30.5 (AI Context Sync API), add new section:**

```markdown
### 30.8 Workflow Management API

Messages for kanban board, task, and agent orchestration.

#### 30.8.1 Kanban Board Management (Face ↔ Brain)

| Message | Direction | Payload | Effect |
|:---|:---|:---|:---|
| `workflow_board_load:<project_path>` | Face → Brain | project_path | Load kanban board definition + tasks |
| `workflow_board_watch:<project_path>` | Face → Brain | project_path | Subscribe to real-time updates |
| `workflow_task_move:<task_id>:<lane_id>` | Face → Brain | task_id, lane_id | Move task to lane (triggers auto-promotion checks) |
| `workflow_task_update:<task_id>` | Face → Brain | JSON (title, description, agent, etc.) | Update task properties |
| `workflow_task_assign:<task_id>:<agent_id>` | Face → Brain | task_id, agent_id | Assign/reassign agent to task |
| `workflow_board_state:<project_path>` | Brain → Face | Full board state JSON | Broadcast board state update |

#### 30.8.2 Agent Orchestration (Face ↔ Brain)

| Message | Direction | Payload | Effect |
|:---|:---|:---|:---|
| `workflow_agent_start:<task_id>` | Face → Brain | task_id | Agent begins task (reads LLM decomposition or generates new) |
| `workflow_agent_step_next:<task_id>:<agent_id>` | Face → Brain | task_id, agent_id | Agent advances to next step |
| `workflow_agent_step_pause:<task_id>:<agent_id>` | Face → Brain | task_id, agent_id | Agent pauses at current step |
| `workflow_agent_step_retry:<task_id>:<agent_id>` | Face → Brain | task_id, agent_id, retry_reason | Retry current step with different approach |
| `workflow_agent_step_skip:<task_id>:<agent_id>` | Face → Brain | task_id, agent_id | Skip to next step (manual override) |
| `workflow_agent_abort:<task_id>:<agent_id>` | Face → Brain | task_id, agent_id | Abort task, move to BLOCKED |
| `workflow_agent_output:<agent_id>:<pane_id>` | Brain → Face | agent_id, pane_id | Route agent terminal to pane |
| `workflow_agent_progress:<agent_id>` | Brain → Face | step_current, step_total, status | Update agent progress (for kanban card) |

#### 30.8.3 LLM Interaction Archive (Brain ↔ Storage)

| Message | Direction | Payload | Effect |
|:---|:---|:---|:---|
| `workflow_llm_history_save:<task_id>` | Brain → Storage | Full LLM interaction object | Archive all LLM requests/responses for task |
| `workflow_llm_history_load:<task_id>` | Brain ← Storage | Full LLM interaction object | Retrieve LLM history for resuming task |
| `workflow_patterns_update:<agent_id>` | Brain → Storage | Learned patterns JSON | Update agent's learned patterns file |
| `workflow_patterns_load:<agent_id>` | Brain ← Storage | Learned patterns JSON | Load agent's patterns for new decomposition |

#### 30.8.4 Dream Consolidation (Brain → Storage)

| Message | Direction | Payload | Effect |
|:---|:---|:---|:---|
| `workflow_dream_consolidate:<project_path>` | Face → Brain | project_path | Consolidate completed tasks' LLM histories into project memory |
| `workflow_dream_update:<project_path>` | Brain → Storage | Project memory markdown | Write synthesized memory to `.tos/memory/project_memory.md` |
| `workflow_dream_query:<project_path>:<tag>` | Face → Brain | project_path, tag | Search project memory by pattern/tag |
| `workflow_memory_export:<project_path>` | Face → Brain | project_path, export_path | Export project memory as markdown file |

---

## Patch 3: Update §3 Process Architecture - Mention Agent Terminals

**Location:** Architecture.md, §3.2 "The Face (UI Thread/Process)"

**After the list of Face responsibilities, add:**

```markdown
- Instantiates terminal output areas for each active agent in the Workflow Manager pane, ensuring isolated output context per agent.
```

---

# USER-MANUAL.md PATCHES

## New Section: Workflow Management & Agent Orchestration

**Location:** User-Manual.md, after Section on "Advanced Terminal Features"

**Insert new section:**

```markdown
## Working with Workflows & Agent Orchestration

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

### Creating Custom Agent Personas

Agents are defined by their **personas** — markdown files that specify their strategy.

```
Settings → Workflows → Agent Personas → [+ New Persona]

Name: my_bot
Strategy: Mix careful testing + fast iteration
```

Or edit directly:

```markdown
# Agent Persona: my_bot

## Identity
- Name: my_bot
- Role: Balanced (thorough but fast)

## Core Strategies

### Testing
- Rule: Run affected tests (not full suite)
- Implementation: Use cargo test --lib

### Error Handling
- Rule: Suggest alternatives on first error
- Implementation: On exit ≠ 0, propose 3 approaches

### Step Sizing
- Rule: Medium-sized steps (not too small, not too large)
- Implementation: 50-200 line edits per step

## Tool Bundle
- read_file, write_file, exec_cmd, run_tests, git_*

## Backend
- Preferred: Local Ollama
- Fallback: Claude API
```

Save to `~/.local/share/tos/personas/my_bot.md`, then use when assigning tasks.

### Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| `Ctrl+Shift+W` | Focus Workflow Manager pane |
| `←/→` (in WIP) | Cycle through agent tabs |
| `Space` | Approve agent's next step |
| `P` | Pause current agent |
| `S` | Skip current step |
| `A` | Abort task |
| `Ctrl+K` | View task (kanban card) details |

### Collaboration Across Devices

Open the same project on multiple sectors (laptop, desktop, tablet):

```
Laptop:
$ tos workflow open ~/projects/tos-desktop
→ @careful_bot working on task_001

Desktop:
$ tos workflow open ~/projects/tos-desktop
→ See same kanban board
→ Start @fast_bot on task_002

Tablet:
$ tos workflow open ~/projects/tos-desktop
→ See both agents working
→ Review completed tasks
```

All sectors share the same kanban board. Changes propagate in real-time.

### Tips & Best Practices

**Start with careful_bot for critical work**
- More thorough validation
- Better for first-time changes to core modules
- Slower but safer

**Use fast_bot for feature development**
- Larger steps, quicker iteration
- Good for exploratory work
- Can refine later with careful_bot

**Review agent reasoning before merging**
- Expand each step's LLM history
- Understand why decisions were made
- Spot any assumptions the agent made

**Use custom personas for domain-specific work**
- Create `api_design_bot` for API changes
- Create `perf_bot` for optimization tasks
- Personas improve over time as they learn

**Consolidate memory regularly**
- After each sprint: `tos dream consolidate`
- Helps future agents work faster
- Builds institutional knowledge

```

---

# DEVELOPER.md PATCHES

## New Section: Building Agent Personas & Extending Workflows

**Location:** Developer.md, in a new "Extending TOS" or "Workflows" section

**Insert new section:**

```markdown
## Agent Personas & Workflow Extension

This section covers how to build custom agent personas, extend the Roadmap Skill, and integrate workflows with external systems.

### Creating a Custom Agent Persona

Agent personas are markdown files in `~/.local/share/tos/personas/`. Any markdown file is automatically discoverable.

**Minimal persona:**

```markdown
# Agent Persona: my_agent

## Identity
- **Name:** my_agent
- **Role:** [your description]

## Core Strategies

### Strategy 1 Name
- **Rule:** [What the agent should do]
- **Implementation:** [How to do it]

### Strategy 2 Name
- **Rule:** [...]
- **Implementation:** [...]

## Tool Bundle
- read_file, write_file, exec_cmd, ...

## Backend Preference
- **Preferred:** [local_ollama / openai_gpt4 / ...]
- **Fallback:** [...]
```

When an LLM decomposes a task using `my_agent`, it reads this markdown and follows the strategies.

### Learned Patterns Storage

As an agent executes tasks, it learns patterns:

```json
{
  "problem_type_your_domain": {
    "count": 3,
    "successful_approaches": [
      {
        "problem": "Description of problem",
        "solution_path": "step1 → step2 → step3",
        "success_rate": 1.0,
        "avg_steps": 5
      }
    ]
  }
}
```

Stored in: `~/.local/share/tos/personas/<agent_id>/patterns.json`

Future task decompositions include these patterns as few-shot examples, improving accuracy.

### Extending Roadmap Planning

The `roadmap_planner` skill can be extended for custom sources:

```toml
# marketplace-extension: jira_roadmap_planner

name = "Jira Roadmap Planner"
version = "1.0.0"
type = "skill"
role = "planner"
description = "Generate tasks from Jira issues"

[capabilities]
interaction_surface = "thought_bubble"
trigger = "manual"

[tool_bundle]
allowed_tools = [
  "read_jira_issue",
  "create_task",
  "assign_agent"
]

[permissions]
network = true
```

When user says: "Plan v0.5 from Jira", the skill:
- Fetches issues from Jira API
- Maps fields to `.tos-task` format
- Creates tasks in kanban board

### Workflow Task Format (.tos-task)

Tasks are YAML files defining discrete work:

```yaml
version: "1.0"
roadmap_id: "v0.5"
roadmap_name: "My Sprint"

tasks:
  - id: "task_001"
    title: "Fix bug #123"
    description: |
      Detailed description of the bug.
      What needs to be done.
    source: "github://org/repo/issues/123"
    depends_on: ["task_000"]
    tags: ["bug", "backend"]
    acceptance_criteria:
      - "Bug is fixed"
      - "Tests pass"
      - "Code reviewed"
```

Custom tools can generate `.tos-task` files from any source (GitHub, Jira, manual input, etc.).

### LLM History Archive Format

Every task's execution is archived for learning:

```json
{
  "task_id": "task_001",
  "llm_history": {
    "initial_decomposition": {
      "timestamp": "2025-04-04T10:22:00Z",
      "request": {
        "task_title": "...",
        "task_description": "...",
        "persona_md": "[full persona markdown]",
        "codebase_context": { ... }
      },
      "response": {
        "reasoning": "...",
        "steps": [ ... ]
      }
    },
    "step_interactions": [
      {
        "step_id": "step_1",
        "executed_at": "...",
        "command_executed": "...",
        "command_output": "...",
        "agent_response": "..."
      }
    ]
  }
}
```

This archive is used to:
- Resume incomplete tasks with full context
- Generate learned patterns
- Train future agents on successful decompositions
- Audit decision-making

### Project Memory Synthesis

The `dream consolidate` process reads completed task archives and generates:

```markdown
# Project Memory: [project_name]

## 🎯 Quick Patterns
- Pattern 1: problem → solution
- Pattern 2: ...

## 📚 Completed Tasks Index
- Task 1: description + decomposition + result
- Task 2: ...

## 🔗 Cross-Task Dependencies
- How tasks relate
- Which approaches worked together

## 💡 Emergent Recommendations
- Lessons learned
- Best practices discovered

## 📊 Project Statistics
- Total tasks, duration, agents used, success rate, etc.
```

Custom synthesis strategies can be provided via marketplace skills.

### IPC Contracts for Agents

Agents communicate with the Brain via IPC:

**Start task decomposition:**
```json
{
  "method": "workflow_agent_start",
  "params": {
    "task_id": "task_001",
    "agent_id": "careful_bot"
  }
}
```

**Advance to next step:**
```json
{
  "method": "workflow_agent_step_next",
  "params": {
    "task_id": "task_001",
    "agent_id": "careful_bot"
  }
}
```

See Architecture §30.8 for complete IPC contract specification.

---

```

---

# SUMMARY OF CHANGES

| File | Section | Type | Description |
|---|---|---|---|
| Features.md | §2.2.2 | Update | Add project context to session schema |
| Features.md | §2.3.2 | Update | Add active_workflow to session file schema |
| Features.md | §2.9.1 | Add | Workflow & agent state persistence |
| Features.md | §7 | **New** | **Kanban-Driven Agent Orchestration** (complete section) |
| Ecosystem.md | §1.4.2.1 | Add | Agent role for skills |
| Ecosystem.md | §1.6 | **New** | **Agent Persona Modules** (complete section) |
| Ecosystem.md | §1.7 | **New** | **Roadmap & Task Generation Skills** (complete section) |
| Ecosystem.md | §1.4.3 | Update | Add workflow tools to Brain Tool Registry |
| Architecture.md | §10.1 | **New** | **Project Context & Shared Kanban Boards** (subsection) |
| Architecture.md | §30.8 | **New** | **Workflow Management API** (complete IPC section) |
| Architecture.md | §3.2 | Update | Mention agent terminals in Face responsibilities |
| User-Manual.md | New Section | **New** | **Working with Workflows & Agent Orchestration** |
| Developer.md | New Section | **New** | **Agent Personas & Workflow Extension** |

---

# NOTES FOR INTEGRATION

1. **Order of patching:** Apply in order (Features → Ecosystem → Architecture → User-Manual → Developer) to maintain cross-references.

2. **Cross-references:** This patch introduces several new IPC messages (§30.8). The Brain implementation should register these messages.

3. **Session file version:** Bump `tos_session_version` from "1.0" to "1.1" to indicate workflow support.

4. **Backward compatibility:** Sessions without `active_workflow` should still load correctly (ignore the field if absent).

5. **Marketplace:** Agent personas and Roadmap Skill extensions can be packaged as `.tos-persona` and `.tos-skill` (marketplace distribution ready).

6. **Files to create:**
   - `~/.local/share/tos/personas/careful_bot.md`
   - `~/.local/share/tos/personas/fast_bot.md`
   - `~/.local/share/tos/personas/creative_bot.md`
   - Built-in `roadmap_planner` skill
   - Built-in `workflow_manager` skill (UI coordinator)

---

**End of Specification Patch**

Questions? References to existing TOS concepts are preserved for consistency. Ready to merge into beta-0 specs.
