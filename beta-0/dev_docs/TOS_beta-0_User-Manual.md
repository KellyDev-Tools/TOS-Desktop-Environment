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

From any level, tap the bottom bezel or swipe up from the bottom edge to open the **Expanded Bezel Command Surface**. This overlay brings the full prompt — with chip columns, AI co-pilot chips, and warning chips — to the foreground, while the current view zooms back slightly. You can run commands from Level 3 without leaving your application.

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

## 8. AI Co-Pilot System

TOS ships with two AI behavior modules pre-installed:

- **Passive Observer** — Watches your terminal silently. If a command fails, it surfaces a correction chip. If a command runs too long, it offers an explanation chip. Always passive, never intrusive.
- **Chat Companion** — Provides a full chat interface in `[AI]` mode. Ask anything in plain English; the AI stages commands for you to review and submit.

Additional AI behaviors and backends can be installed from the Marketplace. All AI is removable via **Settings → AI → Behaviors**.

**Safety guarantee:** The AI never executes a command. Every suggestion ends up staged in the prompt — visible, editable, under your control.

---

## 9. Multi-Sensory Interface

TOS uses immersive feedback loops to minimize cognitive load:

- **Earcons** — Distinct audio cues for mode switches, level zooms, modal actions, and data commits.
- **Haptic Pulses** — Physical confirmation of virtual actions on supported hardware.
- **Alert Levels** — Green (normal), Yellow (caution — ambient audio shifts), Red (critical — repeating tone, haptic escalation).

All audio and haptic feedback is configurable in **Settings → Interface → Audio** and can be disabled entirely.

---

## 10. Global Shortcuts

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

## 11. Configuration: System Settings

Access the **System Settings** modal (⚙ icon, Top Bezel Right) to adjust:

1. **Appearance** — Theme, Terminal Output Module, font size, color palette.
2. **AI** — Backend selection, behavior modules, ghost text, disable master switch.
3. **Security** — Trust configuration for command classes, per-sector overrides, deep inspection toggle.
4. **Interface** — Audio/haptic feedback, animation speeds, Expanded Bezel behaviour, split viewport snap settings.
5. **Network** — Remote access port, mDNS advertisement, view port map.
6. **Sessions** — Import/export session files, browse named sessions.
7. **System** — Default shell, sandboxing tiers, resource limits per sector.
8. **Onboarding** — Replay the guided tour, reset hints, suppress hints.

---

## 12. Session Persistence

TOS automatically saves your workspace state continuously. When you return, your sectors, terminals, histories, and AI chat are exactly where you left them — with no restore notification, no animation, no prompt.

**Named Sessions** allow you to save and recall distinct workspace states per sector (e.g., "rust-project", "client-work"). Save via: secondary select on a sector tile → **Save Session As...**

Session files (`.tos-session`) are portable — copy them to another machine and load them directly via **Settings → Sessions → Import** or by dropping the file onto a sector tile.

---

## 13. Deep Inspection & Recovery (LVL 4)

Level 4 provides three sub-views:

- **Detail View** — Structured metadata: CPU/memory, event history, config, security audit.
- **Buffer View** — Hex dump of the target process's memory (read-only, disabled by default, requires privilege elevation).
- **Tactical Reset (God Mode)** — Low-overhead wireframe diagnostics of the entire system. Press `Ctrl+Alt+Backspace` from anywhere, or use the bezel button.

During Tactical Reset, the prompt is locked and the Expanded Bezel is disabled. Force Kill and other destructive actions require re-authentication. Remote guests cannot initiate or interact with Tactical Reset.

---

*TOS Terminal On Steroids // User Manual*
