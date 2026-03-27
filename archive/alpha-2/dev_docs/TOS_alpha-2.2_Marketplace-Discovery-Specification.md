# TOS Alpha-2.2 Marketplace Discovery & Browse Experience
### Specification v1.0 — Supplement to Ecosystem Specification §2

---

## 1. Philosophy

The marketplace is where TOS grows. A user who discovers the right theme, the right AI behavior, or the right shell module on day two has a fundamentally different relationship with the system than one who never knew those options existed. Discovery is not a feature — it is the mechanism by which TOS becomes personal.

The existing Ecosystem Specification §2 defines the installation pipeline, package formats, and permission model thoroughly. This document specifies what comes before that pipeline: the surface a user actually sees, browses, and acts from. It builds on the existing `tos-marketplaced` daemon without replacing any of its mechanics.

Two principles:

- **CURATED FIRST, EXHAUSTIVE SECOND.** The home view is not a firehose. It is a considered set of picks and categories. Exhaustive search is one tap away but it is not the first thing a user sees.
- **THE DETAIL PAGE EARNS THE INSTALL.** A user should understand what a module does, what it looks like, and what it asks for before they commit. The detail page does that work. The install button is the last thing on the page, not the first.

---

## 2. UI Surface

The marketplace opens as a **Level 3 Application Focus** — the same full-screen surface used by any graphical application in TOS. It is accessed via:

- The **Web Portal satellite button** in the Top Bezel Right section — repurposed as a dual-purpose button: short tap opens the Web Portal sharing overlay as before; long press opens the Marketplace.
- The command `tos marketplace` typed in any Command Hub prompt.
- **Settings → [any module category] → Browse Marketplace** — context-aware entry that opens the marketplace pre-filtered to the relevant category.
- The AI suggestion chip that fires when a module conflict or missing module is detected (e.g. "Install a Chat Companion from the Marketplace").

When opened, the marketplace registers as a standard Level 3 application. The Tactical Bezel remains visible. `Esc` or zoom-out returns the user to their previous level and sector without disrupting any running processes.

> **NOTE:** The marketplace is read from `tos-marketplaced` (ephemeral port, discovered via Brain service registry). This document describes the Face-side rendering of the data that daemon provides. No changes to the daemon or its package verification logic are required.

---

## 3. Home View

The marketplace home view is divided into two vertical sections: **Featured** (top) and **Categories** (below).

### 3.1 Featured Strip

A horizontally scrollable strip of curated module cards selected by the TOS team. Featured picks are served by `tos-marketplaced` as a signed, versioned `featured.json` manifest that updates independently of TOS releases.

Each featured card shows:
- Module name and type badge (e.g. `THEME`, `AI BEHAVIOR`, `SHELL`)
- A single hero screenshot or animation
- A one-line description (max 80 characters)
- Install count and star rating
- **[View]** button — navigates to the detail page

Featured cards are larger than category browse cards — they are meant to catch attention, not just list information.

```
┌─────────────────────────────────────────────────────────────────────┐
│  FEATURED                                              [← →  1/6]  │
├──────────────────┬──────────────────┬──────────────────────────────┤
│  ┌────────────┐  │  ┌────────────┐  │  ┌────────────┐              │
│  │ screenshot │  │  │ screenshot │  │  │ screenshot │  ···         │
│  └────────────┘  │  └────────────┘  │  └────────────┘              │
│  Void Theme      │  Git Expert      │  Starship Shell               │
│  THEME           │  AI BEHAVIOR     │  SHELL                        │
│  Dark LCARS      │  Git-aware chips │  Rust prompt                  │
│  ★ 4.8  12.4k   │  ★ 4.6  8.1k    │  ★ 4.9  21k                  │
│  [View]          │  [View]          │  [View]                       │
└──────────────────┴──────────────────┴──────────────────────────────┘
```

### 3.2 Category Grid

Below the featured strip, all module categories are presented as a grid of tappable category tiles. Each tile shows the category name, module type badge, and a count of available modules.

| Category Tile | Module Type | Description |
| :--- | :--- | :--- |
| Themes | `.tos-theme` | Visual appearance, color palettes, LCARS variants |
| Shells | `.tos-shell` | Shell implementations with OSC integration |
| Terminal Output | `.tos-terminal` | Terminal rendering modules |
| AI Backends | `.tos-ai` | LLM connections — local and remote |
| AI Behaviors | `.tos-aibehavior` | Co-pilot interaction patterns |
| Sector Types | `.tos-sector` | Workspace presets and specialized sector logic |
| Bezel Components | `.tos-bezel` | Dockable bezel slot components |
| Audio Themes | `.tos-audio` | Earcon sets and ambient audio layers |

Tapping a category tile navigates to the **Category Browse View** for that type.

---

## 4. Category Browse View

The category browse view shows all available modules of a given type in a scrollable grid. Each module is represented by a **Browse Card**.

### 4.1 Browse Card

```
┌──────────────────────────────────┐
│  ┌──────────────────────────┐    │
│  │      screenshot          │    │
│  └──────────────────────────┘    │
│  Catppuccin Theme      THEME     │
│  Soothing pastel LCARS variant   │
│  ★ 4.7  ↓ 9.2k        [View]   │
└──────────────────────────────────┘
```

- **Screenshot** — a static preview image. For themes, this shows the TOS interface with the theme applied. For AI behaviors, an annotated screenshot showing the chip or panel it produces.
- **Name and type badge**
- **One-line description** (max 80 characters)
- **Star rating and download count**
- **[View]** — navigates to the detail page. There is no install button on the browse card. Install happens on the detail page only, after the user has reviewed permissions.

### 4.2 Sorting & Filtering

The category browse view has a sort/filter bar at the top:

```
[Most Downloaded ▾]   [All ▾]   [🔍 Search in Themes...]
```

- **Sort:** Most Downloaded / Highest Rated / Newest
- **Filter:** All / Free / By the TOS Team / Compatible with current setup
- **Search:** filters the current category in real time

"Compatible with current setup" filters out modules that declare capability requirements the user's current AI backend or platform cannot satisfy — preventing the frustration of installing something that silently does nothing.

---

## 5. Module Detail Page

Tapping **[View]** on any card navigates to the module's full detail page. This is a dedicated full-screen view within the Level 3 marketplace app.

### 5.1 Layout

```
┌─────────────────────────────────────────────────────────────────────┐
│  ← Back                                                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  [  Screenshot 1  ] [  Screenshot 2  ] [  Screenshot 3  ]  ···     │
│                                                                     │
├────────────────────────────────┬────────────────────────────────────┤
│  Catppuccin Theme              │  THEME                             │
│  by tos-community              │  ★ 4.7  (312 ratings)             │
│                                │  ↓ 9,241 installs                 │
│  A soothing pastel recolor     │                                    │
│  of the default LCARS dark     │  Version: 2.1.0                   │
│  theme. Supports high-contrast │  Updated: Feb 28 2026             │
│  and reduced-motion modes.     │  License: MIT                     │
│                                │                                    │
│                                │  [Review Permissions & Install]   │
├────────────────────────────────┴────────────────────────────────────┤
│  WHAT IT CHANGES                                                    │
│  Full description, changelog, compatibility notes...                │
│                                                                     │
│  PERMISSIONS REQUIRED                                               │
│  • None — theme modules are static assets                          │
│                                                                     │
│  RATINGS & REVIEWS                                                  │
│  ★★★★★  "Best theme in the marketplace" — user_a  Mar 01          │
│  ★★★★☆  "Colours are perfect, wish it had more earcons" — user_b   │
└─────────────────────────────────────────────────────────────────────┘
```

### 5.2 Screenshot Gallery

A horizontally scrollable gallery of screenshots at the top of the page. For modules with a visual component (themes, terminal modules, bezel components), screenshots show the TOS interface with the module active. For non-visual modules (AI backends, shells), screenshots show annotated diagrams or example interactions.

Module authors submit screenshots as part of their package manifest. The marketplace validates that at least one screenshot is present before listing a module.

### 5.3 Permissions Section

The permissions section on the detail page mirrors exactly what will be shown in the install review step — so the user sees permissions before they tap install, not as a surprise during it. This is intentional: a user who notices a permission they are uncomfortable with can decide not to proceed before they are mid-flow.

Permissions are rendered as human-readable statements, not raw manifest keys:

| Manifest Permission | Displayed As |
| :--- | :--- |
| `terminal_read = true` | Can read your terminal output |
| `prompt_read = true` | Can read your current prompt |
| `network = true` | Can make network requests |
| `filesystem = true` | Can read and write files |
| No permissions declared | No special permissions required |

### 5.4 The Install Button

**[Review Permissions & Install]** is the single call to action on the detail page. It is positioned in the top-right metadata block — visible without scrolling, but not so prominent that it draws attention before the user has read the page.

---

## 6. Install Flow

Tapping **[Review Permissions & Install]** from the detail page initiates the install flow. This maps onto the existing Ecosystem Specification §2.2 installation pipeline with a defined Face-side experience.

### 6.1 Permission Review Step

A modal overlay appears on top of the detail page showing the full permissions list for the module. This is the same information shown in the detail page permissions section, but now presented as a formal review before commitment.

```
┌──────────────────────────────────────────────┐
│  Install Catppuccin Theme?                   │
│                                              │
│  This module requires:                       │
│  ✓ No special permissions                   │
│                                              │
│  Signed by: tos-community                   │
│  Verified: ✓ Signature valid                │
│                                              │
│         [Cancel]   [Install]                 │
└──────────────────────────────────────────────┘
```

For modules with no permissions, the review step is lightweight — it primarily confirms the author and signature validity. For modules with significant permissions (e.g. `network = true`, `filesystem = true`), the modal is more prominent and each permission is listed individually with a brief explanation.

There is no second confirmation after this step. **[Install]** triggers the download immediately.

### 6.2 Download Progress

After **[Install]** is tapped:

- The modal closes.
- A download progress bar appears at the bottom of the detail page, below the screenshots and description. It shows percentage and a cancel button.
- The **[Review Permissions & Install]** button is replaced with **[Installing... x%]** and is non-interactive during download.
- The user can navigate away from the detail page. The download continues in the background via `tos-marketplaced`.

### 6.3 Completion

When the install completes:

- A notification is pushed to the **TOS Log** (via the existing `notify_push` IPC): `Module installed: Catppuccin Theme — available in Settings → Appearance → Theme`.
- If the user is still on the detail page, the progress bar disappears and the install button becomes **[Installed ✓]**.
- If the user has navigated away, the log notification is the only signal. No toast, no alert, no interruption.
- The module is immediately available in its relevant Settings category.

### 6.4 Install Failure

If the download or signature verification fails:

- The progress bar turns amber and shows the failure reason.
- The install button returns to **[Review Permissions & Install]** so the user can retry.
- A failure notification is pushed to the TOS Log.

---

## 7. Search

Global marketplace search is accessible from the home view via a persistent search bar at the top of the page. Search queries are sent to `tos-marketplaced` which searches across all module types simultaneously.

Results are grouped by category. Each result renders as a standard browse card. Tapping any result navigates to its detail page.

The AI integration from Ecosystem Specification §2.3 is preserved: typing a natural language query (e.g. "I need a Git integration") in the search bar triggers the AI-assisted discovery path, which returns curated suggestions rather than keyword matches.

---

## 8. Installed State in Browse

When a user browses a category and encounters a module they have already installed, the browse card and detail page reflect this:

- Browse card shows a **[Installed ✓]** badge in place of the star rating row.
- Detail page shows **[Installed ✓]** in place of the install button, with a secondary link: **[Manage in Settings →]** which navigates directly to the relevant Settings category.

There is no separate "My Modules" section in the marketplace. Installed module management lives in Settings, as already specced for each module type. The marketplace is for discovery; Settings is for management.

---

## 9. IPC Contracts

The following IPC messages are added to support the marketplace UI. All communicate with `tos-marketplaced` (ephemeral port, discovered via Brain service registry).

| Message | Effect |
| :--- | :--- |
| `marketplace_home` | Returns featured manifest and category counts |
| `marketplace_category:<type>` | Returns paginated module list for a category |
| `marketplace_detail:<id>` | Returns full module metadata, screenshots, permissions |
| `marketplace_search:<query>` | Full-text search across all module types |
| `marketplace_search_ai:<query>` | AI-assisted natural language discovery |
| `marketplace_install:<id>` | Initiates install after permission review (existing) |
| `marketplace_install_cancel:<id>` | Cancels an in-progress download |
| `marketplace_install_status:<id>` | Returns current install progress (0–100, or error) |

The existing `marketplace_search` and `marketplace_install` messages from Architecture Specification §28.2 are preserved and extended, not replaced.

---

*TOS Alpha-2.2 // Marketplace Discovery & Browse Experience v1.0 // Supplement to Ecosystem Specification §2*
