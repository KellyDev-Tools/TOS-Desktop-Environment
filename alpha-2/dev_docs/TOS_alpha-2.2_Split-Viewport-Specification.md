# TOS Alpha-2.1 Split Viewport & Pane Management
### Specification v1.0 — Replaces Architecture Specification §11

---

## 1. Philosophy

Split viewports are where TOS earns its power-user credentials. A developer running tests in one pane while editing in another, a sysadmin watching logs alongside a shell, a designer previewing output next to the command that generated it — these are the workflows that make a system feel fast.

The split system follows three principles:

- **THE DISPLAY DECIDES ORIENTATION.** The first split direction is determined by the display's aspect ratio, not by the user having to choose. On a landscape display the first split is vertical (side by side). On a portrait display it is horizontal (top and bottom). The geometry makes the obvious decision automatically.
- **INFINITE BUT BOUNDED.** Splitting is recursive — any pane can be split again. But at the point where a pane can no longer contain meaningful content, splitting is blocked. The boundary is calculated, not arbitrary.
- **MANAGEMENT THROUGH THE BEZEL.** Pane management actions are not scattered across right-click menus or toolbar buttons. They surface as chips when the user activates the Expanded Bezel Command Surface at Level 3, keeping the split view itself clean.

---

## 2. Supported Pane Content Types

Any pane in a split layout can contain one of the following content types:

| Content Type | Description |
| :--- | :--- |
| **Terminal (Command Hub)** | A full Command Hub instance — prompt, chip columns, terminal output module. Shares the sector's shell context. |
| **Level 3 Application** | Any running graphical application in Application Focus. |

These can be combined freely. A split can contain two terminals, two applications, or a terminal alongside an application. Terminal + web portal is the same as terminal + Level 3 app — the web portal is an application that runs at Level 3.

---

## 3. Aspect-Ratio-Driven Split Orientation

### 3.1 First Split Direction

When the user creates the first split in a view, the orientation is determined automatically by the current display's aspect ratio:

| Display Aspect Ratio | First Split Orientation | Result |
| :--- | :--- | :--- |
| Wider than tall (16:9, ultrawide) | **Vertical** — left / right | `[ A ][ B ]` |
| Taller than wide (9:16, portrait) | **Horizontal** — top / bottom | `[ A ] / [ B ]` |
| Square or near-square (±10%) | **Vertical** — left / right (default) | `[ A ][ B ]` |

### 3.2 Subsequent Splits

When a pane is split again, the same aspect-ratio logic applies — but now evaluated against the pane's own dimensions, not the full display. A tall narrow pane splits horizontally. A wide short pane splits vertically.

This means split trees grow naturally with the geometry:

**Ultrawide display (e.g. 32:9):**
```
First split:   [ A      ][ B      ][ C      ]   ← 3 vertical panes feel natural
```

**Standard 16:9, first vertical then horizontal on right pane:**
```
              [ A      ][ B      ]
                        [ C      ]
```

**Portrait display:**
```
First split:  [    A    ]
              [    B    ]
```

### 3.3 Orientation Override

The user can override the auto-detected orientation for any individual split by holding `Shift` while triggering the split shortcut. This rotates the split 90 degrees from the auto-detected default for that operation only. Future splits continue to use auto-detection.

---

## 4. Creating Splits

Splits are created via keyboard shortcuts or via the Expanded Bezel action chips (see §8).

| Action | Shortcut |
| :--- | :--- |
| Split focused pane (auto-orientation) | `Ctrl+\` |
| Split focused pane (orientation override) | `Shift+Ctrl+\` |
| Split and open a specific app in new pane | `Ctrl+\` then select from app chip list |
| Close focused pane | `Ctrl+W` |
| Move focus between panes | `Ctrl+Arrow` |
| Resize pane (move divider) | Drag divider |
| Equalize all pane weights | Double-click any divider |

---

## 5. Minimum Pane Size & Split Blocking

Splitting is recursive but bounded. A pane cannot be split if doing so would produce a child pane smaller than the **minimum viable size** for its content type.

### 5.1 Minimum Size Calculation

The minimum is determined by taking the larger of two constraints:

**Constraint 1 — Ratio minimum:**
No pane may be smaller than **1/6 of the total split axis** (width for vertical splits, height for horizontal splits). On a 1920px wide display, no pane may be narrower than 320px.

**Constraint 2 — Content-aware minimum:**

| Content Type | Minimum Width | Minimum Height |
| :--- | :--- | :--- |
| Terminal (Command Hub) | 400px | 200px |
| Level 3 Application | 320px | 240px |

The larger of the ratio minimum and the content-aware minimum applies. If splitting a pane would produce any child pane smaller than this value on the relevant axis, the split is blocked.

### 5.2 Blocked Split Feedback

When a split is blocked, the `Ctrl+\` shortcut produces a brief amber flash on the pane border and an earcon indicating the action is unavailable. No error message. The visual feedback is immediate and self-explanatory.

---

## 6. Divider Behaviour

Dividers between panes are freely draggable to any position within the bounds set by the minimum pane size constraints (§5). There are no fixed snap points by default.

**Snap assist (optional):** when dragging a divider, it softly snaps to 50% if released within 5% of center. This snap can be disabled in **Settings → Interface → Split Viewport → Divider Snap**.

**Divider appearance:** dividers render as thin LCARS-styled lines. On hover or touch, they thicken slightly to indicate draggability. The focused pane always has an amber border; dividers adjacent to the focused pane are slightly brighter than others.

---

## 7. Pane Focus & Input Routing

Only one pane is focused at a time. The focused pane:
- Receives all keyboard input
- Has its amber border active
- Is the target of all Expanded Bezel prompt commands

Unfocused panes:
- Remain fully live — applications keep running, terminals keep streaming output
- Render at full opacity (unlike the bezel expansion zoom-out, which dims the background — split panes are peers, not backgrounds)
- Do not receive keyboard input

Focus moves between panes with `Ctrl+Arrow`. Clicking or tapping any pane also focuses it immediately.

---

## 8. Pane Management via Expanded Bezel

All pane management actions are accessed through the **Expanded Bezel Command Surface** (Expanded Bezel Specification). When the user activates the expanded bezel while at Level 3 with a split layout active, a row of pane management chips appears above the prompt alongside the standard context chips:

```
┌──────────────────────────────────────────────────────────────────┐
│  [⛶ Fullscreen]  [⇄ Swap]  [⊞ Detach →Sector]  [💾 Save Layout] │
└──────────────────────────────────────────────────────────────────┘
```

These chips operate on the **focused pane** at the time the bezel was opened.

### 8.1 Fullscreen — Promote Without Closing

**[⛶ Fullscreen]** expands the focused pane to fill the full viewport temporarily. The split layout is preserved in memory — the other panes continue running in the background. A persistent **[⊞ Return to Split]** chip appears in the Top Bezel to return to the split view at any time.

This is not a level change. `active_level` remains unchanged. It is a temporary visual promotion of one pane, equivalent to a "zoom" within the split.

### 8.2 Swap Pane Positions

**[⇄ Swap]** swaps the focused pane's position with an adjacent pane. If there is only one adjacent pane, the swap happens immediately. If there are multiple adjacent panes, a secondary chip set appears showing each neighbour for the user to select:

```
Swap with: [← Left pane]  [→ Right pane]  [↓ Bottom pane]
```

Swap is purely visual — it changes the rendered position of panes without affecting their shell contexts, processes, or histories.

### 8.3 Detach to Sector

**[⊞ Detach →Sector]** removes the focused pane from the split and promotes it into a new independent sector at Level 1. The user is offered two context options via secondary chips:

```
[📦 Bring Context]   [✦ Fresh Start]
```

**Bring Context:**
The pane's entire terminal output area is moved to the new sector, along with its PTY, process tree, shell history, cwd, and any backgrounded processes. The new sector is an identical continuation of the pane as it existed in the split. From the user's perspective the terminal just moved — nothing restarts, nothing is lost.

Technically: the Brain re-parents the PTY and its process group from the source sector's hub to a new sector's hub. The terminal output module instance is transferred with its full buffer. The source sector's split layout closes the detached pane and rebalances remaining panes.

**Fresh Start:**
The pane's running process is detached from its terminal output area and re-attached to a new terminal output area in the new sector as a **background process**. The original process keeps running — it is not killed — but it is no longer connected to any interactive terminal. The new sector opens with a clean shell in the same `cwd`. The user can reattach to the background process at any time via the Activity mode.

Technically: the Brain sends `SIGCONT` to the process group, detaches the PTY, and attaches the process as a tracked background job in the new sector's process registry. A background job chip appears in the new sector's right chip column indicating the detached process is running.

### 8.4 Save Layout as Template

**[💾 Save Layout]** serialises the current split configuration — pane count, orientations, weights, and content types — as a named sector template (`.tos-template`). The user is prompted for a name. The saved template appears in the sector tile context menu under **New Sector from Template** and in the Marketplace under the user's local templates.

The template saves layout geometry and content types but not content state — it is a structural blueprint, not a snapshot. A "Rust Dev" template might save a 60/40 vertical split of terminal + editor, but when applied it opens fresh instances, not the saved session.

---

## 9. Split State Persistence

Split layouts are part of the sector's `hub_layout` object and are persisted to the session file as specced in the Session Persistence Specification §4. On restore:

- All panes are recreated with their saved weights and orientations.
- Terminal panes have their scrollback histories loaded before the shell spawns.
- Level 3 application panes attempt to relaunch the application in the same position. If the application is no longer running, the pane renders a **[Relaunch]** chip in place of the application.

---

## 10. Relationship with Existing Architecture

### 10.1 Replaces

This specification fully replaces Architecture Specification §11 (Split Viewports), which contained only a brief mention of split viewports without defining behaviour, interactions, or constraints.

### 10.2 Relationship with Terminal Output Modules

Architecture Specification §7.5 notes that the Cinematic Triangular Module supports a "pinwheel" arrangement of multiple terminals. That capability is preserved and complementary to this spec. The pinwheel is a **module-defined layout** (`layout_type: module_defined`) within a single pane — it is not a split viewport. A split viewport can contain a pane running the Cinematic Triangular Module in pinwheel mode alongside another pane running a Level 3 application. The two systems compose without conflict.

### 10.3 Relationship with the Expanded Bezel

Split pane management chips (§8) appear in the Expanded Bezel Command Surface when a split layout is active at Level 3. This follows the same pattern as the busy-PTY chips in the Expanded Bezel Specification — the bezel surface adapts its chip set to the current context. No new UI surface is required.

### 10.4 IPC Contracts

| Message | Effect |
| :--- | :--- |
| `split_create:<orientation>` | Creates a new split in the focused pane (`vertical` / `horizontal` / `auto`) |
| `split_close:<pane_id>` | Closes a pane and rebalances remaining panes |
| `split_focus:<pane_id>` | Moves focus to the specified pane |
| `split_focus_direction:<dir>` | Moves focus directionally (`left` / `right` / `up` / `down`) |
| `split_resize:<pane_id>:<weight>` | Sets a pane's weight (0.0–1.0, sibling weights rebalance) |
| `split_equalize` | Sets all sibling panes to equal weights |
| `split_fullscreen:<pane_id>` | Promotes a pane to fullscreen (layout preserved) |
| `split_fullscreen_exit` | Returns from fullscreen to split layout |
| `split_swap:<pane_id_a>:<pane_id_b>` | Swaps two panes' positions |
| `split_detach:<pane_id>:context` | Detaches pane to new sector, bringing context |
| `split_detach:<pane_id>:fresh` | Detaches pane to new sector as fresh start |
| `split_save_template:<name>` | Saves current layout as a named sector template |

---

*TOS Alpha-2.1 // Split Viewport & Pane Management v1.0 // Replaces Architecture Specification §11*
