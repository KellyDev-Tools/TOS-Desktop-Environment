# TOS Alpha-2.2 Ambient AI & Co-Pilot System
### Specification v1.0 — Supplement to Architecture Specification & Ecosystem Specification

---

## 1. Philosophy

AI in TOS is not a room you walk into. It is a layer that runs underneath everything, watching, learning context, and surfacing help exactly when it is useful — then getting out of the way.

The existing `[AI]` mode is preserved and extended into a full chat interface. But the ambient intelligence that predicts commands, reacts to errors, and surfaces suggestions operates independently of any mode switch. A user who never opens `[AI]` mode still benefits from the co-pilot layer.

Three principles govern all AI design in TOS:

- **STAGE, NEVER RUN.** The AI never executes a command without the user submitting it from the prompt. Every suggestion, correction, and prediction ends up staged — visible, editable, and under user control.
- **PLUGGABLE BY DEFAULT.** The LLM powering the system and the behaviors it exhibits are independent, swappable modules. The user is never locked into one model or one interaction style.
- **REMOVABLE.** All AI behavior wrappers, including the defaults, can be uninstalled or disabled. A user who wants zero AI involvement gets zero AI involvement.

---

## 2. Architecture Overview

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

- **AI Backend modules** (`.tos-ai`) define the LLM connection — what model, what endpoint, what auth. Multiple backends can be installed simultaneously. One is designated the **system default**, selected in **Settings → AI → Backend**. Any behavior module can override this and target a specific installed backend instead.
- **AI Behavior modules** (`.tos-aibehavior`) define how the AI acts, when it speaks, and what UI it renders. Multiple behavior modules can run simultaneously. Each is independently toggled and configured in **Settings → AI → Behaviors**, including which backend it uses.

### Backend Resolution Order

The `AIService` resolves which backend to use for each request using a simple cascade:

1. **Behavior-level override** — if the behavior module has a specific backend set in its config, use that.
2. **System default** — if no override is set, use the system default backend.

This means a power user can run Ollama locally for fast ghost-text predictions, route their chat companion to GPT-4, and let the passive observer fall back to the system default — all simultaneously, with no conflicts.

The Brain's `AIService` brokers all communication between behavior modules and the active backend. Behavior modules never call the backend directly — they submit requests to the `AIService` via IPC, which queues, prioritizes, and routes them.

---

## 3. AI Backend Modules (`.tos-ai`) — Unchanged + Extended

The existing `.tos-ai` module spec (Ecosystem Specification §1.3) is preserved. The following extensions apply:

- Backends now declare a `capabilities` field that behavior modules can query before making requests, preventing incompatible calls (e.g. a vision-capable behavior won't call a text-only backend).
- Backends declare a `latency_profile` hint (`local` / `fast_remote` / `slow_remote`) that behavior modules use to decide whether to show a loading state or fire-and-forget.

```toml
[capabilities]
chat             = true
function_calling = true
vision           = false
streaming        = true
latency_profile  = "local"   # local | fast_remote | slow_remote
```

---

## 4. AI Behavior Modules (`.tos-aibehavior`) — New Module Type

Behavior modules are a new module type packaged as `.tos-aibehavior` files. They define a specific AI interaction pattern and own a specific region of the UI surface.

### 4.1 Manifest Structure

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
chip_color  = "secondary"         # primary | secondary | accent — distinguishes AI chips
runs_always = true                # survives mode switches

[capabilities_required]
function_calling = false
streaming        = true
vision           = false

[permissions]
terminal_read  = true    # Can read terminal output buffer
prompt_read    = true    # Can read current prompt contents
prompt_write   = false   # Cannot write to prompt directly (Brain stages on behalf)
network        = false
```

### 4.2 UI Surfaces

Each behavior module owns exactly one UI surface type. The surface type determines where and how the AI's output appears:

| Surface | Description | Default Color |
| :--- | :--- | :--- |
| `chips` | AI-suggested chips in the left/right chip layout. Visually distinct via `secondary` color scheme to differentiate from system chips. | Teal / cyan accent |
| `ghost_text` | Inline ghost text rendered in the prompt, dimmed, accepted with `Tab` or `→`. | 40% opacity prompt color |
| `chat_panel` | Replaces or augments the `[AI]` mode panel with a full chat interface. | Standard panel |
| `thought_bubble` | Floating dismissable card that appears above the prompt. Can be tapped to expand into chat. | Glassmorphism dark card |

> **VISUAL IDENTITY:** AI chips always render in the secondary color (teal/cyan by default, themeable). This makes them immediately distinguishable from system-generated chips (directory entries, process chips) at a glance, without any labeling required.

### 4.3 The Behavior API

Behavior modules communicate with the Brain's `AIService` via IPC using the existing JSON-RPC format extended with behavior-specific fields:

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

The Brain receives this response and instructs the Face to render the chips. The behavior module never touches the Face directly.

---

## 5. Default Behavior Modules (Shipped with TOS)

Two behavior modules ship pre-installed with TOS. Both are removable and replaceable via the marketplace.

### 5.1 Passive Observer (`tos-observer`)

Watches the terminal output buffer and the prompt passively. Surfaces contextual AI chips in the secondary color when it detects actionable moments. Never interrupts. Never opens a panel unprompted.

**Trigger conditions:**

| Condition | What It Surfaces |
| :--- | :--- |
| Exit code `127` (command not found) | Correction chip: closest matching command staged |
| Non-zero exit code with stderr output | "Explain error" chip + suggested fix chip |
| Partial command typed, 1.5s idle, unsubmitted | Ghost text completion (if Predictor also installed) or chip suggestion |
| Long-running command exceeds 30s | "Explain what this is doing" chip + "Cancel" chip |
| `cd` into directory with no prior visits | "What's in here?" chip (summarizes directory structure) |

**Chip appearance:** chips render in the secondary color with a subtle `✦` prefix glyph to identify them as AI-sourced. They occupy the right chip column by default, below system chips, so they never displace existing context.

**Settings:** `Settings → AI → Behaviors → Passive Observer`
- Toggle on/off
- Trigger sensitivity: Low / Medium / High (controls idle timeout, error threshold)
- Chip position: Right column / Left column / Both

### 5.2 Chat Companion (`tos-chat`)

Provides the full chat interface within `[AI]` mode. When the user switches to `[AI]` mode, the terminal output area is replaced by a scrollable chat panel. The prompt remains the input. Conversation history is maintained per-sector per-session.

**Chat panel features:**
- Full streaming responses with cursor animation
- Code blocks in responses are rendered with a `[Stage →]` button that appends the code/command to the prompt
- Conversation context includes: current directory, last 20 terminal lines, active sector name, current shell
- `[Clear]` button resets conversation history for the current sector
- Switching away from `[AI]` mode preserves conversation history — returning resumes where the user left off

**Difference from current spec:** the chat panel is no longer a hardcoded mode behavior. It is a behavior module. This means it can be replaced by a marketplace alternative (e.g. a specialized DevOps chat companion, a Git expert, a documentation assistant) without changing any Brain or Face code.

---

## 6. Marketplace Behavior Module Types

The following behavior wrapper archetypes are defined as valid `.tos-aibehavior` module types. Third-party developers can build and publish any of these:

### 6.1 Command Predictor

- **Surface:** `ghost_text`
- **Trigger:** `prompt_input` — fires on every keystroke with debounce
- **Behavior:** Renders a dimmed ghost text completion inline in the prompt. `Tab` or `→` accepts the full suggestion. `Esc` dismisses.
- **Context sent to backend:** current prompt text, cwd, last 5 commands run, shell type
- **Latency requirement:** must respond within 300ms or ghost text is suppressed for that keystroke cycle. Modules should declare `latency_profile = "local"` and warn users if connecting to a slow remote backend.

```
user types:  git che
ghost shows: git checkout feature/my-branch█
```

### 6.2 Workflow Agent

- **Surface:** `thought_bubble` + `chat_panel`
- **Trigger:** `manual` — activated explicitly by the user (dedicated keybind or chip)
- **Behavior:** Can plan and stage multi-step command sequences. Each step is presented as an ordered chip list with a tactile confirmation before any step is staged. The agent explains each step before staging it.
- **Permissions required:** `prompt_write = false` (Brain stages on behalf), `terminal_read = true`
- **Safety contract:** Workflow agents MUST use the Brain's tactile confirmation API for any command that modifies the filesystem, network, or process state. The Brain enforces this — any `stage_command` call from a workflow agent for a flagged command class is automatically wrapped in confirmation.

Example interaction:
```
User: "set up a new rust project called torpedo in ~/projects"

Agent proposes:
  Step 1 of 4: cd ~/projects          [Stage]
  Step 2 of 4: cargo new torpedo      [Stage]
  Step 3 of 4: cd torpedo             [Stage]
  Step 4 of 4: git init               [Stage]
```

### 6.3 Domain Expert

- **Surface:** `chips` or `thought_bubble`
- **Trigger:** `passive` — activates based on detected context (e.g. presence of a `Dockerfile`, a `.git` directory, a `Cargo.toml`)
- **Behavior:** A specialist module with a narrowly scoped system prompt tuned to a domain. Examples: Git Expert, Docker Expert, Kubernetes Navigator, SQL Analyst.
- **Context detection:** declared in manifest as `context_signals` — a list of files or environment variables that activate the module.

```toml
[behavior]
trigger          = "passive"
context_signals  = [".git", "Makefile", "Cargo.toml"]
ui_surface       = "chips"
```

### 6.4 Thought Bubble Companion

- **Surface:** `thought_bubble`
- **Trigger:** `passive` — always watching
- **Behavior:** A floating, dismissable card that appears in the corner of the terminal when the AI has something to say. Unlike passive observer chips (which are small and contextual), thought bubble companions are more conversational — they can initiate, ask clarifying questions, and be expanded into a full chat.
- **Appearance:** Glassmorphism dark card with a cloud/bubble shape indicator. A small pulse animation indicates new content. Tap/click expands to full chat panel. `[×]` dismisses for the current session. Long-press `[×]` dismisses permanently until re-enabled.

---

## 7. AI Mode ([AI]) — Extended

The `[AI]` mode is preserved as a first-class Command Hub mode. Its behavior is now driven by whichever Chat Companion behavior module is active. The mode itself is a surface contract, not an implementation.

**What changes:**
- Switching to `[AI]` mode now invokes the active Chat Companion module's `on_mode_enter` callback, which renders into the `chat_panel` surface.
- If no Chat Companion module is installed, `[AI]` mode falls back to a minimal built-in text interface with a notice: "Install a Chat Companion from the Marketplace for a full experience."
- The mode switch earcon (`nav_switch`) plays as before.

**What stays the same:**
- `[AI]` mode is one of three Command Hub modes: `[CMD]`, `[SEARCH]`, `[AI]`
- Mode switching via the top chip buttons or natural language detection
- The prompt remains the input surface in all modes

---

## 8. Context Passed to All Behavior Modules

The `AIService` maintains a rolling context object that is automatically included with every behavior module request. Modules do not need to collect this themselves:

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
  "env_hints": ["RUST_LOG=info", "NODE_ENV=development"]
}
```

Modules declare which context fields they consume in their manifest under `[context_required]`. The `AIService` only sends declared fields, minimizing token usage for backends that charge per token.

---

## 9. Settings Integration

**Settings → AI** gains a new top-level structure:

```
Settings → AI
├── Backend
│   ├── System Default:  [Ollama (local) ▾]
│   ├── Installed:       Ollama  |  OpenAI  |  Anthropic
│   └── Manage Backends → (opens marketplace filtered to .tos-ai)
├── Behaviors
│   ├── Passive Observer
│   │   ├── [●  ON]  [Remove]
│   │   ├── Backend: [System Default ▾]
│   │   └── Trigger Sensitivity: [Medium ▾]
│   ├── Chat Companion
│   │   ├── [●  ON]  [Remove]
│   │   ├── Backend: [OpenAI (gpt-4o) ▾]   ← behavior-level override
│   │   └── Conversation Memory: [Per Session ▾]
│   ├── Command Predictor
│   │   ├── [●  ON]  [Remove]
│   │   ├── Backend: [Ollama (local) ▾]     ← pinned to fast local model
│   │   └── Max Latency: [300ms ▾]
│   └── + Add Behavior → (opens marketplace filtered to .tos-aibehavior)
└── Global
    ├── AI Chip Color:        [Secondary (teal) ▾]
    ├── Ghost Text Opacity:   [40% ▾]
    ├── Disable All AI:       [ ] (master off switch)
    └── Context Sent:         [Standard ▾] (Standard / Minimal / Full)
```

---

## 10. IPC Contracts — New Messages

The following IPC messages are added to support the AI behavior system:

| Message | Effect |
| :--- | :--- |
| `ai_behavior_enable:<id>` | Enables a behavior module by ID |
| `ai_behavior_disable:<id>` | Disables a behavior module by ID |
| `ai_behavior_configure:<id>:<json>` | Updates a behavior module's config |
| `ai_chip_stage:<command>` | Stages an AI-suggested chip command into the prompt |
| `ai_chip_dismiss:<chip_id>` | Dismisses an AI chip without staging |
| `ai_thought_expand` | Expands the active thought bubble into chat panel |
| `ai_thought_dismiss` | Dismisses the thought bubble for current session |
| `ai_thought_dismiss_permanent` | Dismisses thought bubble permanently (adds to suppressed list) |
| `ai_context_request` | Face requests current AI context object from Brain |
| `ai_backend_set_default:<id>` | Sets the system default backend |
| `ai_backend_set_behavior:<behavior_id>:<backend_id>` | Sets a backend override for a specific behavior module |
| `ai_backend_clear_behavior:<behavior_id>` | Removes the override, returns behavior to system default |

---

## 11. Integration with Existing Architecture

This specification extends but does not replace the existing AI module spec in Ecosystem Specification §1.3. Changes are additive:

- **Existing `.tos-ai` modules:** fully compatible. The new `capabilities` and `latency_profile` fields are optional; existing modules without them are treated as `capabilities = {chat: true}` and `latency_profile = "fast_remote"`.
- **Brain `AIService`:** gains a behavior module registry, a context aggregator, and a request queue/router. The existing `ai_query` and `ai_tool_call` IPC messages are preserved as the internal protocol between `AIService` and backends.
- **Face:** gains the `ghost_text` prompt overlay, `thought_bubble` component, and secondary-color chip rendering. The `chat_panel` surface is the existing `[AI]` mode panel, now controlled by the Chat Companion module.
- **Marketplace:** gains a new filterable category `AI Behaviors` alongside the existing `AI Backends`.

---

## 12. Safety Contracts

- **No auto-execution.** No behavior module may call `prompt_submit` directly. All command staging goes through `ai_chip_stage` or `stage_command`, which places text in the prompt without submitting. The user always presses Enter.
- **Workflow agent confirmation.** Any workflow agent staging a command in the classes `filesystem_write`, `network`, `process_kill`, or `privilege_escalation` must route through the Brain's tactile confirmation API. The Brain enforces this regardless of module implementation.
- **Context minimization.** Behavior modules only receive the context fields they declare in their manifest. A module that does not declare `terminal_read = true` never receives terminal buffer content.
- **Backend isolation.** Behavior modules never communicate with the LLM backend directly. All requests route through `AIService`. A misbehaving behavior module cannot exfiltrate context to an unauthorized endpoint.

---

*TOS Alpha-2.2 // Ambient AI & Co-Pilot Specification v1.0 // Supplement Document*
