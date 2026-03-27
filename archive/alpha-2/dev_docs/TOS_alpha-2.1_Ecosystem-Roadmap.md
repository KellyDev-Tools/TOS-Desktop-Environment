# TOS Alpha-2.1 Ecosystem & Services Roadmap

This roadmap tracks the development of the auxiliary services, modular ecosystem, and marketplace infrastructure that plug into the core logic of the TOS Brain.

## Auxiliary Services (§4)
*The following services run as independent processes and communicate with the Brain via IPC.*

- [x] **TOS Log Service (§19):** Implemented the unified logging daemon (`tos-loggerd`). It aggregates stdout/stderr, Brain telemetry, and remote events via a centralized TCP interface.
- [x] **Log Query Engine (§3.3.4):** Implemented structured JSONL logging and a high-performance query interface in `tos-loggerd`, enabling historical telemetry retrieval by source and limit via IPC.
- [x] **Settings Daemon (§26):** Finalize the JSON persistence layer and cascading resolution engine (Global -> Sector -> Application bounds) for user settings, themes, and sandboxing toggles.
- [x] **Global Search & Indexing Service:** Develop the background worker responsible for indexing local file systems, application content, and the unified log. Expose a gRPC/IPC interface for the Brain's generic SEARCH context.
- [x] **Priority Indicator Engine (§21):** Implemented the scoring service (`tos-priorityd`) that ranks tactical alerts (1 to 5) based on real-time system metrics and triggers haptic feedback.

## Shell ABI & Integrations (§27)
- [x] **Universal PTY Integration:** Support directory-aware navigation (OSC 7/1337) and rich telemetry feedback (OSC 900x) across all shell types (Fish, Zsh, Bash).
- [x] **JSON Context Export:** Allow applications to write standard JSON to stdout formatted within an OSC sequence to populate the Left Chip Column dynamically.

## Marketplace & Module Ecosystem (Ecosystem Spec §1 & §2)
- [x] **Application Models (`.tos-appmodel`):** Defined the manifest and interaction rules for deeply integrating standard Wayland/X11 applications into Level 3 (§8.2). Implemented `launch_app` and `close_app` with vertical hierarchy transitions.
- [x] **Sector Types (§2.1.1):** Created the blueprint engine (`SectorTemplate`) allowing users to spawn pre-configured hubs with specific shells, layouts, and initial environments via IPC.
- [x] **Theme Packaging (`.tos-theme`):** Defined the `.tos-theme` manifest and asset resolution logic. Implemented IPC-driven global theme switching (e.g., Classic LCARS to Tactical Amber) with real-time CSS variable injection in the Face.
- [x] **Terminal Output Modules (`.tos-terminal`):** Isolated the visual styling of the Terminal Canvas into modular layouts. Implemented IPC-driven hotswapping between Rectangular and Cinematic 3D terminal rendering engines.
- [x] **Marketplace Client:** Built the `tos-pkg` command-line utility for tactical module management and integrated a visual marketplace explorer into the System Settings modal.
- [x] **Module Contract Implementation:** Formalize the Rust traits and JSON boundaries for external Shell PTY binaries and AI LLM backends (Ecosystem Spec §1).
- [x] **Developer SDK:** Document the authoritative manifest schemas, PTY telemetry standards, and AI boundary protocols for third-party module authorship.

## Collaboration Hub (§13)
- [x] **Multi-User Presence API:** Flesh out the WebRTC data channel payloads to sync dual-sided chip states, active viewport titles, and cursor metadata between connected users.
- [x] **Follow Mode:** Implement the logic forcing connected peers to match zoom levels and viewport context of the "host" user.
- [x] **Web Portal Management:** Implemented the `PortalService` and IPC routines for secure, timed token generation and revocation (§13.8).

## Cross-Roadmap Dependencies
- **Settings Daemon** JSON persistence API must be completed to unblock the **Settings UI Panel** functionality (Face Roadmap).
- **Global Search & Indexing Service** indexes must exist before the backend can implement **Natural Language Search** (Brain Roadmap) embedding routing.
- **Universal OSC Scripts** and JSON Context definitions must be integrated into shells to unblock **Directory Context Previews** (Face Roadmap).
- **Multi-User Presence API** mapping is blocked by the completion of strong **Remote WebRTC Auto-Close / Remote Desktop Protocols (TDP)** (Brain Roadmap).
