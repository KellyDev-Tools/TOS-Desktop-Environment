Unified Implementation Plan — LCARS Spatial Desktop Environment SDE (v4.5)

Executive summary  
This document (v4.5) extends v4.4 by integrating a first‑class command output model and the recommended dual‑mode presentation approach. It preserves the touch/gesture model, Nushell command composition flow, Orbital Context, Payload Mode, Clusters, Subspace Layer, and JSON‑RPC bridge, and adds a complete output surface specification (docked LCARS frame and cinematic scroll), settings, RPC methods, accessibility mirroring rules, security constraints, and developer tasks required to implement reliable, discoverable, and accessible command output.

---

Scope and goals
- Primary goal: Ensure every command has a clear, configurable, secure, and accessible destination while preserving the spatial, touch‑first UX.  
- Included: Input FSM, Orbital UI, Payload Mode, Clusters, Subspace Layer, session/power, portals/sandboxing, packaging, accessibility, and the output subsystem (presentation modes, settings, RPC).  
- Design principle: Default to functionality and accessibility (docked terminal) and offer immersive presentation (cinematic scroll) as an opt‑in enhancement. Per‑command and per‑profile defaults let power users tune behavior.

---

Core architecture and persistence
- Input FSM and gesture plumbing: Multi‑touch parsing, configurable thresholds, compositor integration, telemetry hooks, and cancel rules remain unchanged.  
- Session and persistence: Session store checkpoints window states, clusters, camera transforms, open payloads, and terminal composition buffer. Output transcripts are checkpointed and restorable.  
- Power and multi‑display: Integrate with systemd/logind for suspend/resume; per‑display camera transforms; payloads and outputs may be pinned to displays.  
- Security and portals: All file/device/print transfers and exports use portal mediation. Destructive commands require explicit terminal confirmation tokens before execution. Audit logs are local and opt‑in.

---

Command output handling and presentation modes

Recommendation approach
- Support both modes as first‑class: Docked LCARS Frame (default) and Cinematic Scroll (optional).  
- Default behavior: Docked LCARS Frame for everyday use, accessibility, and persistence. Cinematic Scroll available for demos, non‑critical logs, or user preference.  
- Per‑command and per‑profile rules: Commands and profiles can declare preferred output mode; user settings and quick toggles override defaults.

Docked LCARS Frame features
- Placement and behavior: Docked to bottom by default; configurable to left/right/top or detachable as floating LCARS window. Focused by Orbital compose actions.  
- Capabilities: Scrollback, search, copy, ANSI color, hyperlinks, inline images via portal, progress bars, structured views (JSON/tree), exit status, and pin/export actions.  
- Persistence and undo: Output saved in session checkpoints; lines and command results can be pinned to clusters or exported via portal; undo affordances for recent destructive operations.  
- Accessibility: Full AT‑SPI/ARIA roles, text scaling, high contrast, and screen reader announcements for new output.

Cinematic Scroll features
- Presentation: Camera‑anchored scrolling text plane that recedes into the scene for cinematic effect.  
- Use cases: Demos, long logs, non‑interactive notifications, and aesthetic visualizations.  
- Mirroring rule: Cinematic output must mirror to the docked transcript by default to preserve accessibility, searchability, and persistence unless the user explicitly disables mirroring.  
- Controls and safety: Speed, font size, and persistence toggles; exported or pinned cinematic output must pass portal checks before sharing.

Anchored output and object association
- Anchored surfaces: Outputs can be anchored to objects, clusters, or camera positions for contextual relevance. Anchored outputs behave like docked frames but are visually attached to the object and follow camera transforms.

---

RPC, settings, accessibility, and testing

JSON‑RPC additions
- New methods:  
  - output.create({commandId, mode, title, persistent}) -> outputId  
  - output.append({outputId, chunk, stream})  
  - output.close({outputId, status})  
  - output.pin({outputId, target})  
  - output.search({outputId, query}) -> matches  
  - output.export({outputId, format}) -> path  
  - output.setProfile({outputId, profileName})  
- Contract: Versioned JSON Schema for each method; error codes for permission, not found, quota, and portal denial. All output exports and pins must validate via portal.request before completing.

Settings and profiles
- Profile model: Terminal profiles with GNOME Terminal–style options plus LCARS additions. Profiles can be applied globally or per‑output.  
- Suggested settings keys:  
`json
{
  "terminal.profile.default": "LCARS",
  "terminal.profiles": {
    "LCARS": {
      "font.family": "Monospace",
      "font.size": 13,
      "font.bold": true,
      "color.scheme": "lcars-dark",
      "background.opacity": 0.92,
      "scrollback.lines": 10000,
      "wrap.mode": "word"
    }
  },
  "output.mode.default": "docked",
  "output.cinematic.enabled": true,
  "output.cinematic.font.size": 18,
  "output.cinematic.persistent_copy": true,
  "output.pinonerror": true,
  "output.autosaveto_session": true
}
`

Accessibility and localization
- Mirroring for accessibility: Cinematic output mirrored to docked transcript for screen readers and search.  
- AT‑SPI/ARIA mapping: Expose semantic roles for Orbital arcs, payloads, clusters, outputs, and terminal HUD.  
- Keyboard parity: Full keyboard navigation and shortcuts for toggling output modes, pinning, searching, and exporting.  
- Localization: Translation pipeline and RTL support for all output renderers.

Testing and telemetry
- Tests: Unit tests for output.* RPC, integration tests for mirroring and portal flows, performance tests for cinematic rendering, and accessibility tests for screen reader announcements.  
- A/B testing: Default output mode and cinematic mirroring behavior.  
- Telemetry: Anonymized metrics for mode preference, misfires, and performance; opt‑outable.

---

Developer notes and next steps
- Finalize RPC schema: Add JSON Schema for output.* methods and integrate into the existing RPC contract.  
- Implement docked frame first: Prioritize docked LCARS frame for accessibility and persistence, then add cinematic renderer consuming the same stream.  
- Portal integration: Ensure output.export and output.pin call portal.request and respect sandbox policies.  
- Session store: Extend session checkpointing to include output transcripts and pinned outputs; define WAL and migration hooks.  
- Accessibility checklist: Map all output interactions to AT‑SPI/ARIA roles and verify with screen reader tests.  
- Prototype profile editor: UI for GNOME Terminal–style settings and LCARS theme presets.  
- Security policy: Enforce confirmation tokens for destructive commands and require portal mediation for exports and cross‑app pins.  
- Documentation: Update onboarding to demonstrate output modes and profile switching; include guidance on when cinematic mode is appropriate.