# TOS Alpha-2.1 Session Persistence & Workspace Memory
### Specification v1.0 — Supplement to Architecture Specification

---

## 1. Philosophy

TOS should feel like it never forgot you left. When a user returns after closing the system, their sectors are where they left them, their terminals have their history, and their workspace is ready. No reconfiguration. No reconstructing context from memory. The system picks up the thread.

Two concepts drive this spec:

- **Live State** — the continuously auto-saved snapshot of the current TOS environment. Written silently on every meaningful state change. Restored silently on every launch. The user never thinks about it.
- **Named Sessions** — portable, exportable sector-scoped snapshots the user explicitly saves and switches between. These are the power-user layer: project workspaces, client environments, experiment states.

Both sit on top of the existing Settings Daemon (Architecture Specification §26), which continues to own user preferences. Session state is deliberately kept separate from preference state — the two never mix.

---

## 2. What Persists

### 2.1 Per-Sector Persistent State

Each sector is a workspace. Its persistent state captures everything needed to reconstruct it fully:

| State Field | Description |
| :--- | :--- |
| `name` | Sector display name |
| `position` | Index in Level 1 grid |
| `cwd` | Last working directory |
| `shell` | Active shell module ID |
| `terminal_module` | Active terminal output module ID |
| `environment` | Sector-level env vars |
| `hub_layout` | Multi-terminal layout within the sector (see §4) |
| `terminal_histories` | Scrollback buffers per terminal pane |
| `ai_chat_history` | AI chat conversation for this sector |
| `pinned_chips` | Pinned left/right chip entries |
| `sector_type` | Sector type module ID if non-default |
| `active_mode` | Last active hub mode (`CMD` / `SEARCH` / `AI`) |
| `frozen` | Whether the sector was frozen at close |

### 2.2 Global Persistent State

| State Field | Description |
| :--- | :--- |
| `bezel_slots` | Component assignments for all Top / Left / Right slots |
| `active_sector_index` | Which sector was focused at close |
| `active_level` | Which hierarchy level was active at close |

### 2.3 What Stays in the Settings Daemon

The Settings Daemon (Architecture Specification §26) continues to own everything that is true regardless of which session is active. Session files never duplicate these:

| Owned by Settings Daemon | Owned by Session Files |
| :--- | :--- |
| Theme module selection | Sector layout and positions |
| AI backend default | Terminal scrollback histories |
| AI behavior configs and overrides | AI chat histories per sector |
| Keybind mappings | Hub layouts (splits / tabs / panes) |
| Audio / haptic preferences | Working directories per pane |
| Accessibility settings | Pinned chips |
| Sandboxing tier rules | Active mode per sector |
| Module installation state | Bezel slot assignments |

The Settings Daemon continues to use `~/.config/tos/settings.json` with debounced writes (≤1s) exactly as specced. Session files live separately at `~/.local/share/tos/sessions/` and are managed by a new **Session Service** (`tos-sessiond`).

---

## 3. Session Files

### 3.1 Format & Location

Session files are plain JSON stored at:

```
~/.local/share/tos/sessions/
├── _live.tos-session          # Auto-saved live state (always present)
├── rust-project.tos-session   # Named session
├── client-work.tos-session    # Named session
└── experiments.tos-session    # Named session
```

The `.tos-session` extension makes files identifiable, portable, and openable. A user can copy a session file to another machine and load it directly.

### 3.2 Schema

```json
{
  "tos_session_version": "1.0",
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
      "environment": {
        "RUST_LOG": "info"
      },
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
      "ai_chat_history": [
        { "role": "user", "content": "explain this error", "timestamp": "2025-03-04T17:40:00Z" },
        { "role": "assistant", "content": "The error is a borrow checker violation...", "timestamp": "2025-03-04T17:40:02Z" }
      ],
      "pinned_chips": {
        "left": ["~/projects", "/etc"],
        "right": ["cargo build", "cargo test"]
      }
    }
  ]
}
```

### 3.3 Auto-Save Triggers

The live state file (`_live.tos-session`) is written automatically on every significant state change. Writes are debounced at 2 seconds to avoid thrashing disk on rapid changes.

| Trigger | What Gets Written |
| :--- | :--- |
| Command submitted to PTY | `terminal_histories` for affected pane |
| `cd` or directory change | `cwd` for affected pane |
| Sector created, closed, or renamed | Full sector list |
| Sector frozen / unfrozen | `frozen` flag |
| Bezel slot reconfigured | `global.bezel_slots` |
| AI chat message sent or received | `ai_chat_history` for affected sector |
| Mode switch (`CMD` / `SEARCH` / `AI`) | `active_mode` for affected sector |
| Hub layout changed | `hub_layout` for affected sector |
| TOS graceful shutdown | Full synchronous write of all state |

> **CRASH RECOVERY:** On ungraceful shutdown, the last debounced write is used. A crash mid-write is safe because `tos-sessiond` writes to a temp file (`_live.tos-session.tmp`) and atomically renames it on success. A corrupted or incomplete temp file is discarded on next startup; the previous good state is used instead.

---

## 4. Multi-Terminal Hub Layout

Each sector can contain multiple terminal instances. Their arrangement within the Command Hub is defined by the `hub_layout` object and rendered by the active Terminal Output Module.

### 4.1 Layout Types

| Layout Type | Description | Visual Character |
| :--- | :--- | :--- |
| `splits` | Tiled panes with resizable dividers | Structural — terminals occupy real spatial estate |
| `tabs` | Stacked terminals, one visible at a time | Flat — functional fallback |
| `module_defined` | Layout fully controlled by the Terminal Output Module | Defined by the module's visual language |

`splits` is the default. `tabs` is the fallback for modules that do not declare `multi_terminal = true`.

### 4.2 Splits Layout

A split layout defines a tree of panes, each with a weight (proportional size), its own `cwd`, its own shell instance, and its own terminal history.

| Action | Shortcut |
| :--- | :--- |
| Split focused pane vertically | `Ctrl+\` |
| Split focused pane horizontally | `Ctrl+-` |
| Move focus between panes | `Ctrl+Arrow` |
| Close focused pane | `Ctrl+W` |
| Equalize pane weights | Double-click divider |

The focused pane renders with a distinct amber border. Unfocused panes render at reduced opacity, remaining visible and in context at all times.

### 4.3 The Visual Opportunity

Tabs are functional but spatially flat. The LCARS aesthetic offers a richer option: Terminal Output Modules can define their own multi-terminal visualization. The **Cinematic Triangular Module**, for example, could render inactive panes as smaller angled panels flanking the primary active pane — always visible, spatially expressive, and striking without requiring a divider line.

This is intentionally an **open design space for Terminal Output Modules**. The session spec defines the data (`hub_layout`, `panes`, `weights`). The visual expression is owned by the module.

> **MODULE CONTRACT:** Terminal Output Modules that support multi-terminal layouts must declare `multi_terminal = true` in their manifest and implement the `render_layout(hub_layout)` interface. Modules without this declaration receive the `tabs` fallback automatically.

---

## 5. Named Sessions

### 5.1 Saving a Named Session

Named sessions are sector-scoped. Each sector manages its own sessions independently — a saved state for the `dev` sector has no relation to a saved state for the `client-work` sector.

**To save:** secondary select on a sector tile at Level 1 → **Save Session As...** → enter a name. The Brain serializes the current sector state and passes it to `tos-sessiond` for writing.

Alternatively, from within the Command Hub, tapping the sector name chip in the Top Bezel Left section opens the **Sector Session Popover**:

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

- The **LIVE** badge indicates the current state is the auto-saved live state, not a named session.
- **[Save Current]** saves the live state as a new named session (prompts for name).
- **[Export]** copies the `.tos-session` file to a user-selected path for portability.

### 5.2 Loading a Named Session

Selecting a named session from the popover replaces the current sector's state with the saved one. Name, layout, histories, cwd, and chat history are all restored. The shell is re-spawned fresh in the restored `cwd` — running processes from the previous session are not restored, only the environment they ran in.

The sector tile animates a brief reload pulse as state is applied. No confirmation is required unless the current live state has unsaved changes, in which case a single-tap confirmation appears: **[Load anyway]** / **[Save first]**.

**Conflict handling:** if a named session references a shell or terminal module that is no longer installed, TOS substitutes the system default for that module and renders a yellow alert chip in the restored sector noting the substitution.

### 5.3 Importing a Session

A `.tos-session` file can be imported by dropping it onto a sector tile at Level 1, or via **Settings → Sessions → Import**. `tos-sessiond` validates the file format and version before the Brain applies it. Imported sessions appear in the sector's named session list immediately.

---

## 6. Session Service (`tos-sessiond`)

A new auxiliary daemon, `tos-sessiond`, handles all session file I/O. It joins the existing daemon set on **Port 7006**.

| Component | Binary | Port | Protocol |
| :--- | :--- | :--- | :--- |
| Session Service | `tos-sessiond` | 7006 | TCP |

### 6.1 Responsibilities

- Maintains `_live.tos-session` with 2s debounced auto-save via atomic temp-file rename.
- Serves named session CRUD to the Brain via IPC.
- Validates session file schema on load and import.
- Detects and discards incomplete temp files on startup for crash recovery.

### 6.2 IPC Contracts

| Message | Effect |
| :--- | :--- |
| `session_save:<sector_id>:<name>` | Saves current sector state as a named session |
| `session_load:<sector_id>:<name>` | Loads a named session into the specified sector |
| `session_delete:<sector_id>:<name>` | Deletes a named session |
| `session_list:<sector_id>` | Returns list of named sessions for a sector |
| `session_export:<sector_id>:<name>:<path>` | Exports a session file to the given path |
| `session_import:<path>` | Imports a `.tos-session` file |
| `session_live_write` | Forces an immediate synchronous live state write (used on graceful shutdown) |

### 6.3 Startup & Restore Sequence

On Brain init, the restore sequence is:

1. Brain starts and signals `tos-sessiond` to load `_live.tos-session`.
2. `tos-sessiond` reads and validates the file. If valid, returns the full state object to the Brain.
3. Brain reconstructs all sectors, hub layouts, and bezel slots from the restored state.
4. Each sector's shell is re-spawned in its restored `cwd`. Terminal histories are loaded into each pane's output buffer before the shell prompt appears.
5. The Face receives the fully reconstructed state via the standard WebSocket state sync and renders everything as if it had never closed.
6. If `_live.tos-session` is missing or corrupt, the Brain starts with a single default sector and an empty state. No error is surfaced to the user unless they navigate to **Settings → Sessions**.

> **SILENT BY DESIGN:** There is no restore notification, animation, or prompt. The system is simply there, as the user left it. The only indication that a restore occurred is that everything looks exactly right.

---

## 7. Terminal History Persistence

Terminal scrollback buffers are serialized as ordered arrays of strings per pane in the session file (`terminal_histories`). On restore they are loaded into the terminal output module's buffer before the shell spawns, so history is visible immediately.

The existing `terminal_buffer_limit` setting (default 500 lines, adjustable via `set_terminal_buffer_limit` IPC) governs both the live buffer and what is written to disk. History beyond the cap is not persisted.

The persisted terminal history is the *visual* scrollback — what appeared on screen. It is complementary to, and does not interfere with, the shell's own history file (e.g. `~/.local/share/fish/fish_history`).

---

## 8. AI Chat History Persistence

Each sector's AI chat history is persisted as an ordered array of message objects in the session file (`ai_chat_history`). On restore, the active Chat Companion behavior module receives the history via its `on_session_restore` callback and renders it into the chat panel before the user interacts.

Chat history is capped at 200 messages per sector in the session file. Messages beyond the cap are dropped from persistence but remain accessible within the running session until the Brain restarts.

Loading a named session replaces the chat history with the one from that snapshot. The live state always reflects the most recent conversation.

---

*TOS Alpha-2.1 // Session Persistence & Workspace Memory Specification v1.0 // Supplement Document*
