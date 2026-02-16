# Spatial Model Reconciliation (Modular)

This document establishes a unified, implementation-focused spatial model for TOS, prioritizing a modular, N-level "Zoom" hierarchy.

---

## The Unified Concept: The Modular Zoom Hierarchy

Instead of a fixed structure, TOS uses a **recursive N-level hierarchy**. Navigation is strictly "Zoom In" to increase granularity and "Zoom Out" to increase context.

### Level 1: The Overview (Global/Sectors)
*The "Birds-Eye" view of all virtual workspaces.*
- **Visual**: All **Sectors** (virtual workspaces) are visible as large LCARS panels on the global canvas.
- **Interaction**: Select/Click a Sector to zoom into its Level 2 state.
- **Implementation**: A grid or layout of virtual screen buffers managed by the compositor.

### Level 2: The Sector (App Launcher)
*The workspace for a specific virtual environment.*
- **Visual**: A grid of LCARS buttons representing **Apps** currently running within this specific Sector.
- **Interaction**: 
    - Click an App button $\rightarrow$ Zoom into Level 3.
    - Pinch Out / Escape $\rightarrow$ Zoom out to Level 1.
- **Badge**: Shows window counts (e.g., "Firefox [3]").

### Level 3: The Focus (Window/Content)
*The active application content.*
- **Visual**: A single window (or tiled windows) filling the LCARS frame.
- **Interaction**: 
    - Pinch Out / Escape $\rightarrow$ Zoom out to Level 2.
    - If the app has multiple windows, zooming out first shows a "Window Picker" (Thumbnail grid).

### Level N: Deep Context (Sub-App/Component)
*Optional deeper levels for complex applications.*
- **Visual**: Specific sub-views (e.g., a specific folder in a file manager, a specific layer in a design tool).
- **Interaction**: Continues the Zoom In/Out pattern.

---

## Simplified Navigation Rules

1.  **Uniform Zooming**: Transitions between levels are global within the current **Viewport**. Navigating into an app focuses the entire viewport on that specific application's content. Opening a new window does not change the zoom level; instead, the new window is added to the current app's window set. If the user wishes to view multiple contents simultaneously, they invoke the **Split** action, which divides the physical monitor into separate **Viewports**.
2.  **No Arbitrary Panning**: View navigation is strictly vertical (Zoom In/Out). The user never "pans" across a surface to find content. All content is reached by moving up to a parent context and down into a child context.
3.  **Object Manipulation (The "Move" Exception)**: While *navigation* is vertical, *management* of resources allows for "sideways" movement of objects at the appropriate context level. Specifically:
    - At **Level 1 (Global Overview)**, a user may drag a window from one Sector's thumbnail to another. This is an administrative move of a resource between stacks, not a navigational pan.
    - Windows can also be "sent" to other Sectors via a "Teleport" or "Send to Sector" command in the LCARS frame.
4.  **Strict Ownership**: Each level $L$ contains a set of children at level $L+1$. A child belongs to exactly one parent.
5.  **Automated Vertical Transitions (Formerly "Context Jumps")**: Switching from one task to another across different branches of the hierarchy is handled by the system automatically "Zooming Out" to the nearest common ancestor and "Zooming In" to the target. This ensures the user's mental model of the stack remains intact.

---

## Simplified Window Tiling

Tiling is a **Level N State** achieved via view fragmentation:

1.  User is at Level $N$ (Focus).
2.  User hits "Split".
3.  The physical monitor's viewport divides into two panes.
4.  The current focus stays in one pane.
5.  The other pane/sector reverts to the **Level 1 Overview** (list of all Sectors) or **Level 2** (app launcher for the current Sector) to select a sibling window or a different Sector to tile alongside the current focus.

---

## Implementation Summary

| Component | Logic |
|-----------|-------|
| **State** | Each active **Viewport** (monitor or pane) maintains its own integer `currentDepth` and a stack `path` (e.g., `[SectorID, AppID, WindowID]`). |
| **Focus** | The leaf node of the current Viewport's `path` stack. |
| **Transition** | Scale/opacity interpolation between `path[n]` and `path[n+1]`. |
| **Visual Morph** | The LCARS container border at level $L$ morphs into the window frame at level $L+1$. |
| **Extensibility** | New levels can be injected by adding metadata to the `path` stack. |
