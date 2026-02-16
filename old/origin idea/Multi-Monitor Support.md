# Multi-Monitor Support (Monitors vs. Sectors)

In TOS, we distinguish between **Physical Monitors** and **Sectors**.

*   **Sector**: A persistent virtual workspace at the base of the zoom hierarchy (**Level 1**). 
*   **Viewport**: A logical display window or pane (e.g., a physical monitor or a split-pane) that maintains its own zoom state and focus stack.
*   **Monitor**: A physical output device that can host one or more Viewports.

---

## Level 1: The Global Overview

The Global Overview is the "Command Center" where all available **Sectors** are visible.

### How it Works

*   **Virtual Workspaces**: Sectors are like GNOME workspaces. The user can have as many as they want, and the compositor dynamically adds or removes Sector nodes at Level 1 based on demand.
*   **Monitor Navigation**: Any physical monitor can host multiple viewports. Each viewport can independently navigate to any Sector and zoom into it.
*   **Multi-Monitor Focus**: Different monitors (or viewports on the same monitor) can view different Sectors simultaneously, or "mirror" a single Sector by zooming into the same one.
*   **Sector Management**: The **Sector Management Panel** allows users to spawn new Sectors, delete unused ones, and arrange their logical grouping on the Level 1 Overview.

---

## Workspace Dynamics

### Sector Persistence

Each Sector maintains its own independent zoom hierarchy and state stack `path` (e.g., `[SectorID, AppID, WindowID]`).

*   **Independent state**: Each viewport maintains its own `path` stack. A monitor zooming into Sector A will see Sector A's specific focused app, while another monitor (or split-pane) zoomed into Sector B sees Sector B's launcher.
*   **Context Switching**: A viewport can "leave" a Sector by zooming out to Level 1 and then "enter" a different Sector.

### Sector & Window Movement

*   **Moving Windows Between Sectors**: To move an active window from Sector A to Sector B:
    1.  Zoom the current Viewport out to **Level 1 (Global Overview)**.
    2.  The user will see a list of open windows within Sector A's panel.
    3.  Drag the target window onto Sector B's panel.
    4.  The window is now reparented to Sector B's stack.
*   **Dragging Between Monitors**: Moving a view between physical monitors is a hardware-level assignment of Viewports.

---
## Configuration

*   **Hardware Settings**: A dedicated panel for adjusting physical monitor resolution, scaling, and orientation. This also allows binding two monitors to act as one large monitor, extending the view of a single Sector across both physical displays.
*   **Sector Layout**: Arrangement of virtual Sectors on the Level 1 grid, allowing for logical grouping and the creation of large "tactical displays" that span multiple virtual workspaces.
