# TOS Alpha-2.1 Brain (Backend) Roadmap

This roadmap tracks the progress of the TOS backend, specifically focusing on the Logic Thread ("Brain"), AI engines, System Services, Sandboxing, and hardware-level compositor integrations.

## Architectural Refactoring & IPC Enforcement
- [x] **ServiceManager State Decoupling:** Refactor `ServiceManager` (`logger`, `settings`, `audio`, `ai`) to drop the `Arc<Mutex<TosState>>` payload to eliminate lock contention on the global state tree.
- [x] **Service IPC Routing:** Force all extracted services to communicate with the Brain exclusively through JSON-RPC payloads on the `IpcDispatcher`.
- [x] **State Delta IPC Schema:** Implemented global and granular (Hub/Sector) versioning with `handle_get_state_delta` for optimized synchronization (§3.3.2).
- [ ] **Standalone Service Binaries:** Extract `logger`, `settings`, and `marketplace` modules from the `tos-brain` codebase into standalone binary crates (e.g., `tos-settingsd`) that act as true external IPC clients.

## Backend Compositing & Wayland (§15)
- [x] **DMABUF Native Path:** Optimize the Linux backend for zero-copy frame buffer sharing with the Wayland compositor.
- [x] **Wayland DMABUF Logic:** Surface attachment and composite loop logic resolved.
- [x] **XR/Quest Swapchain:** Swapchain placeholder resolved.
- [ ] **Android NDK/Intents:** Implement real SurfaceControl and Intent pipelines (currently mocked).

## Services & Transports
- [x] **Multi-Sensory Audio Pipeline:** Implemented real `rodio` audio sink initialization and expanded the synth earcon set (§13.1).
- [x] **Remote Desktop Protocol (TDP):** Finalized the custom TOS Display Protocol for low-latency (<50ms) remote sector streaming (§3.3.4).
- [x] **Remote WebRTC Auto-Close:** Implemented graceful ICE/Sector teardown timers; replaced 5-sec sleep with state-verified reaping.
- [x] **Web Portal Security Protocol:** Implemented the `PortalService` for secure, timed token generation and Revocation (§13.8).
- [x] **Default Shell Hardening:** ShellApi now verifies binary existence and fallbacks to trusted system shells (/bin/bash, /bin/sh).

## Production-Grade Security & Isolation (Ecosystem Spec §1.4)
- [x] **Kernel-Level Sandboxing:** Move beyond mock permissions to actual Linux namespaces/cgroups for Standard Modules.
- [x] **Deep Inspection (Level 5) Audit:** Implement a cryptographic audit log for all privilege escalations.
- [x] **Manifest Signing:** Require signed `module.toml` for Marketplace installations to ensure supply-chain security.
- [x] **Sandbox Profile Expansion:** Strictly define and enforce Bubblewrap (`bwrap`) profiles mapping to specific `module.toml` granular permissions.
- [x] **Signature Forging:** `verify_manifest_signature` now performs real Edwards-curve (Ed25519) verification using the system's trusted root key.
- [x] **Interception Loophole:** Replaced naïve string matching with robust regex-based detection for root-level recursive deletes and raw device manipulation.
- [x] **Trusted Root Key:** `get_trusted_public_key()` provides the canonical system root hash for marketplace validation.

## AI Engine & Intelligence (§12)
- [x] **AI Mode Implementation:** Develop the `AiService` in the Brain to handle Natural Language Processing.
- [x] **Staged Command Generation:** Enable the AI to propose complex shell scripts for user review before execution.
- [x] **Contextual Awareness:** Feed system state (active sectors, search results, system logs) into the AI context for smarter assistance.
- [x] **Natural Language Search:** Replace string matching in SEARCH mode with semantic embedding-based retrieval. (Partial: IPC Routing Ready).
- [x] **Real LLM Integration:** LLM NLP is completely mocked with exact substring matching (`p.contains("list")`). Needs real API integration.

## Testing & Infrastructure
- [x] **Stress Testing:** Validated the <16ms IPC threshold locally under heavy load simulation (20+ sectors).
- [x] **stimulator_brain_node IPC Port:** Fix test crash where the `tos-brain` doesn't bind an IPC server port alongside the web-server for testing mocks.
- [ ] **Beta Branch Merge:** Consolidate all backends and services into the main production branch.
- [ ] **Developer SDK Guide:** Finalize documentation for module creators.

## Cross-Roadmap Dependencies
- **Natural Language Search** relies on the index building provided by the **Global Search & Indexing Service** (Ecosystem Roadmap).
- **Multi-Sensory Audio Pipeline** initialization blocks the **Multi-Sensory Audio Hooks** (Face Roadmap) from producing sound.
- **Wayland DMABUF Logic** and compositing pipelines must be finalized to unblock the Face's **Activity Context Live Thumbnails** (Face Roadmap).
- **Remote WebRTC Auto-Close** and **Remote Desktop Protocol (TDP)** stability block the advanced sync layers like **Multi-User Presence API** (Ecosystem Roadmap).
