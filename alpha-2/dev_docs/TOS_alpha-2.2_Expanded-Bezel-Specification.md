# TOS Alpha-2.1 Expanded Bezel Command Surface
### Specification v1.0 — Supplement to Architecture Specification §7.1 & §8.1

---

## 1. Philosophy

The Persistent Unified Prompt exists at every level. It is always visible, always reachable. But in its collapsed state it is just an input — it does not surface output, context chips, or navigation. The Expanded Bezel Command Surface unlocks the full power of the prompt without requiring the user to navigate anywhere.

This is not a new level. It is a persistent overlay state that sits above whatever level the user is currently on. It can be invoked at Level 1 while surveying sectors, at Level 3 while a graphical app is in focus, or anywhere in between. The user's current view zooms back spatially to make room. Output appears. Action chips offer what to do next. The user decides whether to dive in or dismiss and continue what they were doing.

---

## 2. Triggers

The Expanded Bezel Command Surface is triggered by any of three equivalent gestures:

| Trigger | Description |
| :--- | :--- |
| **Tap bottom bezel** | Tap anywhere on the collapsed bottom bezel bar |
| **Swipe up from bottom edge** | Upward swipe gesture from the bottom bezel edge |
| **Split button in Top Bezel** | A dedicated split/expand button in the Top Bezel Center section |

The Top Bezel split button is the primary discoverable trigger — it is always visible and labeled. The bottom bezel tap and swipe-up are power-user shortcuts once the gesture is learned.

All three triggers are equivalent and produce the same expansion animation and state.

---

## 3. Expansion Animation

When triggered, the current view undergoes a **spatial zoom-out**: the content scales down slightly along the z-axis, as if the user has stepped back from the screen. This is consistent with the existing zoom model used between levels — the same depth language, applied to a new gesture.

The bottom bezel animates upward, expanding to reveal:
- The full **Persistent Unified Prompt** with all active bezel overlays visible
- The **left and right chip columns** populated with context from the current sector
- All active **ambient hint chips**, **AI co-pilot chips**, and **warning chips**

The expanded surface occupies the lower portion of the viewport. The zoomed-out current view remains visible behind it — dimmed slightly but not occluded. The user can see what they were doing while they type.

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
│              │                      │               │
└──────────────┴──────────────────────┴───────────────┘
│  BOTTOM BEZEL (expanded)                            │
└─────────────────────────────────────────────────────┘
```

---

## 4. Level 3 App Navigation

While the Expanded Bezel Command Surface is open, the zoomed-out content layer becomes a **lateral swipe surface** for Level 3 applications. The user can swipe left/right (or press `←`/`→`) to cycle through open Level 3 applications without closing the expanded bezel.

This allows a workflow like:
1. Open the expanded bezel while a text editor is in focus.
2. Type a command. While typing, swipe right to check a running process in another app.
3. Submit the command. Output appears. Swipe back left to the editor.
4. Dismiss the bezel — the editor is still in focus, unchanged.

The lateral navigation does not change the active sector. It is a visual preview gesture — the same app that was in focus when the bezel was opened remains the active Level 3 context for the prompt's shell.

---

## 5. Shell Context

When a command is submitted from the Expanded Bezel, the shell context is resolved as follows:

### 5.1 Active PTY Available

If the current sector's active Command Hub PTY is idle (not running a command), the submitted command is routed to that PTY. The user gets the full context of their working directory, environment variables, and shell history. This is the default and most common case — the bezel prompt is a direct extension of the Command Hub.

### 5.2 Active PTY Busy

If the active PTY is currently running a command and has not returned, the prompt reflects this state visually. The standard input area is overlaid with three options presented as chips:

```
┌──────────────────────────────────────────────────────────────┐
│  [⏹ Stop (Ctrl+C)]   [⧉ New Terminal]   [⏳ Wait...]        │
└──────────────────────────────────────────────────────────────┘
```

- **[⏹ Stop (Ctrl+C)]** — sends `SIGINT` to the running process, freeing the PTY. The prompt returns to normal input state immediately.
- **[⧉ New Terminal]** — spawns a fresh ephemeral shell pane in the current sector. Commands run in this pane are associated with the sector but do not interfere with the running process. The new pane closes automatically when the bezel is dismissed unless the user chooses to promote it (see §7).
- **[⏳ Wait...]** — dismisses the chip overlay and returns the prompt to a waiting state. The user can continue watching the output and try again when the process completes.

The Stop button is always visible regardless of which chip is selected — `Ctrl+C` is never more than one tap away.

---

## 6. Output Display

When a command completes, its output is rendered by the active **Terminal Output Module** in an overlay panel that expands upward from the prompt within the bezel surface. The zoomed-out background view remains visible behind the output.

The output panel has a maximum height of 40% of the viewport. If output exceeds this height it becomes scrollable within the panel. Long outputs do not push the current view further away.

### 6.1 Action Chips

When output appears, a row of **action chips** renders immediately below it. These give the user explicit choices for what to do next:

| Chip | Action |
| :--- | :--- |
| **[→ Command Hub]** | Zooms into the sector's Level 2 Command Hub with the output visible in the terminal buffer. The bezel collapses. |
| **[⊞ Split View]** | Opens a split layout — output and terminal on one side, current Level 3 app on the other. Both remain interactive. |
| **[✕ Dismiss]** | Collapses the bezel. Output is saved to the sector's terminal history but not displayed. |
| **[⧉ Keep Open]** | Pins the output panel open and returns focus to the background view. The bezel remains expanded but unobtrusive. |

The action chips appear automatically after output completes. They do not appear during streaming output — they wait for the command to return.

### 6.2 Error Output

If the command exits with a non-zero code, the output panel border renders in amber. The AI Passive Observer chip (if installed and active) may surface a correction or explanation chip alongside the standard action chips, exactly as it would in the Command Hub.

---

## 7. Dismiss Behaviour

The default dismiss behaviour after output appears is user-configurable in **Settings → Interface → Expanded Bezel**:

| Setting | Behaviour |
| :--- | :--- |
| **Stay open** | The bezel remains expanded after output. User must explicitly dismiss via chip or gesture. |
| **Auto-collapse on complete** | The bezel collapses automatically when a command completes with exit code 0. Errors keep it open. |
| **Auto-collapse after timeout** | The bezel collapses N seconds after output completes if no further input is detected. N is configurable (default: 5s). |

Regardless of the setting, the user can always manually collapse the bezel by:
- Tapping the Top Bezel split button again
- Swiping down on the expanded bezel area
- Pressing `Esc`

---

## 8. Ephemeral Pane Promotion

When a **[⧉ New Terminal]** pane is created from the busy-PTY state (§5.2), it is ephemeral by default — it exists only for the duration of the expanded bezel session and closes when the bezel is dismissed.

If the user runs commands in the ephemeral pane that they want to keep, a **[⊞ Promote to Split]** chip appears alongside the existing action chips after the first command completes. Tapping it converts the ephemeral pane into a persistent split pane in the sector's hub layout, saved to the session file as per the Session Persistence Specification §4.

---

## 9. Architectural Position

The Expanded Bezel Command Surface is **not a level** in the six-level hierarchy. It does not change `active_level` in the session state. It is a persistent overlay that can be opened and closed at any level without affecting navigation state.

The Brain tracks a single boolean flag: `bezel_expanded`. When true, the Face renders the expanded surface. When false, the standard collapsed bezel is shown. This flag is not persisted to the session file — the bezel always opens collapsed on launch.

### 9.1 IPC Contracts

| Message | Effect |
| :--- | :--- |
| `bezel_expand` | Opens the Expanded Bezel Command Surface |
| `bezel_collapse` | Collapses the surface back to standard bezel |
| `bezel_output_action:<action>` | Triggers a post-output action chip (`hub`, `split`, `dismiss`, `keep`) |
| `bezel_pane_promote` | Promotes the ephemeral pane to a persistent sector split |
| `bezel_swipe:<direction>` | Navigates between Level 3 apps in the zoomed-out layer (`left` / `right`) |

### 9.2 Relationship with Existing Bezel Spec

The Bottom Bezel Segment is described in Architecture Specification §8.1 as housing the Persistent Unified Prompt in a **strictly static, locked assembly with no configurable slots**. This specification extends that definition:

- The collapsed state is unchanged — the bottom bezel remains locked and static as specced.
- The expanded state is a new mode of the same segment, not a new segment. The lock applies to slot configurability, not to expansion capability.
- The Persistent Unified Prompt at Level 6 (Tactical Reset) remains locked and non-expandable as per Architecture Specification §20. The expansion gesture is disabled during Tactical Reset.

---

## 10. Updated Level Hierarchy Note

The six-level hierarchy (Architecture Specification §2) is unchanged. For documentation clarity, the Expanded Bezel state should be noted in the hierarchy table as a cross-cutting overlay:

| Level | Name | Expanded Bezel Available? |
| :--- | :--- | :--- |
| LVL 1 | Global Overview | ✓ Yes |
| LVL 2 | Command Hub | ✓ Yes (extends the existing prompt) |
| LVL 3 | Application Focus | ✓ Yes — primary use case |
| LVL 4 | Detail Inspector | ✓ Yes |
| LVL 5 | Buffer View | ✓ Yes |
| LVL 6 | Tactical Reset | ✗ Disabled |

---

*TOS Alpha-2.1 // Expanded Bezel Command Surface v1.0 // Supplement to Architecture Specification §7.1 & §8.1*
