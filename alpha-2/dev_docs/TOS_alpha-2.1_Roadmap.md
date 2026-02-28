# TOS Alpha-2.1: High-Fidelity & AI Integration Roadmap

Following the successful completion of the Alpha-2 functional prototype, the Alpha-3 phase focuses on "Sensory & Cognitive Intelligence." This phase moves away from ASCII-simulated UI and basic command sniffing toward a rich, AI-augmented experience with production-grade security.

## Phase 10: AI Engine & Intelligence (§12)
- [x] **AI Mode Implementation:** Develop the `AiService` in the Brain to handle Natural Language Processing.
- [x] **Staged Command Generation:** Enable the AI to propose complex shell scripts for user review before execution.
- [x] **Contextual Awareness:** Feed system state (active sectors, search results, system logs) into the AI context for smarter assistance.
- [/] **Natural Language Search:** Replace string matching in SEARCH mode with semantic embedding-based retrieval. (Partial: IPC Routing Ready)

## Phase 11: High-Fidelity Rendering & Themes (§9, §10, §18)
- [x] **Web Profile DOM Implementation:** Transition the Face layer from terminal-mock to a rich React/Vite-based LCARS interface.
- [x] **Cinematic Modules:** Implement the "Cinematic Triangular Module" with GPU-accelerated transitions.
- [x] **Dynamic Theme Engine:** Implement runtime CSS variable injection and multi-sensory asset swapping (SFX/Haptics).
- [ ] **DMABUF Native Path:** Optimize the Linux backend for zero-copy frame buffer sharing with the Wayland compositor.

## Phase 12: Production-Grade Security & Isolation (§18.4)
- [x] **Kernel-Level Sandboxing:** Move beyond mock permissions to actual Linux namespaces/cgroups for Standard Modules.
- [x] **Deep Inspection (Level 5) Audit:** Implement a cryptographic audit log for all privilege escalations.
- [x] **Manifest Signing:** Require signed `module.toml` for Marketplace installations to ensure supply-chain security.

## Phase 13: Hardware & Mobile Finalization (§15)
- [ ] **Android NDK Integration:** Replace the Android mock with the actual SurfaceControl and input event pipeline.
- [ ] **OpenXR / Meta Quest:** Implement the spatial UI shell for VR/XR sectors, enabling floating Hub windows in 3D space.
- [ ] **Remote Desktop Protocol (TDP):** Finalize the custom TOS Display Protocol for low-latency (<50ms) remote sector streaming.

## Phase 14: Final Release Candidate (RC)
- [ ] **Stress Testing:** Validate the <16ms IPC threshold under heavy load (20+ sectors).
- [ ] **End-to-End Documentation:** Finalize the "User Manual" and "Developer SDK Guide."
- [ ] **Beta Branch Merge:** Consolidate all backends and services into the main production branch.
