# Notifications (Spatial System Alerts)

Notifications in TOS are integrated into the system's **Modular Zoom Hierarchy**, ensuring they inform the user without disrupting the vertical navigation flow.

---

## Visual Theme: LCARS-Style Alert Panels

Notifications are rendered as LCARS-styled panels that respect the current zoom level's context.

### Design Configuration

*   **Temporary Overlays**: Most notifications and transient interactions (like the **Terminal Entry Point**) are rendered as high-level overlays that sit "above" the current zoom level.
    - They do not change the `currentDepth` or `path` of the viewport.
    - They slide in from the screen edges (typically top-right for alerts, bottom-left for terminal input).
*   **Color Coding**: Urgency is indicated via standard LCARS color conventions (Blue=Info, Orange=Warning, Red=Critical).
*   **Contextual Sizing**: When at **Level 1 (Overview)**, notifications are badges on the specific Sector panel. When zoomed in, they are translucent LCARS panels.

---

## Spatial Integration & Layout

### The Notification Log (Level 1 Sector)
The **Notification Log** is a persistent, dedicated **Sector** available in the Level 1 Overview for historical record-keeping.

*   **Access**: Zoom out to Level 1, then zoom into the "Security/Logs" Sector.
*   **Interaction**: Selecting a notification in the log triggers an **Automated Vertical Transition** (formerly Context Jump). The system automatically zooms out to Level 1 and then zooms into the target App's Sector and Focus levels.

### Contextual Placement
Notifications can be configured to follow the user:
- **Sector-Locked**: Notifications only appear on the monitor where the app is located.
- **Focus-Locked**: Notifications follow the `currentDepth` stack and appear on whichever monitor is currently receiving user input.
