# Spec Patch: Migrate Activity Mode → Level 4 btop-Style Monitor

## Goal

Remove Activity Mode from the Level 2 Mode Selector and migrate all process management to Level 4, redesigning Level 4's visual presentation as a **btop-style system monitor** with resource graphs, process trees, and interactive process management.

## Design Decisions (From User Feedback)

1. **Level 2 Activity Mode → removed.** The Mode Selector has exactly **three buttons: `CMD`, `SEARCH`, `AI`**. There is no Activity button. Directory context and process context are not modes — they are **dynamic chip overlay behaviors** that activate automatically based on terminal output, without changing the mode selector.

2. **Process output → chip scraper populates inspection chips.** When a process-related command (`ps`, `top`, `htop`, `kill`) runs in CMD mode, the Brain's output scraper recognizes the output and populates the chip overlay with PID and process name chips. Tapping a PID chip navigates to **Level 4 Process Monitor**, pre-focused on that process. This is the same pattern as Directory context (which populates file/folder chips when `ls`/`cd` output is detected) — neither changes the Mode Selector state.

3. **Level 4 Detail View → sector-scoped btop-style visualization.** When entering Level 4 from Level 3 (inspecting an app), shows the **focused app + its parent sector's process tree** in a btop-style layout with CPU/Memory/I/O graphs, sortable process tree, and interactive process management.

4. **Level 4 Tactical Reset → global btop-style visualization.** God Mode becomes a full-system btop-style monitor showing **all sectors, all services, all OS processes**.

5. **System services + all processes → accessible from Level 1.** The Global Overview bezel provides a shortcut to God Mode.

6. **Clone/Move after elevation in God Mode.** After privilege elevation, process management actions include:
   - **Clone to Sector** — select an existing sector or create new
   - **Move to Sector** — re-parent the process to an existing or new sector

---

## Proposed Changes

### Architecture Specification

#### [MODIFY] [TOS_v0.1.0_Architecture.md](file:///z:/repos/TOS-Desktop-Environment/docs/spec/TOS_v0.1.0_Architecture.md)

**§5 The Extended Hierarchy (line ~869–876)**
Update Level 4 description:

```diff
-| **4** | **Deep Inspection & Recovery** | Unified diagnostic level with three sub-views: Detail View (structured metadata), Buffer View (raw hex dump, privileged), and Tactical Reset (God Mode wireframe recovery). |
+| **4** | **System Monitor & Recovery** | Unified diagnostic level with three sub-views: Process Monitor (btop-style sector-scoped system monitor), Buffer View (raw hex dump, privileged), and Tactical Reset (btop-style global system monitor with emergency recovery). |
```

---

**§6.4 Global Overview Bezel (line ~936–942)**
Add a shortcut to global process view from Level 1:

Add to the Expanded bezel items under **System:**
```
- **System Processes:** Jump to Level 4 Tactical Reset (God Mode) for global process management.
```

---

**§7.1 Persistent Unified Prompt (line ~985)**
Remove ACTIVITY from mode selector:

```diff
-- **Left Section (Origin):** Universal Mode Selector (CMD, SEARCH, AI, ACTIVITY). An integral part of the prompt assembly, not a dockable module.
+- **Left Section (Origin):** Universal Mode Selector (CMD, SEARCH, AI). An integral part of the prompt assembly, not a dockable module.
```

---

**§7.3 Context-Aware Terminal Augmentation (line ~1009–1015)**
Remove Activity row. Clarify that Directory and Process contexts are **chip overlay behaviors**, not mode selector states:

```diff
-TOS treats the **Terminal Canvas** and the **Dual-Sided Chip Layout** as a unified interface. The system context dictates what appears in the terminal and how chips are populated:
+TOS treats the **Terminal Canvas** and the **Dual-Sided Chip Layout** as a unified interface. The Brain's output scraper detects command families in terminal output and dynamically populates the chip overlay without changing the Mode Selector state:

 | Context | Terminal Canvas | Chip Layout Integration |
 |---------|-----------------|------------------------|
 | **Command** | Standard shell `stdout`/`stderr`. | Chips show command history, autocomplete suggestions, tool flags. |
 | **Search** | Semantic or exact search results. | Chips populate with search scopes, filters, quick-action buttons. |
 | **AI** | The LLM's rationale, thought process, or raw output. | Chips act as command staging buttons for AI-suggested shell operations. |
-| **Directory** | Raw directory listing (`ls` / `cd`). | Chips populate with interactive file and folder paths. Chips also provide file or image previews when applicable. |
-| **Activity** | Raw process table (`top` / `ps`). | Chips populate with process-handling actions (kill, renice, monitor). Running apps show 10Hz live thumbnails. |
+| **Directory** (chip overlay) | Raw directory listing (`ls` / `cd`). | Chips populate with interactive file and folder paths. Chips also provide file or image previews when applicable. Does not change the Mode Selector. |
+| **Process** (chip overlay) | Process-related output (`ps`, `top`, `htop`, `kill`). | Chips populate with PID and process name entries. Tapping a PID chip navigates to Level 4 Process Monitor (§9.1), pre-focused on that process. Does not change the Mode Selector. |
```

---

**§7.5.2 Process & App Chips (line ~1051–1057)**
Update chip actions — these appear via the process output scraper in CMD mode and in Level 4:

```diff
 - **[Tactical Signal...]:** Sub-menu to send `SIGINT`, `SIGTERM`, or `SIGKILL` to the PTY/Process.
 - **[Renice Priority]:** Adjust process priority (LCARS levels 1–5).
 - **[Inspect Buffer]:** Transition to Level 4 Buffer View.
 - **[Isolate Process]:** Force the process into a more restrictive sandbox tier.
-- **[Clone to Sector]:** Duplicate the process state in a new terminal sector.
+- **[Inspect Process]:** Navigate to Level 4 Process Monitor (§9.1), pre-focused on this PID.
+- **[Clone to Sector]:** Duplicate the process state to an existing or new sector (sector picker dialog).
+- **[Move to Sector]:** Re-parent the process to an existing or new sector. The process continues running in the target sector.
```

---

**§7.7 Context-Aware Mode Switching (line ~1065–1078)**
Rewrite to reflect that these are **chip overlay reactions**, not mode switches:

```diff
-### 7.7 Context-Aware Mode Switching
+### 7.7 Context-Aware Chip Overlay

-Certain shell commands signal an intent to change the active context. TOS can detect these and switch the Mode Selector (§7.1) accordingly. This behaviour is user-configurable per sector: **Off**, **Suggest**, or **Auto**.
+Certain shell commands produce output that the Brain's output scraper recognizes. When detected, the chip overlay populates with contextual chips **without changing the Mode Selector state**. The mode stays on CMD. This behaviour is user-configurable per sector: **Off**, **Suggest**, or **Auto**.

 | Command Family | Example Commands | Resulting Context |
 |---|---|---|
-| Filesystem | `ls`, `cd`, `cp`, `mv`, `find`, etc. | Directory |
-| Process Management | `kill`, `ps`, `top`, `htop`, etc. | Activity |
+| Filesystem | `ls`, `cd`, `cp`, `mv`, `find`, etc. | Directory chip overlay (file/folder chips) |
+| Process Management | `ps`, `top`, `htop`, etc. | Process chip overlay (PID/name chips → Level 4) |

-**Off:** No automatic switching. Mode stays as-is.
-**Suggest:** A chip appears in the Right Region offering the switch. The user taps to confirm.
-**Auto:** The context switch fires immediately, accompanied by a mode-transition earcon (Visual Design §3) and a brief visual indicator on the Mode Selector.
+**Off:** No chip overlay reaction. Output renders normally.
+**Suggest:** A chip appears in the Right Region offering the contextual overlay. The user taps to populate.
+**Auto:** The chip overlay populates immediately.

-**Effect on the Chip Layer:** A context switch — whether confirmed by the user in Suggest mode or automatic in Auto mode — immediately drives chip column repopulation as defined in §7.3. The Left Region reflects the new context's static options; the Right Region reflects its predictive action set. Any open Autocomplete Overlay (§7.6) is dismissed and rebuilt for the new context.
+**Effect on the Chip Layer:** The overlay populates the chip columns as defined in §7.3. The Left Region reflects the context's static options (paths or process tree); the Right Region reflects its predictive action set (file actions or process signals/inspection links). Any open Autocomplete Overlay (§7.6) is dismissed and rebuilt. The Mode Selector indicator does not change.
```

---

**§8.2 Application Models (line ~1146)**
Remove Activity Mode thumbnail reference:

```diff
-- Thumbnail for Activity Mode.
+- Thumbnail for Level 1 sector tile and Level 4 Process Monitor.
```

---

**§9.1 Detail View → Process Monitor (line ~1160–1170)**
**Major rewrite.** Replace the structured metadata modal with a btop-style sector-scoped process monitor:

```markdown
### 9.1 Process Monitor (Sector-Scoped)

A btop-style interactive system monitor presenting the focused application and its parent sector's process tree. The view is composed of modular panes arranged in a dashboard layout.

#### 9.1.1 Layout

| Pane | Content | Position |
|---|---|---|
| **CPU Graph** | Per-core utilization over time (CSS-rendered line graphs or Unicode Braille-pattern equivalents). Auto-scaling. | Top-left |
| **Memory Graph** | RAM and swap usage over time. | Top-right |
| **Process Tree** | Hierarchical parent-child process list for the sector. Sortable by CPU, MEM, PID, user. Filterable by name. Keyboard-navigable with process following (lock onto a PID). | Center (primary, largest pane) |
| **Disk I/O** | Read/write rates per disk. | Bottom-left |
| **Network** | Up/down rate graphs with auto-scaling. | Bottom-right |

Pane arrangement is user-configurable. Panes can be toggled on/off. The layout uses clean border boxes to visually separate metrics (LCARS-styled).

#### 9.1.2 Scope

- **Entry from Level 3 (inspecting an app):** The process tree is pre-filtered to show the focused application's process subtree, expanded within the parent sector's full process tree.
- **Entry from a process chip at Level 2:** The process tree scrolls to and highlights the selected PID.
- **Scope is always the parent sector's process tree** — not global. For global scope, use Tactical Reset (§9.3).

#### 9.1.3 Process Interaction

Each process row supports:
- **Select:** Click to highlight. Details (env vars, args, open files) appear in the Metadata Panel (§9.1.4).
- **Signal:** Right-click or chip → send SIGINT, SIGTERM, SIGKILL.
- **Renice:** Adjust priority (LCARS levels 1–5).
- **Isolate:** Force into a more restrictive sandbox tier.
- **Clone to Sector:** Duplicate to an existing or new sector (sector picker dialog).
- **Move to Sector:** Re-parent the process to an existing or new sector (sector picker dialog).
- **Inspect Buffer:** Transition to Buffer View (§9.2) for the selected PID.

#### 9.1.4 Metadata Panel

When a process is selected, a collapsible side panel shows structured metadata:
- PID, PPID, user, session ownership
- Environment variables and args
- Permissions and sandbox status
- Event history (from TOS Log)
- Security audit excerpts

Export as JSON/plain text.
```

---

**§9.3 Tactical Reset → Global System Monitor & Recovery (line ~1180–1200)**
**Major rewrite.** Transform God Mode from a wireframe map to a global btop-style monitor:

```markdown
### 9.3 Tactical Reset (Global System Monitor & Recovery)

The Tactical Reset is the system's ultimate diagnostic layer — a btop-style global system monitor that shows **all** Brain sectors, services, and associated OS processes. It uses the same modular pane layout as the Process Monitor (§9.1) but with global scope.

#### 9.3.1 Global Resource Diagnostics

- **Layout:** Same btop-style pane arrangement as §9.1.1 (CPU, Memory, Process Tree, Disk I/O, Network), but the process tree shows **every process across all sectors and system services**.
- **Sector Grouping:** The process tree groups processes by sector. Each sector is a collapsible node showing its process subtree. TOS system services appear under a "System" group.
- **Resource Monitoring:** Real-time CPU, memory, and I/O pressure gauges for every active PID.
- **Emergency Management:** After privilege elevation (§9.5), integrated "Force Kill" capabilities that send `SIGKILL` directly via the Brain's root-tier services. Also: Clone to Sector, Move to Sector (§9.3.4).
- **Recovery Logic:** Triggering a Tactical Reset flushes all transient diagnostic buffers and resets the Face-Brain IPC sync to a known stable state.

#### 9.3.2 Initiation
- **Manual Trigger:** Bezel "Tactical Reset" button, `Ctrl+Alt+Backspace`, or **Level 1 Global Overview bezel → System Processes**.
- **Safety Fallback:** Automatically triggered if the Face detects sustained latency >500ms or if the Brain reports a service-level deadlock.

#### 9.3.3 Security & Privilege Isolation
(unchanged — read-only by default, explicit elevation required)

#### 9.3.4 Cross-Sector Process Management (Requires Elevation)

After privilege elevation, the following actions become available on any process in the global tree:

| Action | Description |
|---|---|
| **Force Kill** | Sends SIGKILL. Re-authentication required. |
| **Clone to Sector** | Opens a sector picker: select an existing sector or create new. Duplicates the process state to the target sector. |
| **Move to Sector** | Opens a sector picker: select an existing sector or create new. Re-parents the process (PID, PTY, process group) to the target sector's hub. The process continues running. |
| **Renice** | Adjust priority. |
| **Isolate** | Force into a more restrictive sandbox tier. |
```

---

**§14 Semantic Event Tables (line ~1551)**
Remove `set_mode_activity`:

```diff
-| Mode Control | `cycle_mode`, `set_mode_command`, `set_mode_directory`, `set_mode_activity`, `set_mode_search`, `set_mode_ai`, `toggle_hidden_files` |
+| Mode Control | `cycle_mode`, `set_mode_command`, `set_mode_directory`, `set_mode_search`, `set_mode_ai`, `toggle_hidden_files` |
```

> [!NOTE]
> `set_mode_directory` is retained despite Directory not being a Mode Selector button. It controls whether the chip overlay activates — the semantic event triggers chip population, not a mode switch. Consider renaming to `chip_overlay_directory` in a future pass, but this is cosmetic.

---

**§25.2 Reserved IPC Prefixes (line ~2043)**
Update the sniffing description:

```diff
-**Prompt Interception Layer:** The `prompt_submit:` message is an exception. The Brain performs a "sniffing" pass on the submitted string to detect `ls` or `cd` and trigger mode switches. This logic lives entirely in the Brain's command dispatcher.
+**Prompt Interception Layer:** The `prompt_submit:` message is an exception. The Brain performs a "sniffing" pass on the submitted string to detect command families (`ls`/`cd` for directory context, `ps`/`top`/`htop` for process context) and trigger chip overlay population. This does not change the Mode Selector state. This logic lives entirely in the Brain's command dispatcher.
```

---

**§25.2 Reserved IPC Prefixes (line ~2064–2065)**
Replace Activity-mode IPC with Level 4 process management:

```diff
-| `app_toggle_select:` | Toggle app in Activity mode | N/A |
-| `app_batch_kill`, `app_batch_signal:` | Batch process management | Semicolon (`;`) |
+| `l4_process_signal:<pid>:<signal>` | Send signal to process in Level 4 monitor | N/A |
+| `l4_process_clone:<pid>:<sector_id>` | Clone process to sector (existing or `new`) | N/A |
+| `l4_process_move:<pid>:<sector_id>` | Move process to sector (existing or `new`) | N/A |
+| `l4_process_renice:<pid>:<priority>` | Adjust process priority | N/A |
+| `l4_process_isolate:<pid>` | Force process into restrictive sandbox | N/A |
+| `l4_inspect:<pid>` | Navigate to Level 4 Process Monitor focused on PID | N/A |
```

---

**§29 Implementation Roadmap (line ~2255)**
Update Activity Mode step:

```diff
-7. **Activity Mode** — Visual process management via `ps` parsing.
+7. **Level 4 System Monitor** — btop-style process management with sector-scoped and global views. Process chip overlay for CMD mode output scraping.
```

---

### User Manual

#### [MODIFY] [TOS_v0.1.0_User-Manual.md](file:///z:/repos/TOS-Desktop-Environment/docs/spec/TOS_v0.1.0_User-Manual.md)

**§3 The Command Hub (LVL 2) Modes (line ~30–39)**
Remove Activity Context. Clarify Directory is a chip overlay, not a mode:

```diff
 - **[CMD] Command Mode** — Standard interactive PTY terminal. Chips populate with command history, autocomplete suggestions, and tool flags.
 - **[SEARCH] Search Mode** — Semantic or global filesystem indexing with instant results. Chips populate with search scopes, filters, and quick-action buttons.
 - **[AI] AI Augmentation** — Natural language shell queries with AI explanation and command staging. The AI never executes commands without your confirmation from the prompt.
-- **Directory Context** — Triggered automatically by `ls` or `cd`. Shows real-time file and folder chips for rapid prompt building. File and image previews where applicable.
-- **Activity Context** — Triggered automatically by `top` or `ps`. Shows process-handling action chips (kill, renice, monitor):
-  - **Live View:** 10Hz snapshots for active, graphical applications.
-  - **Resource View:** App icon and name for inactive or non-graphical applications.
-  - **System View:** Symbolic placeholders for background and system processes.
+
+**Dynamic Chip Overlays** — In CMD mode, the chip columns react to terminal output without changing the mode selector:
+- **Directory chips** — Triggered by `ls` or `cd`. Shows file and folder chips for rapid prompt building. File and image previews where applicable.
+- **Process chips** — Triggered by `ps`, `top`, `htop`. Shows PID and process name chips. Tap a PID to inspect it in the Level 4 Process Monitor.
```

---

**§4 The Persistent Unified Prompt (line ~47)**
Remove ACTIVITY from mode selector:

```diff
-- **Left (Origin):** Universal Mode Selector (CMD, SEARCH, AI, ACTIVITY). Not removable.
+- **Left (Origin):** Universal Mode Selector (CMD, SEARCH, AI). Not removable.
```

---

**§14 Deep Inspection & Recovery (LVL 4) (line ~453–461)**
Rewrite to describe btop-style monitor:

```diff
-Level 4 provides three sub-views:
-
-- **Detail View** — Structured metadata: CPU/memory, event history, config, security audit.
-- **Buffer View** — Hex dump of the target process's memory (read-only, disabled by default, requires privilege elevation).
-- **Tactical Reset (God Mode)** — Low-overhead wireframe diagnostics of the entire system. Press `Ctrl+Alt+Backspace` from anywhere, or use the bezel button.
+Level 4 provides three sub-views:
+
+- **Process Monitor** — btop-style interactive system monitor showing CPU/memory graphs, process tree, disk I/O, and network stats. Scoped to the current sector's process tree. Select processes to signal, renice, clone, or move to another sector. When entered from a Level 2 process chip, pre-focuses on the selected PID.
+- **Buffer View** — Hex dump of the target process's memory (read-only, disabled by default, requires privilege elevation).
+- **Tactical Reset (God Mode)** — Full-system btop-style monitor showing all sectors, all services, all processes. Press `Ctrl+Alt+Backspace` from anywhere, or use the bezel button. After privilege elevation, Force Kill, Clone to Sector, and Move to Sector become available.
```

---

## Summary of All Affected Lines

| File | Sections | Change Type |
|---|---|---|
| [Architecture.md](file:///z:/repos/TOS-Desktop-Environment/docs/spec/TOS_v0.1.0_Architecture.md) | §5 (~869), §6.4 (~936), §7.1 (~985), **§7.3 (~1009)**, §7.5.2 (~1057), **§7.7 (~1065)**, §8.2 (~1146), **§9.1 (~1160)**, **§9.3 (~1180)**, §14 (~1551), **§25.2 (~2043, ~2064)**, §29 (~2255) | Remove ACTIVITY mode, reframe Directory/Process as chip overlays, rewrite §9.1 + §9.3 |
| [User-Manual.md](file:///z:/repos/TOS-Desktop-Environment/docs/spec/TOS_v0.1.0_User-Manual.md) | §3 (~30), §4 (~47), §14 (~453) | Remove ACTIVITY, add Dynamic Chip Overlays section, rewrite §14 |

---

## Verification Plan

### Manual Verification
- Read through each modified section to confirm no orphaned references to "Activity Mode" remain
- Grep all spec files for `activity` (case-insensitive) and verify each remaining hit is about "recent activity" (the noun), not the removed mode
- Confirm §7.3 and §7.7 consistently describe Directory and Process as chip overlay behaviors, never as mode selector states
- Confirm the btop-style §9.1 and §9.3 sections are internally consistent with the scope model: Process Monitor = sector, Tactical Reset = global
- Confirm the Level 2 → Level 4 navigation path via process chips works: CMD mode → process command → scraper → PID chip → tap → Level 4 Process Monitor (§9.1)
