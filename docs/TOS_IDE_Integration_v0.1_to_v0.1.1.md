# TOS IDE Integration & Cortex Architecture updates from TOS v0.1 to TOS v0.1.1
 
**Date:** April 30, 2026  
**Status:** Design Phase (Ready for Implementation)  
**Scope:** IDE integration strategy, Cortex expansion, File Context Pane specification

---

## Executive Summary

This document captures the architectural decisions for integrating IDEs (Zed, Vim, Neovim, Emacs, VS Code) into TOS via the **Cortex Orchestration Layer**, replacing the planned TOS Editor pane at Level 2 with a lightweight File Context Viewer and full-featured IDE orchestration at Level 3.

**Key Decision:** IDE integration is **not a separate protocol**—it is **part of Cortex**. IDEs become:
- **Curators** (IDE State Curator): Sources of editor context (file, cursor, selection, diagnostics)
- **Action Targets** (IDE Action Executor): Receive and execute orchestrated commands from agents

**Outcome:** Agent personas defined in Cortex can optionally, if IDE supported, be propagated to IDE AI layers, ensuring unified AI voice across TOS chat, IDE suggestions, and terminal completions. TOS relies on external IDEs for heavy text editing, syntax, and LSP diagnostics, significantly reducing the core codebase's maintenance burden.

---

## Table of Contents

1. [Core Architecture Decision](#1-core-architecture-decision)
2. [IDE Integration Model](#2-ide-integration-model)
3. [Cortex Expansion](#3-cortex-expansion)
4. [File Context Pane (L2 Replacement)](#4-file-context-pane-l2-replacement)
5. [IDE State Curator](#5-ide-state-curator)
6. [IDE Action Executor](#6-ide-action-executor)
7. [Agent Context Propagation](#7-agent-context-propagation)
8. [Cross-Device Session Handoff](#8-cross-device-session-handoff)
9. [Implementation Architecture](#9-implementation-architecture)
10. [IDE-Specific Integration](#10-ide-specific-integration)
11. [Workflow Example: Task-Driven Development](#11-workflow-example-task-driven-development)
12. [Specification Updates](#12-specification-updates)
13. [Implementation Roadmap](#13-implementation-roadmap)

---

## 1. Core Architecture Decision

### 1.1 Problem Statement

**Original Approach:**
- TOS would include a built-in code editor pane at Level 2
- Requires maintenance of syntax highlighting, LSP integration, multi-cursor, refactoring
- ~5,000 lines of new code; opportunity cost vs. TOS's core mission

**IDE Integration Dilemma:**
- Separate "IDE Integration Protocol" + Cortex = two orchestration systems
- Agents cannot directly invoke IDE actions
- IDE state is external to Cortex context
- Redundant infrastructure for state management and IPC

### 1.2 Decision: IDE Integration as Cortex Subsystem

**IDEs are not external tools—they are orchestration components within Cortex.**

```
┌──────────────────────────────────────────┐
│            Cortex Layer                  │
│ ┌──────────────┐      ┌────────────────┐ │
│ │ Assistants   │      │ Curators       │ │
│ │ + Agents     │      │ - Filesystem   │ │
│ │              │      │ - Terminal     │ │
│ │              │      │ - Git          │ │
│ │              │      │ - IDE State ✨ │ │
│ └──────────────┘      └────────────────┘ │
│                                          │
│ ┌──────────────────────────────────────┐ │
│ │ Action Executors                     │ │
│ │ - Terminal (shell commands)          │ │
│ │ - Workflow (agent sequencing)        │ │
│ │ - IDE (editor operations) ✨         │ │
│ └──────────────────────────────────────┘ │
└──────────────────────────────────────────┘
```

**Benefits:**
- ✅ Agents directly invoke IDE actions
- ✅ IDE state is a queryable context source
- ✅ Single IPC protocol (Cortex API)
- ✅ Scalable: new IDEs integrate without core changes

### 1.3 Scope: What TOS Does NOT Build

TOS **removes** from its responsibility:
- ❌ Full-featured code editor (syntax highlighting, multi-cursor, refactoring)
- ❌ Language-specific IDE features
- ❌ LSP Management (`.tos-language` modules are deprecated for LSP and used only for simple Level 2 syntax highlighting)

---

## 2. IDE Integration Model

### 2.1 Three-Layer Integration

```
┌──────────────────────────────────────┐
│ Layer 1: IDE Plugin/Extension        │
│ (Vim plugin, Zed extension, etc.)    │
└──────────────────────────────────────┘
↓ (IPC: sockets/shared memory)
┌──────────────────────────────────────┐
│ Layer 2: IDE Integration Service     │
│ (TOS daemon subprocess)              │
└──────────────────────────────────────┘
↓ (Cortex API: MCP/JSON-RPC)
┌──────────────────────────────────────┐
│ Layer 3: Cortex (TOS Brain)          │
└──────────────────────────────────────┘
```

### 2.2 Supported IDEs (Priority Order)

**Phase 1 (MVP):**
1. **Neovim** (Lua plugin) — simplest, terminal-native, testing ground
2. **Zed** (Rust extension) — native ecosystem, strong AI integration

**Phase 2:**
3. **Vim** (VimScript plugin) — similar to Neovim but older API
4. **Emacs** (elisp package) — Lisp community support

**Phase 3 (Community & Fallbacks):**
5. **VS Code** (TypeScript extension)
6. **Local Mobile Fallback (Native Android Editor):** A lightweight, native Android text editor that registers via the `IDE Integration Protocol` to ensure offline, local editing works when no desktop instance can be streamed to the Handheld/XR profiles.

---

## 3. Cortex Expansion

### 3.1 New Curator: IDE State Curator

**Purpose:** Expose current editor state to Cortex agents.

**Provided Context:**
| Field | Type | Purpose |
|:---|:---|:---|
| `current_file` | object | What is being edited |
| `cursor_position` | object | Where is cursor |
| `selection` | string | Selected text |
| `unsaved_changes` | object | Unsaved state (hunk counts) |
| `diagnostics` | array | Lint/error info provided entirely by the IDE |

### 3.2 New Action Target: IDE Action Executor

**Purpose:** Execute orchestrated operations in the IDE. Uses context-aware matching instead of absolute line numbers to mitigate race conditions.

**Action Types:**

| Action | Parameters | Description | Confirm? |
|:---|:---|:---|:---|
| `open_file` | `path, search_pattern?` | Open file at position | No |
| `goto_line` | `search_pattern` | Jump to match | No |
| `select_range` | `start_pattern, end_pattern` | Select text range | No |
| `insert_text` | `search_pattern, expected_context_before, expected_context_after, text` | Insert text safely | Yes |
| `replace_range` | `search_pattern, expected_context_before, expected_context_after, text` | Replace text safely | Yes |
| `open_diff` | `target_file, proposed_content` | Open IDE's native diff viewer for Vibe Coder review | Yes |
| `run_command` | `command, context?` | Run IDE command (format, lint, build) | Yes |
| `save_all` | `none` | Trigger save across all buffers | No |

**Race Condition Handling:** If an agent attempts `insert_text` but the `expected_context_before` no longer matches the active buffer (e.g., user kept typing), the IDE Plugin returns an `action_error`. The Cortex intercepts this, interrupts the agent, and triggers a re-read of the `IDEStateCurator` to try again.

---

## 4. File Context Pane (L2 Replacement)

### 4.1 Purpose

A **lightweight, read-only** viewer for Level 2 that tracks the IDE's state. It replaces the heavy TOS Editor.

### 4.2 Features

- ✅ Displays syntax-highlighted file content (using basic Tree-sitter)
- ✅ Subscribes to IDE State Curator to show real-time cursor position
- ✅ Selection → prompt integration
- ✅ IDE switcher UI
- ❌ **No editing, no LSP, no multi-cursor.** (All editing is deferred to the Level 3 IDE).

---

## 5. IDE State Curator

(See previous draft for Data Model and Polling Updates. No structural changes needed beyond adding hashed state verifications for race-condition tracking.)

---

## 6. IDE Action Executor

### 6.1 Confirmation & Safety

**Staging Philosophy:** "STAGE, NEVER RUN"

All destructive IDE actions are **staged in the prompt** before execution. 

When the Vibe Coder AI proposes a change, it executes an `open_diff` action. The IDE opens a native diff view, and the user accepts or rejects the change directly in the IDE UI or via the TOS prompt. 

---

## 7. Agent Context Propagation

### 7.1 Overview

When a Kanban task is activated, the associated Agent stack propagates to the active IDE if the IDE supports it. The IDE's AI layer receives the merged Agent system prompt and task context.

### 7.2 Unified AI Voice (Optional API)

This feature is **optional**. If an IDE (e.g., standard VS Code or an older Vim setup) does not expose an API to dynamically rewrite its system prompt, the integration simply degrades gracefully. The user continues to use the TOS Level 2 Chat Companion for the unified voice, while the IDE is used purely as a text actuator.

---

## 8. Cross-Device Session Handoff

To preserve work when jumping between devices (e.g., moving from laptop to phone), the `tos-sessiond` requires an explicit handshake with the IDE.

**The Protocol:**
1. User requests session handoff (`session_handoff:<sector_id>`).
2. Brain dispatches a `save_all` action to the `IDEActionExecutor`.
3. If the IDE saves successfully, the handoff token is generated.
4. If the IDE returns an error (e.g., a file is invalid or locked), the handoff is aborted and an amber warning chip is displayed in the prompt: `[⚠ Handoff Failed: Unsaved buffers in IDE] [Force Handoff]`.

---

## 9. Implementation Architecture

### 9.1 System Components

```
┌─────────────────────────────────────────────────┐
│            TOS Brain (Rust)                     │
│                                                 │
│  ┌──────────────────────────────────────────┐   │
│  │ Cortex Layer                             │   │
│  │ ├─ Assistants (LLM backends)             │   │
│  │ ├─ Curators                              │   │
│  │ │  ├─ Filesystem                         │   │
│  │ │  ├─ Terminal                           │   │
│  │ │  ├─ Git                                │   │
│  │ │  └─ IDE State ✨                       │   │
│  │ ├─ Agents (instruction stacks)           │   │
│  │ └─ Action Executors                      │   │
│  │    ├─ Terminal (shell)                   │   │
│  │    ├─ Workflow (agents)                  │   │
│  │    └─ IDE ✨                             │   │
│  └──────────────────────────────────────────┘   │
│                                                 │
│  IPC: Cortex API (MCP / JSON-RPC)               │
└─────────────────────────────────────────────────┘
         ↓ ↑ (socket: ide-integration.sock)
┌─────────────────────────────────────────────────┐
│   IDE Integration Service (Rust daemon)         │
│                                                 │
│  ┌──────────────────────────────────────────┐   │
│  │ IDE Plugin Bridge                        │   │
│  │ ├─ Receive state from IDE plugins        │   │
│  │ ├─ Aggregate multi-IDE state             │   │
│  │ └─ Route Cortex actions to IDEs          │   │
│  └──────────────────────────────────────────┘   │
│                                                 │
│  IPC: Plugin sockets (per IDE)                  │
└─────────────────────────────────────────────────┘
         ↓ ↑ (socket: per IDE)
┌─────────────────────────────────────────────────┐
│      IDE Instances (Zed, Vim, Neovim, etc.)    │
│                                                 │
│  ┌──────────────┐ ┌──────────┐ ┌──────────┐    │
│  │ Zed+Extension│ │ Vim+Plugin│ │Neovim etc│    │
│  │ ├─ Monitor   │ │├─ Monitor │ │├─ Monitor │   │
│  │ │  cursor    │ ││ state    │ ││ state   │    │
│  │ ├─ Listen    │ │├─ Listen  │ │├─ Listen  │   │
│  │ │  actions   │ ││ for acts │ ││ for acts│    │
│  │ └─ Execute   │ │└─ Execute │ │└─ Execute│    │
│  │    in IDE    │ │   in IDE  │ │   in IDE │    │
│  └──────────────┘ └──────────┘ └──────────┘    │
└─────────────────────────────────────────────────┘
```

### 9.2 File & Module Organization

```
TOS codebase (Rust)
├── crates/
│   ├── tos-brain/
│   │   └── src/
│   │       ├── cortex/
│   │       │   ├── curators/
│   │       │   │   ├── filesystem.rs
│   │       │   │   ├── terminal.rs
│   │       │   │   ├── git.rs
│   │       │   │   └── ide_state.rs ✨ (NEW)
│   │       │   ├── executors/
│   │       │   │   ├── terminal.rs
│   │       │   │   ├── workflow.rs
│   │       │   │   └── ide.rs ✨ (NEW)
│   │       │   └── agent_stack.rs
│   │       │
│   │       └── face/
│   │           ├── pane_types/
│   │           │   ├── terminal.rs
│   │           │   ├── file_context.rs ✨ (NEW)
│   │           │   └── ...
│   │           │
│   │           └── views/
│   │               ├── editor.rs ❌ (REMOVED)
│   │               └── ...
│   │
│   └── tos-ide-integration/ ✨ (NEW crate)
│       ├── src/
│       │   ├── lib.rs
│       │   ├── service.rs (main daemon)
│       │   ├── plugin_bridge.rs
│       │   ├── protocol.rs (IDE IPC protocol)
│       │   ├── ide/
│       │   │   ├── mod.rs
│       │   │   ├── zed.rs
│       │   │   ├── neovim.rs
│       │   │   ├── vim.rs
│       │   │   └── emacs.rs
│       │   └── action_router.rs
│       │
│       └── plugins/ ✨ (IDE plugin code)
│           ├── zed-extension/
│           │   ├── Cargo.toml
│           │   ├── src/
│           │   │   └── lib.rs
│           │   └── extension.toml
│           │
│           ├── nvim-plugin/
│           │   ├── lua/
│           │   │   ├── tos-integration/
│           │   │   │   ├── init.lua
│           │   │   │   ├── state_reporter.lua
│           │   │   │   ├── action_dispatcher.lua
│           │   │   │   └── protocol.lua
│           │   │   └── ...
│           │   └── plugin/tos-integration.lua
│           │
│           ├── vim-plugin/
│           │   ├── plugin/tos.vim
│           │   ├── autoload/tos/
│           │   │   ├── state.vim
│           │   │   ├── protocol.vim
│           │   │   └── actions.vim
│           │   └── ...
│           │
│           └── emacs/
│               ├── tos-integration.el
│               └── ...
```

### 9.3 Socket Protocol: IDE ↔ IDE Integration Service

**Socket Location:** `~/.tos/ide-{IDE_NAME}.sock` (Unix domain socket)

**Message Format:** Line-delimited JSON

**Handshake (IDE Plugin → Service):**
```json
// IDE plugin connects
{"type": "register", "ide": "zed", "version": "1.0", "pid": 1234}

// Service acknowledges
{"type": "ready", "service_version": "0.1"}
```

**State Reports (IDE Plugin → Service, continuous):**
```json
// On cursor move
{"type": "state_update", "event": "cursor_moved", "file": "src/main.rs", "line": 42, "col": 10}

// On file save
{"type": "state_update", "event": "file_saved", "file": "src/main.rs"}

// On unsaved changes
{"type": "state_update", "event": "buffer_modified", "file": "src/main.rs", "hunk_count": 3}
```

**Action Commands (Service → IDE Plugin):**
```json
// Open file at line
{"type": "action", "id": "act-42", "action": "open_file", "path": "src/main.rs", "line": 42}

// IDE plugin executes, sends back result
{"type": "action_result", "id": "act-42", "status": "success", "message": "File opened"}

// On error
{"type": "action_result", "id": "act-42", "status": "error", "error": "File not found: src/main.rs"}
```

---

## 10. IDE-Specific Integration

### 10.1 Zed Extension (Rust → WASM)

**File:** `crates/tos-ide-integration/plugins/zed-extension/src/lib.rs`

**Key Responsibilities:**
1. Monitor editor state (cursor, file, selection, unsaved)
2. Report state to IDE Integration Service over socket
3. Listen for IDE actions, execute in Zed
4. Receive agent context updates, inject into Zed's AI layer

**Implementation Sketch:**

```rust
use zed_extension_api as zed;
use std::net::UnixStream;
use serde_json::json;

pub struct TosIntegration {
    socket: Option<UnixStream>,
    active_agents: Vec<Agent>,
    active_task: Option<TaskContext>,
}

impl zed::Extension for TosIntegration {
    fn new() -> Self {
        let mut ext = Self {
            socket: None,
            active_agents: vec![],
            active_task: None,
        };
        
        // Connect to IDE Integration Service
        if let Ok(socket) = UnixStream::connect("~/.tos/ide-zed.sock") {
            ext.socket = Some(socket);
            ext.register_with_service();
        }
        
        ext
    }
    
    fn on_editor_event(&mut self, event: EditorEvent) {
        if let Some(ref mut socket) = self.socket {
            let msg = json!({
                "type": "state_update",
                "event": match event {
                    EditorEvent::CursorMoved => "cursor_moved",
                    EditorEvent::FileSaved => "file_saved",
                    EditorEvent::BufferModified => "buffer_modified",
                    _ => "unknown"
                },
                "file": event.file_path,
                "line": event.cursor.line,
                "col": event.cursor.column,
            });
            
            // Send to service
            let _ = socket.write_all(serde_json::to_string(&msg).unwrap().as_bytes());
        }
    }
    
    fn listen_for_actions(&mut self) {
        // In event loop: listen on socket for actions from TOS
        // Execute actions in Zed
    }
}
```

**Status:** Phase 1 candidate (MVP)

### 10.2 Neovim Plugin (Lua)

**File:** `crates/tos-ide-integration/plugins/nvim-plugin/lua/tos-integration/init.lua`

**Key Responsibilities:**
1. Monitor buffer/cursor state
2. Send state updates to IDE Integration Service
3. Handle incoming actions
4. Inject agent context into Neovim's AI plugin (if exists)

**Implementation Sketch:**

```lua
local M = {}
local socket = nil
local active_agents = {}
local active_task = nil

function M.setup()
  -- Connect to IDE Integration Service
  socket = vim.fn.sockconnect('unix', vim.fn.expand('~/.tos/ide-neovim.sock'))
  if socket <= 0 then
    vim.notify("TOS: Failed to connect to IDE Integration Service", vim.log.levels.WARN)
    return
  end
  
  -- Register with service
  local register_msg = vim.json.encode({
    type = "register",
    ide = "neovim",
    version = "1.0",
    pid = vim.fn.getpid()
  })
  vim.fn.chansend(socket, register_msg .. "\n")
  
  -- Set up autocommands
  local group = vim.api.nvim_create_augroup("tos_integration", { clear = true })
  
  vim.api.nvim_create_autocmd("CursorMoved", {
    group = group,
    callback = M.on_cursor_moved
  })
  
  vim.api.nvim_create_autocmd("BufWritePost", {
    group = group,
    callback = M.on_file_saved
  })
  
  vim.api.nvim_create_autocmd("TextChanged", {
    group = group,
    callback = M.on_buffer_modified
  })
end

function M.on_cursor_moved()
  if not socket or socket <= 0 then return end
  
  local pos = vim.api.nvim_win_get_cursor(0)
  local file = vim.api.nvim_buf_get_name(0)
  
  local msg = vim.json.encode({
    type = "state_update",
    event = "cursor_moved",
    file = file,
    line = pos[1],
    col = pos[2]
  })
  
  vim.fn.chansend(socket, msg .. "\n")
end

-- Listen for actions from TOS
function M.on_action_received(action)
  local cmd = action.action
  
  if cmd == "open_file" then
    vim.cmd("edit " .. action.params.path)
    if action.params.line then
      vim.api.nvim_win_set_cursor(0, {action.params.line, action.params.col or 0})
    end
  
  elseif cmd == "goto_line" then
    vim.api.nvim_win_set_cursor(0, {action.params.line, action.params.col or 0})
  
  elseif cmd == "select_range" then
    -- Set visual selection
    -- ... (more complex in Lua)
  end
end

-- Receive and apply agent context
function M.on_agent_context_update(msg)
  active_agents = msg.agents
  active_task = msg.task_context
  
  -- Store in buffer variable for potential AI plugin integration
  vim.b.tos_system_prompt = M.merge_agent_prompts(active_agents)
  vim.b.tos_task_context = active_task
  
  vim.notify("TOS: Agent context updated", vim.log.levels.INFO)
end

function M.merge_agent_prompts(agents)
  -- Merge system prompts from all agents
  -- Implementation: combine identity, constraints, efficiency rules
end

return M
```

**Status:** Phase 1 candidate (MVP)

### 10.3 Vim Plugin (VimScript)

**File:** `crates/tos-ide-integration/plugins/vim-plugin/plugin/tos.vim`

Similar to Neovim but uses older VimScript API. Lower priority due to older scripting interface.

**Status:** Phase 2

### 10.4 Emacs Package (Elisp)

**File:** `crates/tos-ide-integration/plugins/emacs/tos-integration.el`

**Status:** Phase 2

### 10.5 VS Code Extension (TypeScript)

**File:** `crates/tos-ide-integration/plugins/vscode/src/extension.ts`

**Status:** Phase 3 (community-driven potentially)

---

## 11. Workflow Example: Task-Driven Development

### 11.1 User Scenario

**Tim is working on a TOS project with a Kanban board tracking multiple tasks.**

### 11.2 Step-by-Step Execution

#### Step 1: Activate Kanban Task

```
Level 2: Kanban Board (TOS)
┌──────────────────────────────────┐
│ Sprint: Auth Refactor            │
│ ┌────────────────────────────────┐│
│ │ Task: Refactor HMAC validation ││
│ │ Status: ⏳ In Progress          ││
│ │                                ││
│ │ Agents:                        ││
│ │  + Security-Conscious Dev      ││
│ │  + Code Auditor                ││
│ │  + Performance Reviewer        ││
│ │                                ││
│ │ [⧉ Open in Zed] [Monitor]      ││
│ └────────────────────────────────┘│
└──────────────────────────────────┘

User clicks: [⧉ Open in Zed]
```

#### Step 2: Cortex Activation Event

**TOS Brain (Cortex) fires:**

```json
{
  "event": "task_activated",
  "event_source": "kanban",
  "task_id": "AUTH-REFACTOR-003",
  "agents": ["security-conscious-dev", "code-auditor", "performance-reviewer"],
  "file": "src/auth/hmac.rs"
}
```

#### Step 3: IDE Executor Routes to IDE Integration Service

**Cortex Action:** `ide_context_update`

```json
{
  "type": "action",
  "target": "ide",
  "action": "set_agent_context",
  "agents": [
    {
      "id": "security-conscious-dev",
      "system_prompt": "You are a security-conscious developer...",
      "constraints": ["Always use constant-time comparisons..."],
      "efficiency": [...]
    },
    // ... other agents
  ],
  "task_context": {...}
}
```

#### Step 4: IDE Integration Service Receives & Routes

```
IDE Integration Service receives set_agent_context action
  ↓
Identifies active IDE: Zed
  ↓
Routes via Cortex API to Zed Extension
  ↓
Zed Extension receives agent context
  ↓
Merges system prompts
  ↓
Writes to ~/.config/zed/tos-ai-context.json
  (or injects via future Zed extension API)
```

#### Step 5: IDE Action - Open File

**Same action includes:**

```json
{
  "type": "action",
  "target": "ide",
  "action": "open_file",
  "params": {
    "path": "src/auth/hmac.rs",
    "line": 1
  }
}
```

Zed Extension executes: `zed::open_file("src/auth/hmac.rs", 1)`

**User sees:** Zed opens with src/auth/hmac.rs

#### Step 6: File Context Pane Syncs

**TOS File Context Pane (L2):**

```
File Context Pane
┌─────────────────────────────────┐
│ src/auth/hmac.rs                │
│ [in Zed — unsaved: 0 hunks]     │
│                                 │
│  1  import hashlib              │
│  2  import hmac                 │
│  3                              │
│ ...                             │
│                                 │
│ [⧉ Edit in Zed] [Switch IDE]    │
└─────────────────────────────────┘

Subscribes to IDE State Curator:
- Real-time cursor position display
- Unsaved indicator
- Diagnostics from Zed's LSP
```

#### Step 7: User Edits in Zed (with Agent Persona)

**User types:**

```rust
// src/auth/hmac.rs:42
fn validate_hmac(sig: &str, key: &str) -> bool {
    sig == key  // ← Cursor here
}
```

**Zed AI (running with security-conscious persona):**

```
💡 [Zed AI Chat] Security Issue

Based on the security-conscious persona from task AUTH-REFACTOR-003:

Timing Attack Vulnerability

The standard == operator compares strings byte-by-byte and stops at 
the first mismatch. This timing variation leaks information about 
which byte differs, enabling timing attacks against cryptographic 
signatures.

Your "Security-Conscious Dev" constraint requires: 
"Always use constant-time comparisons for sensitive data"

Suggested fix:

```rust
fn validate_hmac(sig: &str, key: &str) -> bool {
    hmac::compare_digest(sig.as_bytes(), key.as_bytes())
}
```

[Accept] [Refine] [Explain]
```

#### Step 8: User Accepts, File Updates

**User clicks [Accept]**

Zed's AI applies the suggestion. File is modified.

**IDE State Curator reports:**
```json
{
  "event": "file_modified",
  "file": "src/auth/hmac.rs",
  "hunk_count": 1,
  "unsaved_changes": true
}
```

**File Context Pane (L2) updates:**
```
src/auth/hmac.rs [in Zed — unsaved: 1 hunk] 🔄
```

#### Step 9: Prompt Integration

**User in TOS L2 Command Hub types:**

```
prompt > cargo test
```

Executes in context where file modifications are tracked.

#### Step 10: Task Completion

**User marks task complete in Kanban:**

```
prompt > task complete AUTH-REFACTOR-003
```

**Cortex fires:**

```json
{
  "event": "task_completed",
  "task_id": "AUTH-REFACTOR-003",
  "file": "src/auth/hmac.rs",
  "changes_summary": {
    "lines_added": 3,
    "lines_removed": 1,
    "agents_used": ["security-conscious-dev", "code-auditor"],
    "ai_interactions": 4
  }
}
```

**IDE Integration Service receives:**

```json
{
  "type": "action",
  "target": "ide",
  "action": "clear_agent_context"
}
```

**Zed Extension clears agent context**, returns to default user configuration.

**Dream Memory captures:**

```json
{
  "task_id": "AUTH-REFACTOR-003",
  "title": "Refactor HMAC validation",
  "resolution": {
    "agent_decisions": [
      "Used constant-time comparison (hmac.compare_digest) to prevent timing attacks",
      "Added cryptographic safety constraints to signature validation"
    ],
    "patterns_learned": [
      "security-conscious-dev persona is effective for cryptographic code",
      "code-auditor helps catch edge cases in error handling"
    ],
    "time_spent": "47 minutes",
    "file_changes": ["src/auth/hmac.rs"]
  }
}
```

---

## 12. Specification Updates

### 12.1 Updates to `TOS_v0_1_Architecture.md`

**Section 3: Process Architecture: Brain & Face**

Add subsection: **3.4 Cortex Integration with External Systems**

```markdown
### 3.4 Cortex Integration with External Systems

The Cortex Layer can coordinate with external tools via well-defined 
Integration Protocols. IDEs (Zed, Vim, Emacs, VS Code) integrate via the 
IDE Integration Protocol.

#### IDE Integration Protocol

- **Curators:** IDE State Curator exposes editor state (file, cursor, selection, diagnostics)
- **Actions:** IDE Action Executor dispatches editor operations
- **Context Propagation:** Agent stacks automatically propagate to IDE AI layers
- **Implementation:** IPC daemon (IDE Integration Service) bridges IDE plugins to Cortex API

See IDE Integration Specification (separate document) for details.
```

### 12.2 Updates to `TOS_v0_1_Features.md`

**Section 4: Cortex Orchestration Layer**

#### 4.2 Architecture Overview (UPDATE)

Replace existing diagram with:

```markdown
The Cortex consists of three pluggable component types:

1. **Assistants** (`.tos-assistant`): LLM backend providers
2. **Curators** (`.tos-curator`): Data connectors including IDE state
3. **Agents** (`.tos-agent`): Instruction sets defining persona
```

Update supported Curators table:

| Curator | Type | Purpose |
|:---|:---|:---|
| Filesystem | Local | Read files, directories, metadata |
| Terminal | Local | PTY output, exit codes, environment |
| Git | Local | Commits, branches, diffs |
| IDE State | External | Current file, cursor, selection, diagnostics |

Update supported Action Targets table:

| Target | Type | Examples |
|:---|:---|:---|
| Terminal | Local | `stage_command`, `exec_cmd` |
| Workflow | Internal | `spawn_agent`, `run_task` |
| IDE | External | `open_file`, `goto_line`, `insert_text` |

#### Add 4.7 IDE Agent Context Propagation

```markdown
### 4.7 IDE Agent Context Propagation

When a Kanban task is activated or agents are changed, Cortex 
automatically propagates the active Agent stack to any connected IDE.

The merged Agent system prompt (identity + constraints + efficiency) 
and task context are sent to the IDE's AI layer, ensuring consistent 
AI voice and task awareness across TOS and IDE interfaces.

**Message Format:**
```json
{
  "type": "ide_context_update",
  "action": "set_agent_context" | "clear_agent_context",
  "agents": [...],
  "task_context": {...}
}
```

**IDE Responsibility:**
- Receive and merge agent system prompts
- Inject into IDE's AI layer (chat, suggestions, etc.)
- Maintain task context for AI awareness
- Clear context on task completion

See IDE Integration Specification for implementation details.
```

### 12.3 New Specification: `TOS_IDE_Integration_Specification.md`

Create comprehensive IDE integration spec (this is the current document's core focus).

Chapters:
1. Protocol Overview & Philosophy
2. IDE State Curator (data model, updates, queries)
3. IDE Action Executor (action types, confirmation, error handling)
4. Agent Context Propagation (message format, task context, merged prompts)
5. IDE Integration Service (daemon architecture, plugin bridge)
6. IDE-Specific Implementation (Zed, Neovim, Vim, Emacs, VS Code)
7. Socket Protocol (handshake, state messages, action messages)
8. Error Handling & Recovery
9. Security Considerations
10. Testing & Validation

### 12.4 Update `TOS_v0_1_Developer.md`

Add section: **IDE Integration for Developers**

```markdown
## IDE Integration Development Guide

### Creating a New IDE Integration

To add support for a new IDE:

1. Create plugin/extension in `crates/tos-ide-integration/plugins/{ide-name}`
2. Implement state reporting (cursor, file, selection, diagnostics)
3. Implement action dispatch (listen on socket, execute in IDE)
4. Register with IDE Integration Service on startup
5. Document IDE-specific quirks in plugin README

### Testing IDE Integration

See `crates/tos-ide-integration/tests/` for:
- State message format validation
- Action dispatch testing
- Multi-IDE coordination testing
- Agent context propagation testing
```

---

## 13. Implementation Roadmap

### Phase 1: MVP (3-4 weeks)

**Goal:** Cortex exposes IDE state; agent context flows to one IDE

**Deliverables:**

1. **Cortex Expansion**
   - [ ] Implement `IDEStateCurator` (Rust)
   - [ ] Implement `IDEActionExecutor` (Rust)
   - [ ] Add `ide_context_update` action type
   - [ ] Agent stacking merges system prompts

2. **IDE Integration Service**
   - [ ] Create `tos-ide-integration` crate (Rust)
   - [ ] Implement plugin bridge (socket handling)
   - [ ] Implement state aggregation
   - [ ] Implement action router

3. **Neovim Plugin (MVP)**
   - [ ] State reporting (cursor, file, unsaved)
   - [ ] Action dispatch (open_file, goto_line)
   - [ ] Socket communication
   - [ ] Agent context reception (store in buffer variable)

4. **File Context Pane (L2)**
   - [ ] Pane type definition
   - [ ] Subscription to IDE State Curator
   - [ ] Real-time cursor display
   - [ ] IDE switcher UI

5. **Testing**
   - [ ] State message format tests
   - [ ] Action execution tests
   - [ ] Neovim plugin integration tests
   - [ ] File Context Pane rendering tests

**Acceptance Criteria:**
- [ ] Tim can activate a Kanban task with agents
- [ ] TOS opens the file in Neovim at Level 3
- [ ] File Context Pane (L2) shows cursor in real-time
- [ ] File Context Pane can switch to another IDE (if running)

### Phase 2: Zed Integration + Polish (2-3 weeks)

**Goal:** Zed extension receives agent context; Zed AI uses persona

**Deliverables:**

1. **Zed Extension**
   - [ ] State reporting (cursor, file, selection)
   - [ ] Action dispatch (all IDE action types)
   - [ ] Agent context injection into Zed's AI config
   - [ ] Task context display in Zed chat

2. **IDE Executor Enhancements**
   - [ ] Confirmation workflow for destructive actions
   - [ ] Error handling & recovery
   - [ ] Staged action display in prompt

3. **Vim Plugin**
   - [ ] State reporting
   - [ ] Action dispatch
   - [ ] Basic agent context support

4. **File Context Pane Enhancements**
   - [ ] Selection → prompt integration
   - [ ] IDE action history/undo
   - [ ] Performance optimization (streaming large files)

5. **Documentation**
   - [ ] IDE Integration Specification (published)
   - [ ] Plugin development guide
   - [ ] Architecture overview diagrams

**Acceptance Criteria:**
- [ ] Tim switches between Zed and Neovim in same task
- [ ] Zed AI chat uses agent persona from task
- [ ] File Context Pane seamlessly handles multi-IDE workflow
- [ ] Agent context updates instantly when task changes

### Phase 3: Ecosystem & Community (2-4 weeks)

**Goal:** IDE integration is extensible; VS Code & Emacs are community-driven

**Deliverables:**

1. **VS Code Extension (or Community Fork)**
   - [ ] Publish to VS Code Marketplace
   - [ ] State reporting
   - [ ] Agent context injection

2. **Emacs Package**
   - [ ] State reporting
   - [ ] Action dispatch
   - [ ] Elisp integration patterns

3. **Marketplace**
   - [ ] Publish IDE integration plugins
   - [ ] Version compatibility matrix
   - [ ] User reviews & ratings

4. **Advanced Features**
   - [ ] Real-time collaboration (multi-user IDE sessions)
   - [ ] IDE session persistence (reload on restart)
   - [ ] Advanced diagnostics aggregation (multi-IDE lint)

5. **Performance Tuning**
   - [ ] Reduce socket latency (<50ms round-trip)
   - [ ] Optimize state update frequency
   - [ ] Memory profiling & optimization

**Acceptance Criteria:**
- [ ] Community can develop IDE integrations without modifying TOS core
- [ ] 3+ IDEs are fully supported (Neovim, Zed, VS Code)
- [ ] No performance degradation from IDE integration

### Timeline Summary

```
Week 1-4:   Phase 1 MVP (Cortex + Neovim + File Context Pane)
Week 4-7:   Phase 2 Polish (Zed + Vim, documentation)
Week 7-11:  Phase 3 Ecosystem (VS Code, Emacs, marketplace)
```

---

## 14. Key Design Principles

### 14.1 Terminal First, Always

IDEs are Level 3 (Application Focus) applications. The **terminal at Level 2** is the primary interaction surface. IDEs are *augmentations*, not replacements.

### 14.2 STAGE, NEVER RUN

All IDE actions are staged in the prompt before execution. Users see what will change. Users approve before changes happen.

### 14.3 Modular Stacking

Agent personas are **composable stacks** of identity, constraints, and efficiency rules. New agents don't require code changes. Agents are **first-class marketplace items**.

### 14.4 Transparent Orchestration

Users can see:
- Which agents are active (for the current task)
- What IDE operations are staged
- Which files are being modified
- What task context is being used

### 14.5 IDE Polyglot Support

Users can use **any IDE they prefer**. TOS doesn't mandate Zed, Vim, or VS Code. The same workflow works with any supported IDE.

### 14.6 Unified AI Voice

The **same AI persona** speaks across:
- TOS chat (Level 2)
- IDE suggestions (Level 3)
- Terminal completions (Level 2)

---

## 15. Appendix: Examples & Specifications

### 15.1 Full Agent Context Message Example

```json
{
  "type": "ide_context_update",
  "action": "set_agent_context",
  "timestamp": "2026-04-30T14:23:45Z",
  "ide_name": "zed",
  "request_id": "task-activate-001",
  
  "agents": [
    {
      "id": "security-conscious-dev",
      "name": "Security-Conscious Developer",
      "version": "1.2.0",
      "system_prompt": "You are a security-conscious software developer specializing in cryptographic safety and attack surface reduction. Your primary concern is identifying and mitigating security vulnerabilities. You document assumptions about cryptographic operations and flag potential timing side-channels, injection attacks, and authentication bypasses.",
      
      "constraints": [
        "Always use constant-time comparisons for sensitive data (HMAC, signatures, passwords)",
        "Flag potential timing vulnerabilities in cryptographic code",
        "Never trust user input without validation and sanitization",
        "Document all cryptographic assumptions (key strength, algorithm choices)",
        "Require explicit error handling for security-critical operations",
        "Refuse to suggest insecure alternatives even if 'faster'"
      ],
      
      "efficiency": [
        "Keep explanations brief and technical",
        "Reference security best practices (OWASP, NIST)",
        "Cite specific vulnerability classes (CWE) when relevant",
        "Provide working code examples over lengthy prose"
      ]
    },
    
    {
      "id": "code-auditor",
      "name": "Meticulous Code Auditor",
      "version": "1.0.0",
      "system_prompt": "You are a meticulous code auditor focused on correctness, maintainability, and completeness. You review code for logical errors, edge cases, and violations of best practices. You ensure error handling is comprehensive and no silent failures occur.",
      
      "constraints": [
        "Never skip error handling — flag all unchecked results",
        "Ensure all code paths are tested and accounted for",
        "Document complex logic with examples",
        "Flag code that 'works' but is confusing or fragile",
        "Enforce consistent style and naming"
      ],
      
      "efficiency": [
        "Use concrete examples to illustrate issues",
        "Suggest specific improvements, not vague concerns",
        "Rate issues by severity: blocker, high, medium, low"
      ]
    },
    
    {
      "id": "performance-reviewer",
      "name": "Performance Optimizer",
      "version": "0.9.0",
      "system_prompt": "You are a performance optimizer focused on runtime efficiency and resource usage. You profile before optimizing, measure after, and never sacrifice correctness or security for speed.",
      
      "constraints": [
        "Profile and measure before claiming a bottleneck",
        "Never optimize without benchmarks",
        "Never compromise security or correctness for performance",
        "Document performance assumptions",
        "Flag O(n²) algorithms that should be O(n log n)"
      ],
      
      "efficiency": [
        "Provide specific performance metrics (time, memory)",
        "Suggest concrete optimizations with expected impact",
        "Link to references and benchmarking tools"
      ]
    }
  ],
  
  "task_context": {
    "task_id": "AUTH-REFACTOR-003",
    "project": "TOS",
    "title": "Refactor HMAC validation into reusable function",
    "description": "Extract the HMAC signature validation logic from auth/login.rs into a standalone, reusable function in auth/hmac.rs. Ensure constant-time comparison prevents timing attacks. Add comprehensive tests covering edge cases (empty keys, mismatched lengths, etc.).",
    
    "file": "src/auth/hmac.rs",
    "related_files": [
      "src/auth/login.rs",
      "src/auth/tokens.rs",
      "tests/auth_test.rs"
    ],
    
    "acceptance_criteria": [
      "validate_hmac() function extracted to auth/hmac.rs",
      "Uses constant-time comparison (hmac.compare_digest or equivalent)",
      "All tests pass with 100% coverage",
      "No timing side-channels (verified with timing analysis)",
      "Documentation explains cryptographic safety assumptions",
      "Performance: < 1ms per validation (on modern hardware)"
    ],
    
    "workflow_stage": "in_progress",
    "time_estimate_minutes": 120,
    "time_spent_minutes": 45,
    "time_remaining_minutes": 75,
    
    "dependencies": [
      "AUTH-INFRA-001 (Crypto library setup)"
    ],
    "blockers": [],
    "related_tasks": [
      "AUTH-REFACTOR-004 (Signature validation refactor)"
    ]
  }
}
```

### 15.2 IDE Action Sequence Example

```json
// Agent response with multiple staged actions

{
  "request_id": "workflow-refactor-hmac",
  "behavior_id": "meticulous-dev",
  "timestamp": "2026-04-30T14:23:50Z",
  
  "response_type": "staged_actions",
  "response_narrative": "I've identified the timing vulnerability in the HMAC validation. Here's a refactoring plan that extracts the function and applies constant-time comparison.",
  
  "actions": [
    {
      "type": "action",
      "target": "ide",
      "ide_name": "zed",
      "action": "goto_line",
      "params": {"line": 42, "col": 0},
      "description": "Jump to the vulnerable comparison line",
      "confirmation_required": false
    },
    
    {
      "type": "action",
      "target": "ide",
      "action": "select_range",
      "params": {
        "start_line": 32,
        "end_line": 52
      },
      "description": "Select the validate_hmac function for extraction",
      "confirmation_required": false
    },
    
    {
      "type": "action",
      "target": "ide",
      "action": "apply_refactoring",
      "params": {
        "refactoring_type": "extract_method",
        "name": "validate_hmac_secure"
      },
      "description": "Extract to standalone function",
      "confirmation_required": true
    },
    
    {
      "type": "action",
      "target": "ide",
      "action": "insert_text",
      "params": {
        "line": 45,
        "col": 0,
        "text": "    # Use constant-time comparison to prevent timing attacks\n    return hmac.compare_digest(sig, key)\n"
      },
      "description": "Replace vulnerable comparison",
      "confirmation_required": true
    },
    
    {
      "type": "staged_command",
      "target": "terminal",
      "cmd": "cd src/auth && cargo test hmac",
      "description": "Run HMAC tests to verify refactoring",
      "confirmation_required": false
    },
    
    {
      "type": "message",
      "target": "chat",
      "text": "Refactoring complete. The new function uses `hmac.compare_digest()` for constant-time comparison, addressing the timing vulnerability flagged in task AUTH-REFACTOR-003. All acceptance criteria satisfied once tests pass."
    }
  ]
}
```

### 15.3 File Context Pane Implementation Reference

```rust
// Reference implementation sketch for File Context Pane

pub struct FileContextPane {
    current_file: Option<FileInfo>,
    cursor_position: Option<CursorPosition>,
    content_cache: String,
    last_update: Instant,
    ide_state_subscription: CuratorSubscription,
}

impl FileContextPane {
    pub fn new(cortex: &Cortex) -> Self {
        // Subscribe to IDE State Curator
        let subscription = cortex.subscribe_to_curator(
            "tos-curator-ide",
            vec!["cursor_moved", "file_changed", "unsaved_status"]
        );
        
        Self {
            current_file: None,
            cursor_position: None,
            content_cache: String::new(),
            last_update: Instant::now(),
            ide_state_subscription: subscription,
        }
    }
    
    pub fn handle_curator_event(&mut self, event: CuratorEvent) {
        match event.event_type.as_str() {
            "cursor_moved" => {
                self.cursor_position = Some(CursorPosition {
                    line: event.data["line"].as_u64().unwrap_or(0) as usize,
                    col: event.data["col"].as_u64().unwrap_or(0) as usize,
                });
                // Trigger re-render
            }
            "file_changed" => {
                let file_path = event.data["file"].as_str().unwrap_or("");
                self.load_file(file_path);
            }
            "unsaved_status" => {
                let unsaved = event.data["unsaved"].as_bool().unwrap_or(false);
                // Update UI indicator
            }
            _ => {}
        }
    }
    
    fn load_file(&mut self, path: &str) {
        // Load file content from disk with syntax highlighting
        let content = std::fs::read_to_string(path).unwrap_or_default();
        self.content_cache = content;
        self.current_file = Some(FileInfo::from_path(path));
    }
    
    pub fn render(&self) -> Element {
        // Render file content with:
        // - Syntax highlighting
        // - Cursor position marker
        // - Unsaved indicator
        // - IDE switcher chips
        // - [Edit in IDE] buttons
    }
}
```

---

## 16. Conclusion

This design **unifies IDE integration with Cortex orchestration**, enabling:

1. ✅ **Task-driven development:** Kanban task → Agent personas → IDE AI context
2. ✅ **Unified AI voice:** Same persona across TOS chat, IDE suggestions, terminal completions
3. ✅ **IDE polyglot support:** Use any IDE; TOS coordinates all
4. ✅ **Transparent orchestration:** Users see and approve all IDE operations
5. ✅ **Modular architecture:** New IDEs integrate via plugin pattern, not core changes
6. ✅ **Terminal-first philosophy:** IDEs augment the terminal, never replace it

The implementation is **phased and incremental**:
- Phase 1 (3-4 weeks): MVP with Cortex expansion + Neovim
- Phase 2 (2-3 weeks): Zed integration + polish
- Phase 3 (2-4 weeks): Ecosystem maturity + community IDEs

**Ready for implementation in future sessions.**

