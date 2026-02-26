# TOS Architectural Specification v1.3 - Extensions

---

> **Status**: Ratified — derived from implementation experience with `alpha-1`.  
> **Supersedes**: Ambiguous or silent areas of v1.0 §3.1, §3.2, §4.2, §13, and v1.2 §14.3.  
> **Does not modify**: Any behaviour explicitly defined in v1.0 or v1.2.

---

# Table of Contents

1. [Directory Mode: Pick Mode and Prompt Integration](#1-directory-mode-pick-mode-and-prompt-integration)
2. [Command Auto-Detection in the Prompt](#2-command-auto-detection-in-the-prompt)
3. [Shell API: Output Capture and Streaming](#3-shell-api-output-capture-and-streaming)
4. [Shell API: Fallback (No Integration)](#4-shell-api-fallback-no-integration)
5. [Bezel IPC Message Contracts](#5-bezel-ipc-message-contracts)
6. [Terminal Output Rendering](#6-terminal-output-rendering)
7. [Settings: Data Model, Scope, and IPC Contracts](#7-settings-data-model-scope-and-ipc-contracts)

---

## 1. Directory Mode: Pick Mode and Prompt Integration

Extends §3.2 of v1.0.

### 1.1 Two Interaction Modes

When the user is in **Directory Mode**, the behaviour of clicking a file or folder item depends on whether a command is currently staged in the **Persistent Unified Prompt**:

| Prompt state         | Item type | Click action                                    |
|----------------------|-----------|-------------------------------------------------|
| **Empty**            | File      | Insert absolute path into prompt (cursor at end)|
| **Empty**            | Folder    | Navigate into folder (default browse behaviour) |
| **Command staged**   | File      | Append absolute path to prompt as next argument |
| **Command staged**   | Folder    | Append absolute path to prompt as next argument |

Breadcrumb navigation and the parent `..` entry **always navigate** regardless of prompt state.

### 1.2 Pick Mode Visual State

When a command is staged in the prompt and the hub is in Directory Mode, the compositor MUST display a **staging banner** above the file grid. The banner contains:

- The keyboard/input icon (`⌨`).
- The current staged command (truncated if longer than ~300 px).
- A hint string: *"Click files/folders to append — press Enter to run"*.

The banner MUST animate (e.g., gentle border pulse) to draw attention without being distracting.

All file and folder items in the grid MUST receive a **pick-mode** visual treatment (e.g., amber/orange left border, subtle background glow, crosshair cursor on hover) to signal that they are clickable as path arguments.

### 1.3 IPC Message Definitions

| IPC message              | Description                                                                  |
|--------------------------|------------------------------------------------------------------------------|
| `dir_pick_file:<name>`   | Append the full path of `<name>` in `current_directory` to the prompt.       |
| `dir_pick_dir:<name>`    | Append the full path of `<name>` in `current_directory` to the prompt.       |
| `dir_navigate:<path>`    | Navigate into `<path>` (absolute or relative to `current_directory`).        |

`dir_pick_file` and `dir_pick_dir` apply the same path-append logic (§1.1 above). They are kept as separate messages to allow the compositor to distinguish file vs folder for future use (e.g., trailing slash, completions).

### 1.4 Multi-Select and Pick Mode

Multi-select (via checkboxes, §3.2 v1.0) MUST remain available in pick mode. When the user submits the prompt with multiple files selected, **all selected file paths are appended** in the order they were selected, space-separated.

---

## 2. Command Auto-Detection in the Prompt

Extends §3.1 of v1.0.

The compositor MUST perform lightweight command analysis on the text submitted via `prompt_submit:` **before** forwarding it to the PTY, and adjust hub state accordingly.

### 2.1 `ls` Detection

If the submitted command matches `ls` or begins with `ls ` (case-insensitive):

1. Resolve the target path:
   - If no argument: use `hub.current_directory`.
   - If argument is absolute: use as-is.
   - If argument is relative: join with `hub.current_directory`.
2. Set `hub.current_directory` to the resolved path.
3. Set `hub.mode` to `Directory`.
4. Clear `hub.shell_listing` (stale listing from previous directory).
5. Clear `hub.selected_files`.
6. Forward the command to the PTY as normal.

The compositor does **not** synthesise a directory listing itself; it waits for the PTY/shell to return output (either raw lines pushed to `terminal_output`, or an OSC `directory` sequence if shell integration is present).

### 2.2 `cd` Detection

If the submitted command matches `cd` or begins with `cd ` (case-insensitive):

1. Resolve the target path:
   - If no argument: resolve to `$HOME`.
   - If argument is absolute: use as-is.
   - If argument is relative: join with `hub.current_directory`.
2. If the resolved path is an existing directory, set `hub.current_directory` to it and clear `hub.selected_files`.
3. Do **not** change `hub.mode`.
4. Forward the command to the PTY as normal.

### 2.3 No False Positives

Auto-detection MUST only match the command word at the start of the trimmed input. Commands like `rls`, `lcd`, or `echo ls` MUST NOT trigger auto-detection.

---

## 3. Shell API: Output Capture and Streaming

Extends §13 of v1.0.

### 3.1 `command_result` Payload Format

The OSC `command_result` (code `9002`) payload format is extended to a three-field semicolon-delimited string:

```
<command>;<exit_status>;<base64(stdout+stderr)>
```

| Field               | Type    | Description                                             |
|---------------------|---------|---------------------------------------------------------|
| `command`           | string  | The command that was run.                               |
| `exit_status`       | integer | Exit code (0 = success).                                |
| `base64(output)`    | string  | Base64-encoded UTF-8 concatenation of stdout and stderr.|

The base64 encoding is required to prevent stray newlines and control characters from breaking OSC sequence parsing.

The third field is **optional** for backwards compatibility. If absent, only the status line is displayed.

### 3.2 Shell Integration Script Requirements

Shell integration scripts (Fish, Bash, Zsh) MUST:

1. Capture the full combined stdout+stderr of each executed command.
2. Base64-encode the output.
3. Emit `ESC]9002;<command>;<exit_status>;<base64_output>BEL` after command completion.
4. Also emit `ESC]9003;<cwd>BEL` after directory changes (existing requirement from v1.0).

Scripts MUST NOT attempt to capture TOS-internal commands like `EXEC`, `CD`, `LS`, `COMPLETE`, or `SETENV` sent from the compositor.

### 3.3 Compositor Decoding

Upon receiving a `command_result` OSC sequence with a third field, the compositor MUST:

1. Base64-decode the output field.
2. Split on newlines.
3. Push each non-empty line (after ANSI stripping, §6.1) into `hub.terminal_output`.
4. Cap `hub.terminal_output` at 500 lines (dropping oldest).

---

## 4. Shell API: Fallback (No Integration)

Extends §13.3 of v1.0.

### 4.1 Fallback Obligation

Shell integration scripts are **opt-in** and will not be present on all systems. The compositor MUST provide a fully functional experience without them, using raw PTY output.

### 4.2 Raw PTY Output Path

When no shell integration is present, the PTY reader thread collects raw bytes from the master file descriptor. For each read:

1. Convert bytes to UTF-8 (lossily).
2. Pass the string through the OSC parser (`shell_api::OscParser::parse`). Any recognized OSC sequences are handled; the remaining clean text is used for step 3.
3. **Strip ANSI/VT100 escape sequences** from the clean text (see §6.1).
4. Split on newlines; push each non-empty line to `hub.terminal_output`.
5. Cap at 500 lines.

### 4.3 Filesystem Fallback for Directory Mode

When the hub switches to Directory Mode and `hub.shell_listing` is `None`, the compositor MUST attempt to populate the file grid by reading `hub.current_directory` directly via the OS filesystem API (`std::fs::read_dir` or equivalent).

Entries are sorted: directories first (alphabetical), then files (alphabetical). Hidden entries (names starting with `.`) are filtered by `hub.show_hidden_files`.

The filesystem fallback is superseded by `hub.shell_listing` whenever the shell provides an OSC `directory` sequence.

### 4.4 `DirectoryChanged` PTY Event

Some terminals and shells emit `ESC]1337;CurrentDir=<path>BEL` on directory change. When the PTY reader detects this:

1. Parse the path from the sequence.
2. If the path is a valid existing directory, set `hub.current_directory` to it.
3. Clear `hub.selected_files` and `hub.shell_listing`.
4. Do not change `hub.mode`.

---

## 5. Bezel IPC Message Contracts

Extends §4 of v1.0.

### 5.1 Action-Identifier Rule

All IPC messages sent from bezel buttons and UI controls MUST use **action identifiers**, not display labels. Display labels are for rendering only and MUST NOT be forwarded to the shell or IPC dispatcher.

**Correct pattern:**
```html
<button onclick="window.ipc.postMessage('zoom_out')">ZOOM OUT</button>
```

**Incorrect (MUST NOT be used):**
```html
<button onclick="window.ipc.postMessage(this.innerText)">ZOOM OUT</button>
```

Violating this rule causes the compositor to forward the button label text (e.g., `"Prompt"`, `"Zoom Out"`) to the PTY as a shell command, producing errors like `Command 'Prompt' not found`.

### 5.2 Reserved IPC Prefixes

All IPC messages fall into one of the following namespaces:

| Prefix              | Purpose                                              |
|---------------------|------------------------------------------------------|
| `prompt_submit:`    | Submit the current prompt value to the PTY.          |
| `prompt_input:`     | Update the staged prompt text (live typing).         |
| `stage_command:`    | Pre-populate the prompt with a command string.       |
| `set_mode:`         | Switch hub mode (`Command`, `Directory`, etc.).      |
| `dir_navigate:`     | Navigate the directory view.                         |
| `dir_pick_file:`    | Append file path to prompt (see §1.3).               |
| `dir_pick_dir:`     | Append directory path to prompt (see §1.3).          |
| `dir_toggle_select:`| Toggle file selection (checkbox).                    |
| `dir_toggle_hidden` | Toggle hidden-file visibility.                       |
| `zoom_in`           | Zoom in one level.                                   |
| `zoom_out`          | Zoom out one level.                                  |
| `zoom_to:`          | Jump to a named level (`GlobalOverview`, `CommandHub`, etc.). |
| `append_output:`    | Append a line of text to a hub's terminal output (for testing/IPC injection). |

Any IPC message not matching a known prefix MUST be logged as a warning and silently ignored — it MUST NOT be forwarded to the PTY.

---

## 6. Terminal Output Rendering

### 6.1 ANSI Stripping

Before any string is stored in `hub.terminal_output`, the compositor MUST strip:

- **CSI sequences**: `ESC[` followed by any number of parameter bytes and a final letter (e.g., `ESC[1;32m`, `ESC[2J`).
- **OSC sequences**: `ESC]` followed by any content until `BEL` (`0x07`) or String Terminator (`ESC\`).
- **Other C0/C1 control characters**: all bytes in the ranges `0x00–0x08`, `0x0B–0x0C`, `0x0E–0x1F`. Tab (`0x09`), LF (`0x0A`), and CR (`0x0D`) are preserved.

The result MUST be valid printable UTF-8 (or ASCII) suitable for embedding in HTML without further escaping beyond standard HTML entity encoding (`<`, `>`, `&`, `"`).

### 6.2 Buffer Limits

`hub.terminal_output` is capped at **500 lines**. When the buffer exceeds this limit, the oldest lines are removed first (FIFO eviction). This limit applies whether lines arrive via OSC `command_result`, raw PTY output, or the `append_output:` IPC message.

### 6.3 Rendering Requirements

The terminal output pane MUST:

- Use a monospace font.
- Preserve whitespace (`white-space: pre-wrap`).
- Scroll automatically to the latest line on new output.
- Have a fixed or maximum height with vertical scrolling (the pane MUST NOT push other UI elements off-screen).
- Render each stored line as a distinct HTML element (e.g., `<div class="log-line">`) to allow per-line styling and selection.

The initial echo of a submitted command (e.g., `> ls /tmp`) MUST be visually distinct from command output (e.g., different colour or prefix sigil).

---

## 7. Settings: Data Model, Scope, and IPC Contracts

Extends §4.2 (v1.0 bezel shortcut), §14.3 (v1.2 privacy controls), and the "Settings Management" row of `architecture_analysis.md`.

### 7.1 The Settings Gap in Previous Versions

v1.0 and v1.2 reference settings in multiple places (logging master toggle, accessibility panel, audio volumes, performance targets, per-sector overrides) but never define:

- What the settings object looks like or where it lives in state.
- How global settings, per-sector overrides, and per-hub overrides relate to each other.
- Which settings are persisted to disk and in what format.
- What IPC messages correspond to each control in the Settings modal.

This section closes those gaps.

---

### 7.2 Settings Data Model

Settings are split into three clearly typed layers on `TosState`:

| Layer | Field | Type | Scope |
|-------|-------|------|-------|
| **Global scalar fields** | `state.fps`, `state.performance_alert`, etc. | native Rust types | All sectors/hubs |
| **Global key-value bag** | `state.settings: HashMap<String, String>` | string pairs | All sectors/hubs; arbitrary extension point |
| **Per-sector key-value bag** | `sector.settings: HashMap<String, String>` | string pairs | One sector and its hubs |
| **Per-application key-value bag** | `app.settings: HashMap<String, f32>` | float pairs | One application surface |

**Inheritance / precedence rule (cascade):**

```
per-application > per-sector > global key-value bag > global scalar field defaults
```

When the compositor resolves a setting for a hub or application, it checks each layer in this order and returns the first match.

---

### 7.3 Canonical Global Settings Keys

The following keys MUST be treated as canonical when read from `state.settings`:

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `theme` | `"lcars"` \| `"high-contrast"` | `"lcars"` | Visual theme. |
| `master_volume` | `"0"–"100"` | `"80"` | Master audio volume (percentage). |
| `ui_feedback_volume` | `"0"–"100"` | `"60"` | UI earcon volume (percentage). |
| `logging_enabled` | `"true"` \| `"false"` | `"true"` | Master toggle for TOS Log (v1.2 §14.3). |
| `sandboxing_enabled` | `"true"` \| `"false"` | `"true"` | Enable process sandboxing. |
| `confirm_destructive` | `"true"` \| `"false"` | `"true"` | Require tactile confirmation for destructive actions. |
| `deep_inspection` | `"true"` \| `"false"` | `"false"` | Enable Level 5 deep inspection (v1.2 §11.6). |
| `shell` | string | `"bash"` | Default shell binary path or name. |
| `accessibility_high_contrast` | `"true"` \| `"false"` | `"false"` | High contrast mode. |
| `accessibility_reduce_motion` | `"true"` \| `"false"` | `"false"` | Disable animations. |
| `show_minimap` | `"true"` \| `"false"` | `"true"` | Show tactical mini-map. |

`state.fps` remains a native `f32` field (not in the key-value bag) because it is used in the render loop and must avoid string parsing overhead.

---

### 7.4 Settings IPC Contracts

The following IPC messages MUST be handled by the compositor. Add these to the reserved prefix table (§5.2):

| IPC message | Effect |
|---|---|
| `open_settings` | Set `state.settings_open = true`. |
| `close_settings` | Set `state.settings_open = false`. |
| `set_fps:<value>` | Parse `<value>` as `f32`, clamp to [1, 240], write to `state.fps`. |
| `set_master_volume:<value>` | Parse `<value>` as `f32` (0–100), store in `state.settings["master_volume"]`, forward to audio manager. |
| `set_ui_feedback_volume:<value>` | Store in `state.settings["ui_feedback_volume"]`, forward to earcon player. |
| `set_theme:<name>` | Store in `state.settings["theme"]`; trigger CSS class swap or re-render. |
| `toggle_sandboxing` | Toggle `state.settings["sandboxing_enabled"]` between `"true"` / `"false"`. |
| `toggle_confirm_destructive` | Toggle `state.settings["confirm_destructive"]`. |
| `enable-deep-inspection` | Set `state.settings["deep_inspection"] = "true"`. Requires elevated privilege in production (see v1.2 §11.6). |
| `disable-deep-inspection` | Set `state.settings["deep_inspection"] = "false"`. |
| `optimize_system` | Trigger compositor-level performance optimisation (reduce background frame rate, flush texture caches). Implementation-defined. |
| `set_setting:<key>:<value>` | Generic setter: write `<value>` to `state.settings[<key>]`. Intended for extensibility. |
| `set_sector_setting:<key>:<value>` | Write `<value>` to the active sector's `sector.settings[<key>]`. |

Controls in the Settings modal that do not yet send an IPC message (e.g., Accessibility tab, Sector Overrides tab, UI Feedback slider) MUST be wired to the appropriate messages above before being shipped.

---

### 7.5 Settings Modal Navigation

Each sidebar tab in the Settings modal MUST send an IPC message to switch the visible panel:

| Tab label | IPC message |
|---|---|
| GENERAL & SYSTEM | `settings_tab:general` |
| ACCESSIBILITY & INPUT | `settings_tab:accessibility` |
| TACTICAL & SECURITY | `settings_tab:security` |
| SECTOR OVERRIDES | `settings_tab:sector` |

The compositor stores the active tab in `state.settings["_settings_tab"]` (underscore prefix = UI-only, not persisted). The settings renderer reads this key to decide which panel content to show.

---

### 7.6 Settings Persistence

Settings MUST be persisted to disk so they survive restarts.

**File location:**  
`~/.config/tos/settings.json` (Linux) or platform-equivalent app config directory.

**Format:** A flat JSON object containing only the canonical keys (§7.3) and any user-defined extension keys. Native scalar fields (`fps`) are serialized by name:

```json
{
  "fps": 60.0,
  "theme": "lcars",
  "master_volume": "80",
  "logging_enabled": "true",
  "deep_inspection": "false"
}
```

**Load on startup:** Required. If the file does not exist, defaults (§7.3) are used and the file is created.  
**Save on change:** Each IPC setter (§7.4) MUST trigger a debounced write (≤ 1 s delay) so settings survive unexpected exits.  
**Fields marked `#[serde(skip)]`** (e.g., `settings_open`, `force_redraw`) are **never** persisted — they are runtime-only state.

---

### 7.7 Per-Sector Settings Persistence

Per-sector settings (`sector.settings`) are persisted as part of the sector's serialized state, within the main TOS state snapshot at `~/.local/share/tos/state.json`.

Per-sector overrides for canonical keys (e.g., a sector with its own `master_volume` or `theme`) MUST be applied whenever that sector becomes the active sector in any viewport.

---

*This document extends the TOS Architectural Specification without modifying v1.0 or v1.2.*
