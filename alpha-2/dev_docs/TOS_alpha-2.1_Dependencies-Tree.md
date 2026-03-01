# TOS Alpha-2.1 Cross-System Dependencies Map

This document maps out the hard execution blocks across the **Ecosystem**, **Brain**, and **Face** roadmaps. Tasks must be executed bottom-up according to this dependency tree to prevent development gridlock and orphaned UI states.

## 1. Ecosystem Blocks (Data & Services Foundation)
The Ecosystem's background services and IPC integrations are the bedrock of the system. Their omission directly blocks major logic routing and UI rendering features.

*   **Settings Daemon** (JSON persistence layer and cascading state resolution)
    *   **Blocks [FACE]:** **Settings UI Panel**. The Face cannot map dual-sided chips or read theme configurations without the daemon persisting and returning the data.
*   **Global Search & Indexing Service** (Daemon indexing the file system, apps, and logs)
    *   **Blocks [BRAIN]:** **Natural Language Search**. The Brain cannot implement LLM semantic embedding routing without the underlying database index to query.
*   **Universal OSC Scripts & JSON Context Export** (Shell hooks)
    *   **Blocks [FACE]:** **Directory Context Previews**. The Face cannot render inline file or image previews without the shell physically emitting the JSON context metadata upon `ls` or `cd`.

## 2. Brain Blocks (Logic & Hardware Transports)
The Brain's system-level hardware APIs and core connection protocols must be initialized before the UI can visualize them or the Ecosystem can sync them securely.

*   **Wayland DMABUF Logic & Compositing Pipelines** (Zero-copy surface attachment)
    *   **Blocks [FACE]:** **Activity Context Live Thumbnails**. The UI cannot render 10Hz live application previews on process chips without the backend compositor extracting and routing the DMABUF handles.
*   **Multi-Sensory Audio Pipeline** (Initialization of OS audio sinks via `cpal`/`rodio`)
    *   **Blocks [FACE]:** **Multi-Sensory Audio Hooks**. The React frontend cannot trigger "Earcons" upon zooming/mode-switching if the backend Rust audio sink is not open.
*   **Remote WebRTC Auto-Close & Remote Desktop Protocol (TDP)** (Connection teardown and stability)
    *   **Blocks [ECOSYSTEM]:** **Multi-User Presence API**. The ecosystem cannot map cursor sharing, follow modes, or active viewport syncs if the underlying WebRTC socket transport drops randomly or fails to close properly.

## 3. Recommended Execution Priority (Bottom-Up)
To safely navigate these blockers, Alpha-2.1 development MUST proceed in this order:

1.  **Phase 1 (The Bedrock):** Build out the Ecosystem Auxiliary Services (Settings Daemon, Global Search) and Brain Hardware APIs (Wayland Compositor, Audio Sinks, WebRTC).
2.  **Phase 2 (The Translators):** Build the Ecosystem Shell Scripts (OSC emission) and Brain AI Routing (Natural Language integrations).
3.  **Phase 3 (The Interface):** Build the Face UI Overlays (Settings Panel, Live Thumbnails, Directory Previews, Audio Hooks) which simply consume the structured data pipelines established in Phases 1 & 2.
