# TOS Visual Design Specification

**Purpose:** This document defines the visual language, UI layout, animation system, and multi-sensory feedback model for **TOS** (**Terminal On Steroids**). It is the authoritative reference for how TOS looks, sounds, and feels. For system structure and IPC contracts, refer to the Architecture Specification.

**Version:** 0.1.3

> **Spec 0.1.1 — 2026-11-17:** Carried forward from 0.1.0; the 0.1.1 revision defines the chip system as a logical dynamic user experience and lives in the [Architecture Specification](./architecture.md) (front note, §7.7) and the [Feature Specifications](./features.md) (§1). The 0.1.0 text is preserved in git history.
> **Spec 0.1.2 — 2026-11-21:** Carried forward from 0.1.1; the 0.1.2 revision makes the chip system dynamic against in-progress typing (type-ahead filtering, the panel's dotfiles switch) and sets the face terminal rendering requirement (canvas, not DOM). It lives in the [Architecture Specification](./architecture.md) (Spec 0.1.2 note). The 0.1.1 text is preserved in git history.
> **Spec 0.1.3 — 2026-11-22:** Adds §5, defining the ambient terminal plane for the three-surface face: the perspective (vanishing-point) rendering of the background terminal, the wallpaper layering and legibility rules, the chip panels' overlap zone, and the focus/dismiss fade timings. The perspective treatment is the Cinematic Triangular treatment (see the [Ecosystem Specification](./ecosystem.md) §1.5), defined here as the background terminal's goal-state rendering. §5.5 records the design-language decision: LCARS is an influence and a possible future theme, not the product's identity or default skin. The 0.1.2 text is preserved in git history.

> **Direction update — 2026-11-15:** TOS is being refocused as an ambient layer inside a host desktop (see the [Architecture Specification](./architecture.md), Direction Update). This document becomes **more** central, not less: the zoom/slide motion language and visual hierarchy defined here are the core of the background terminal, chips overlay, and prompt overlay. Apply it to those three surfaces first. See [Scope Split](../decisions/2026-11-15-scope-split.md) and [Roadmap](../roadmap.md).

---

## Table of Contents

1. [Priority-Weighted Visual Indicators](#1-priority-weighted-visual-indicators)
2. [Tactical Mini-Map](#2-tactical-mini-map)
3. [Auditory and Haptic Interface](#3-auditory-and-haptic-interface)
4. [Accessibility](#4-accessibility)
5. [The Ambient Terminal Plane](#5-the-ambient-terminal-plane)

---

## 1. Priority-Weighted Visual Indicators

Non-intrusive indicators convey relative importance without altering size or position.

### 1.1 Indicator Types

| Type | Description |
|---|---|
| **Border Chips** | Small coloured notches along tile border; number reflects priority level (1–5). |
| **Chevrons** | Arrows; pulsing indicates pending notification or critical status. |
| **Glow / Luminance** | Subtle inner/outer glow; intensity varies with priority. |
| **Status Dots** | Small coloured circles (blue=normal, yellow=caution, red=critical). |

**Color Shifts by Urgency:** Subtle accents at Level 1; dominant hazard colors (Orange/Red) at Level 4. Critical alerts may gently pulse their border opacity. High-priority visual changes trigger synchronized UI sounds or haptic pulses on Android/XR.

### 1.2 Priority Scoring & Configuration

Weighted factors (user-configurable):
- Recency of focus (40%)
- Frequency of use (20%)
- Activity level (CPU, memory, I/O) (15%)
- Notification priority (10%)
- Collaboration focus (10%)
- AI suggestion (5%)
- User pinning (override)
- Sector-specific rules

**Configuration:** Master toggle, colour per factor, sensitivity, per-factor visibility, hover tooltips.

### 1.3 Behaviour by Depth

- Level 1: Sector tiles show aggregate priority.
- Level 2: Application tiles show individual priority; chip regions use indicators.
- Level 3: Bezel may show priority chevron/glow; split viewport borders.
- Level 4: Inspection panels show inspected surface's priority and sibling mini-map.

---

## 2. Tactical Mini-Map

Ephemeral overlay providing spatial awareness.

### 2.1 Passive & Active States

- **Passive:** Semi-transparent, input passes through.
- **Active:** Activated by hover (dwell), keyboard (`Ctrl+M`), modifier+click, double-tap, voice. Captures input; shows close button.

### 2.2 Content by Depth

- Level 1: All sectors as miniature tiles.
- Level 2: Current sector with hubs, active hub highlighted.
- Level 3: Focused app highlighted, other viewports shown.
- Level 4: Current surface and siblings.

### 2.3 Monitoring Layer (Resource Usage)

Optional overlay (toggle) showing live resource usage:
- Level 1: Aggregated CPU/memory per sector.
- Level 2: All apps with CPU%, memory%, sparkline.
- Level 3: Detailed stats for focused app + compact for others.
- Throttled to 1–2 Hz.

### 2.4 Bezel Integration (Slot Projection)

The Tactical Mini-Map is docked within a slot in the **Left Bezel Segment**.
- **Docked State:** Occupies the 1.5rem width of the left bezel, showing only high-alert status lines.
- **Projected State:** When activated, it projects a wide glassmorphism overlay into the viewport center without expanding the sidebar.
- **Contextual Anchors:** Clicking tiles within the projected overlay triggers immediate level transitions.

---

## 3. Auditory and Haptic Interface

### 3.1 Three-Layer Audio Model

| Layer | Purpose | Characteristics |
|---|---|---|
| **Ambient** | Atmosphere | Continuous, depth-varying background. |
| **Tactical** | Action confirmation | Discrete earcons for zoom, commands, notifications, alerts, collaboration. |
| **Voice** | Speech synthesis | TTS for announcements, screen reader, AI responses. |

Each layer has independent volume and enable/disable.

### 3.2 Context Adaptation (Green/Yellow/Red Alerts)

- **Green:** Normal.
- **Yellow:** Ambient shifts urgent, tactical adds periodic pulse, voice more verbose.
- **Red:** Ambient replaced by repeating tone; tactical suppresses non-critical earcons; voice prioritises critical messages.

### 3.3 Spatial Audio (VR/AR)

Sounds positioned in 3D space. Notifications from the left sector sound left.

### 3.4 Haptic Feedback Taxonomy

| Event | Pattern |
|---|---|
| `zoom_in` | Ascending pulses |
| `select` | Quick click |
| `dangerous_command` | Sharp, insistent buzz |
| `red_alert` | Pulsing, escalating |

Scrolling the Cinematic Triangular terminal triggers subtle haptic detents. Spatial haptics in VR/AR (directional).

### 3.5 Theming & Configuration

- Audio themes (`.tos-audio`) installable via Marketplace (see Ecosystem Specification).
- Applications can contribute custom sounds.
- Configuration: master volume, per-category toggles, test patterns, hearing-impaired mode (route tactical to haptics).

---

## 4. Accessibility

### 4.1 Visual

- High-contrast themes, font scaling, colourblind filters.
- Screen reader support (AT-SPI/Orca on Linux, TalkBack on Android).
- Braille display support.
- Focus indicators (thick border, optional haptic/auditory).
- **High-Visibility Mode:** Forced thick borders, monochromatic glassmorphism for better contrast, increased font sizes.
- **Screen Reader Bridge:** Every UI element publishes a semantic role (button, line, chip) to the platform's accessibility bridge (AT-SPI / TalkBack).

### 4.2 Auditory

- Screen reader via Voice layer.
- Earcons for navigation and feedback.
- Voice notifications (TTS) with adjustable verbosity.

### 4.3 Motor

- Switch device support (single/multi-switch scanning, linear/row-column).
- Dwell clicking (gaze/head tracking).
- Sticky keys, slow keys.
- **Voice Confirmation:** Users can confirm commands via speech using a randomized challenge-response system if a tactile prompt is physically impossible.
- Haptic confirmation for actions.
- Customisable input mapping.

### 4.4 Cognitive

- Simplified mode (reduced clutter, larger elements, limited features).
- Built-in tutorials (eval-help mapping, interactive guides).
- Consistent spatial model (four levels, context-aware modes).

### 4.5 Profiles & Platform Integration

- Central Accessibility panel with profiles (save/load/export).
- Per-sector overrides.
- Integration with platform accessibility services (AT-SPI, TalkBack, Switch Access).

---

## 5. The Ambient Terminal Plane

How the background terminal of the three-surface face (terminal,
chips, prompt) is rendered and how it moves. This section defines
the visual treatment only: the terminal's content and behaviour
stay those of a completely normal terminal, and nothing here
changes what the terminal is — only how the background surface
presents it.

### 5.1 The perspective plane

At rest, the background terminal renders as a plane in
perspective — the Cinematic Triangular treatment (Ecosystem
Specification §1.5). For this surface the perspective rendering
is the goal state, not an option: the background terminal's
goal is cinematic, and a flat rectangular rendering of it is a
transitional form of the young slice, not an alternative goal:

- Output is anchored at the **bottom** of the screen. The newest
  lines sit at full size and full legibility — this is the
  **readable portion**, and it reads exactly like a normal
  terminal.
- History recedes from the readable portion toward a **vanishing
  point** in the upper part of the screen, each line scaling down
  and fading as it ages, until the oldest visible history is
  ambience rather than text.
- Scrolling is travel along the plane: history pulled back toward
  the viewer grows and sharpens as it re-enters the readable
  portion.

### 5.2 Layering and legibility

- A traditional desktop background (the user's wallpaper) sits
  behind everything. The host desktop keeps ownership of it; the
  face does not replace it.
- The terminal plane is translucent over that background — its
  opacity lets the background ghost through the receding region.
- Terminal text is rendered at high contrast against whatever is
  behind it, so the text stands out and the readable portion is
  never hard to read.

### 5.3 Chips over the plane

The dual-sided chip panels flank the plane and may **overlap the
edges of the receding region** — across nearly the top
three-quarters of the screen, the plane carries only distant
history, and the chips take precedence there. The readable
portion is never overlapped: where the terminal is meant to be
read, nothing sits on top of it.

### 5.4 Focus and dismiss motion

The surfaces have two visual states — **ambient** (at rest:
perspective plane, surfaces softened) and **focused** (full
visibility).

- Pulling focus to the terminal fades the readable portion and
  the chips **into full visibility over 400 ms**.
- Dismissing focus fades them **back to ambient over 300 ms** —
  dismissal is deliberately quicker than arrival.

### 5.5 The design language

The primary goal governs every visual choice: a terminal on
steroids that lowers the bar for terminal use and brings it to
every device in a coherent, dynamic, and elegant way. User
experience comes first; no visual identity outranks it.

LCARS informed this design's perspective — its clean,
functional, glanceable qualities are an influence the design
keeps. But LCARS is not the product's identity and not its
default skin, and it must never hinder the primary goal. The
face's built-in look is its own. The end look is themeable:
theme modules (Ecosystem Specification §1.8) may restyle the
surfaces, and an LCARS theme is a legitimate future module —
one theme among others, arriving with the theme system, not
before it.
