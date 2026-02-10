LCARS Spatial Desktop Environment SDE — Unified Implementation Plan v4.3

---

Executive summary
This document (v4.3) extends the LCARS SDE plan with a complete touch input model and the missing ZUI features: Orbital Context, Payload Mode, Clusters, Subspace Layer, and Computer Query. The persistent Nushell command bar remains the canonical action surface; contextual actions generate or append structured Nushell commands via a JSON‑RPC bridge. This version adds the Input FSM, gesture thresholds, telemetry hooks, and RPC contracts required to implement the long press and short press behaviors and the command composition flow.

---

Additions to core architecture input FSM and gesture plumbing
I/O and Kernel libinput extension  
- Multi‑touch parsing extended to expose tap, double‑tap, long‑press, deep‑press, press‑and‑drag, and three‑finger tap.  
- Configurable thresholds exposed to the UI via the settings registry.

Core Engine and Scene Graph  
- Input FSM integrated into the compositor to disambiguate short press versus long press before dispatching events to the UI layer.  
- State telemetry hooks added for tuning gesture thresholds during testing.

Bridge JSON RPC new methods  
- input.stateTransition(state) — notifies Nushell and UI of FSM changes.  
- orbital.open(objectId) — request orbital options and command templates.  
- payload.create(sourcePath) — create floating payload and return payloadId.  
- cluster.create(name, coords, members) — persist cluster bookmark.

---

Orbital Context Nushell integration and command templating
Orbital Context radial UI  
- Trigger Long press default ≥500 ms on an object.  
- Visual Camera zooms ~1.5× and the object is encircled by four LCARS arcs: North, East, South, West.  
- Arc semantics  
  - North Execute Open; Open With  
  - East Modify Rename; Edit Tags  
  - South Destructive Delete; Shred  
  - West Transport Move; Copy (activates Payload Mode)

Nushell Touch to Syntax Bridge  
- Selecting an orbital arc prepares a command template in the terminal (for example mv /path/file ) rather than immediately executing.  
- Tapping object tokens while in Composing state appends path tokens or flags to the command bar.  
- The terminal remains editable; the user can type, speak, or accept defaults before pressing Enter to execute.

Command templating examples

| Action | Template | Behavior |
|---|---|---|
| Rename | mv /src/file  | Focus cursor after trailing space for new name |
| Copy Payload create | cp /src/file /dest/ | Payload holds /src/file until drop |
| Delete | rm /src/file | Shows confirmation token in terminal |

---

Payload Mode Spatial Grouping and navigation
Payload Mode visible clipboard  
- Create Selecting Copy or Move from Orbital creates a Payload widget docked to the viewport edge.  
- Transport User pans and zooms to destination and drags Payload onto target; drop triggers cp or mv command execution via Nushell.  
- Preview While dragging, the terminal shows a live preview of the command to be executed.

Clusters named regions  
- Definition A Cluster is a saved bounding region with metadata name, x, y, z.  
- Creation Lasso select windows then Create Cluster: <name>.  
- Navigation Command Bar exposes Jump to <Cluster> which triggers a smooth camera flight.  
- Persistence Clusters stored in settings_registry.json and exposed to the UI module.

---

Subspace Layer HUD and Computer Query global search
Subspace Layer HUD  
- System status elements such as Wi‑Fi, battery, and time live on the Layer Shell and do not zoom with the canvas.  
- Notifications use edge pulses; swiping from the edge materializes a Comm Badge that can be dragged onto the canvas.

Computer Query global search  
- Trigger Three‑finger tap or voice command “Computer”.  
- Behavior Background dims and the Command Bar highlights; query runs fd or Nushell search.  
- Reveal Matches are highlighted on the canvas and the camera zooms to the best match.

---

Interaction model short press long press thresholds and FSM
Default timing user configurable

| Interaction | Default Duration | Primary Effect |
|---|---:|---|
| Short Press Tap | ≤ 200 ms | Open, Run, or Append token when composing |
| Long Press | ≥ 500 ms | Enter Orbital Context to inspect and compose |
| Press and drag | Hold plus move | Selection or Payload drag |
| Three finger tap | Instant | Computer Query global search |

Finite State Machine FSM core states and transitions  
- Idle Tap opens; Long press opens Orbital.  
- Composing Entered after Orbital selection or explicit compose toggle; Tap appends token; Long press opens advanced orbital options.  
- Payload After Copy or Move; Drag moves payload; Drop executes command.  
- Selecting Press and drag lasso windows to create a Cluster.

UX safeguards  
- Touch down feedback with immediate ripple or scale and subtle haptic.  
- Cancel rules: moving finger more than 12 to 20 px during long press cancels Orbital entry.  
- Undo buffer: appended tokens and executed commands present an undo affordance in the terminal HUD.  
- Accessibility: keyboard and mouse equivalents (right click opens Orbital; Enter confirms; Space toggles compose) and voice command parity.

---

Roadmap updates and testing plan
Phase 1 Engine — unchanged, plus implement input FSM and gesture telemetry hooks.  
Phase 2 WPE Integration — add Orbital UI components and Layer Shell hooks.  
Phase 3 Command Link — implement JSON RPC methods for orbital, payload, cluster, and input state; wire Nushell templates.  
Phase 4 Legacy and Polish — add XWayland, onboarding, gesture sensitivity settings, and telemetry driven threshold tuning.

Testing and rollout  
- A/B testing for default thresholds (200, 300, 400 ms tap boundary).  
- Telemetry for misfires and false positives.  
- Interactive onboarding that demonstrates tap versus long press, Payload Mode, and Cluster navigation.  
- Power user beta to validate Nushell templates and command composition flow.

---

Developer notes and next steps
- Settings exposure Add gesture.sensitivity, longPress.duration, and compose.mode to settings_registry.json.  
- RPC contract Finalize JSON RPC schema for orbital.open, payload.create, cluster.create, and input.stateTransition.  
- Security Ensure destructive actions such as shred and rm require explicit terminal confirmation before execution.  
- Telemetry privacy Collect only anonymized gesture metrics and expose an opt out in settings.

---

Appendix suggested files and registry entries
settings_registry.json suggested keys

`json
{
  "gesture.sensitivity": 50,
  "longPress.duration": 500,
  "compose.mode": "auto",
  "clusters": [
    {
      "name": "Web Dev",
      "x": 5000,
      "y": 2000,
      "z": 1.0,
      "members": ["windowId1", "windowId2"]
    }
  ]
}
`

---
