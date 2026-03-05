# TOS Alpha-2.2 Onboarding & First-Run Experience
### Specification v1.0 — Supplement to Architecture Specification

---

## 1. Philosophy

TOS is a dense, powerful system. Its depth is a feature, not a bug — but that depth must not be a barrier. The onboarding experience exists to compress the learning curve without patronizing experienced users or hiding the system's true nature.

Three principles govern all onboarding design:

- **SKIP** — Respect the skip. Every onboarding element must be skippable by a user who already knows what they are doing. No forced flows, no unskippable animations beyond 2 seconds.
- **DO** — Teach through doing. Users learn TOS by using TOS, not by reading about it. Guided steps happen inside the live system, not in a sandbox.
- **FADE** — Fade gracefully. Onboarding hints become less visible as user confidence grows. The system should feel like it trusts the user increasingly over time.

---

## 2. Onboarding State Model

The Brain tracks a persistent onboarding state object in the Settings Daemon under the namespace `tos.onboarding`. This state governs which onboarding elements are active, suppressed, or completed.

> **IPC KEY:** `tos.onboarding.state` — persisted via Settings Daemon, read on every Brain init

State schema (TOML):

```toml
[onboarding]
first_run_complete   = false   # Has the cinematic intro been shown?
wizard_complete      = false   # Has the guided demo workflow finished?
hints_dismissed      = []      # List of hint IDs the user has explicitly closed
hint_suppressed      = false   # Master kill-switch for all hints
sessions_count       = 0       # Incremented on each Brain start
commands_run         = 0       # Incremented on each PTY submit
levels_visited       = []      # Tracks which levels the user has reached
```

The Brain evaluates this state on startup and passes the relevant flags to the Face via the standard IPC state sync. The Face is responsible for rendering all onboarding UI elements; the Brain manages state persistence only.

---

## 3. First-Run Flow

The first-run experience fires once: when `first_run_complete` is false. It consists of two sequential stages with a hard skip available at any point.

### 3.1 Stage Overview

| Stage | Name | Trigger | Duration | Skip? |
| :--- | :--- | :--- | :--- | :--- |
| **S1** | Cinematic Intro | `first_run_complete = false` | ~12 seconds | YES — any key / tap |
| **S2** | Guided Demo | `wizard_complete = false` | ~4 minutes | YES — skip button always visible |
| **S3** | Ambient Hints | Ongoing (fadeable) | Indefinite | YES — per-hint or master off |

---

### 3.2 Stage 1: Cinematic Intro

A short, skippable cinematic sequence that plays before the system is interactive. This is the user's first impression of TOS's aesthetic identity — it should feel like booting a tactical system, not loading a desktop.

#### 3.2.1 Sequence

- **Duration:** 12 seconds total. Skip available immediately via any keypress, tap, or click.
- **Frame 0–2s:** Black screen. The TOS wordmark fades in with amber glow. Subtle startup earcon plays.
- **Frame 2–5s:** LCARS grid lines sweep in from edges, forming the bezel skeleton. No content yet — just structure.
- **Frame 5–9s:** Sector tiles fade in at Level 1, one by one. The Brain console output area activates with scrolling boot log text (real Brain init output streamed live). The system is visibly waking up.
- **Frame 9–12s:** Zoom transition inward to Level 2. The Command Hub assembles. Prompt cursor blinks. Text fades in: `SYSTEM READY.`
- **Frame 12s:** Cinematic ends. If `wizard_complete` is false, Stage 2 begins automatically. Otherwise, the system is live.

> **NOTE:** The boot log shown during Frame 5–9 is the actual Brain init output — not simulated. This reinforces the terminal-first identity from the very first second.

#### 3.2.2 Skip Behavior

Any keypress, mouse click, or touch during the cinematic immediately cuts to the end state (Level 2 Command Hub, live). `first_run_complete` is set to true. The skip is non-destructive: the Brain has already been initializing during the cinematic.

---

### 3.3 Stage 2: Guided Demo Workflow

The Guided Demo is an interactive walkthrough that runs inside the live system. The user is never moved to a sandbox or fake environment. All commands they run during the demo are real.

A non-blocking overlay panel appears in the bottom-left of the viewport above the bezel. It presents one step at a time with a brief instruction, an optional "Show me" shortcut, and a persistent **[SKIP TOUR]** button.

#### 3.3.1 Overlay Panel Design

- **Position:** Docked above bottom-left bezel, z-index above all content but below modals.
- **Style:** Glassmorphism card with LCARS amber accent border. Semi-transparent dark background.
- **Controls:** `[NEXT →]` to advance manually, `[SKIP TOUR]` to exit, `[← BACK]` to revisit the previous step.
- **"Show me" button:** Where applicable, auto-executes the step action so the user can just watch. One tap to auto-type and run the example command.

#### 3.3.2 Guided Demo Steps

| Step | Instruction Shown | "Show Me" Action | What It Teaches |
| :--- | :--- | :--- | :--- |
| **1** | This is your Command Hub. The terminal is always here — it never goes away. | Highlights the terminal output area | Core terminal-first identity |
| **2** | Type a command, any command. Try: `ls` | Auto-types `ls` in the prompt | Basic prompt interaction |
| **3** | Notice the chips that appeared? Click one to append it to your command. | Highlights the nearest file chip | Directory context chips |
| **4** | Hold `Ctrl+Tab` to see all your Sectors at Level 1. | Triggers `Ctrl+Tab` zoom out | Level 1 navigation |
| **5** | Click your sector to zoom back in. | Highlights the default sector tile | Level zoom mechanics |
| **6** | Press `Ctrl+M` to bring up the Minimap. | Fires `Ctrl+M` shortcut | Minimap / bezel slot |
| **7** | Type a question in plain English. Try: `show me running processes` | Auto-types the query, switches to AI mode | AI Augmentation mode |
| **8** | You're ready. Explore freely — press `[?]` any time for help. | Dismisses overlay, pulses the `?` badge | Completion + help shortcut |

> **DESIGN RULE:** The demo never blocks the user. At any step they can ignore the overlay and just use the system. The overlay tracks completion by detecting the relevant system event (e.g. a successful `ls` completing Step 2), not by enforcing sequence.

#### 3.3.3 Completion

On Step 8 or on `[SKIP TOUR]`, `wizard_complete` is set to true in the Settings Daemon. The overlay animates out. Ambient hints (Stage 3) activate.

---

## 4. Ambient Hints System

After the guided demo, TOS continues to teach through non-blocking contextual hints. These are small tooltip-style overlays that appear when a user encounters a feature for the first time, and fade permanently once dismissed or after the user has clearly learned the feature.

### 4.1 Hint Anatomy

- A hint consists of: a target element (bezel slot, chip type, shortcut key), a brief label (max 12 words), and an optional action link.
- Hints appear with a 300ms fade-in. They never occlude the prompt or terminal output.
- **Dismissal:** clicking the `[x]` on a hint adds its ID to `hints_dismissed` permanently. The hint never reappears.
- **Auto-dismissal:** if the user performs the hinted action independently, the hint fades out and is added to `hints_dismissed`.

### 4.2 Hint Suppression

A master toggle in **Settings → Interface → Onboarding** sets `hint_suppressed = true`, immediately hiding all active hints and preventing future ones. This is the power-user escape hatch. It is also offered as a one-tap option at the end of the Guided Demo for users who felt the tour was sufficient.

### 4.3 Hint Decay

Hints become progressively less visible as user confidence grows, measured by `sessions_count` and `commands_run`:

| Threshold | Opacity | Pulse |
| :--- | :--- | :--- |
| Sessions 1–3 / Commands 0–50 | 100% | Amber pulse border active |
| Sessions 4–7 / Commands 51–200 | 70% | No pulse |
| Sessions 8–14 / Commands 200–499 | 40% | Whisper — barely visible |
| Sessions 15+ / Commands 500+ | Auto-suppressed | Re-enable manually in Settings |

> **RATIONALE:** A user who has run 500 commands knows what they are doing. The system should trust them. Continued hints at this stage would feel condescending.

### 4.4 Hint Registry (Initial Set)

| Element | Tooltip Text | Condition |
| :--- | :--- | :--- |
| Bezel Lateral Slots | These slots are configurable — dock any component here | First time Level 2 loads |
| `[AI]` Mode Button | Ask anything in plain English — commands are staged, never auto-run | First time CMD mode used without AI |
| Right-Click on Chip | Right-click any chip for deep options: inspect, signal, renice | First chip rendered |
| `Ctrl+Tab` | See all your Sectors from Level 1 | After 3 commands run |
| `Ctrl+Alt+Backspace` | Emergency recovery: Tactical Reset (God Mode) | After first error exit code |
| Status Badge (top-right) | Generate a secure link to share this session remotely | After session 2 |
| Earcons | Audio cues are configurable in Settings → Interface | First mode switch |
| `[?]` Help Badge | Replay the guided tour or browse the full manual | End of tour / any time |

---

## 5. Re-Access & Persistent Help

Onboarding is not a one-time event. Users must be able to return to it at any time without resetting their system state.

### 5.1 The [?] Help Badge

A persistent `[?]` badge lives in the Top Bezel Right section alongside the status badge. It is always visible and never disappears. Clicking it opens a non-blocking Help Modal with three options:

- **Replay Tour** — restarts the Guided Demo overlay from Step 1 without resetting `wizard_complete` or any system state.
- **Open Manual** — opens the TOS User Manual in an Application Focus window (Level 3).
- **Reset Hints** — clears `hints_dismissed`, re-enabling all ambient hints from the beginning.

### 5.2 IPC Contracts

The following IPC messages support the onboarding system:

| Message | Effect |
| :--- | :--- |
| `onboarding_skip_cinematic` | Immediately ends Stage 1 |
| `onboarding_skip_tour` | Sets `wizard_complete = true`, dismisses overlay |
| `onboarding_advance_step` | Advances guided demo to next step |
| `onboarding_hint_dismiss:<hint_id>` | Permanently dismisses a specific hint |
| `onboarding_hints_suppress` | Sets `hint_suppressed = true` |
| `onboarding_replay_tour` | Re-opens guided demo overlay from Step 1 |
| `onboarding_reset_hints` | Clears `hints_dismissed` array |

---

## 6. Integration with Existing Architecture

The onboarding system is a thin layer on top of existing TOS architecture. It introduces no new subsystems and requires only the following integration points:

- **Settings Daemon:** New `tos.onboarding` namespace (see §2). No schema conflicts with existing settings.
- **Brain IPC Handler:** New `onboarding_*` message prefix handled by a dedicated `OnboardingService` module. All existing IPC prefixes are unaffected.
- **Face (Web UI):** New `<OnboardingOverlay>` component rendered at the root level, z-indexed above content but below modals. Reads onboarding state from the Brain state sync WebSocket.
- **Earcon Service:** Two new earcons — `onboarding_start` (cinematic begin) and `onboarding_complete` (tour end). Registered in the audio asset manifest.
- **Top Bezel Right Section:** `[?]` badge added as a permanent non-configurable slot element alongside the existing status badge.

> **DEPENDENCY:** The cinematic intro requires the Brain to be fully initialized before Frame 5 begins. The init sequence should target completion within 4 seconds to allow buffer. If Brain init exceeds 4s, the cinematic holds on Frame 2–5 until ready.

---

## 7. Power User Fast Path

A user who has used a terminal their entire life should be able to go from launch to a working prompt in under 5 seconds. The following guarantees that:

- Any keypress during the cinematic skips it instantly. The Brain is already initialized in parallel.
- The Guided Demo overlay appears but does not intercept any input. The prompt is live immediately.
- A single `[SKIP TOUR]` click removes the overlay permanently and suppresses hints.
- Total interaction cost for a power user who skips everything: **2 clicks, under 3 seconds.**

**Acceptance criterion:** A power user must be able to reach a live, unobstructed prompt within 5 seconds of launch. This is the single testable bar for the onboarding implementation.

---

*TOS Alpha-2.2 // Onboarding Specification v1.0 // Supplement Document*
