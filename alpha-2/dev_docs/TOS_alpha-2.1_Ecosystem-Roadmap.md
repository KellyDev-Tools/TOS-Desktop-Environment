# TOS Alpha-2.1 Ecosystem & Services Roadmap

This roadmap tracks the development of the auxiliary services, modular ecosystem, and marketplace infrastructure that plug into the core logic of the TOS Brain.

## Auxiliary Services (§4)
*The following services run as independent processes and communicate with the Brain via IPC.*

- [ ] **TOS Log Service (§19):** Implement the unified logging daemon. It must aggregate standard stdout/stderr from user applications, background telemetry from the Brain, and remote sector events into a unified, queryable database (`~/.local/share/tos/logs/`).
- [ ] **Settings Daemon (§26):** Finalize the JSON persistence layer and cascading resolution engine (Global -> Sector -> Application bounds) for user settings, themes, and sandboxing toggles.
- [ ] **Global Search & Indexing Service:** Develop the background worker responsible for indexing local file systems, application content, and the unified log. Expose a gRPC/IPC interface for the Brain's generic SEARCH context.
- [ ] **Priority Indicator Engine (§21):** Implement the scoring service that listens to system events, ranks alerts (1 to 5), and triggers visual or auditory haptic events for high-priority notifications.

## Shell ABI & Integrations (§27)
- [ ] **Universal OSC Scripts:** Finalize shell integration scripts for `Bash`, `Zsh`, and `Fish` to automatically emit OSC 1337 sequences for directory changes, command timings, and exit codes.
- [ ] **JSON Context Export:** Allow applications to write standard JSON to stdout formatted within an OSC sequence to populate the Left Chip Column dynamically.

## Marketplace & Module Ecosystem (Ecosystem Spec §1 & §2)
- [ ] **Application Models (`.tos-app`):** Define the manifest and interaction rules for deeply integrating standard Wayland/X11 applications into Level 3 (e.g., overriding controls, defining zoom behavior, and search indexing).
- [ ] **Sector Types (`.tos-sector`):** Create the blueprint engine allowing users to spawn pre-configured hubs with specific shells, layouts, and pre-pinned dual-sided chips.
- [ ] **Theme Packaging (`.tos-theme`):** Finalize the distribution mechanism for custom CSS variables, fonts, and Audio themes.
- [ ] **Terminal Output Modules (`.tos-terminal`):** Isolate the React/CSS styling of the Terminal Canvas into installable packages to allow switching between Rectangular and Cinematic layouts.
- [ ] **Marketplace Client:** Build a command-line utility (`tos-pkg`) and an accompanying visual application to browse, sign-verify, and install these modules.

## Collaboration Hub (§13)
- [ ] **Multi-User Presence API:** Flesh out the WebRTC data channel payloads to sync dual-sided chip states, active viewport titles, and cursor metadata between connected users.
- [ ] **Follow Mode:** Implement the logic forcing connected peers to match zoom levels and viewport context of the "host" user.

## Cross-Roadmap Dependencies
- **Settings Daemon** JSON persistence API must be completed to unblock the **Settings UI Panel** functionality (Face Roadmap).
- **Global Search & Indexing Service** indexes must exist before the backend can implement **Natural Language Search** (Brain Roadmap) embedding routing.
- **Universal OSC Scripts** and JSON Context definitions must be integrated into shells to unblock **Directory Context Previews** (Face Roadmap).
- **Multi-User Presence API** mapping is blocked by the completion of strong **Remote WebRTC Auto-Close / Remote Desktop Protocols (TDP)** (Brain Roadmap).
