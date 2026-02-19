# TOS UX Design Specification

This document defines the complete user experience design for the TOS (Tactical Operating System), covering the spatial interface hierarchy, interaction patterns, collaboration features, and accessibility considerations.

---

## Table of Contents

1. [Global Overview – Level 1](#1-global-overview--level-1)
2. [Command Hub – Level 2](#2-command-hub--level-2)
3. [Application Focus – Level 3](#3-application-focus--level-3)
4. [Deep Inspection – Levels 4 & 5](#4-deep-inspection--levels-4--5)
5. [Priority Indicators](#5-priority-indicators)
6. [Tactical Mini-Map](#6-tactical-mini-map)
7. [Collaboration UI](#7-collaboration-ui)
8. [Input Abstraction Layer](#8-input-abstraction-layer)
9. [TOS Log](#9-tos-log)
10. [Auditory and Haptic Interface](#10-auditory-and-haptic-interface)
11. [Security Model](#11-security-model)
12. [Application Models and Sector Types](#12-application-models-and-sector-types)
13. [Shell API](#13-shell-api)
14. [Tactical Reset](#14-tactical-reset)
15. [Sector Templates and Marketplace](#15-sector-templates-and-marketplace)
16. [Accessibility](#16-accessibility)

---

## 1. Global Overview – Level 1

The Global Overview displays all sectors (local and remote) as zoomable tiles. Each tile acts as a miniature representation of its corresponding Command Hub.

### 1.1 Sector Tile as a Mini Command Hub

· Borders – The four borders of a sector tile mirror the structural elements of a full Command Hub:
  · Top border – Represents the Tactical Bezel (collapsed state). A thin coloured strip may indicate the sector’s alert status or active collaboration.
  · Bottom border – Embodies the Persistent Unified Prompt – a solid or subtly animated line.
  · Left and right borders – House mode indicators (CMD, DIR, ACT, SEARCH) as small coloured chips or icons, and priority indicator chips (see §5.1 of v1.2).
· Conveyed Information:
  · Active Mode – A coloured chip on the left or right border glows to indicate the sector’s current hub mode.
  · Priority State – Border chips along the edges reflect the sector’s urgency or activity level.
  · Recent Activity – A subtle “wave” animation along the bottom border hints at recent command output or notifications.
  · Collaboration Presence – Tiny avatar dots along the top border show active guests.
· Zoom Transition – When selected, the tile’s borders smoothly expand and resolve into the full Command Hub interface:
  · Top border widens into the Tactical Bezel.
  · Bottom border grows into the Persistent Unified Prompt.
  · Side borders become the left and right “wings” of the hub (chip regions).
  · Priority chips slide into position around the hub’s content area.

### 1.2 Global Overview Bezel

The Tactical Bezel at Level 1 provides system‑level controls.

· Collapsed State – Thin strip along the top edge containing:
  · Settings icon (gear) – one‑click access to global settings.
  · Add Sector button (+).
  · Expand Handle (down chevron).
  · Collaboration Indicator (avatars of active shared sectors) – far right.
· Expanded State – Activated by dragging the handle, clicking, or Ctrl+Space. Reveals a command strip:
  · Navigation – Zoom Out (if applicable), Home (reset overview layout).
  · Sector Management – New Sector, Import Sector, Remote Connection.
  · System – Settings, Updates, Security Dashboard.
  · Collaboration – Share Overview, Active Sessions, Invite Users.
  · View Controls – Toggle Mini‑Map, Toggle Sector Labels, Arrange Tiles.
  · Power – Sleep, Restart TOS, Log Out (with tactile confirmation).
· Settings Panel – Opens as a modal overlay with left sidebar categories, right content area, and integrated search. The bezel remains visible (collapsed) with the Settings icon highlighted.

## 2. Command Hub – Level 2

The Command Hub is the central control point for a sector. It consists of a top bezel, a main display area with dual‑sided chip layout and output background, and the Persistent Unified Prompt at the bottom.

### 2.1 Top Bezel

· Output Mode Toggle – Button (or icon) to switch between Standard and Centered Perspective output configurations (see §2.4). Positioned on the bezel, visible even when collapsed.
· Zoom Out – Returns to Global Overview.
· Left Region Toggle – Optional button to show/hide the left favourites/context chip region.
· Additional Controls – May include split buttons, sector name, etc., depending on context.

### 2.2 Persistent Unified Prompt

Fixed at the bottom of the Command Hub, spanning the full width. It consists of three distinct sections:

· Left Section – 3‑position mode selector (CMD | SEARCH | AI). The active mode is visually highlighted. Tapping or clicking a mode switches the hub’s behaviour.
· Center Section – Text input field. Supports typing, pasting, and real‑time autocomplete suggestions. In CMD mode, syntax highlighting may be applied. In SEARCH mode, it acts as a search query field. In AI mode, it accepts natural language.
· Right Section – Contains two controls:
  · Mic Button (microphone icon) – Toggles voice input mode. When activated, the system listens for speech and transcribes it into the input field. The button may change appearance (e.g., pulsing red) while recording. Voice input can be cancelled by tapping the mic button again or using the stop button.
  · Stop Button (⏹️) – Cancels the current operation: interrupts a running command, stops an ongoing search, halts AI response generation, or deactivates voice input.

Voice Interaction Notes:

· Voice input can also be triggered by a wake word (e.g., “Hey TOS”) or a dedicated hardware button, depending on platform capabilities.
· The mic button provides a manual, always‑available fallback.
· Transcribed text appears in the input field; the user can edit it before execution.
· In AI mode, voice is particularly useful for natural language queries; the AI response may be spoken back via TTS if enabled.

Layout Consistency:

· The mode selector visually aligns with the left chip region above.
· The mic and stop buttons align with the right chip region.
· The input field dynamically resizes as the left and right sections occupy fixed widths.

### 2.3 Main Display Area – Dual-Sided Chip Layout

The area between the top bezel and the prompt contains two overlapping layers:

· Background – Terminal output (see §2.4).
· Foreground – Left and right chip regions, rendered with semi‑transparent backgrounds.

Left Region – Favourites & Context

· Position – Left edge, expanding rightward.
· Content:
  · Favourites – User‑pinned commands (global or per‑sector). Each chip shows command name and a star icon.
  · Context Chips – Generated from directory awareness, Application Model hooks, Sector Type defaults, or active processes.
· Visibility – Can be toggled off (via bezel button or keyboard shortcut). When hidden, the right region expands to full width.
· Interaction – Tapping a left‑region chip populates the prompt with the associated command (or executes it directly if auto‑execute is enabled). Chips that open submenus display a secondary chip list in the same region.

Right Region – Prioritized Chips

· Position – Right edge, expanding leftward up to 3/4 of the hub’s width (or full width if left region hidden).
· Content – Dynamically ranked suggestions based on priority scoring (§5.1 of v1.2):
  · Eval‑help flags (parsed from --help output of the current command).
  · Command history (relevant to context).
  · File/path completions.
  · AI‑suggested actions.
  · System alerts (e.g., low disk space).
· Visual Priority Indicators – Each chip may display border chips, chevrons, or status dots to convey urgency.
· Interaction – Tapping a right‑region chip appends its content to the prompt at the cursor position (or replaces the current token). For flags that accept arguments, tapping inserts the flag and positions the cursor for the argument; a secondary chip list may appear for possible values.

### 2.4 Output Area – Two Configurations

The background terminal output can be viewed in two modes, toggled by the bezel button (see §2.1). The output area always scrolls vertically; new lines appear at the bottom.

#### 2.4.1 Standard Rectangular Configuration

· Full‑width rectangle spanning between left and right chip regions (or full hub width if chips hidden).
· Uniform text, vertical scrolling.
· Ideal for reviewing logs or continuous output.

#### 2.4.2 Centered Perspective Configuration

· Output lines recede toward a central vanishing point, creating a sense of depth.
· Bottom line (most recent) retains the full width of the Persistent Unified Prompt.
· Previous lines progressively narrow and move toward the center as they scroll upward.
· Older lines may become too narrow to read; hovering or clicking expands a tooltip with full content.
· Left and right chip regions expand outward, using the freed space to show more chips.
· Transition is animated with a smooth “tunnel” effect, accompanied by an earcon and optional haptic feedback.

### 2.5 Autocomplete – Bezel-Born Overlay

When the user types in the prompt (CMD mode), a temporary overlay extends downward from the right side of the top bezel.

· Appearance – Unfurls like a drawer, attached to the bezel. Maximum height ~3/4 of the hub.
· Content – Comprehensive, scrollable list of completions (flags, file paths, command names, etc.), presented as chips with descriptions.
· Dismissal – Tapping outside, pressing Escape, clicking a close chevron, or executing the command retracts the overlay.
· Relationship with Chip Regions – The overlay complements the persistent right‑region chips by providing a fuller set of options; it temporarily overlays the chip layout but can be dismissed to return to the persistent view.

### 2.6 Context-Aware Mode Switching

The Command Hub can automatically switch modes based on the command being typed.

· Filesystem Commands (e.g., ls, cd, cp, rm, find) – Triggers a switch to Directory Mode (either automatically or via suggestion chip). In Directory Mode, the file grid displays the current working directory, and file selections populate the prompt.
· Process Commands (e.g., kill, ps, top, renice) – Triggers a switch to Activity Mode. The tactical grid of running processes appears, with relevant processes highlighted.
· Configuration – User setting in Command Hub preferences:
  · Off – No automatic switching.
  · Suggest – A chip appears offering to switch; tapping it changes mode.
  · Auto – The mode switches immediately without confirmation.
· Command List Customisation – Users can extend or override the list of commands that trigger each mode.
· Visual Feedback – When a mode switch occurs, a subtle animation highlights the mode selector and the new mode’s icon. A brief earcon or haptic pulse may accompany the transition. Chip regions update to reflect the new mode’s context.

3. Application Focus – Level 3

Application Focus is the deepest interactive level in the standard hierarchy. When the user zooms into an application from the Command Hub (Level 2), the view transitions smoothly to a full‑screen (or tiled) surface displaying the application’s native window. This level is where the user interacts directly with graphical applications, while the Tactical Bezel remains the only system‑level overlay, guaranteeing navigational escape and providing contextual controls.

### 3.1 Application Surface

· The application runs in its own window, rendered as a native Wayland surface (or X11 forward‑compatible surface).
· The surface occupies the entire viewport (or a tile in a split configuration) with no window decorations other than the Tactical Bezel.
· For legacy X11 applications, TOS suppresses native decorations where possible and overlays the bezel; if suppression is not possible, the bezel may be positioned above the application’s own title bar.

### 3.2 Tactical Bezel – Application Context

The bezel at Level 3 follows the same design as at higher levels but is tailored for application interaction.

#### 3.2.1 Collapsed State

· Thin, semi‑transparent strip along the top edge of the application surface (position user‑configurable).
· Contains:
  · Zoom Out button – Returns to Level 2 (Command Hub).
  · Application Icon and Title – Provides immediate context.
  · Expand Handle – A down‑chevron that reveals the expanded bezel when dragged, clicked, or activated via keyboard (Ctrl+Space).

#### 3.2.2 Expanded State

Activated by any of the above methods. The expanded bezel extends downward, revealing a command strip with the following sections:

Section Controls
Navigation Zoom Out, Split View, Teleport, Close Application
Window Controls Minimize, Full‑screen Toggle, Always on Top (where applicable)
Application‑Specific Actions Provided by the Application Model (e.g., “New Tab” for browser, “Find” for editor)
System Shortcuts Open Command Hub, Toggle Mini‑Map, Settings
Collaboration Indicators Avatars of active participants, share button

· The expanded bezel may also display priority indicators (border chips, chevrons) reflecting the application’s current importance (e.g., a pending notification in a communication app).
· Tapping any control either executes an action or populates the Command Hub’s prompt (if the action involves a command).

### 3.3 Split Viewports from Level 3

Splitting is initiated from the expanded bezel:

· Split Button – After choosing orientation (horizontal or vertical), the user selects what to place in the new viewport:
  · New Command Hub – Creates a fresh Level 2 hub.
  · Parent Command Hub – Shows the hub that launched the current app.
  · Choose Hub… – Lists all hubs in the sector.
· From Activity Mode – In Level 2 Activity Mode, multi‑selecting application tiles and choosing “Open in Split View” creates tiled Level 3 viewports for the selected apps and zooms to Level 3.

Each split viewport operates independently: it can contain an application (Level 3) or a Command Hub (Level 2), with its own zoom state, mode, and content. Viewports can be resized by dragging dividers; closing a viewport causes the remaining ones to expand.

### 3.4 Application Models

Application Models (see §12 of v1.0 Core) customise the behaviour of specific applications at Level 3. They provide:

· Custom bezel actions – Additional buttons or menus relevant to the application.
· Zoom behaviour – Some applications (e.g., IDEs) may support internal deep zoom levels; the model can define how the TOS zoom interacts with the app’s own interface.
· Legacy decoration policy – How the bezel should integrate with applications that have their own window decorations (Suppress, Overlay, or Native).
· Thumbnail generation – For Activity Mode at Level 2.
· Searchable content – Applications can expose internal data to the unified search (e.g., browser tabs, document titles).

Models are installed locally and run sandboxed with user‑granted permissions.

### 3.5 Deep Inspection Access

From the expanded bezel, an Inspect button (or similar) allows the user to zoom into Level 4 (Detail View) for the current application. This reveals structured metadata such as CPU/memory usage, uptime, event history, and configuration. A further zoom (Level 5) provides raw memory inspection, but requires explicit privilege elevation and may be unavailable on some platforms (see §11.6 of v1.2 Extensions).

### 3.6 Auditory and Haptic Feedback at Level 3

· Zoom transition – A distinct earcon confirms entry into Application Focus.
· Bezel actions – Tapping bezel controls triggers appropriate haptic feedback (e.g., a light click for selection, a buzz for dangerous actions).
· Spatial audio (VR/AR) – Application sounds may be positioned in 3D space relative to the user; bezel interactions also have spatialised feedback.

### 3.7 Platform Adaptations

· Linux Wayland – Full native performance; the bezel is rendered by the TOS compositor as an overlay.
· Android XR – The application surface becomes a virtual screen in 3D space; the bezel appears as a floating panel attached to the virtual screen, operable via gaze, pinch, or hand tracking.
· Android Phone – The application fills the screen; the bezel is a swipe-down drawer from the top, with touch-optimised controls.

### 3.8 Accessibility

· The bezel is fully navigable via keyboard (Tab, arrow keys, Enter) and screen reader (announcing button labels and states).
· High‑contrast variants and adjustable font scaling ensure visibility.
· Haptic feedback provides confirmation for users with visual impairments.

## 4. Deep Inspection – Levels 4 & 5

Deep Inspection extends the standard three-level hierarchy to provide detailed introspection of any surface (sector, application, or process). These levels are accessible from any point in the hierarchy where deeper analysis is required, typically via an Inspect button in the expanded bezel or a contextual command.

### 4.1 Level 4 – Detail View

The Detail View presents structured metadata about the inspected surface in a clear, organised panel.

· Access – From Level 3 (Application Focus), the expanded bezel includes an Inspect button. From Level 2 (Command Hub), right‑clicking/long‑pressing a tile in Activity Mode or a file in Directory Mode may offer "Inspect" as an option. From Level 1, sector tiles can be inspected similarly.
· Appearance – A modal overlay that slides up from the bottom or expands from the bezel, occupying most of the viewport while preserving context of the underlying surface. The overlay follows LCARS design language: clean panels, colour‑coded sections, and interactive chips.

Content Categories:

Category Information Displayed
System Resources CPU usage (current/average), memory consumption, uptime, network I/O, disk I/O
Event History Scrollable timeline of lifecycle events (creation, focus, moves, closes), commands executed, inspections accessed (see §14 – TOS Log)
Configuration Environment variables, command-line arguments, application settings (if exposed via Application Model), sector preferences
Metadata Surface UUID, process ID (if applicable), parent surface, session ownership
Security Permissions granted, sandbox status, audit log excerpts (critical events only)
Collaboration Active guests, recent guest actions (if any)

· Interactive Elements – Certain data points may be interactive:
  · Clicking a process ID switches to Activity Mode with that process highlighted.
  · Tapping a log entry may expand it or offer to search for similar events.
  · Configuration values that are editable appear with an edit icon; changing them may require elevation.
· Export – A button in the panel allows exporting the current detail view as JSON or plain text for further analysis.

4.2 Level 5 – Buffer View

The Buffer View provides raw memory inspection of the target surface's process space. Due to its sensitivity, this level is privileged and subject to strict controls.

· Access – From Level 4, a button labelled "Memory View" or "Buffer" appears, but is disabled by default. Enabling it requires explicit privilege elevation (see §4.3).
· Appearance – A hex dump viewer fills the main area, with columns for offset, hexadecimal representation, and ASCII interpretation. Controls at the top allow:
  · Seek – Jump to a specific memory address.
  · Search – Find a byte sequence or ASCII string.
  · Export – Save the buffer (or selected range) to a file.
  · Refresh – Update the view (memory may change rapidly).
· Limitations:
  · On Android, Level 5 is generally unavailable due to platform restrictions; attempting to access it shows a message explaining the limitation.
  · Applications may opt out via their Application Model manifest; attempting to inspect such apps at Level 5 shows a permission denied notice.
  · The view is read‑only; no memory modification is permitted through TOS.

4.3 Security & Privilege Elevation

Access to Level 5 (and certain sensitive data in Level 4) requires explicit user consent and may be gated by platform‑specific authentication.

· Default State – Level 5 access is disabled globally. Level 4 is fully accessible.
· Enabling Deep Inspection:
  · Linux Wayland – User must run a privileged command (sudo tos enable-deep-inspection) or authenticate via Polkit dialog when first attempting to access Level 5. Once enabled globally, individual applications may still be inspected only if they haven't opted out.
  · Android XR/Phone – Deep inspection is typically not available; if the platform allows, a biometric prompt (fingerprint/face) may grant temporary access to Level 4 extended metadata, but Level 5 remains inaccessible.
· Visual Indicator – When deep inspection is enabled (globally or for a session), a 🔓 indicator appears in the Tactical Bezel (all levels). Clicking this indicator immediately disables deep inspection and closes any open Level 5 views.
· Auditing – All enable/disable events and every access to Level 5 are recorded in the system audit log (non‑disableable). Level 4 access is logged in the TOS Log (see §14) but may be disabled by user privacy settings.

4.4 Relationship with TOS Log

· Level 4 includes an Event History section that pulls from the TOS Log (see §14), displaying a filtered timeline relevant to the inspected surface.
· From Level 4, the user can click "View Full Log" to open the global TOS Log sector at that surface's filtered view.
· Log entries related to deep inspection (e.g., "Level 5 accessed for process 1234") appear in both the surface's log and the global audit trail.

4.5 Platform Notes

Platform Level 4 Availability Level 5 Availability
Linux Wayland Full Available with privilege elevation (sudo/Polkit)
Android XR Partial (no raw memory) Not available
Android Phone Partial (limited metadata) Not available

4.6 Use Cases

· Debugging – A developer inspecting a misbehaving application can view its resource usage and recent log entries at Level 4, then drop to Level 5 to examine memory for corruption or unexpected data.
· Security Analysis – An advanced user investigating a suspicious process can review its configuration and event history at Level 4, and if necessary, examine its memory space for anomalies.
· System Optimisation – Identifying memory‑leaking applications by comparing live memory dumps over time.

4.7 Accessibility

· The hex viewer in Level 5 supports screen reader output (announcing offset, byte values, and ASCII equivalents).
· Keyboard navigation: arrow keys move through the hex dump; Tab focuses controls.
· High‑contrast and large‑text modes apply to all inspection panels.

## 5. Priority Indicators

Priority indicators provide a consistent, non-intrusive visual language to convey the relative importance or urgency of elements across all levels of the TOS interface.

### 5.1 Indicator Types

Four primary indicator types are used, often in combination:

Type Description Appearance
Border Chips Small, pill‑shaped coloured accents placed along the border of a tile or viewport. The number of chips reflects the priority score (e.g., one chip for low, four for critical). Positioned at corners or along edges; colour varies by factor (e.g., blue for recency, red for alerts).
Chevrons LCARS‑style arrow shapes that can be static or animated. Pulsing chevrons indicate a pending notification, active collaboration, or critical status. Usually placed near the top‑right corner; direction may indicate type (up for alerts, right for activity).
Glow / Luminance A subtle inner or outer glow around the element. Intensity varies with priority; can be combined with colour to convey mood. Applied to the entire tile or its border; may pulse for high priority.
Status Dots Small circles in a corner of the tile. Colour‑coded (blue=normal, yellow=caution, red=critical). Multiple dots can appear to indicate multiple concurrent factors (e.g., both a notification and high activity). Typically bottom‑right or top‑left; dots may have tooltips on hover.

### 5.2 Priority Scoring

Each element’s priority is determined by a weighted score computed from multiple factors. The score maps to a specific indicator configuration (e.g., number of chips, chevron state, glow intensity).

Weighted Factors (user‑configurable):

Factor Default Weight Description
Recency of focus 40% How recently the element was interacted with.
Frequency of use 20% How often the element is accessed.
Activity level 15% CPU, memory, I/O activity (for processes/applications).
Notification priority 10% Urgency of pending notifications.
User pinning Override Manually pinned elements always show at least medium priority.
Collaboration focus 10% Whether the element is currently being viewed or edited by collaborators.
Sector‑specific rules Variable Custom rules defined by Sector Types (e.g., a monitoring sector may boost priority for failing services).
AI suggestion 5% Elements suggested by the AI assistant may receive a temporary boost.

Score to Indicator Mapping (example):

Score Range Border Chips Chevron Glow Status Dots
0–20 None None None None (or blue dot for baseline)
21–40 1 chip Static Subtle Blue dot
41–60 2 chips Static Low glow Yellow dot
61–80 3 chips Pulsing Medium glow Yellow dot (pulsing)
81–100 4 chips Pulsing + direction High glow, possibly red Red dot(s)

The exact mapping is fully customisable per user or per sector.

### 5.3 Behaviour by Depth

Priority indicators adapt to the current zoom level, showing aggregated or detailed information as appropriate.

Level Indicator Behaviour
Level 1 – Global Overview Sector tiles display overall sector priority, combining the priorities of their contained hubs and applications. Border chips reflect the sector’s aggregate score; collaboration presence may add chevrons.
Level 2 – Command Hub Application tiles in Activity Mode show individual priority. The left (favourites) and right (prioritized) chip regions themselves use indicators to highlight the most important suggestions. The hub’s own bezel may display a subtle glow if the sector as a whole has elevated priority.
Level 3 – Application Focus The Tactical Bezel may show a priority chevron or glow if the application has a pending notification or critical status. Within split viewports, indicators appear along the shared borders of each viewport, allowing quick comparison.
Level 4 – Detail View The inspection panel includes a mini‑map of sibling surfaces, each with priority indicators. The inspected surface’s own priority is shown prominently, and a timeline of priority changes (from the TOS Log) may be displayed.
Level 5 – Buffer View Priority indicators are minimal, as the focus is on raw data. However, a chevron may indicate if the process is in a critical state (e.g., high memory pressure).

### 5.4 Configuration

Users have extensive control over priority indicators through a dedicated settings panel (accessible from the global Settings or per‑sector).

· Master Toggle – Enable/disable priority indicators entirely.
· Indicator Type Selection – Choose which indicator types to use (e.g., some users may prefer only status dots).
· Colour Customisation – Assign colours per priority level or per factor.
· Sensitivity – Adjust the weightings or create custom scoring rules.
· Per‑Factor Visibility – Decide which factors contribute to the score and how they are displayed.
· Hover Tooltips – When hovering over an indicator, a tooltip can show the contributing factors and their scores.
· Accessibility – Options to enlarge indicators, replace colours with patterns, or route priority information to audio/haptic channels.

5.5 Integration with Other Systems

· TOS Log – Every change in priority score is logged, allowing users to review why an element became important at a certain time.
· Collaboration – When a collaborator focuses on an element, its priority may temporarily increase, indicated by a special chevron or a collaborator’s avatar merging with the indicator.
· AI Assistant – The AI can suggest priority adjustments based on learned patterns (e.g., “I noticed you often check this log at this time – would you like to pin it?”).
· Auditory Interface – Priority changes can be accompanied by earcons; for example, a rising tone when an element becomes critical.

5.6 Examples

· A sector tile with three border chips and a pulsing chevron indicates high aggregate activity and a pending notification.
· In Activity Mode, an application tile with a red status dot and a glow signals a process consuming excessive resources.
· A file in Directory Mode with a single blue chip is a recently accessed document.
· A search result with two chips and a yellow dot is a high‑relevance match based on frequency and recency.

## 6. Tactical Mini-Map

The Tactical Mini‑Map is an ephemeral overlay that provides spatial awareness of the entire sector hierarchy without blocking interaction. It appears as a small, semi‑transparent panel (default bottom‑right corner) and adapts its content based on the current zoom level. Users can quickly orient themselves, jump to different areas, or monitor resource usage – all without leaving their current context.

6.1 Overview

· Purpose – Maintain situational awareness across sectors, hubs, and applications. The mini‑map shows the user’s current position within the tree, nearby elements, and optionally live resource metrics.
· Persistence – The mini‑map is always available but remains passive (input passes through to underlying UI) until explicitly activated. This ensures it never interferes with interaction.
· Activation Methods (configurable):
  · Hover (dwell time) over the mini‑map area.
  · Keyboard shortcut (Ctrl+M or Super+M).
  · Modifier + click (Alt+click) on any empty area.
  · Double‑tap (touch) on the edge of the screen.
  · Game controller button (e.g., View/Back button).
  · Voice command (“show mini‑map” or “activate mini‑map”).

6.2 Visual Design

· Shape – A rounded rectangle or LCARS‑style curved panel, sized approximately 200×150 pixels (scales with UI). The panel has a subtle glow and a semi‑transparent background (blur effect) to maintain readability over content.
· Elements – The mini‑map displays a simplified topological view:
  · Current sector – Highlighted (e.g., with a bright border or colour fill).
  · Other sectors – Dimmed, shown as smaller outlines.
  · Viewports – Within the current sector, each split viewport is represented as a rectangle; the active viewport is highlighted.
  · Depth indicator – A small icon or text (e.g., “L2”) shows the current zoom level.
  · Collaboration presence – Tiny avatars or coloured dots may appear on sectors/viewports where collaborators are active.
· Active State – When activated, the mini‑map becomes opaque, its border thickens, and it captures input. A small close button (×) appears in its corner for dismissal.

6.3 Activation and Interaction

· Passive State – The mini‑map displays information but does not capture mouse/touch events. Users can click through it to interact with underlying elements.
· Active State – Once activated, the mini‑map captures all input:
  · Click/tap on a sector tile jumps to that sector (zoom out if necessary, then zoom in).
  · Click/tap on a viewport within the current sector focuses that viewport.
  · Drag – The mini‑map can be repositioned by dragging its title area (if any) or edges.
  · Scroll wheel/pinch – May zoom the mini‑map’s view (if supported) to show more or less detail.
  · Close – Click the close button, press Escape, or repeat the activation gesture to return to passive state.
· Deactivation – The mini‑map automatically reverts to passive state after a configurable timeout of inactivity (default 5 seconds) or when the user explicitly closes it.

6.4 Content by Depth

The mini‑map’s content adapts to the user’s current zoom level, providing the most relevant spatial information.

Level Content Shown
Level 1 – Global Overview All sectors as miniature tiles. The current sector (if any, since Level 1 is the top) may be highlighted, but typically the user is already viewing all sectors.
Level 2 – Command Hub The current sector is shown in the centre, with other sectors dimmed around it. Within the sector, each Command Hub (if multiple exist due to splits) is represented as a small tile, with the active hub highlighted. The depth indicator reads “L2”.
Level 3 – Application Focus The current sector is shown, with the focused application’s viewport highlighted. Other viewports (splits) are shown as smaller rectangles. The path from the sector down to the current app may be visually indicated (e.g., with lines). Depth: “L3”.
Level 4 – Detail View The mini‑map may show the current surface and its siblings, helping the user understand where they are in the inspection hierarchy. Depth: “L4”.
Level 5 – Buffer View Simplified view – may only show the current surface and an indicator that deep inspection is active. Depth: “L5”.

6.5 Monitoring Layer (Resource Usage)

Introduced in v1.2 (§18.5), an optional overlay within the mini‑map displays live resource usage of processes relevant to the current depth.

· Toggle – An icon on the mini‑map (or a separate keyboard shortcut) toggles the monitoring layer on/off.
· Content by Depth:
  · Level 1 – Aggregated CPU and memory usage per sector (e.g., small bar graphs or percentages next to each sector tile).
  · Level 2 – All applications in the current sector shown with CPU%, memory%, and a sparkline of recent activity.
  · Level 3 – Detailed stats for the focused application, plus compact usage indicators for other viewports (e.g., a small bar).
  · Level 4/5 – Resource usage of the inspected surface, plus optionally its children.
· Update Rate – Throttled to 1–2 Hz to minimise performance impact.
· Visual Style – Small, unobtrusive bars or numeric readouts, colour‑coded (green = normal, yellow = high, red = critical). Hovering over a metric shows a tooltip with exact values.

6.6 Configuration

Users can customise the mini‑map through the Settings panel:

· Position – Choose corner (top‑left, top‑right, bottom‑left, bottom‑right) or free‑floating with remembered position.
· Size – Adjust base size (small, medium, large) or enable auto‑scaling based on screen resolution.
· Opacity – Separate opacity for passive and active states.
· Activation Behaviour – Choose which methods are enabled (hover, keyboard, etc.) and set dwell time for hover activation.
· Content – Toggle display of other sectors, viewport details, depth indicator, collaboration avatars.
· Monitoring Layer – Enable/disable, choose metrics to display (CPU, memory, network, disk), and set colour thresholds.
· Accessibility – Options to enlarge the mini‑map, use high‑contrast colours, or route its information to audio (e.g., spoken summary on hover).

6.7 Platform Adaptations

· Linux Wayland – Rendered as a compositor overlay; input pass‑through handled via Wayland protocols.
· Android XR – The mini‑map appears as a floating panel in 3D space, attached to the user’s field of view (HUD) or anchored to a virtual wrist. Activation via gaze dwell or hand gesture.
· Android Phone – Positioned as a small overlay; touch interaction follows standard mobile conventions. May be temporarily hidden during landscape full‑screen apps.

6.8 Accessibility

· Screen Reader – The mini‑map’s content can be announced on activation or hover; users can navigate its elements with keyboard or switch scanning.
· High Contrast – The mini‑map respects system‑wide high‑contrast themes; its colours can be overridden for better visibility.
· Auditory Cues – When the monitoring layer detects a critical resource threshold, an earcon may play, and the mini‑map can briefly highlight the affected element.
· Simplified Mode – A “simple mini‑map” option reduces complexity, showing only the current sector and depth.

7. Collaboration UI

Collaboration in TOS transforms a sector into a shared workspace where multiple users can interact in real time. The collaboration interface is designed to be minimally intrusive while providing clear awareness of other participants’ presence, actions, and intent. All collaboration features are built on a host‑owned model: the sector resides on one host, and guests connect via secure tokens or invitations.

7.1 Visual Presence Indicators

Collaborators are represented consistently across all levels through a combination of avatars, coloured borders, and cursors.

· Avatars – Small circular or square icons displaying the user’s profile picture or initials. Avatars appear in:
  · Global Overview (Level 1) – On sector tiles, indicating active guests in that sector. Multiple avatars may stack or show a count badge.
  · Command Hub (Level 2) – Near the mode selector or in the top bezel, showing all participants in the current sector. Clicking an avatar reveals a menu with options (follow, message, etc.).
  · Application Focus (Level 3) – In the expanded bezel, avatars of guests currently viewing or interacting with that application.
  · Tactical Mini‑Map – Tiny avatars or coloured dots on sector/viewport representations.
· Coloured Borders and Cursors – Each participant is assigned a distinct colour (user‑configurable or auto‑assigned). This colour is used for:
  · Cursor outlines – When a guest’s cursor is visible, it appears with their colour.
  · Viewport borders – If a guest is focused on a particular split viewport, that viewport’s border may glow with their colour.
  · Selection highlights – Text or elements selected by a guest are highlighted in their colour (if view‑synchronised).
  · Priority indicators – Collaboration focus may temporarily boost an element’s priority, indicated by a special chevron or a collaborator’s avatar merging with the indicator.
· Follow Mode Indicator – When a guest is following another user, a small “following” icon (e.g., an eye or footsteps) appears next to the follower’s avatar, and their viewport may show a semi‑transparent outline of the target’s view.

7.2 Collaboration Controls in the Bezel

The expanded bezel at any level includes a Collaboration section with the following controls (subject to role permissions):

· Share Sector – Generates an invite link or token (with optional expiration and role limit). Available to hosts and co‑owners.
· Active Participants – List of current guests with their roles (Viewer, Commenter, Operator, Co‑owner). Clicking a name opens a menu to change role, send a message, or follow.
· Raise Hand – A button that sends a gentle alert to all participants, indicating a request for attention (e.g., to ask a question). The requester’s avatar pulses yellow.
· Request Control / Release Control – For guests in following mode, a button to request temporary control or release it.
· Follow / Unfollow – Toggle to synchronise viewport with another participant.
· Leave Sector – Exit the shared session.

7.3 Collaboration Alerts

Key collaboration events trigger non‑intrusive alerts to maintain awareness without disrupting workflow. Alerts are visual, auditory, and haptic (configurable).

Event Visual Indicator Auditory Haptic
User joins sector Avatar fades in; brief highlight on sector tile Soft chime Short pulse
User leaves sector Avatar fades out; brief dimming Soft click Short pulse
Guest role changes Role badge updates; brief notification chip Gentle tone —
Guest raises hand Pulsing yellow border around avatar; “Hand raised” chip Two‑tone chime Double pulse
Guest requests follow “X wants to follow” chip; accept/decline buttons Soft query tone —
Guest shares cursor Cursor becomes visible with guest’s colour — —
Host ends session Countdown notification; session closes Alert tone Long vibration

All collaboration alerts are recorded in the TOS Log (host side) for later review.

7.4 Guest View and Permissions

Guests experience the same TOS interface as the host, but with certain restrictions based on their role:

· Viewer – Can see all content but cannot issue commands. The prompt is visible but disabled. All controls in the bezel are read‑only.
· Commenter – Can type in the prompt, but commands are executed in a restricted shell (or not at all, depending on host configuration). Comments may appear in a separate chat overlay (optional).
· Operator – Full control: can execute any command, create splits, launch applications, and change viewports.
· Co‑owner – Same as Operator, plus ability to invite others and change roles.

Guests always see the host’s sector tree; they cannot access other sectors on the host machine unless explicitly shared. Their own local sectors remain private.

7.5 Following Mode

Following mode synchronises a guest’s view with another participant’s (usually the host or an operator). When following:

· The guest’s viewport mirrors the target’s zoom level, splits, and focused elements.
· The guest can still move their own cursor independently, but any interaction (clicking, typing) is either disabled or subject to role permissions.
· A “break follow” button appears in the bezel; clicking it restores independent control.
· The target may receive a notification when someone starts following them.

7.6 Chat and Communication

While TOS emphasises command‑first interaction, a lightweight chat overlay is available for collaboration.

· Activation – A chat bubble icon in the expanded bezel, or keyboard shortcut (Ctrl+Shift+C).
· Appearance – Slides in from the right edge, overlaying the chip regions but not the prompt. Shows a scrollable list of messages with timestamps and sender avatars.
· Input – A text field at the bottom of the chat panel; messages are sent with Enter.
· Integration – Commands typed in chat can be executed by the host if prefixed with /run (subject to permissions). Chat messages are also logged in the TOS Log.

7.7 AI Assistant in Collaboration

The AI assistant (see §2.2, AI Mode) gains collaboration‑aware capabilities when a sector is shared:

· Summarise Activity – “What has everyone been working on for the last hour?” – the AI scans the TOS Log and provides a summary.
· Translate Commands – If guests speak different languages, the AI can translate commands and chat messages in real time (with appropriate backend support).
· Suggest Collaboration Actions – “Should I share this log with the team?” or “X has been idle for a while – would you like to reassign them?”
· Explain Guest Intent – “What is Y trying to do?” – the AI can interpret a guest’s recent actions.
· Mediate Role Changes – “Promote Z to operator?” – the AI can suggest role changes based on activity.

Guests are notified if their actions may be processed by the AI, and they can opt out if privacy concerns arise.

7.8 Privacy and Auditing

· Guest Action Logging – All guest actions (commands executed, files accessed, etc.) are recorded in the host’s TOS Log (see §8). Guests do not have access to this log unless granted explicit permission.
· Privacy Notice – When joining a shared sector, guests see a brief notice explaining what data may be logged and whether AI processing is enabled. They must acknowledge before continuing.
· Audit Trail – Critical events (role changes, invite usage, security‑relevant commands) are written to a non‑disableable audit log on the host.

7.9 Platform Adaptations

· Linux Wayland – Full collaboration features, including cursor sharing and viewport synchronisation, implemented via custom Wayland protocols.
· Android XR – Avatars appear as 3D models floating near the user; collaboration alerts are spatialised. Following mode may include gaze and hand tracking.
· Android Phone – Simplified avatars and chat overlay; following mode may show a small inset view of the target’s screen.

7.10 Accessibility

· Screen readers announce when users join/leave, when hands are raised, and when following mode is activated.
· Haptic feedback provides tactile confirmation of collaboration events.
· High‑contrast colours for borders and avatars ensure visibility for users with colour vision deficiencies.

8. Input Abstraction Layer

TOS is fundamentally input‑agnostic, designed to support any interaction modality equally. The Input Abstraction Layer normalises all physical input devices into a common set of semantic events, which are then mapped to TOS actions through a flexible, user‑configurable mapping layer. This ensures that whether the user is typing, touching, speaking, or gesturing, the system responds consistently and predictably.

8.1 Semantic Event Categories

All input devices generate events that fall into one of several high‑level categories. These semantic events are what the core TOS logic understands, independent of the physical source.

Category Events Description
Navigation zoom_in, zoom_out, next_element, previous_element, next_viewport, previous_viewport, focus_left, focus_right, focus_up, focus_down, home (Level 1), command_hub (jump to Level 2) Moving through the spatial hierarchy and between elements.
Selection select, secondary_select, multi_select_toggle, select_all, clear_selection, drag_start, drag_end, drop Choosing, activating, or manipulating elements.
Mode Control cycle_mode, set_mode_command, set_mode_directory, set_mode_activity, set_mode_search, set_mode_ai, toggle_hidden_files Switching between Command Hub modes and toggling view options.
Bezel Control toggle_bezel_expanded, split_view, close_viewport, inspect, teleport, show_bezel_actions Interacting with the Tactical Bezel at any level.
System Commands open_hub, open_global_overview, tactical_reset_sector, tactical_reset_system, open_settings, toggle_minimap, show_help Global system actions.
Text Input text_input (with content), command_history_prev, command_history_next, autocomplete_request, autocomplete_select Entering and editing text in the Persistent Unified Prompt.
Voice voice_command_start, voice_command_end, voice_transcription (with confidence) Voice interaction; transcription may populate the prompt.
AI Interaction ai_submit, ai_stop, ai_mode_toggle, ai_suggestion_accept AI‑specific actions.
Collaboration show_cursor, follow_user, unfollow, raise_hand, share_sector, leave_sector Multi‑user actions.
Stop Operation stop_operation Universal cancel (maps to stop button).

8.2 Device Support and Mapping

Physical devices are supported through pluggable input modules that translate raw input into semantic events. The user can remap any physical action to any semantic event via a graphical configuration interface.

Device Class Supported Inputs Default Semantic Mapping (examples)
Keyboard Key presses, key combinations, chorded input Arrow keys → navigation; Enter → select; Esc → zoom_out (at Level 3) or cancel; Ctrl+Space → toggle_bezel_expanded
Mouse / Trackpad Click, right‑click, double‑click, scroll, drag, hover Left click → select; right click → secondary_select; scroll wheel → zoom_in/zoom_out; hover → focus indication
Touch Tap, long press, pinch, spread, swipe, multi‑finger gestures Single tap → select; double tap → zoom_in; two‑finger pinch → zoom_out; swipe from edge → bezel expansion
Game Controller Analog sticks, D‑pad, triggers, bumpers, face buttons, gyro Right trigger → zoom_in; left trigger → zoom_out; D‑pad → navigation; A button → select; B button → back/cancel; Start → open_hub
VR/AR Controllers Trigger, grip, thumbstick, controller pose, touchpad Trigger pull → select; grip squeeze → grab/drag; thumbstick up/down → zoom; thumbstick click → toggle_minimap
Hand Tracking Pinch, grab, point, two‑hand spread, swipe gestures Pinch → select; two‑hand spread/pinch → zoom_in/zoom_out; point dwell → focus; grab + move → drag
Gaze / Eye Tracking Gaze point, dwell, blink patterns, smooth pursuit Gaze + dwell (configurable time) → select; gaze at bezel edge → toggle_bezel_expanded; blink pattern → stop_operation
Voice Wake word, natural language commands, dictation Wake word + "zoom in" → zoom_in; dictation → text_input with transcription
Accessibility Switches Single switch, multiple switches, sip‑and‑puff, eye blink Switch 1 → next_element; switch 2 → select; long press switch → secondary_select

8.3 Concurrent Input

TOS supports simultaneous use of multiple input devices, intelligently merging streams to provide a seamless experience.

· Last Active Device – The cursor appearance may change based on the last used device (e.g., mouse cursor appears after mouse movement, then fades after keyboard use).
· Conflict Resolution – When multiple devices generate conflicting events (e.g., simultaneous zoom_in from keyboard and controller), the system processes both but may prioritise one based on user configuration (e.g., keyboard over touch).
· Device‑Specific Feedback – Haptic and auditory feedback can be routed to the active device (e.g., controller vibrates when used for selection).
· Accessibility Priority – Users can assign higher priority to specific devices (e.g., a switch device always takes precedence).

8.4 Input Configuration

Users can customise input mappings through a dedicated panel in Settings, accessible from any level.

· Per‑Device Mapping – Select any connected device and remap its physical inputs to semantic events. Multiple mappings can be saved as profiles.
· Gesture Recording – For touch and controller gestures, users can record custom gestures and assign them to actions.
· Voice Command Training – Users can teach the system custom voice commands or import command sets.
· Sensitivity and Dead Zones – Adjustable for analog inputs (controller triggers, thumbsticks, touch pressure).
· Profiles – Save and load input configurations per user, per sector, or per application.

8.5 Accessibility Integration

The Input Abstraction Layer is the foundation for TOS’s accessibility features.

· Switch Scanning – The system can automatically cycle through elements; a switch press triggers select. Scanning speed and patterns are configurable.
· Sticky Keys – Modifier keys (Ctrl, Alt, Super) can be latched for users who cannot hold multiple keys simultaneously.
· Slow Keys – A delay before key presses are registered, accommodating users with motor difficulties.
· Dwell Clicking – For gaze or head tracking, dwelling on an element for a configurable time triggers select.
· Voice Commands – All semantic events can be triggered by voice, with custom phrases.
· Haptic Feedback as Input – On supported devices, haptic patterns can be used as input triggers (e.g., a specific vibration pattern to confirm a dangerous action).

8.6 Platform‑Specific Input Sources

Each platform implementation provides appropriate input modules:

Platform Input Sources
Linux Wayland evdev/libinput for keyboards, mice, touchpads, touchscreens; SDL2 for game controllers; OpenXR for VR/AR controllers; speech recognition via pocketsphinx or cloud APIs; eye tracking via Tobii or Pupil Labs drivers
Android XR OpenXR action system (gaze, hand tracking, controllers); Android touch events for phone‑mode fallback; Google Speech Recognition for voice; platform accessibility services
Android Phone Android touch events; hardware keys; Bluetooth controllers (via Android gamepad API); Google Speech Recognition; Accessibility Service API for switch devices

8.7 Semantic Event Flow

```
Physical Input → Device Driver → Raw Event → Input Module → Semantic Event → Action Mapper → TOS Core
```

· Raw Event – Device‑specific data (e.g., key code, touch coordinates, controller axis position).
· Input Module – Normalises raw events into a common format; applies dead zones, sensitivity, and gesture recognition.
· Semantic Event – Platform‑independent representation (e.g., zoom_in, select).
· Action Mapper – Applies user mappings (e.g., remap zoom_in to select if desired).
· TOS Core – Consumes the semantic event and triggers the appropriate response (change depth, select element, execute command).

8.8 Example Workflows

· Keyboard User – Presses Ctrl+Alt+T (configured as open_hub), types ls -la, presses Enter (text_input with submission). The output scrolls in the terminal.
· Touch User – Pinches to zoom out from an application (zoom_out), taps a sector tile (select), then taps a file in Directory Mode (select), which appends its path to the prompt.
· Voice User – Says “Hey TOS, search for budget files” (wake word + set_mode_search + text_input). The system switches to SEARCH mode, populates the prompt with “budget files”, and displays results.
· VR User – Gazes at a sector tile for 500ms (gaze + dwell = select), then pinches with both hands to zoom into the Command Hub (zoom_in). Uses hand tracking to tap chips in the right region.
· Switch User – Single switch: each press cycles focus to the next element (next_element); a long press triggers select. Scanning speed is set to 1.5 seconds.


9. TOS Log

The TOS Log is a system‑wide, per‑surface event history that provides a complete timeline of user and system actions. It is designed for auditability, debugging, and quick recall of past activities. The log is integrated into the UI at multiple levels, allowing users to review events without leaving their current context.

9.1 Overview

· Purpose – Record all significant events within TOS, including commands executed, lifecycle changes, inspections, collaboration events, and system alerts.
· Storage – Logs are stored locally in ~/.local/share/tos/logs/ (Linux) or app‑private storage (Android) in a structured format (JSON Lines or SQLite). Critical security events are stored in a separate, non‑disableable audit log.
· Privacy – Users have granular control over what is logged, with options to opt out per surface, set retention policies, and exclude sensitive patterns.

9.2 Recorded Events

The log captures a wide range of event types, each with relevant metadata:

Event Type Examples Metadata
Lifecycle Surface creation, focus change, move, resize, close Timestamp, surface ID, surface type, user
Commands Command executed in Command Hub, exit status, duration Command string, working directory, exit code, duration, user, sector
Inspections Level 4 (Detail) or Level 5 (Buffer) views accessed Surface inspected, level, timestamp
Telemetry Periodic resource snapshots (CPU, memory, I/O) – if enabled Metrics values, surface/process ID
Collaboration User join/leave, role changes, guest actions Guest identity, action type, outcome
System Events Notifications, alerts, security events, updates Event type, severity, details
Priority Changes Changes in priority score and indicator configuration Surface, old score, new score, contributing factors
AI Interactions Queries submitted, responses generated (if enabled) Query, response summary, backend used

9.3 Access Methods

Users can access logs through three primary interfaces:

9.3.1 Per‑Surface Log (Level 4 Detail View)

· When inspecting any surface (sector, application, process) at Level 4, a Event History section displays a scrollable timeline of events relevant to that surface.
· Events are shown in reverse chronological order, with filters for event type and time range.
· Clicking an event expands it to show full metadata; double‑clicking may jump to the context (e.g., zoom to the surface at the time of the event).

9.3.2 Global TOS Log Sector

· A dedicated sector, accessible from Level 1, aggregates logs from all surfaces.
· The sector behaves like a special Command Hub with a single mode: Log Viewer.
· Layout:
  · Left region: Filters (by surface, event type, user, date range, etc.) as interactive chips.
  · Right region: Prioritized log entries (ranked by recency, severity, or custom priority).
  · Main area: Scrollable list of log entries, each displayed as a card with icon, summary, and timestamp.
  · Prompt: Can accept log‑specific commands (e.g., log --since 1h --level error).
· Users can export the current filtered view as JSON, CSV, or plain text.

9.3.3 Prompt Queries

· In any Command Hub, users can type log‑query commands directly:
  · log – Shows recent events in the current sector (output appears in terminal area).
  · log --surface browser --since 10min – Filters to a specific surface.
  · log --user guest --level error – Shows errors from a collaborator.
  · log --help – Displays query syntax.
· Results are displayed in the output area, with clickable links to jump to the relevant surface or time.

9.3.4 OpenSearch Compatibility

· TOS provides an OpenSearch description document, allowing the browser address bar to query logs (e.g., tos log failed command).
· With user consent, logs can be forwarded to an external OpenSearch cluster for advanced analysis and visualisation.

9.4 Log Viewer UI Components

· Entry Cards – Each log entry is displayed as a card with:
  · Icon representing event type (command, lifecycle, collaboration, etc.).
  · Summary line (e.g., “Executed rm -rf temp in sector Dev”).
  · Timestamp (relative or absolute, user‑configurable).
  · Severity indicator (colour‑coded dot: info, success, warning, error, critical).
  · Expand/collapse arrow for details.
· Details Panel – When expanded, shows full metadata:
  · Event ID (UUID)
  · Surface path (e.g., “Dev Sector > Command Hub A”)
  · User (local or guest)
  · Full command line (if applicable)
  · Exit status and duration
  · Tags and custom fields
· Filters – Accessible via left chip region or a dedicated filter bar. Filters include:
  · Time range (last hour, today, custom)
  · Event type (checkboxes or chips)
  · Surface (searchable dropdown)
  · User (local/guest)
  · Severity
  · Text search (within event summaries or metadata)
· Export Button – Saves the current filtered view.

9.5 Privacy and User Control

· Master Toggle – Global setting to enable/disable logging entirely (except critical security events, which are always logged).
· Per‑Surface Opt‑Out – Users can disable logging for specific surfaces (e.g., a private terminal session).
· Retention Policies – Automatic deletion of logs older than a user‑defined period (e.g., 30 days).
· Exclude Patterns – Users can specify regex patterns to redact sensitive information (e.g., passwords) from logs before storage.
· Audit Log – Critical events (security, privilege changes) are stored in a separate, append‑only log that cannot be disabled or cleared by the user (requires system administrator access).

9.6 Integration with Other Systems

· Priority Indicators – Logged priority changes help explain why an element became important; users can click a priority indicator to view the log entry for that change.
· Collaboration – Guest actions are recorded in the host’s log, tagged with guest identity. Guests cannot access the host’s log unless explicitly granted permission.
· AI Assistant – The AI can query logs to answer questions like “What commands did I run yesterday?” or “Summarise recent errors in the web server.”
· Marketplace – Log analysis modules may be available to provide advanced visualisations or anomaly detection.

9.7 Platform Adaptations

Platform Log Storage Access
Linux Wayland ~/.local/share/tos/logs/ Full read/write access; audit log in /var/log/tos/ (requires root)
Android XR App‑private storage Logs accessible via TOS Log sector; export via Storage Access Framework
Android Phone App‑private storage Same as XR; limited by platform sandbox

9.8 Example Use Cases

· Debugging – A developer notices an application crash. They open the TOS Log sector, filter by the application surface, and see the last commands executed before the crash, along with resource spikes.
· Security Audit – A system administrator reviews the audit log for any privilege escalation attempts or failed authentication.
· Collaboration Recap – After a shared session, a host reviews guest actions to understand what was changed.
· Personal Productivity – A user searches their log for all instances of git commit to estimate time spent on version control.

10. Auditory and Haptic Interface

TOS integrates a rich auditory and haptic feedback system to enhance situational awareness, provide confirmation of actions, and create an immersive experience across all platforms. The interface is designed as a three‑layer model, with independent control over each layer and deep integration with the semantic event system.

10.1 Three‑Layer Audio Model

Layer Purpose Characteristics
Ambient Atmosphere and spatial context Continuous, depth‑varying background sound that changes subtly as the user zooms between levels. Provides an auditory sense of “where” the user is in the hierarchy.
Tactical Action confirmation and alerts Discrete earcons (short, distinctive sounds) for specific events: zoom in/out, command execution, mode changes, notifications, split actions, collaboration events, and alerts.
Voice Speech output Text‑to‑speech for announcements, screen reader output, AI responses, and contextual help. Supports multiple languages and voices.

· Volume Control – Each layer has independent volume control and can be enabled/disabled globally or per‑sector.
· Sound Themes – Users can install custom sound themes (.tos-audio packages) from the Marketplace, replacing default earcons and ambient tracks.

10.2 Context Adaptation

The auditory interface adapts dynamically to the user’s current context, particularly zoom level and alert state.

· Depth Variation – Ambient sounds shift subtly as the user moves between levels:
  · Level 1 (Global Overview) – Open, spacious ambience (e.g., soft wind or distant hum).
  · Level 2 (Command Hub) – Focused, “control room” ambience with subtle technical undertones.
  · Level 3 (Application Focus) – Intimate, close ambience that may incorporate application‑specific sounds (if provided by Application Model).
  · Level 4/5 (Inspection) – Quiet, analytical ambience, with minimal background distraction.
· Alert State Adaptation – When an alert is triggered, the audio mix changes to draw attention:
  · Green (Normal) – All layers as configured.
  · Yellow Alert – Ambient layer shifts to a slightly more urgent tone; tactical layer adds a periodic pulse (e.g., soft heartbeat) every few seconds; voice layer becomes more verbose (e.g., announces non‑critical events).
  · Red Alert – Ambient layer is replaced by a repeating, attention‑grabbing tone (or silenced entirely, depending on theme); tactical layer suppresses non‑critical earcons to avoid overload; voice layer prioritises critical messages (e.g., “System overheating – immediate action required”).

10.3 Spatial Audio (VR/AR)

In virtual and augmented reality environments, sounds are positioned in 3D space to match their visual origin.

· Notifications – A notification from a sector to the user’s left will sound as if coming from that direction.
· Collaboration – A collaborator’s voice (if voice chat is enabled) appears to emanate from their avatar’s location.
· Zoom Transitions – The “whoosh” of zooming in/out is spatialised to match the direction of movement.
· Bezel Interactions – Clicking a bezel control produces a sound that feels attached to the bezel’s position.

10.4 Theming and Extensibility

· Audio Themes – Users can browse and install audio themes from the Marketplace. A theme package (.tos-audio) includes:
  · Ambient tracks for each level and alert state.
  · Earcons for all tactical events.
  · Voice configuration (voice, speed, pitch).
· Application‑Supplied Sounds – Applications can provide custom tactical sounds via their Application Model, subject to user approval.
· User Customisation – Advanced users can replace individual sound files or adjust the mapping of events to earcons.

10.5 Haptic Feedback

Haptics parallel the tactical audio layer, providing tactile confirmation of events on supported devices.

· Device Support:
  · Game controllers (Xbox, PlayStation, Switch Pro, Steam Deck)
  · VR/AR controllers (HTC Vive, Oculus Touch, etc.)
  · Haptic touchpads (Apple Force Touch, some Windows Precision touchpads)
  · Mobile devices (Android vibration motor)
  · Accessibility switches (with configurable haptic output)
· Haptic Event Taxonomy – Each semantic event (see §8.1) maps to a specific haptic pattern:

Category Events Pattern Suggestion
Navigation zoom_in, zoom_out Ascending/descending pulse train
Selection select, secondary_select Quick, sharp click
Mode Control cycle_mode, set_mode_command Mode‑specific pulse sequences (e.g., 1 pulse for CMD, 2 for DIR, 3 for ACT)
Bezel Control toggle_bezel_expanded Light buzz or soft thud
System Commands tactical_reset_sector Distinctive long vibration with pause
Text Input text_input (keystroke) Subtle tick (can be disabled)
Voice voice_command_start Short “listening” pulse
Collaboration user_joined, raise_hand Gentle ping‑like vibration
Dangerous Actions dangerous_command_confirmation Sharp, insistent buzz, increasing in intensity
Alerts red_alert Pulsing vibration that escalates with alert level

· Spatial Haptics (VR/AR) – Haptic feedback is directional:
  · A notification from the left triggers vibration in the left controller.
  · Zooming in/out creates a sensation of “pulling” or “pushing” with both hands.
  · Dragging a surface produces continuous vibration that varies with speed.

10.6 Configuration

Users can fine‑tune both auditory and haptic feedback through a unified panel in Settings.

· Master Toggle – Enable/disable all non‑voice audio and haptics.
· Per‑Layer Volume – Sliders for Ambient, Tactical, and Voice.
· Per‑Category Enable – Checkboxes to enable/disable earcons/haptics for specific event categories (e.g., disable navigation sounds but keep alerts).
· Test Patterns – Buttons to play each earcon and trigger each haptic pattern for preview.
· Haptic Intensity – Global slider, plus per‑category intensity adjustments.
· Hearing‑Impaired Mode – Route tactical audio to haptic feedback (where supported) and increase visual indicators.
· Motor‑Impaired Mode – Haptics can confirm switch input or dwell selections; patterns may be simplified.

10.7 Platform Implementation

Platform Audio Haptics
Linux Wayland ALSA/PulseAudio (PipeWire ready) evdev haptic events for supported touchpads; SDL2 for controller haptics
Android XR OpenXR audio spatialisation; Android AudioManager OpenXR haptic feedback for controllers; Android Vibrator for phone mode
Android Phone Android AudioManager Android Vibrator (pattern support)

10.8 Accessibility Integration

· Screen Reader – Voice layer provides the foundation for screen reader output, with configurable verbosity (off, brief, verbose).
· Auditory Cues for Visual Indicators – Priority indicators and status dots can be announced via voice or represented by earcons.
· Haptic Confirmation – All actions can be confirmed haptically, benefiting users with visual or hearing impairments.
· Custom Patterns – Users can record custom haptic patterns and assign them to events.

11. Security Model

TOS is designed with a defence‑in‑depth security architecture, ensuring that the innovative interface does not compromise system integrity. The security model encompasses authentication, authorisation, process isolation, dangerous command handling, auditing, and platform‑specific considerations. All remote connections, module installations, and privileged operations are subject to explicit user consent and, where appropriate, multi‑factor confirmation.

11.1 Authentication

· Local Login – On Linux, TOS integrates with PAM (Pluggable Authentication Modules), supporting passwords, biometrics (via fprintd or similar), and smart cards. On Android, the system uses the Android Keystore and can prompt for biometric (fingerprint/face) or PIN authentication when required.
· Remote Connections – The TOS Remote Server (see §7) uses mutually authenticated TLS (mTLS) with optional SSH key fallback. Invite tokens for shared sectors are cryptographically secure, time‑limited, and single‑use by default.
· Session Management – Users remain authenticated for the duration of their TOS session. Inactivity timeouts can be configured to re‑lock the session, requiring re‑authentication.

11.2 Authorisation (RBAC)

Access to resources and actions is governed by a role‑based access control (RBAC) model, particularly in collaborative contexts.

Role Capabilities
Viewer Can see content but cannot issue commands or interact with the prompt. Read‑only access to files and applications (subject to underlying filesystem permissions).
Commenter Can type in the prompt and send messages, but commands are either restricted (executed in a sandbox) or require host approval. May be able to highlight areas or annotate.
Operator Full control over the sector: execute any command, launch/close applications, create splits, change viewports. Equivalent to local user privileges.
Co‑owner Same as Operator, plus the ability to invite other users, change roles, and modify sector settings (including sharing and security options).
Host (Owner) Implicitly has all rights; can terminate the session and revoke access at any time.

· Permission Enforcement – All guest actions are enforced on the host side; the host’s kernel and filesystem permissions ultimately determine what can be done.
· Granular Permissions – Future versions may allow finer‑grained permissions (e.g., “can access only this directory”, “can run only these commands”).

11.3 Process Isolation

Applications and modules run with the least privilege necessary, leveraging Linux security features.

· User Processes – Applications launched from TOS run as the user’s own processes, inheriting the user’s permissions. This is the standard Linux model.
· Optional Sandboxing – Users can enable additional sandboxing per application or per sector:
  · Flatpak – If an application is installed as a Flatpak, TOS respects its sandbox.
  · Firejail / Bubblewrap – TOS can launch applications inside these lightweight containers with configurable profiles (network access, filesystem visibility, etc.).
  · Docker / Podman – For server or development environments, sectors can be backed by containers.
· Module Isolation – Sector Types, Application Models, and AI backends (see §16.4) are sandboxed via the TOS module API. They run in isolated processes with limited capabilities, and any access to system resources (files, network, devices) must be declared in their manifest and explicitly granted by the user at installation time.
· Android – On Android, each TOS component respects the Android application sandbox; additional isolation is provided by the platform.

11.4 Dangerous Command Handling

Certain commands pose a risk to system stability or data integrity. TOS provides a multi‑modal confirmation mechanism for such commands.

· Configurable Dangerous Command List – A default list includes commands like rm -rf /, dd if=/dev/zero of=/dev/sda, chmod -R 000 /, etc. Users can extend or modify this list.
· Confirmation Methods – When a dangerous command is detected (via shell integration or pattern matching), TOS requires explicit confirmation:
  · Tactile Confirmation – The user must perform a specific physical action, such as holding a button for 2 seconds, sliding a slider, or performing a multi‑touch gesture.
  · Voice Confirmation – The user must speak a confirmation phrase (e.g., “yes, delete”).
  · Biometric Prompt – On supported hardware, a fingerprint or face scan may be required.
  · Multi‑user Approval – In collaborative sectors, a dangerous command may require approval from another operator or the host.
· Audit Trail – All dangerous command attempts (successful or denied) are logged in the audit log (see §11.6).
· User Education – When a dangerous command is first attempted, a brief explanation of the risk is shown, with a link to documentation.

11.5 Module Security

Modules (Sector Types, Application Models, AI backends) extend TOS functionality but introduce potential risks. A strict security model governs their installation and execution.

· Manifest Declarations – Every module includes a manifest (module.toml) that declares:
  · Permissions – Required access to filesystem paths, network domains, devices, environment variables, etc.
  · Capabilities – What the module can do (e.g., execute arbitrary code, access microphone, spawn subprocesses).
  · Dependencies – Other modules it requires.
· User Consent – During installation, TOS displays the requested permissions and capabilities. The user must explicitly accept them. Permissions can be granted permanently, for the session only, or denied.
· Sandboxing – Modules run in isolated processes with restricted system calls (seccomp), namespaces (where available), and network filtering. On Linux, this may be implemented via bubblewrap or similar; on Android, the platform’s own sandbox is used.
· Updates – When a module is updated, any new or escalated permissions are highlighted, and the user must re‑consent.
· Revocation – Users can revoke permissions or disable modules at any time via the Settings panel.

11.6 Deep Inspection Privilege

Access to Level 5 (raw memory) and certain sensitive metadata at Level 4 is considered privileged and requires explicit elevation.

· Default State – Level 5 access is disabled globally. Level 4 is fully accessible.
· Enabling Deep Inspection:
  · Linux Wayland – The user must run sudo tos enable-deep-inspection or authenticate via Polkit when first attempting to access Level 5. This enables the feature globally until explicitly disabled.
  · Android – Deep inspection is generally unavailable due to platform restrictions. If the device allows, a biometric prompt may grant temporary access to extended Level 4 metadata.
· Visual Indicator – When deep inspection is enabled, a 🔓 indicator appears in the Tactical Bezel (all levels). Clicking this indicator immediately disables deep inspection and closes any open Level 5 views.
· Auditing – All enable/disable events and every access to Level 5 are recorded in the system audit log. Level 4 access is logged in the TOS Log but may be disabled by user privacy settings.
· Application Opt‑Out – Applications can declare in their Application Model manifest that they should not be inspected at Level 5 (or at all). TOS respects this and will block or redact such inspection attempts.

11.7 Auditing

TOS maintains a comprehensive audit trail of security‑relevant events.

· Audit Log Contents:
  · Authentication successes and failures (local and remote).
  · Role changes in shared sectors.
  · Invite token generation and usage.
  · Module installations, updates, and permission changes.
  · Deep inspection enable/disable and accesses.
  · Dangerous command attempts (with outcome).
  · System‑level changes (e.g., updates, configuration changes).
· Storage – The audit log is stored separately from the main TOS Log, typically in /var/log/tos/audit.log on Linux (requires root read access) or in a protected system directory on Android. It is append‑only and cannot be cleared by the user.
· Review – Authorised users (e.g., system administrators) can view the audit log via a dedicated Security Dashboard or by querying the log with elevated privileges.

11.8 Platform Comparison

Aspect Linux Wayland Android XR / Phone
Authentication PAM (password, biometric, smart card) Android Keystore, biometric, PIN
Authorization Local user accounts + TOS roles Android permissions (per‑app) + TOS roles
Process Isolation Optional Flatpak/Firejail; native processes run as user Android sandbox (each app isolated); TOS modules run within TOS app sandbox
Dangerous Commands Tactile confirmation (hold, slider, etc.) Biometric prompt for sensitive actions
Deep Inspection Level 5 via sudo/Polkit; Level 4 full Level 5 unavailable; Level 4 limited
Module Sandboxing Bubblewrap, seccomp, network filters Android platform sandbox + additional checks
Audit Log System log (/var/log/tos/audit.log) Protected app storage; may be forwarded to system log

11.9 Security Dashboard

A centralised dashboard (accessible from Settings or via the command tos security) provides an overview of the system’s security state:

· Current Status – Indicates whether deep inspection is enabled, number of active remote connections, pending module updates, etc.
· Recent Alerts – List of recent security events (e.g., failed login attempts, dangerous command blocks).
· Module Permissions – Overview of installed modules and their granted permissions, with options to revoke.
· Audit Log Viewer – For authorised users, a read‑only view of the audit log with filtering.
· Configuration – Settings for dangerous command list, confirmation methods, auto‑lock timeout, etc.

11.10 Security Best Practices

TOS encourages secure usage through defaults and user education:

· Least Privilege – Modules and applications are granted only the permissions they explicitly request and need.
· Secure Defaults – Remote sharing is off by default; invite tokens expire; deep inspection is disabled.
· User Awareness – Confirmation prompts for dangerous actions include clear explanations; the Security Dashboard highlights potential risks.
· Regular Updates – The Marketplace notifies users of module updates, especially security‑related ones.

12. Application Models and Sector Types

TOS is designed to be extensible through two kinds of local modules: Application Models and Sector Types. These modules allow deep integration with specific applications or entire workspaces, tailoring the TOS experience to the user’s workflow while maintaining the core hierarchical model.

12.1 Application Models

An Application Model is a module that customises how a specific application (or class of applications) integrates with TOS at Level 3 (Application Focus) and influences behaviour at Level 2 (Command Hub). It encapsulates logic that would otherwise be application‑agnostic, enabling a richer, more context‑aware interface.

12.1.1 Capabilities

An Application Model can provide:

· Custom Bezel Actions – Additional buttons or menus in the expanded bezel that are specific to the application (e.g., “New Tab” for a browser, “Find” for an editor, “Build” for an IDE).
· Zoom Behaviour – Some applications (e.g., IDEs, document viewers) have internal hierarchical structures. The model can define how TOS zoom interacts with the application’s own interface (e.g., zooming into a function definition within an IDE).
· Legacy Decoration Policy – For X11 or non‑native applications, the model can specify whether TOS should suppress the application’s own window decorations, overlay the bezel on top, or leave decorations native.
· Thumbnail Generation – Provides a live thumbnail or icon for the application in Activity Mode (Level 2).
· Searchable Content – Exposes internal application data to the unified search (e.g., browser tabs, document titles, recent files). This content appears in SEARCH Mode results.
· Priority Factor Definitions – Custom weights or rules that influence the priority scoring of the application (e.g., a communication app might boost priority during an active call).
· Command Suggestions – Provides context‑sensitive command chips in the Command Hub’s right (prioritized) region when the application is focused (e.g., a Git model might suggest git status, git log when in a repository).
· Opt‑Out from Deep Inspection – The model can declare that the application should not be inspectable at Level 5 (raw memory) or even Level 4, protecting sensitive data.

12.1.2 API and Implementation

· Rust Trait – The primary API is a Rust trait that model implementors must satisfy. Key methods include:
  ```rust
  fn bezel_actions(&self) -> Vec<BezelAction>;
  fn handle_command(&self, command: &str) -> Option<CommandResult>;
  fn decoration_policy(&self) -> DecorationPolicy;
  fn thumbnail(&self, surface: &Surface) -> Option<Thumbnail>;
  fn searchable_content(&self) -> Vec<SearchableItem>;
  fn priority_factors(&self) -> Vec<PriorityFactor>;
  fn can_inspect(&self, level: InspectionLevel) -> bool;
  ```
· Scripting Support – Models can also be implemented in a lightweight scripting language (Lua or JavaScript) for rapid prototyping or less performance‑critical integrations. The script runs in a sandboxed environment with limited access to system resources.
· Hot‑Loading – Models are loaded dynamically. Changes to a model’s code can be applied without restarting TOS (subject to the application being restarted or the model’s update policy).

12.1.3 Installation and Location

· Linux: Models are installed in ~/.local/share/tos/app-models/ as shared objects (.so) or script files. System‑wide models can be placed in /usr/share/tos/app-models/.
· Android: Models are distributed as Android library plugins (.apk or dynamic feature modules) and installed via the TOS Marketplace or manually.

12.1.4 Security

· Manifest – Each model includes a manifest (model.toml) declaring:
  · Required permissions (filesystem access, network domains, etc.).
  · Capabilities (e.g., ability to spawn subprocesses).
  · Dependencies on other models or sector types.
· User Consent – On installation, TOS displays the requested permissions. The user must explicitly grant them. Permissions can be granted permanently, per session, or denied.
· Sandboxing – Models run in isolated processes with restricted system calls (seccomp), namespaces (where available), and network filtering. On Linux, this is implemented via bubblewrap; on Android, the platform’s app sandbox is used.

12.2 Sector Types

A Sector Type is a module that defines the default behaviour and environment for a sector at Level 2. When a new sector is created, the user can choose a type, which pre‑configures the sector with appropriate settings, favourites, and available Application Models.

12.2.1 Capabilities

A Sector Type can provide:

· Command Favourites – A set of user‑pinned commands that appear in the left chip region of the Command Hub, tailored to the sector’s purpose (e.g., a “Development” sector might have git status, make, cargo build).
· Context Chip Generation – Logic to generate context‑sensitive chips based on the current directory or active application (e.g., in a “Design” sector, if an image file is selected, offer “Open in GIMP”).
· Interesting Directory Detection – Rules to automatically switch to Directory Mode or highlight certain paths (e.g., a “Projects” sector might treat any directory containing a .git folder as “interesting”).
· Environment Variables – Default environment variables to set when launching shells or applications in the sector.
· Available Hub Modes – Some sectors may restrict which modes are available (e.g., a “Kiosk” sector might only allow Directory Mode).
· Default Guest Role – When the sector is shared, new guests are assigned this role unless overridden.
· Associated Application Models – A list of Application Models that are commonly used in this sector; they may be automatically loaded or suggested.
· Custom Zoom Behaviour – Defines whether zooming within the sector should respect internal application hierarchies (e.g., a “Database” sector might allow zooming into table schemas).

12.2.2 API and Implementation

· Rust Trait – Similar to Application Models, Sector Types implement a Rust trait:
  ```rust
  fn command_favourites(&self) -> Vec<CommandChip>;
  fn context_chips(&self, context: &Context) -> Vec<CommandChip>;
  fn is_interesting_directory(&self, path: &Path) -> bool;
  fn environment(&self) -> Vec<(String, String)>;
  fn available_modes(&self) -> Vec<HubMode>;
  fn default_guest_role(&self) -> Role;
  fn associated_app_models(&self) -> Vec<AppModelId>;
  ```
· Scripting Support – Also available for rapid development.

12.2.3 Installation and Location

· Linux: ~/.local/share/tos/sector-types/ (.so or script files).
· Android: Similar to Application Models, distributed as plugins.

12.2.4 Security

· Manifest and Permissions – Sector Types also declare required permissions (e.g., access to certain filesystem paths, network). They are sandboxed and require user consent.

12.3 Interaction with the UI

· Command Hub Integration – Both Application Models and Sector Types contribute to the left (favourites/context) and right (prioritized) chip regions. Their suggestions are merged with system‑generated ones and ranked according to the priority system.
· Bezel Customisation – Application Models directly influence the expanded bezel at Level 3, adding buttons that execute model‑defined actions.
· Search Integration – Models that expose searchable content populate SEARCH Mode results.
· Priority Scoring – Custom priority factors from models are incorporated into the weighted score for the associated surfaces.

12.4 Example: Git Application Model

```rust
// Hypothetical implementation
impl ApplicationModel for GitModel {
    fn bezel_actions(&self) -> Vec<BezelAction> {
        vec![
            BezelAction::new("Status", "git status"),
            BezelAction::new("Commit", "git commit"),
            BezelAction::new("Push", "git push"),
        ]
    }
    
    fn searchable_content(&self) -> Vec<SearchableItem> {
        // Expose recent commits as searchable items
        get_recent_commits().into_iter().map(|c| SearchableItem {
            title: c.message,
            subtitle: format!("{} by {}", c.hash, c.author),
            domain: "git".to_string(),
            action: Box::new(|| open_commit(c.hash)),
        }).collect()
    }
    
    fn priority_factors(&self) -> Vec<PriorityFactor> {
        vec![
            PriorityFactor::new("unpushed commits", 0.2, |app| has_unpushed_commits()),
        ]
    }
}
```

12.5 Example: Development Sector Type

```toml
# sector-type.toml
name = "Development"
version = "1.0.0"
description = "Sector configured for software development"

[ favourites ]
commands = ["git status", "cargo build", "cargo test", "make"]

[ context ]
# Generate chips when in a Cargo project
[[ context.rules ]]
pattern = "**/Cargo.toml"
chips = ["cargo update", "cargo doc --open"]

[ environment ]
RUST_BACKTRACE = "1"
EDITOR = "code"

[ modes ]
available = ["CMD", "DIR", "SEARCH"]  # Activity mode optional

[ guest ]
default_role = "Commenter"

[ associated_models ]
models = ["git", "rust-analyzer"]
```

12.6 Marketplace Distribution

Both Application Models and Sector Types can be packaged and distributed via the TOS Marketplace (see §15). Packages include the compiled module (or scripts), manifest, and optional icons/documentation. Installation follows the same permission‑granting flow as other modules.

12.7 Platform Notes

· Linux Wayland – Full support; modules are native shared objects.
· Android XR / Phone – Modules are Android library plugins; some system‑level integrations may be limited by platform sandboxing.


13. Shell API

The Shell API is the communication bridge between the TOS compositor (specifically the Command Hub) and the underlying shell. It enables bi‑directional exchange of state, suggestions, and commands, allowing the spatial UI to reflect the shell’s reality and the shell to be controlled through graphical interactions. The API is implemented via a set of standardised OSC (Operating System Command) escape sequences and custom event protocols, ensuring shell‑agnostic operation.

13.1 Purpose and Goals

· Real‑Time Synchronisation – Keep the Command Hub’s directory view, environment, and command suggestions in sync with the shell’s actual state.
· Rich Command Construction – Enable the hub to provide context‑aware completions, flag suggestions (eval‑help mapping), and dangerous command warnings.
· Shell‑Agnostic Design – Any shell can be used, as long as a corresponding Shell Provider module implements the API.
· Extensibility – Allow third‑party tools to inject custom command‑line GUI panels into the Command Hub via the API.

13.2 Shell Providers

A Shell Provider is a module that encapsulates the integration logic for a specific shell. It supplies:

· The shell executable and any required arguments.
· Integration scripts (e.g., for Fish, a config script that sets up the necessary OSC hooks; for Bash/Zsh, PROMPT_COMMAND and DEBUG traps).
· Spawning logic (how to create a PTY and attach the shell).

The reference implementation is Fish, which offers the deepest out‑of‑the‑box integration due to its event‑driven architecture. Providers for Bash, Zsh, and others are available through the same interface.

13.3 Communication Protocol

The API uses two channels:

· Shell‑to‑Compositor – The shell sends OSC escape sequences embedded in its output. These are intercepted by the TOS terminal emulator and parsed into semantic events.
· Compositor‑to‑Shell – The compositor writes special commands or data to the shell’s PTY, which the shell’s integration scripts interpret.

13.3.1 OSC Sequences (Shell → Compositor)

OSC Code Purpose Data Format
OSC 133 ; A Command start (no data) – indicates beginning of command input
OSC 133 ; B Command end (no data) – indicates command execution completed
OSC 133 ; C Command output start (optional) – marks beginning of command output
OSC 133 ; D Command output end (no data)
OSC 777 ; suggestions Provide command suggestions JSON array of suggestion objects (text, description, type)
OSC 777 ; directory Send current working directory Path string
OSC 777 ; command_result Report exit status and output preview { "exit": 0, "output": "..." }
OSC 777 ; cwd Inform of current working directory Path string
OSC 777 ; env Environment variable updates { "var": "NAME", "value": "..." }
OSC 777 ; dangerous_command Flag a command as dangerous Command string
OSC 777 ; completions Provide completions for a partial token JSON array of completion strings

All OSC sequences are terminated with ST (ESC \ or 0x9c).

13.3.2 Compositor Commands (Compositor → Shell)

These are written to the PTY as specially formatted strings that the shell’s integration script recognises:

Command Format Description
EXEC \x1b[2tEXEC <command>\n Execute a command (as if typed)
CD \x1b[2tCD <path>\n Change directory
COMPLETE \x1b[2tCOMPLETE <partial>\n Request completions for a partial token
LS \x1b[2tLS <path>\n Request directory listing (fallback if OSC not supported)
SETENV \x1b[2tSETENV <var>=<value>\n Set environment variable

The shell’s integration script traps these sequences (e.g., using PROMPT_COMMAND in Bash, or a custom key binding in Fish) and executes the appropriate action, sending results back via OSC where applicable.

13.4 Integration with the Command Hub

The Shell API powers several key features of the Command Hub:

· Directory Mode – When the shell’s working directory changes, the shell sends an OSC 777 ; directory sequence. The Command Hub updates its path bar and file grid accordingly.
· Command Execution – Commands typed in the prompt are sent via EXEC. The shell executes them, and output is captured and displayed in the terminal area. The shell signals command boundaries with OSC 133 ; A and OSC 133 ; B, allowing the hub to highlight commands separately from output.
· Autocomplete – As the user types, the hub sends COMPLETE requests. The shell returns completions via OSC 777 ; completions. These appear in the bezel‑born overlay and as prioritized chips.
· Eval‑Help Mapping – When the user types a command name, the hub can request its --help output (by executing the command with --help in a separate PTY or parsing cached help). The output is parsed to generate flag chips in the right region.
· Dangerous Command Detection – The shell (or a configurable list) flags dangerous commands. When such a command is about to be executed, the shell sends OSC 777 ; dangerous_command. The hub then triggers the tactile confirmation flow (see §11.4).
· Environment Variables – Changes to environment variables are reported via OSC 777 ; env. The hub can display relevant variables (e.g., $PWD, $PATH) in the Detail View (Level 4).

13.5 Implementation Details

· PTY Management – Each Command Hub maintains a pseudo‑terminal (PTY) for its shell session. The PTY’s master side is read by the TOS terminal emulator; the slave side is attached to the shell process.
· Integration Script Injection – When spawning a shell, TOS passes an initialisation command (e.g., --init-file for Bash, or a config file for Fish) that sources the integration script. This script sets up the necessary traps and OSC hooks.
· Fallback Mechanisms – If a shell does not support OSC sequences (or if the integration script fails), TOS falls back to parsing the terminal output directly (e.g., detecting prompts, parsing ls output for directory listings). This provides basic functionality even without full API support.

13.6 Security Considerations

· Command Injection Prevention – All commands sent via EXEC are properly escaped to prevent injection. The shell’s integration script must treat the received string as a single command, not as additional input that could be manipulated.
· Permission Checks – The EXEC command is only honoured if the session is at the Command Hub and the user has appropriate permissions (local user always does; guests are subject to role restrictions).
· Dangerous Command Filtering – The shell’s dangerous command detection is advisory; the final confirmation is handled by the hub’s security layer.

13.7 Platform Adaptations

Platform PTY Implementation Shell Availability
Linux Wayland Standard Unix PTY (/dev/pts) via forkpty or similar. Full shell access. Fish (reference), Bash, Zsh, any POSIX shell.
Android XR / Phone TOS bundles a lightweight shell (based on mksh) or integrates with Termux if installed. Remote shell execution via TOS Remote Server is also supported. Limited local shell; remote shells provide full functionality.

13.8 Example Flow: Directory Change

1. User types cd projects in the Command Hub prompt.
2. Hub sends EXEC cd projects to the shell’s PTY.
3. Shell executes the command, changes directory.
4. Shell’s integration script detects the directory change (e.g., via PROMPT_COMMAND in Bash) and emits OSC 777 ; directory /home/user/projects.
5. Hub receives the OSC sequence and updates the Directory Mode view to show the contents of /home/user/projects.
6. The shell also sends a new prompt (e.g., via PS1), which is displayed in the terminal output.

13.9 Extensibility

The Shell API is itself extensible. Third‑party tools can register custom OSC codes (in the OSC 777 space) to provide specialised functionality. For example, a version control plugin might send OSC 777 ; vcs_status with JSON data about the current repository, which the hub could display as chips or in a dedicated panel. This extensibility is managed through the module system, with appropriate permission prompts.


14. Tactical Reset

Tactical Reset is a two‑level emergency recovery system that allows users to quickly restore order when a sector or the entire TOS session becomes unresponsive or cluttered. It is designed to be fast, intuitive, and safe, with clear visual feedback and configurable confirmation steps.

14.1 Level 1 – Sector Reset

Resets the current sector to a clean state, closing all applications and returning to a fresh Command Hub.

· Trigger – Configurable, default:
  · Keyboard: Super+Backspace (Linux) or a similar system‑level shortcut.
  · Command: tos sector reset typed in any Command Hub prompt.
  · Bezel: In the expanded bezel at any level, a Reset Sector button may be present (user‑configurable).
  · Voice: “Reset sector” (with confirmation).
· Action:
  · Sends SIGTERM to all processes owned by the current sector (applications, background jobs).
  · Closes all split viewports within the sector.
  · Returns the sector to a single Level 2 Command Hub with a fresh shell session (preserving the sector’s type and configuration).
· Confirmation – By default, no confirmation is required (instant reset). Users can enable a 5‑second undo option in Settings, which displays an “Undo” button after reset, allowing the sector to be restored to its previous state (if process termination can be rolled back – limited to non‑destructive actions).
· Visual Feedback – The screen briefly flashes or dims, and a notification appears: “Sector reset complete.” An earcon (a short, distinctive sound) confirms the action.

14.2 Level 2 – System Reset

Resets the entire TOS session, affecting all sectors. This is a more drastic action with safety measures.

· Trigger – Configurable, default:
  · Keyboard: Super+Alt+Backspace (Linux) or equivalent.
  · Command: tos system reset (requires elevated confirmation).
  · Bezel: In the expanded bezel at Level 1 (Global Overview), a System Reset button is available (may be hidden by default).
  · Voice: “System reset” (requires confirmation).
· Dialog – When triggered, a modal dialog appears with three options:
  · Restart Compositor – Terminates all sectors, restarts the TOS compositor, and returns to the Global Overview with the user still logged in. All running applications are closed (data loss may occur; unsaved work should be saved beforehand).
  · Log Out – Ends the TOS session and returns to the system login manager (or Android home screen). All processes are terminated.
  · Cancel – Aborts the reset.
· Confirmation – The dialog requires tactile confirmation (see §11.4). The user must perform a specific action (e.g., hold a button for 3 seconds, slide a slider, speak a confirmation phrase) to proceed. A countdown (configurable, default 10 seconds) is shown; if the countdown expires without confirmation, the action is cancelled.
· Visual and Auditory Feedback – The dialog is prominent, with a warning colour (red/orange). A warning earcon plays when the dialog appears, and a continuous tone may sound during the countdown. Haptic feedback (intense, repeating pulses) accompanies the countdown on supported devices.
· Audit Trail – All system reset attempts (successful or cancelled) are logged in the audit log (see §11.7).

14.3 Configuration

Users can customise Tactical Reset behaviour in Settings:

· Enable/Disable Level 1 Reset – Toggle sector reset availability.
· Level 1 Confirmation – Choose between “No confirmation”, “Undo button (5s)”, or “Tactile confirmation” (same as Level 2).
· Level 2 Confirmation – Enable/disable countdown, adjust countdown duration, choose confirmation method (hold, slider, voice, etc.).
· Keyboard Shortcuts – Customise the key combinations for both reset levels.
· Bezel Buttons – Choose whether to show reset buttons in the expanded bezel (at appropriate levels).

14.4 Platform Adaptations

Platform Level 1 Reset Level 2 Reset
Linux Wayland Full support; processes are terminated via signals. Full support; compositor restart or logout via systemd/logind.
Android XR Supported; applications are closed via Android activity manager. Compositor restart may not be applicable; logout returns to Android home.
Android Phone Supported; apps are closed via Android activity manager. Logout returns to home screen; restart compositor may simply restart the TOS app.

14.5 Use Cases

· Runaway Application – An application becomes unresponsive and cannot be closed normally. The user triggers a sector reset, which forcefully terminates the application and returns to a clean Command Hub.
· Cluttered Workspace – After a long session with many splits and open applications, the user wants to start fresh without logging out. A sector reset clears everything in the current sector.
· System Glitch – If the TOS compositor itself becomes sluggish or behaves unexpectedly, a system reset (restart compositor) can resolve the issue without a full logout.

14.6 Relationship with Other Systems

· TOS Log – Both sector and system resets are recorded in the TOS Log (and audit log for system reset), including the trigger method and outcome.
· Collaboration – If a sector is shared, a sector reset affects all guests: their viewports are closed, and they are returned to the host’s fresh Command Hub. A system reset ends the session for all guests (they are disconnected).
· Application Models – Models may receive a shutdown notification before termination, allowing them to save state or perform cleanup (if they support it).


15. Sector Templates and Marketplace

TOS includes a flexible system for packaging, sharing, and discovering sector configurations and modules. The Marketplace provides a centralised (but user‑configurable) repository where users can browse, install, and update Sector Templates, Sector Types, Application Models, AI Backends, and Audio Themes. All packages are installed locally and run with explicit user‑granted permissions.

15.1 Package Types

Package Type Extension Description
Sector Template .tos-template A complete sector configuration export, including sector type, favourites, environment variables, pinned applications, and layout. Contains no executable code – only configuration data.
Sector Type .tos-sector A module that defines a sector's default behaviour (see §12.2). Contains executable code (Rust or script) and requires permissions.
Application Model .tos-appmodel A module that customises integration for a specific application (see §12.1). Contains executable code and requires permissions.
AI Backend .tos-ai A module providing an AI assistant backend (e.g., Ollama, OpenAI, Gemini). Contains connection logic and may include model files or API wrappers.
Audio Theme .tos-audio A collection of sound files and mappings for the auditory interface (see §10.4). No executable code.

15.2 Marketplace Architecture

· Repository Indices – The Marketplace is defined by one or more repository indices, each a JSON file over HTTPS. Indices list available packages with metadata: name, version, description, author, size, hash, download URL, dependencies, and required permissions (for code packages).
· User‑Configurable Repositories – Users can add, remove, or reorder repositories via Settings. Default repositories may be provided by the TOS project, but users are free to host their own.
· Search and Discovery – The Marketplace is integrated into the Command Hub's SEARCH Mode. Typing a query shows relevant packages as tiles, with installation status, ratings (if available), and permission requirements.

15.3 Package Contents and Structure

15.3.1 Sector Template (.tos-template)

A template is simply a TOML or JSON file capturing a sector's configuration:

```toml
name = "Web Development"
description = "Sector configured for web development with Node.js and VS Code"
version = "1.0.0"
sector_type = "development"  # References an installed Sector Type

[favourites]
commands = [
    "npm start",
    "npm test",
    "git status",
    "code ."
]

[environment]
NODE_ENV = "development"
EDITOR = "code"

[pinned_applications]
apps = ["firefox", "code", "terminal"]

[layout]
# Optional saved split layout
splits = [
    { type = "app", app = "code", size = 0.6 },
    { type = "hub", size = 0.4 }
]
```

When applied, the template creates a new sector with the specified configuration, installing any missing dependencies (Sector Types, Application Models) if the user consents.

15.3.2 Code Packages (.tos-sector, .tos-appmodel, .tos-ai)

Code packages include:

· Compiled binary (.so for Linux) or script files (.lua, .js).
· Manifest (module.toml) declaring metadata, permissions, and capabilities.
· Optional assets (icons, documentation).

Example manifest for an AI backend (from v1.2 Extensions):

```toml
name = "OpenAI GPT-4"
version = "1.0.0"
type = "ai-backend"
description = "Connect to OpenAI's GPT-4 model for AI assistance."
icon = "openai.svg"

[capabilities]
chat = true
function_calling = true
vision = false
streaming = true

[connection]
protocol = "https"
default_endpoint = "https://api.openai.com/v1/chat/completions"
auth_type = "api-key"  # or "oauth2", "none"

[permissions]
network = ["api.openai.com"]
filesystem = false

[configuration]
model = { type = "string", default = "gpt-4", options = ["gpt-4", "gpt-3.5-turbo"] }
temperature = { type = "float", default = 0.7, min = 0, max = 2 }
```

15.3.3 Audio Theme (.tos-audio)

Contains:

· Sound files (WAV, OGG, or MP3) for each earcon and ambient track.
· Mapping file (theme.toml) that maps semantic event names to sound files.
· Optional metadata (author, version, preview audio).

15.4 Installation Flow

1. Discovery – User finds a package via SEARCH Mode, browsing the Marketplace in Settings, or by opening a downloaded .tos-* file directly.
2. Details Panel – Clicking a package tile opens a details panel showing:
   · Description, version, author, size.
   · Screenshots (if available).
   · Required permissions (for code packages).
   · Dependencies (other packages that must be installed first).
   · User ratings and reviews (if repository supports them).
3. Permission Review – For code packages, TOS displays the requested permissions in a clear, non‑technical language (e.g., “This module will be able to access the internet (api.openai.com only)”). The user can choose:
   · Install – Grants all requested permissions permanently.
   · Install for this session only – Grants permissions until TOS restarts.
   · Cancel – Aborts installation.
4. Dependency Resolution – If the package has dependencies, TOS checks if they are already installed. Missing dependencies are presented for installation (with their own permission prompts) before the main package.
5. Installation – Files are copied to the appropriate local directory (~/.local/share/tos/ on Linux, app‑private storage on Android). The package is registered with the system and becomes available immediately (no restart required for most package types; AI backends may require a settings panel refresh).
6. Post‑Install – A confirmation notification appears. The package may appear in relevant UI locations (e.g., new AI backend in AI Mode settings, new Sector Type in sector creation dialog).

15.5 Security and Isolation

· Code Packages – Run in sandboxed processes with restricted capabilities (see §11.5). Network access is limited to domains declared in the manifest. Filesystem access is restricted to specified paths (if any). System calls are filtered via seccomp.
· Configuration‑Only Packages – No code execution; considered safe. Still subject to validation to prevent malformed data from causing issues.
· Signature Verification – Packages can be signed with GPG or minisign. If the user has imported the signer’s public key, TOS verifies the signature before installation and warns if it's invalid or missing. This is optional but recommended for official repositories.
· Updates – When an update is available, TOS notifies the user (Yellow Alert). The update details panel highlights any changes in permissions or capabilities. The user must re‑consent if permissions have been added or escalated.

15.6 Marketplace Discovery Enhancements

· Search Mode Integration – As noted, SEARCH Mode includes packages as a search domain. Results show package tiles with install buttons (or “Installed” status). Typing a query like ai backend ollama will find relevant packages.
· AI‑Assisted Discovery – In AI Mode, the assistant can help find packages based on natural language queries: “I need a Git integration for my terminal” might suggest relevant Application Models.
· Update Alerts – When an installed module has an update available, a Yellow Alert appears (see §8.7), with a notification chip in the Command Hub’s right region. Clicking it opens the updates panel.
· Ratings and Reviews – If the repository supports it, users can rate and review packages. Reviews are displayed in the details panel.

15.7 Creating and Sharing Packages

Users can create their own packages:

· Export Sector as Template – From any sector, a “Export as Template” option in the expanded bezel (Level 2) creates a .tos-template file. The user can choose which aspects to include (favourites, environment, pinned apps, layout).
· Package a Module – Developers can create Sector Types, Application Models, or AI backends by writing code and a manifest, then packaging them into a .tos-sector, .tos-appmodel, or .tos-ai file (essentially a tar/zip archive with a specific structure). Documentation and tools for this process will be provided.
· Submit to Repository – Users can submit their packages to public repositories (if the repository accepts submissions). The process varies by repository.

15.8 Platform Notes

Platform Installation Location Sandboxing
Linux Wayland ~/.local/share/tos/ (user) or /usr/share/tos/ (system) Bubblewrap, seccomp, network filtering
Android XR / Phone App‑private storage Android platform sandbox; modules run within TOS app sandbox

15.9 Example Workflow

1. User wants to add AI assistance to their TOS. They open SEARCH Mode and type ai assistant.
2. Results show several AI backend packages: “Ollama (local)”, “OpenAI GPT-4”, “Gemini”. They click on “Ollama”.
3. The details panel shows: description, size, permissions (“network access to localhost only”). They click Install.
4. TOS downloads the package, verifies its signature (if configured), and copies it to the appropriate directory.
5. A notification confirms installation. The user now sees “Ollama” as an option in the AI Mode backend selector.
6. Later, they receive an update alert for the Ollama package. They review the update details (no new permissions) and approve the update.


16. Accessibility

TOS is designed with accessibility as a first‑class concern, ensuring that the innovative spatial interface is usable by people with diverse abilities. The accessibility features are deeply integrated into every layer of the system, from input handling to visual presentation and auditory feedback. All features are configurable and can be combined to suit individual needs.

16.1 Visual Accessibility

· High‑Contrast Themes – TOS includes built‑in high‑contrast colour schemes that meet WCAG guidelines. Users can choose from several presets or customise colours per element (background, text, borders, chips). The interface automatically adapts to system‑wide high‑contrast settings where supported.
· Font Scaling and Customisation – All text elements (terminal output, chip labels, bezel controls) can be scaled independently of the UI size. Users can choose font families, sizes, and line spacing. A “large text” mode increases all UI text by a configurable percentage.
· Colourblind Filters – Built‑in colour filters (protanopia, deuteranopia, tritanopia, monochromacy) can be applied globally. Priority indicators (border chips, chevrons, status dots) can also be configured to use patterns or symbols in addition to colour.
· Focus Indicators – The currently focused element is always clearly marked with a thick, high‑contrast border. The indicator style (colour, thickness, animation) is user‑configurable. Haptic and auditory focus indicators can be enabled as alternatives.
· Screen Reader Support – TOS integrates with platform screen readers:
  · Linux: AT‑SPI (Orca) – all UI elements expose appropriate roles, states, and labels.
  · Android: TalkBack – full compatibility, with custom actions and navigation hints.
  · Braille displays are supported via the platform’s braille infrastructure.
· Reduced Motion – Users can disable or reduce animations (zoom transitions, chip movements, bezel expansions) to prevent disorientation.

16.2 Auditory Accessibility

· Screen Reader Output – The Voice layer (see §10.1) provides TTS for all UI elements. Users can adjust speech rate, pitch, and voice. Verbosity levels control how much information is spoken (e.g., “brief” announces only element names, “verbose” includes descriptions and state).
· Earcons for Navigation – All navigation actions (zoom in/out, mode changes, selection) have distinct earcons that can be enabled independently. Users can choose from different sound themes or replace individual sounds.
· Auditory Priority Indicators – Priority changes can be accompanied by earcons (e.g., rising tone for increased priority). The intensity and pitch can be mapped to priority level.
· Spatial Audio Cues – In VR/AR, sounds are positioned to indicate the location of events (e.g., a notification from a sector to the left is heard from the left).
· Voice Commands – Full voice control is available (see §8), allowing users to navigate, select, and execute commands without touching any device.

16.3 Motor Accessibility

· Switch Device Support – TOS supports single‑switch and multiple‑switch scanning:
  · Scanning Modes – Automatic (system cycles through elements at user‑set speed) or manual (user advances with switch press). Scanning patterns include linear (through all elements) or row‑column (for grids).
  · Switch Mapping – Any switch input (physical button, sip‑and‑puff, eye blink) can be mapped to scanning actions (next element, select, previous element, stop scanning).
  · Visual Scanning Indicator – The current element is highlighted during scanning; auditory cues can announce each element.
· Dwell Clicking – For gaze tracking, head tracking, or any pointing device, dwelling on an element for a configurable time triggers a selection. Dwell time and activation area size are adjustable.
· Sticky Keys – Modifier keys (Ctrl, Alt, Super) can be latched, allowing sequential key presses for keyboard shortcuts.
· Slow Keys – A delay before a key press is registered, accommodating users with unintentional presses.
· Haptic Feedback – All actions can be confirmed haptically (see §10.5). For switch users, haptic feedback confirms switch presses and scanning progress.
· Gesture Alternatives – Every gesture (pinch, swipe, etc.) has a keyboard or switch alternative. For example, zoom in can be triggered by Ctrl++ or a switch‑based command.
· Customisable Input Mapping – Users can remap any input device to any TOS action (see §8.4), creating personalised control schemes.

16.4 Cognitive Accessibility

· Simplified Mode – A system‑wide setting that reduces visual clutter, enlarges elements, and limits available features. In this mode:
  · Only essential UI components are shown (prompt, basic chips, output area).
  · Complex features (splits, deep inspection, collaboration) are hidden or simplified.
  · Navigation is restricted to the core three levels.
  · Tutorials and help are more prominently displayed.
· Built‑in Tutorials – Context‑sensitive tutorials (eval‑help mapping, interactive guides) help users learn the interface. Tutorials can be triggered manually or appear automatically for new users.
· Consistent Spatial Model – The strict three‑level hierarchy (with clear visual and auditory cues) provides a predictable mental model, reducing cognitive load.
· Notification Management – Users can control which notifications appear, how they are presented (visual, auditory, both), and set quiet hours.
· Command History and Favourites – Frequently used commands are easily accessible via the left chip region, reducing the need to remember syntax.

16.5 Configuration and Profiles

· Central Accessibility Panel – All accessibility settings are gathered in a dedicated section of the global Settings panel, organised by category (Visual, Auditory, Motor, Cognitive).
· User Profiles – Users can save and load accessibility profiles. For example, a “Low Vision” profile might enable high contrast, large text, and screen reader; a “Switch User” profile configures scanning and switch mappings. Profiles can be switched quickly from the bezel or via voice command.
· Per‑Sector Settings – Some accessibility features can be set per sector (e.g., simplified mode in a “Kids” sector, high contrast in a “Reading” sector).
· Import/Export – Profiles can be exported and shared, allowing users to transfer their settings to another TOS installation.

16.6 Platform Integration

Platform Screen Reader Braille Switch Access Dwell
Linux Wayland Orca via AT‑SPI BRLTTY Input remapping via evdev; scanning implemented by TOS Via gaze/head tracking drivers
Android XR TalkBack (when in phone mode); OpenXR may provide spatial audio cues Platform support via BrailleBack Android Switch Access service Via OpenXR gaze or hand tracking
Android Phone TalkBack BrailleBack Android Switch Access Via Accessibility Service (dwell)

16.7 Testing and Validation

TOS development includes accessibility testing with real users and adherence to WCAG 2.1 AA standards where applicable. Automated tests verify that all UI components expose correct accessibility metadata (labels, roles, states).

16.8 Future Directions

· AI‑Powered Accessibility – The AI assistant can adapt to user needs, e.g., suggesting simplified layouts for users who frequently activate simplified mode, or providing real‑time descriptions of complex visual content.
· Custom Accessibility Modules – The module system could allow third‑party developers to create specialised accessibility tools (e.g., eye‑tracking keyboards, advanced switch scanning algorithms).



