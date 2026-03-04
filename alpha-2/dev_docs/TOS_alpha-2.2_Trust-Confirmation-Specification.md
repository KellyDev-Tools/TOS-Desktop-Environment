# TOS Alpha-2.1 Command Trust & Confirmation System
### Specification v1.0 — Replaces Tactile Confirmation (Architecture Specification §17.2)

---

## 1. Philosophy

TOS does not decide what is dangerous. The user does. The trust system exists to ensure that a user who has not yet expressed an opinion about a command class gets a moment to notice what they are about to do — nothing more. Once they have expressed an opinion, the system remembers it and never asks again.

There is no floor. There are no unpromotable patterns. A user who wants `rm -rf /` to run without any friction can configure exactly that. The system's job is to make the first encounter with a command class visible, and then get out of the way permanently.

Two principles:

- **INFORM, DON'T BLOCK.** Warning chips are non-blocking. The user can proceed immediately. The chip is information, not a gate.
- **EXPLICIT PROMOTION.** Trust is never earned automatically by use count or time. The user explicitly decides when a command class is trusted. The system never makes that decision on their behalf.

> **REPLACES:** The Tactile Confirmation Slider (Architecture Specification §17.2) is retired. The slider interaction is removed entirely. The `update_confirmation_progress` IPC message is deprecated. All dangerous command handling is replaced by the system described in this document.

---

## 2. Trust Configuration

### 2.1 First-Run Trust Setup

During the onboarding flow (Onboarding Specification §3.3, Step 0 — inserted before Step 1), the user is presented with a single trust configuration screen. This is the only time TOS actively prompts the user to think about trust posture. It does not happen again unless the user navigates to **Settings → Security → Trust**.

The screen presents each command class with a toggle: **Warn** or **Trust**. No default is pre-selected. The user must make an explicit choice for each class before proceeding. They can also choose **Skip** to defer all classes to **Warn** and configure later.

This replaces the onboarding profile question entirely — instead of picking "muggle" or "power user," the user simply states their intent for each class directly.

### 2.2 Trust Levels

Each command class has exactly one of two trust levels at any time:

| Level | Behaviour |
| :--- | :--- |
| **WARN** | A non-blocking warning chip appears in the prompt area when the command is staged. The user can proceed immediately or dismiss. |
| **TRUST** | The command runs without any chip or interruption. No UI. No friction. |

There are no intermediate levels. No countdowns. No sliders. No re-authentication. Trust is binary and user-controlled.

### 2.3 Command Classes

The trust system covers two categories of command:

**Explicit Classes** — specific command families identified by the Brain's command classifier:

| Class ID | Covers | Default at First-Run Setup |
| :--- | :--- | :--- |
| `privilege_escalation` | `sudo`, `su`, `doas`, `pkexec` | User choice (no pre-selection) |
| `recursive_bulk` | Any command operating on 10+ files, or using `-r`/`-R`/`--recursive` flags with a destructive verb | User choice |

**Implicit Bulk Detection** — commands not in an explicit class but detected as operating on a large file set at execution time. When the Brain's PTY analysis estimates a command will affect 10 or more filesystem objects, it is temporarily treated as `recursive_bulk` for that invocation only. This is not a persistent class assignment — it fires per-command based on argument analysis.

> **SCOPE NOTE:** The original spec listed six command classes. This spec narrows to two based on the decision to focus on privilege escalation and recursive/bulk operations. The other classes (network changes, process kills, package management, disk operations) are not covered by the trust system — they run without friction regardless of trust configuration. This keeps the system focused rather than exhaustive.

---

## 3. The Warning Chip

When a command in a **WARN** class is staged in the prompt, a warning chip appears in the right chip column immediately above the prompt. It is non-blocking — the user presses Enter as normal to run the command. The chip is information, not a gate.

### 3.1 Chip Anatomy

```
┌──────────────────────────────────────────────────┐
│  ⚠  sudo apt remove nginx          [Trust Class] │
│     Privilege escalation — runs as root           │
└──────────────────────────────────────────────────┘
```

- **⚠ Icon** — amber warning glyph, visually distinct from AI chips (✦) and system chips.
- **Command echo** — repeats the staged command so the user sees it in context.
- **Class label** — human-readable description of why this chip appeared.
- **[Trust Class]** — a single tap permanently promotes this command class to TRUST. The chip disappears immediately and never appears for this class again.
- The chip dismisses automatically when the prompt is cleared or a different command is staged.

### 3.2 Chip Behaviour for Bulk Detection

For implicitly detected bulk operations, the chip shows an estimated file count:

```
┌──────────────────────────────────────────────────┐
│  ⚠  rm *.log                       [Trust Class] │
│     Affects ~47 files in /var/log                 │
└──────────────────────────────────────────────────┘
```

The file count estimate is produced by the Brain's argument analyser before the command runs. It is an estimate, not a guarantee — the chip makes this clear with a `~` prefix.

### 3.3 What the Warning Chip Does Not Do

- It does not prevent Enter from running the command.
- It does not add a delay or countdown.
- It does not require any interaction to proceed.
- It does not appear twice for the same command if the user has already seen it and chosen not to promote.
- It does not appear at all once the class is promoted to TRUST.

---

## 4. Promoting and Demoting Trust

### 4.1 Promoting to TRUST

Trust is promoted explicitly in one of two ways:

**Inline:** tapping **[Trust Class]** on a warning chip immediately promotes that class to TRUST. The chip vanishes. No confirmation. No "are you sure." The user said trust, the system trusts.

**Settings panel:** **Settings → Security → Trust** lists all command classes with their current level. Each has a toggle. Flipping a toggle takes effect immediately for all sectors.

### 4.2 Per-Sector Trust Override

The global trust configuration applies system-wide by default. Individual sectors can override any class:

- A `prod` sector might have `privilege_escalation` set to WARN even if the global config has it as TRUST — a deliberate extra layer of caution for sensitive environments.
- A `scratch` sector might have everything set to TRUST for rapid experimentation.

Per-sector overrides are set via **Sector Settings** (accessible from the sector tile context menu at Level 1 → **Sector Settings → Trust**).

The resolution order mirrors the existing Settings Daemon cascade:

```
Sector Override → Global Config
```

If a sector has no override for a class, the global config applies.

### 4.3 Demoting to WARN

A TRUST class can be returned to WARN at any time via **Settings → Security → Trust** or via the sector settings panel. Demotion takes effect immediately — the next invocation of a command in that class will show the warning chip again.

There is no automatic demotion. Trust does not expire. The user chose it; only the user removes it.

---

## 5. Brain Implementation

### 5.1 Command Classification

The Brain's command dispatcher runs a classification pass on every staged command before PTY submission. The classifier operates in two stages:

**Stage 1 — Explicit class matching:** checks the command verb against the class registry. `sudo`, `su`, `doas`, `pkexec` match `privilege_escalation`. Commands using `-r`, `-R`, or `--recursive` flags with destructive verbs (`rm`, `mv`, `cp`, `chmod`, `chown`) match `recursive_bulk`.

**Stage 2 — Implicit bulk detection:** for commands not matched in Stage 1, the Brain estimates the number of filesystem objects affected by expanding globs and counting directory entries where applicable. If the estimate reaches 10 or more, the command is flagged for the implicit bulk warning for that invocation only.

### 5.2 Trust Check Flow

```
Command staged in prompt
        │
        ▼
Run classifier (Stage 1 + Stage 2)
        │
   Match found?
   ┌────┴────┐
  YES       NO
   │         │
Check trust  Execute
level        normally
   │
 TRUST?
 ┌──┴──┐
YES    NO
 │      │
Execute Emit warning chip IPC
normally then execute on Enter
```

The command is never held. In the WARN path, the chip is emitted and the prompt remains live. The user presses Enter; the command runs.

### 5.3 IPC Contracts

| Message | Effect |
| :--- | :--- |
| `trust_promote:<class_id>` | Promotes a command class to TRUST globally |
| `trust_demote:<class_id>` | Returns a command class to WARN globally |
| `trust_promote_sector:<sector_id>:<class_id>` | Sector-level TRUST override |
| `trust_demote_sector:<sector_id>:<class_id>` | Sector-level WARN override |
| `trust_clear_sector:<sector_id>:<class_id>` | Removes sector override, falls back to global |
| `trust_get_config` | Returns full trust config for all classes (global + all sector overrides) |

### 5.4 Deprecated IPC

The following messages from Architecture Specification §28.2 are deprecated by this spec and should be removed from the IPC handler:

| Deprecated Message | Reason |
| :--- | :--- |
| `update_confirmation_progress:` | Tactile slider retired |

---

## 6. Settings Integration

**Settings → Security → Trust** presents the full trust configuration:

```
Settings → Security → Trust

  Command Classes                    Global      This Sector
  ─────────────────────────────────────────────────────────
  Privilege Escalation               [ WARN ●]   [Use Global ▾]
  (sudo, su, doas, pkexec)

  Recursive / Bulk Operations        [● TRUST]   [ WARN ▾]
  (10+ files, -r/-R flags)

  ─────────────────────────────────────────────────────────
  Bulk Detection Threshold           [10 files ▾]
  (implicit bulk warning triggers above this count)
```

The sector column shows **Use Global** when no override is set, or the override value if one exists. Changing the sector column sets or clears a sector-level override without affecting the global config.

---

## 7. Relationship with Existing Architecture

### 7.1 Replaces

This spec fully replaces Architecture Specification §17.2 (Tactile Confirmation). All references to the confirmation slider, `update_confirmation_progress` IPC, and the "drag-controlled element in the bezel" from the Alpha-2.2 Production Roadmap §3 are superseded by this document.

### 7.2 Preserves

Architecture Specification §17.3 (Voice Confirmation fallback for dangerous actions in voice input mode) is unaffected. Voice-submitted commands that match a WARN class still trigger the voice confirmation path as specced — the warning chip system applies to prompt-submitted commands only.

Architecture Specification §17.1 (Prompt Locking during Tactical Reset / Level 6) is unaffected.

The Workflow Agent behavior module (AI Co-Pilot Specification §6.2) routes staged commands through the same trust check. A workflow agent staging a `privilege_escalation` command in a WARN sector will produce a warning chip exactly as a manually typed command would.

---

*TOS Alpha-2.1 // Command Trust & Confirmation System v1.0 // Replaces Architecture Specification §17.2*
