# Terminal On Steroids — User Story Backlog
**Beta 0 · Development Reference**

---

## Table of Contents

1. [Navigation & Hierarchy](#1-navigation--hierarchy)
2. [Command Hub & Persistent Unified Prompt](#2-command-hub--persistent-unified-prompt)
3. [AI Co-Pilot System](#3-ai-co-pilot-system)
4. [SEARCH Mode](#4-search-mode)
5. [Trust & Security Model](#5-trust--security-model)
6. [Multi-Sensory Feedback](#6-multi-sensory-feedback)
7. [Onboarding & First-Run Experience](#7-onboarding--first-run-experience)
8. [Marketplace & Module System](#8-marketplace--module-system)
9. [Collaboration](#9-collaboration)
10. [Performance & Accessibility](#10-performance--accessibility)
11. [Appendix A — Story ID Index](#appendix-a--story-id-index)

---

## 1. Navigation & Hierarchy

Stories covering the five-level zoom model, sector management, and the core navigation contract.

| ID | As a… | I want to… | So that… |
|---|---|---|---|
| NAV-01 | power user | zoom in from the Global Overview into a specific sector with a single gesture | I can move from bird's-eye oversight to active terminal work without touching a mouse |
| NAV-02 | power user | zoom back out to the Global Overview from any active sector | I can context-switch between projects in one fluid motion |
| NAV-03 | developer | drill into Deep Inspection (Level 4) from an active Command Hub | I can inspect raw buffer contents and process metadata without leaving the session |
| NAV-04 | new user | see a clear visual indicator of my current depth level | I always know where I am in the hierarchy and how to return |
| NAV-05 | operator | create a new sector and have it auto-labeled based on my working directory | I spend less time naming contexts and more time working |
| NAV-06 | operator | freeze, clone, or close a sector from the tile context menu | I can manage session lifecycle without disrupting other running workloads |
| NAV-07 | collaborator | see the navigation depth of remote guests in real time | I understand what each collaborator is currently viewing |

### NAV-01 Acceptance Criteria
- Zoom-in gesture triggers the `ZoomIn` semantic event, not a raw key binding.
- The `recursive-zoom` keyframe animation plays during the transition (< 300 ms).
- Focus lands on the previously active Command Hub of the target sector.

### NAV-05 Acceptance Criteria
- When `cwd` changes to a directory containing a known project file (e.g., `Cargo.toml`, `.git`), the sector label updates within 2 s.
- The user can lock the label via the tile context menu to prevent further auto-rename.
- Locked sectors display a padlock badge on the tile.

---

## 2. Command Hub & Persistent Unified Prompt

Stories covering the four hub modes, the always-visible prompt, and context-aware chip layout.

| ID | As a… | I want to… | So that… |
|---|---|---|---|
| HUB-01 | developer | switch between CMD, SEARCH, and AI modes from the prompt without lifting my hands from the keyboard | I stay in flow regardless of what I need to do next |
| HUB-02 | developer | have the hub automatically enter Directory Mode when I type `ls` or `cd` | I get rich file chips without changing my natural shell habits |
| HUB-03 | operator | have the hub automatically enter Activity Mode when I run `top` or `ps` | I can act on processes directly from the chip layout without a secondary tool |
| HUB-04 | developer | see the left chip region populate with contextual favorites and pinned paths | I can stage common commands in one tap |
| HUB-05 | developer | see the right chip region surface predictive completions and AI-suggested commands | I can accept smart suggestions without retyping |
| HUB-06 | power user | have the prompt remain visible and accessible at all navigation levels | I never lose my command line no matter how deep I zoom |
| HUB-07 | developer | click on a "Focus Error" chip after a failed build | I am immediately scrolled to the authoritative error line in the output |
| HUB-08 | developer | have typo correction chips appear when I submit a mistyped command | I can fix errors in one tap and re-run without retyping |

### HUB-02 Acceptance Criteria
- Brain command dispatcher detects `ls` prefix (case-insensitive) with no false positives (`rls`, `echo cd`).
- Mode switches to Directory Context; file and folder chips populate within 500 ms.
- Clicking a folder chip navigates into it; clicking a file chip appends the path to the staged command.

### HUB-07 Acceptance Criteria
- When PTY output includes at least one line tagged Priority 4 or higher, a "Focus Error" chip appears in the right region.
- Clicking the chip scrolls the terminal output to the first Priority 4+ line.
- The authoritative error line is rendered with increased visual weight (color accent, bold).

---

## 3. AI Co-Pilot System

Stories covering the Passive Observer, Chat Companion, and the AI safety boundary — the AI never executes commands directly.

| ID | As a… | I want to… | So that… |
|---|---|---|---|
| AI-01 | developer | receive a non-blocking correction chip when a command fails | I can recover immediately without manually diagnosing the error |
| AI-02 | developer | ask a question in plain English in AI mode and receive a staged command | I can explore unfamiliar tools without memorizing syntax |
| AI-03 | developer | review and edit the AI-staged command before it executes | I stay in full control — nothing runs behind my back |
| AI-04 | operator | install an alternative AI backend from the Marketplace | I can use my preferred LLM provider or a local model |
| AI-05 | operator | toggle individual AI behavior modules on or off independently | I can disable the Chat Companion without losing the Passive Observer |
| AI-06 | developer | have the AI silently watch for long-running commands and surface an explanation chip | I understand what a hung process is doing without interrupting it |
| AI-07 | team lead | have AI behavior automatically activate based on project context signals | Domain-specific assistance appears without me configuring it manually per session |
| AI-08 | developer | have AI chat history restored when I return to a sector | I can resume multi-turn conversations without losing context |

### AI-03 Acceptance Criteria
- Every AI suggestion is placed into the prompt input field — it is never auto-submitted.
- The staged command is fully editable before the user submits.
- The AI explanation is visible in the terminal canvas alongside the staged command.

### AI-07 Acceptance Criteria
- Behavior modules declare `context_signals` in their manifest (e.g., `.git`, `Cargo.toml`).
- The AI Engine evaluates signals against the current `cwd` and activates the matching module.
- Activation is logged with the sector name and signal matched.

---

## 4. SEARCH Mode

Stories covering semantic and filesystem search, result chips, and cross-domain filtering.

| ID | As a… | I want to… | So that… |
|---|---|---|---|
| SRC-01 | developer | switch to SEARCH mode and get instant results across the filesystem | I find files without leaving the terminal or opening a separate app |
| SRC-02 | developer | filter search results by scope (files, processes, history, docs) using chips | I narrow results without typing additional flags |
| SRC-03 | power user | use semantic natural-language queries in SEARCH mode | I can describe what I'm looking for without knowing the exact filename |
| SRC-04 | developer | execute a quick action (open, copy path, delete) directly from a search result chip | I act on results without staging a separate command |

### SRC-01 Acceptance Criteria
- Switching to SEARCH mode via the mode selector triggers the `SearchModeEnter` semantic event.
- First results appear within 300 ms of the first keystroke.
- The terminal canvas displays results; the left chip region populates with filter scopes.

---

## 5. Trust & Security Model

Stories covering the non-blocking warning system, explicit trust promotion, and role-based access in collaboration.

| ID | As a… | I want to… | So that… |
|---|---|---|---|
| SEC-01 | operator | see a non-blocking warning chip when I stage a command in a WARN trust class | I am informed about privilege escalation without being blocked from proceeding |
| SEC-02 | operator | permanently promote a command class to TRUST by tapping the chip label | I eliminate friction for commands I run routinely |
| SEC-03 | operator | revert a TRUST promotion back to WARN from Settings | I can tighten security after granting trust without reinstalling |
| SEC-04 | new user | configure trust classes during the first-run wizard without any pre-selected defaults | I make an explicit, informed decision for each class |
| SEC-05 | team lead | assign collaborators the Viewer, Commenter, Operator, or Co-owner role | Guests can contribute at the appropriate privilege level |
| SEC-06 | auditor | have all guest actions recorded in the host's TOS Log with guest identity | I can audit who did what and when during a collaboration session |

### SEC-01 Acceptance Criteria
- Warning chip renders above the prompt in the right bezel area — it does not block the input field.
- Pressing Enter while the chip is visible submits the command immediately.
- The chip is dismissed automatically when the command executes.

### SEC-02 Acceptance Criteria
- Tapping `[Trust Class]` on the chip triggers the `trust_promote` IPC message with the command class identifier.
- The Settings Daemon persists the promotion synchronously before the next command can stage.
- Subsequent commands in the same class run with no chip and no friction.

---

## 6. Multi-Sensory Feedback

Stories covering earcons, haptic pulses, alert levels, and the notification center.

| ID | As a… | I want to… | So that… |
|---|---|---|---|
| MSF-01 | power user | hear a distinct earcon for each mode switch and level zoom | I receive audio confirmation of my navigation state without looking at the screen |
| MSF-02 | power user | receive a haptic pulse when a command I submitted completes | I know the result without watching the terminal |
| MSF-03 | operator | have the interface shift to Red alert level with a repeating tone when a critical notification arrives | I cannot miss an urgent system event |
| MSF-04 | accessibility-focused user | disable all audio and haptic feedback from a single settings toggle | I can use TOS comfortably in silent environments |
| MSF-05 | operator | dismiss a Priority 3 notification with a single tap; critical notifications require explicit interaction | High-priority alerts cannot be accidentally cleared |

### MSF-03 Acceptance Criteria
- A Priority 5 system event sets the global alert level to Red.
- A repeating earcon plays at the audio engine level — not via the Face — to ensure it sounds even if the UI is partially frozen.
- The notification unfurls in the Right Lateral Bezel with a red border and remains until the user explicitly dismisses it.

---

## 7. Onboarding & First-Run Experience

Stories covering the cinematic intro, guided demo, ambient hints, and the power-user escape hatch.

| ID | As a… | I want to… | So that… |
|---|---|---|---|
| ONB-01 | new user | watch a skippable cinematic intro that establishes the TOS aesthetic identity | I understand what kind of system I am entering before I interact with it |
| ONB-02 | new user | step through a guided demo that uses the live system, not a sandbox | I learn by doing in real context rather than a simulated environment |
| ONB-03 | power user | reach a live, unobstructed prompt within 5 seconds of launch | I am never forced through an onboarding flow I have already completed |
| ONB-04 | new user | dismiss individual ambient hints and have that choice persisted | Hints I have absorbed stop appearing without me disabling hints globally |
| ONB-05 | new user | configure trust classes during the wizard with no defaults pre-selected | I understand what I am authorizing before granting any trust |

### ONB-03 Acceptance Criteria
- If `wizard_complete = true` and `first_run_complete = true`, the system boots directly to Level 1 with no intro or guided overlay.
- From a cold start, the time from application launch to an interactive prompt must be ≤ 5 seconds on reference hardware.
- This criterion is enforced as a performance integration test in `tests/headless_brain.rs`.

---

## 8. Marketplace & Module System

Stories covering module discovery, installation, sandboxing, and the permission model.

| ID | As a… | I want to… | So that… |
|---|---|---|---|
| MKT-01 | developer | browse and install a new Terminal Output Module from the Marketplace | I can replace the default rectangular terminal with an alternative visual style |
| MKT-02 | operator | see the declared permissions before installing any module | I make an informed consent decision before granting system access |
| MKT-03 | developer | sideload a community module by adding its developer public key | I can use trusted unsigned packages without compromising the broader security model |
| MKT-04 | operator | have Standard Tier modules run in a sandboxed environment automatically | Third-party code cannot access system resources it did not declare |
| MKT-05 | developer | install an alternative AI backend that replaces the default LLM provider | I can point TOS at a local model without modifying core system files |

### MKT-02 Acceptance Criteria
- The permission review screen lists every entry in the module's `[permissions]` manifest block.
- The user must scroll to the bottom of the permission list or explicitly tap "I have reviewed permissions" before the Install button becomes active.
- If the manifest declares no permissions, the screen still renders with a "No special permissions required" notice.

---

## 9. Collaboration

Stories covering real-time presence, following mode, role management, and the audit trail.

| ID | As a… | I want to… | So that… |
|---|---|---|---|
| COL-01 | team lead | invite a collaborator to a sector via a secure one-time token | I can share a live session without exposing persistent credentials |
| COL-02 | viewer | see live avatar presence for each active collaborator on the Global Overview | I know who is working where without asking |
| COL-03 | collaborator | follow another user's viewport so it stays synchronized with theirs | I can observe a colleague's actions in real time during a pairing session |
| COL-04 | team lead | promote a Viewer to Operator and have the change take effect immediately | I can dynamically adjust access without ending the session |
| COL-05 | team lead | have all guest actions tagged with guest identity in the TOS Log | I maintain a full audit trail for compliance purposes |

### COL-01 Acceptance Criteria
- One-time token expires after 30 minutes of inactivity.
- Token generation triggers a `collaboration_session_created` log entry at `LogType::Security`.
- The invited user sees a privacy notice before joining.

---

## 10. Performance & Accessibility

Stories covering frame-rate targets, headless testing, and keyboard/screen reader accessibility.

| ID | As a… | I want to… | So that… |
|---|---|---|---|
| PER-01 | desktop user | maintain 60 FPS during all UI transitions and mode switches | The system feels responsive and premium at all times |
| PER-02 | VR user | maintain 90 FPS in OpenXR environments under normal workloads | I avoid nausea caused by frame drops in immersive mode |
| PER-03 | operator | see a Tactical Alert if frame rate drops below target for more than 2 seconds | I am informed of performance degradation without it blocking my work |
| PER-04 | keyboard user | navigate the sector tile context menu entirely with the keyboard | I can manage sectors without a pointing device |
| PER-05 | screen reader user | have all sector tiles and mode buttons announced correctly by the system screen reader | TOS is operable with assistive technology |

### PER-03 Acceptance Criteria
- Frame rate is sampled every 500 ms by the rendering subsystem.
- If the rolling 2-second average falls below 60 FPS (desktop) or 90 FPS (VR), the Tactical Alert chip appears non-intrusively in the corner of the screen.
- The chip displays current FPS and a one-tap link to the Performance diagnostics panel.
- The alert auto-dismisses once frame rate recovers for 3 consecutive seconds.

---

## Appendix A — Story ID Index

| Story ID | Epic | Summary |
|---|---|---|
| NAV-01 | Navigation | Zoom in to a sector from Global Overview |
| NAV-02 | Navigation | Zoom out to Global Overview from any sector |
| NAV-03 | Navigation | Drill into Deep Inspection (Level 4) |
| NAV-04 | Navigation | Visual depth level indicator |
| NAV-05 | Navigation | Auto-label sectors from cwd |
| NAV-06 | Navigation | Sector lifecycle via tile context menu |
| NAV-07 | Navigation | Remote guest depth visibility |
| HUB-01 | Command Hub | Keyboard mode switching |
| HUB-02 | Command Hub | Auto Directory Mode on ls / cd |
| HUB-03 | Command Hub | Auto Activity Mode on top / ps |
| HUB-04 | Command Hub | Left chip region context favorites |
| HUB-05 | Command Hub | Right chip predictive completions |
| HUB-06 | Command Hub | Prompt visible at all levels |
| HUB-07 | Command Hub | Focus Error chip on build failure |
| HUB-08 | Command Hub | Typo correction chips |
| AI-01 | AI Co-Pilot | Non-blocking correction chip on failure |
| AI-02 | AI Co-Pilot | Plain-English to staged command |
| AI-03 | AI Co-Pilot | Review and edit staged command |
| AI-04 | AI Co-Pilot | Install alternative AI backend |
| AI-05 | AI Co-Pilot | Toggle behavior modules independently |
| AI-06 | AI Co-Pilot | Long-running command explanation chip |
| AI-07 | AI Co-Pilot | Context-signal behavior activation |
| AI-08 | AI Co-Pilot | AI chat history restored on sector restore |
| SRC-01 | Search | Instant filesystem search results |
| SRC-02 | Search | Scope filter chips |
| SRC-03 | Search | Semantic natural-language queries |
| SRC-04 | Search | Quick actions from result chips |
| SEC-01 | Trust | Non-blocking WARN chip |
| SEC-02 | Trust | Promote command class to TRUST |
| SEC-03 | Trust | Revert TRUST to WARN |
| SEC-04 | Trust | First-run trust config with no defaults |
| SEC-05 | Trust | Assign collaborator roles |
| SEC-06 | Trust | Guest action audit log |
| MSF-01 | Multi-Sensory | Earcons on mode/level changes |
| MSF-02 | Multi-Sensory | Haptic on command completion |
| MSF-03 | Multi-Sensory | Red alert on critical notification |
| MSF-04 | Multi-Sensory | Disable all audio/haptic |
| MSF-05 | Multi-Sensory | Priority-gated notification dismissal |
| ONB-01 | Onboarding | Skippable cinematic intro |
| ONB-02 | Onboarding | Guided demo in live system |
| ONB-03 | Onboarding | Live prompt within 5 seconds |
| ONB-04 | Onboarding | Persistent ambient hint dismissal |
| ONB-05 | Onboarding | Trust config in wizard |
| MKT-01 | Marketplace | Install Terminal Output Module |
| MKT-02 | Marketplace | Permission review before install |
| MKT-03 | Marketplace | Sideload via developer public key |
| MKT-04 | Marketplace | Standard Tier sandboxing |
| MKT-05 | Marketplace | Install alternative AI backend |
| COL-01 | Collaboration | One-time token invite |
| COL-02 | Collaboration | Live avatar presence |
| COL-03 | Collaboration | Following mode viewport sync |
| COL-04 | Collaboration | Dynamic role promotion |
| COL-05 | Collaboration | Guest identity in audit log |
| PER-01 | Performance | 60 FPS desktop target |
| PER-02 | Performance | 90 FPS VR target |
| PER-03 | Performance | Tactical Alert on FPS drop |
| PER-04 | Performance | Full keyboard nav of context menu |
| PER-05 | Performance | Screen reader accessibility |
