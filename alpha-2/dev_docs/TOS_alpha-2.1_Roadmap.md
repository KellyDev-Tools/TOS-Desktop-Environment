# TOS Alpha-2.1: High-Fidelity & AI Integration Roadmap

Following the successful completion of the Alpha-2 functional prototype, the Alpha-2.1 phase focuses on "Sensory & Cognitive Intelligence." This phase moves away from ASCII-simulated UI and basic command sniffing toward a rich, AI-augmented experience with production-grade security.

## AI Engine & Intelligence (§12)
- [x] **AI Mode Implementation:** Develop the `AiService` in the Brain to handle Natural Language Processing.
- [x] **Staged Command Generation:** Enable the AI to propose complex shell scripts for user review before execution.
- [x] **Contextual Awareness:** Feed system state (active sectors, search results, system logs) into the AI context for smarter assistance.
- [/] **Natural Language Search:** Replace string matching in SEARCH mode with semantic embedding-based retrieval. (Partial: IPC Routing Ready)

## High-Fidelity Rendering & Themes (§9, §10, §18)
- [x] **Web Profile DOM Implementation:** Transition the Face layer from terminal-mock to a rich React/Vite-based LCARS interface.
- [x] **Cinematic Modules:** Implement the "Cinematic Triangular Module" with GPU-accelerated transitions.
- [x] **Dynamic Theme Engine:** Implement runtime CSS variable injection and multi-sensory asset swapping (SFX/Haptics).
- [x] **DMABUF Native Path:** Optimize the Linux backend for zero-copy frame buffer sharing with the Wayland compositor.
- [x] **Configurable Bezel Architecture:** Implement omni-directional slots (Top, Left, Right) with modular component docking and downward/lateral slot projection.
- [x] **Top Bezel Evolution:** Partition the Top Bezel into Left/Center/Right zones (Left: Handles, Center: Telemetry, Right: System Controls).

## Production-Grade Security & Isolation (§18.4)
- [x] **Kernel-Level Sandboxing:** Move beyond mock permissions to actual Linux namespaces/cgroups for Standard Modules.
- [x] **Deep Inspection (Level 5) Audit:** Implement a cryptographic audit log for all privilege escalations.
- [x] **Manifest Signing:** Require signed `module.toml` for Marketplace installations to ensure supply-chain security.

## Hardware & Mobile Finalization (§15)
- [ ] **Android NDK Integration:** Replace the Android mock with the actual SurfaceControl and input event pipeline.
- [ ] **OpenXR / Meta Quest:** Implement the spatial UI shell for VR/XR sectors, enabling floating Hub windows in 3D space.
- [ ] **Remote Desktop Protocol (TDP):** Finalize the custom TOS Display Protocol for low-latency (<50ms) remote sector streaming.

## Final Release Candidate (RC)
- [ ] **Stress Testing:** Validate the <16ms IPC threshold under heavy load (20+ sectors).
- [ ] **End-to-End Documentation:** Finalize the "User Manual" and "Developer SDK Guide."
- [ ] **Beta Branch Merge:** Consolidate all backends and services into the main production branch.

## Alpha-2.1 Validation Findings & Pending Tasks
The following gaps, stubs, and hardcoded assumptions were identified during the Alpha-2 validation and must be addressed:

### Rendering Engine (`src/platform/*`)
- [x] **Wayland DMABUF (Linux):** `update_surface` drops real attachment logic; `composite()` only logs. Needs actual compositor attachment.
- [x] **XR/Quest Swapchain:** `QuestRenderer` only creates a placeholder handle and polls empty inputs.
- [ ] **Android NDK/Intents:** Implement real SurfaceControl and Intent pipelines (currently mocked).
- [ ] **Level 3-5 Rendering:** `ApplicationFocus`, `DetailView`, and `BufferView` fall completely flat on the terminal proxy UI. Currently a hardcoded output block.

### Security & Sandboxing (`src/modules/sandbox/mod.rs` & `src/brain/ipc_handler.rs`)
- [ ] **Signature Forging:** `verify_manifest_signature` returns a hardcoded `true`. Must check real Edwards-curve signatures.
- [ ] **Interception Loophole:** `is_dangerous` uses naïve string matching (`rm -rf`); `rm -r /` bypasses it. Needs abstract semantic analysis.
- [ ] **Mock Public Key:** `get_trusted_public_key()` hardcodes an Ed25519 hash literal purely to pass tests.

### System Services & AI
- [ ] **Multi-Sensory Audio:** `AudioService::play_earcon` pipes to stdout; needs real `cpal`/`rodio` audio sink initialization.
- [ ] **AI Context Queries:** LLM NLP is completely mocked with exact substring matching (`p.contains("list")`). Needs real LLM integration.
- [ ] **Remote WebRTC Auto-Close:** Uses an arbitrary 5-second `tokio::time::sleep` on close without heartbeat negotiation.
- [ ] **Default Shell:** `std::env::var("SHELL")` fallback misses setting validation (default `fish`).

### Testing & Infrastructure
- [ ] **stimulator_brain_node.rs Timeout:** The test crashes because `tos-brain` doesn't bind an IPC server port alongside the web-server for the testing context. Needs an IPC port.
